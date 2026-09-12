use crate::geometry::ExPolygon;
use crate::project_slice::{
    extruders,
    island_print_order::{self, PreparedPostIslandPrintOrder},
    perimeters::classic::traversal::PreparedPostClassicTraversal,
};

mod brim;
mod cooling;
mod expression;
mod fan_mover;
mod file_start;
mod finish;
pub(super) mod footprint;
mod header;
mod layer_gcode;
mod layers;
mod lexer;
mod machine;
pub(in crate::project_slice) mod motion;
mod object;
mod offset;
mod placeholders;
mod processor;
mod skirt;
mod small_area;
mod spiral_vase;
mod tags;
mod template;
#[cfg(test)]
mod tests;
mod timelapse;
mod value;
use crate::{GenerationMetadata, SliceError};

pub(super) fn emit(
    prepared: &mut PreparedPostIslandPrintOrder,
    metadata: GenerationMetadata,
) -> Result<Vec<u8>, SliceError> {
    let traversal = &prepared
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let brim = brim::BrimPlan::generate(traversal)?;
    let skirt = skirt::SkirtPlan::generate(traversal, brim.as_ref())?;
    let first_layer_bounds =
        footprint::first_layer_bounds(traversal, skirt.as_ref(), brim.as_ref());
    let mut output = Vec::new();
    file_start::append(&mut output, traversal, metadata, first_layer_bounds)?;
    let tags = tags::Tags::of(traversal);
    header::append_header(&mut output, metadata, traversal);
    // `GCode.cpp` + Orca export layout: BBL keeps the config block up front;
    // the compatible flavor moves it after the tail statistics.
    if tags.is_bbl()
        && let Some(config) = &traversal.config_block
    {
        output.extend_from_slice(config);
    }
    header::append_width_block(&mut output, traversal);
    output.extend_from_slice(b"; EXECUTABLE_BLOCK_START\n");
    object::append_definitions(&mut output, traversal);
    machine::append_first_line_m73(&mut output);
    // The Marlin-family machine envelope prints before the start G-code
    // (`GCode.cpp:2819`), followed by the start G-code (`GCode.cpp:3137`).
    machine::append_limits(&mut output, traversal);
    let (bed_cache, start_position) =
        machine::append_start(&mut output, traversal, metadata, first_layer_bounds)?;
    let options = motion::MotionOptions::from_traversal(traversal);
    let small_area_flow = small_area::from_traversal(traversal)?;
    let model_offset = footprint::model_center(traversal).unwrap_or_default();
    let model_offset = (
        traversal
            .scale
            .unscale(traversal.scale.checked_scale(model_offset.0).unwrap()),
        traversal
            .scale
            .unscale(traversal.scale.checked_scale(model_offset.1).unwrap()),
    );
    let extruder_offset = offset::initial_extruder(traversal);
    let offset = (
        model_offset.0 - extruder_offset.0,
        model_offset.1 - extruder_offset.1,
    );
    let mut state = motion::EmitState {
        offset,
        origin: model_offset,
        extruder_offset,
        scale_factor: traversal.scale.factor(),
        travel_feedrate: options.first_layer_travel_feedrate,
        extrusion_feedrate: options.initial_layer_speed * 60.0,
        options,
        small_area_flow,
        tags: tags::Tags::of(traversal),
        spiral_vase: traversal.resolved.views.full.process.print.spiral_mode.0,
        ..Default::default()
    };
    let fan_layers_start = output.len();
    let (max_layer_z, mut fan_mover_handle) = layers::append(
        prepared,
        &mut output,
        &mut state,
        layers::Context {
            metadata,
            first_layer_bounds,
            start_position,
            bed_cache,
            extruder_offset,
            brim: &brim,
            skirt: &skirt,
        },
    )?;
    let traversal = &prepared
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let fan_layers_end = output.len();
    let emitted_layer_count =
        header::finalize_layer_count(&mut output, header::plate_layer_count(traversal));
    // The final compatible layer has no following layer marker to flush its
    // deferred retraction. Flush only retract/wipe (not a travel lift) before
    // end G-code (`GCode.cpp` final object teardown).
    motion::retract_for_print_end(&mut output, &mut state);
    motion::append_exclude_end(&mut output, &mut state);
    finish::append(
        &mut output,
        traversal,
        max_layer_z,
        metadata,
        first_layer_bounds,
    )?;
    machine::append_completion_controls(&mut output, traversal);
    output.extend_from_slice(b"M73 P100 R0\n; EXECUTABLE_BLOCK_END\n\n");
    let used_filament = finish::account_used_filament(&output);
    let (total_weight, total_cost) =
        finish::append_filament_stats(&mut output, traversal, used_filament);
    finish::append_compatible_stats(
        &mut output,
        traversal,
        total_weight,
        total_cost,
        emitted_layer_count,
    );
    output.push(b'\n');
    // Upstream applies FanMover ONLY to layer-chunk output (the tbb
    // pipeline after the cooling filter, GCode.cpp:3749), never to the
    // machine start g-code; a pending buffer is flushed per chunk.
    if let Some(mover) = fan_mover_handle.as_mut() {
        let mut layers = output.split_off(fan_layers_start);
        let tail = layers.split_off(fan_layers_end - fan_layers_start);
        let text = String::from_utf8(layers).expect("generated G-code is UTF-8");
        if std::env::var_os("ARES_DUMP_FANMOVER").is_some() {
            std::fs::write("/tmp/fanmover-input.gcode", &text).ok();
        }
        let flushed = mover.process_gcode(&text, true);
        output.extend(flushed.into_bytes());
        output.extend(tail);
    }
    Ok(processor::process(
        output,
        !traversal.resolved.views.full.printer.gcode.disable_m73.0,
        traversal
            .resolved
            .views
            .full
            .printer
            .gcode
            .machine_load_filament_time
            .0,
        used_filament,
        processor::ProcessorLimits::from_config(
            &traversal.resolved.views.full.printer.machine,
            traversal.resolved.views.full.printer.gcode.gcode_flavor,
            state.tags.is_bbl(),
        ),
    ))
}
/// The Z of the last G0/G1 move in the emitted prefix — what the
/// `GCodeWriter`'s Z position would be after emitting the same g-code
/// (feeds the `change_layer` `will_move_z` gate).
/// The avoid-crossing boundary covers every object's slices at the layer's
/// slice_z (`Layer::lslices`, `AvoidCrossingPerimeters.cpp:1100`): copies
/// of one source object are separate traversal objects here, but one
/// PrintObject upstream — their islands union into one boundary. The
/// cache holds one union per layer_index (all copies share the layout).
fn layer_boundary_slices<'a>(
    traversal: &'a PreparedPostClassicTraversal,
    object_index: usize,
    layer_index: usize,
    cache: &'a mut std::collections::HashMap<usize, std::rc::Rc<[ExPolygon]>>,
) -> &'a [ExPolygon] {
    let Some(record) = traversal.objects[object_index]
        .records
        .get(layer_index)
        .and_then(Option::as_ref)
    else {
        return traversal.objects[object_index]
            .slices(layer_index)
            .unwrap_or(&[]);
    };
    let _ = record;
    cache.entry(layer_index).or_insert_with(|| {
        let mut all: Vec<ExPolygon> = Vec::new();
        for slices in traversal.objects[object_index].occurrence_slices(layer_index) {
            all.extend(slices.iter().cloned());
        }
        std::rc::Rc::from(crate::geometry::union_expolygons(&all, &[]).unwrap_or_default())
    })
}

fn trailing_gcode_z(output: &[u8]) -> f64 {
    std::str::from_utf8(output)
        .ok()
        .and_then(|text| {
            text.lines().rev().find_map(|line| {
                let code = line.split(';').next()?.trim();
                let command = code.split_whitespace().next()?;
                if !matches!(command, "G0" | "G1") {
                    return None;
                }
                code.split_whitespace()
                    .find_map(|word| word.strip_prefix('Z'))
                    .and_then(|value| value.parse::<f64>().ok())
            })
        })
        .unwrap_or(0.0)
}

/// The final XY the start g-code left the nozzle at — mirrors
/// `GCodeWriter::m_pos.head<2>()` after the start g-code renders. X
/// and Y words apply independently (a move may update only one).
fn trailing_gcode_xy(output: &[u8]) -> Option<(f64, f64)> {
    let text = std::str::from_utf8(output).ok()?;
    let mut x = None;
    let mut y = None;
    for line in text.lines() {
        let code = line.split(';').next()?.trim();
        let command = code.split_whitespace().next()?;
        if !matches!(command, "G0" | "G1") {
            continue;
        }
        for word in code.split_whitespace() {
            if let Some(value) = word.strip_prefix('X') {
                x = value.parse::<f64>().ok();
            } else if let Some(value) = word.strip_prefix('Y') {
                y = value.parse::<f64>().ok();
            }
        }
    }
    match (x, y) {
        (Some(x), Some(y)) => Some((x, y)),
        _ => None,
    }
}

fn append_layer_end_timelapse(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    inserted: bool,
    traditional: bool,
    context: timelapse::Context<'_>,
) -> Result<(), SliceError> {
    if inserted {
        return Ok(());
    }
    if traditional {
        // `GCode.cpp:5538-5546` `add_object_change_labels`: pending
        // object-end labels flush before the layer-end template.
        motion::append_exclude_end(output, state);
    }
    // Only the traditional path emits timelapse at layer end
    // (`GCode.cpp:5264-5300`). BBL renders it inside the next layer's
    // CHANGE_LAYER block (`GCode.cpp:5205-5210`); non-BBL non-traditional
    // already emitted it in the layer-change template
    // (`GCode.cpp:4667-4676`).
    if !traditional {
        return Ok(());
    }
    timelapse::append_and_track(output, state, context)
}

pub(super) fn format_processor_float(value: f64) -> String {
    if value == 0.0 {
        return "0".to_owned();
    }
    let magnitude = value.abs().log10().floor() as i32;
    let precision = (5 - magnitude).max(0) as usize;
    let mut formatted = format!("{value:.precision$}");
    if formatted.contains('.') {
        while formatted.ends_with('0') {
            formatted.pop();
        }
        if formatted.ends_with('.') {
            formatted.pop();
        }
    }
    formatted
}
