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
    header::append_header(&mut output, metadata, &prepared.objects, traversal);
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
        scale_factor: traversal.scale.factor(),
        travel_feedrate: options.first_layer_travel_feedrate,
        extrusion_feedrate: options.initial_layer_speed * 60.0,
        options,
        small_area_flow,
        tags: tags::Tags::of(traversal),
        spiral_vase: traversal.resolved.views.full.process.print.spiral_mode.0,
        ..Default::default()
    };
    let emit_labels = traversal
        .resolved
        .views
        .full
        .process
        .print
        .gcode_label_objects
        .0;
    let mut cooling = cooling::CoolingState::from_traversal(traversal);
    let mut spiral = spiral_vase::SpiralVaseFilter::from_traversal(traversal, brim.is_some());
    let max_layer_z = traversal
        .objects
        .first()
        .into_iter()
        .flat_map(|object| object.records.iter())
        .filter_map(|record| record.as_ref())
        .map(|record| record.layer_height)
        .sum();
    let layer_change_template =
        layer_gcode::LayerChangeTemplate::new(traversal, metadata, first_layer_bounds);
    let runtime_gcode = &traversal.resolved.views.runtime_gcode;
    let traditional_timelapse = !runtime_gcode.time_lapse_gcode.0.is_empty()
        && ((runtime_gcode.printer_structure == crate::PrinterStructure::I3
            && !traversal.resolved.views.full.process.print.spiral_mode.0)
            || traversal
                .resolved
                .views
                .full
                .project
                .print
                .nozzle_diameter
                .0
                .len()
                > 1);
    // The mid-layer insert only fires on I3 printers (`GCode.cpp:5455-5461`);
    // corexy/multi-nozzle traditional prints fall through to the layer-end
    // sequence (`GCode.cpp:5527-5546`).
    let traditional_interlude =
        traditional_timelapse && runtime_gcode.printer_structure == crate::PrinterStructure::I3;
    state.traditional_timelapse = traditional_timelapse;
    let mut second_layer_done = false;
    let object_count = prepared.objects.len();
    // Avoid-crossing boundaries are built per slice_z across every object
    // (`Layer::lslices` covers all instances of the print object;
    // `AvoidCrossingPerimeters.cpp:1100`). Copies of one source share the
    // layer layout, so records are matched by slice_z.
    let mut layer_boundary_cache: std::collections::HashMap<usize, std::rc::Rc<[ExPolygon]>> =
        std::collections::HashMap::new();
    // FanMover construction mirrors GCode.cpp:3727-3740 (gate:
    // fan_speedup_time != 0 || fan_kickstart > 0).
    let fan_mover_gate = (|| {
        let gcode = &traversal.resolved.views.full.printer.gcode;
        let speedup_time = gcode.fan_speedup_time.0;
        let kickstart = gcode.fan_kickstart.0;
        (speedup_time != 0.0 || kickstart > 0.0).then(|| {
            let relative_e = gcode.use_relative_e_distances.0;
            fan_mover::FanMover::new(
                speedup_time,
                kickstart,
                gcode.fan_speedup_overhangs.0,
                relative_e,
                gcode.gcode_flavor,
            )
        })
    })();
    let mut fan_mover_handle = fan_mover_gate;
    let fan_layers_start = output.len();
    for (object_index, object) in prepared.objects.iter_mut().enumerate() {
        let labels = object::ObjectLabels::from_traversal(traversal, object_index);
        let object_layer_count = object.len();
        let mut precise_layer_z = 0.0;
        let mut previous_layer_z = 0.0_f32;
        for (layer_index, layer) in object.iter_mut().enumerate() {
            if layer_index == 0 {
                layer_gcode::append_print_preamble(
                    &mut output,
                    traversal,
                    metadata,
                    start_position.as_ref(),
                    first_layer_bounds,
                )?;
                // Track the Z the start g-code left the nozzle at, mirroring
                // `GCodeWriter::m_pos(2)` for the `change_layer`
                // `will_move_z` gate (`GCode.cpp:5693`).
                state.writer_z = Some(trailing_gcode_z(&output));
                // The brim split target (`loop.split_at(last_pos)`) uses
                // the nozzle XY the start g-code left — track it in the
                // live gcode coordinates.
                if let Some((x, y)) = trailing_gcode_xy(&output) {
                    state.x = x;
                    state.y = y;
                }
            }
            let layer_output_start = output.len();
            cooling.begin_layer(&mut output, layer_index);
            state.part_fan_speed = cooling.provisional_part_speed();
            state.physical_fan_speed = state.part_fan_speed;
            let tags = state.tags;
            output.extend_from_slice(tags.layer_change().as_bytes());
            output.push(b'\n');
            let record_layer_height = traversal
                .objects
                .first()
                .and_then(|object| object.records.get(layer_index))
                .and_then(|record| record.as_ref())
                .map_or(0.0, |record| record.layer_height);
            precise_layer_z += record_layer_height;
            let layer_z = precise_layer_z as f32;
            let layer_height = layer_z - previous_layer_z;
            // Upstream's writer z at the layer-change retract is still the
            // previous layer's z (`change_layer` does not move z); the
            // start-gcode z remains authoritative for the first layer.
            if layer_index > 0 {
                state.writer_z = Some(f64::from(previous_layer_z));
            }
            previous_layer_z = layer_z;
            let header = format!(
                "{}\n{}\n",
                tags.z(&format_processor_float(f64::from(layer_z))),
                tags.height(&format_processor_float(f64::from(layer_height))),
            );
            output.extend_from_slice(header.as_bytes());
            let timelapse_context = timelapse::Context {
                traversal,
                layer: timelapse::TimelapseLayer {
                    index: layer_index,
                    z: f64::from(layer_z),
                    max_z: f64::from(layer_z),
                },
                metadata,
                first_layer_bounds,
            };
            let timelapse_at_layer_change =
                !tags.is_bbl() && !runtime_gcode.time_lapse_gcode.0.is_empty();
            layer_gcode::append_before_layer_change_gcode(
                &mut output,
                layer_gcode::LayerTemplateContext {
                    traversal,
                    layer_index,
                    layer_z: f64::from(layer_z),
                    totals: state.extrusion_totals(),
                    context: &layer_change_template,
                },
            )?;
            // Upstream's `change_layer` retract (`retract_when_changing_layer`)
            // also DEFERS the hop (`maybe_zlift`, `GCodeWriter.cpp:626-648`):
            // `m_to_lift` survives into the new layer's first travel, which
            // raises to layer+hop and descends at the target. Capture the
            // flag BEFORE the wipe flush consumes it; defer only here (the
            // c7ea935f lesson: no already-retracted mid-print deferrals).
            let layer_retract_pending = state.pending_layer_retract
                && state.options.z_hop > 0.0
                && state.options.retraction_length > 0.0
                && !state.lifted
                && state.pending_lift.is_none();
            motion::flush_pending_retract_wipe(&mut output, &mut state);
            if layer_retract_pending {
                motion::defer_layer_change_lift(&mut state);
            }
            // Pending object-end labels flush after the layer-change
            // retract/wipe, before the layer-change gcode
            // (`GCode.cpp:5699` `change_layer`).
            motion::append_exclude_end(&mut output, &mut state);
            // The layer-start retraction mirrors the `change_layer`/BBS
            // layer-start retract, which is gated on
            // `retract_when_changing_layer` (`GCode.cpp:5206`, `GCode.cpp:5693`);
            // the compatible-flavor `change_layer` variant additionally
            // requires `will_move_z` — the nozzle must actually change Z
            // from wherever the start g-code left it (`GCode.cpp:5693`).
            // BBL layer starts retract whenever the flag is set.
            let will_move_z = state
                .writer_z
                .is_none_or(|writer_z| (f64::from(layer_z) - writer_z).abs() > 1.0e-4);
            if layer_index == 0
                && state.options.retract_when_changing_layer
                && (state.tags.is_bbl() || will_move_z)
            {
                motion::retract_before_layer(&mut output, &mut state);
            }
            if timelapse_at_layer_change {
                timelapse::append_and_track(&mut output, &mut state, timelapse_context)?;
            }
            spiral.append_layer_z(&mut output, layer_index, f64::from(layer_z));
            layer_gcode::append_layer_change(
                &mut output,
                layer_gcode::LayerTemplateContext {
                    traversal,
                    layer_index,
                    layer_z: f64::from(layer_z),
                    totals: state.extrusion_totals(),
                    context: &layer_change_template,
                },
            )?;
            // A deferred previous-layer retraction lifts above the new layer's
            // print Z (`GCodeWriter::travel_to_z` during layer transition).
            // The lift gate evaluates at the writer's z before the layer move
            // (`GCode.cpp:5690` change_layer retract precedes travel_to_z;
            // `GCodeWriter.cpp:633-639` gates on `m_pos.z()`).
            let previous_layer_z = state.layer_z;
            state.layer_z = f64::from(layer_z);
            state.source_layer_z = precise_layer_z;
            state.layer_index = layer_index;
            motion::flush_pending_retract_lift(&mut output, &mut state, previous_layer_z);
            motion::begin_layer(
                &mut output,
                &mut state,
                layer_index,
                f64::from(layer_z),
                f64::from(layer_height),
            );
            // Second-layer transition: bed temperature for the remaining
            // layers (`GCode.cpp:4777-4830`), once per slice.
            if layer_index == 1 && !second_layer_done {
                second_layer_done = true;
                machine::append_second_layer_transition(&mut output, traversal, bed_cache);
            }
            // Orca `GCode.cpp:5205-5210`: core-xy BBL printers emit the eager
            // change-layer lift (the second change-layer retract is a no-op
            // for E once retracted) and the timelapse gcode inside the new
            // layer's window, after the layer-change acceleration.
            let bbl_layer_start = state.tags.is_bbl() && !state.traditional_timelapse;
            if bbl_layer_start && state.options.retract_when_changing_layer {
                motion::flush_pending_retract_eager(&mut output, &mut state);
            }
            if bbl_layer_start {
                timelapse::append_and_track(&mut output, &mut state, timelapse_context)?;
            }
            let lower_boundary_lines = traversal.objects[object_index]
                .lower_slices(layer_index)
                .into_iter()
                .flatten()
                .flat_map(crate::geometry::ExPolygon::lines)
                .collect::<Vec<_>>();
            let lower_boundary = (!lower_boundary_lines.is_empty())
                .then(|| crate::geometry::LineDistanceTree::new(&lower_boundary_lines));
            let top_surfaces = prepared.top_surfaces[object_index]
                .get(layer_index)
                .map(|expolygons| expolygons.iter().collect::<Vec<_>>())
                .unwrap_or_default();
            let nearest_penalties = prepared
                .nearest_seam_plans
                .get(object_index)
                .and_then(|plans| plans.get(layer_index));
            let staggered_inner = nearest_penalties.is_some()
                && traversal.resolved.objects[object_index]
                    .object
                    .staggered_inner_seams
                    .0;
            let geometry = motion::LayerGeometry {
                nearest_seam_penalties: nearest_penalties,
                staggered_inner,
                internal_surfaces: island_print_order::internal_surfaces(
                    &prepared.predecessor,
                    object_index,
                    layer_index,
                ),
                scale: traversal.scale,
                previous_layer_boundary: lower_boundary.as_ref(),
                avoid_crossing: motion::AvoidCrossingGeometry {
                    layer_slices: layer_boundary_slices(
                        traversal,
                        object_index,
                        layer_index,
                        &mut layer_boundary_cache,
                    ),
                    perimeter_spacing: traversal.objects[object_index]
                        .perimeter_spacing(layer_index)
                        .unwrap_or_default(),
                    external_perimeter_width: traversal.objects[object_index]
                        .external_perimeter_width(layer_index)
                        .unwrap_or_default(),
                    top_surfaces: &top_surfaces,
                },
            };
            // The skirt prints once per layer before any object content
            // (`GCode.cpp:4388+`), on the layers it covers.
            if object_index == 0
                && let Some(plan) = &skirt
            {
                plan.emit(
                    &mut output,
                    skirt::SkirtLayer {
                        index: layer_index,
                        height_mm: f64::from(layer_height),
                    },
                    geometry,
                    &mut state,
                );
            }
            if layer_index == 0
                && object_index == 0
                && let Some(plan) = &brim
            {
                plan.emit(&mut output, geometry, &mut state);
            }
            let spiral_body_layer = spiral.is_body_layer(layer, layer_index, f64::from(layer_z));
            state.spiral_vase_layer = spiral_body_layer;
            if let Some(labels) = &labels {
                labels.queue_start(&mut output, &mut state, emit_labels);
            }
            let timelapse_inserted =
                motion::emit_layer(&mut output, layer, geometry, &mut state, |output, state| {
                    timelapse::append_traditional(
                        traditional_interlude,
                        output,
                        state,
                        timelapse_context,
                    )
                })?;
            if let Some(labels) = &labels {
                labels.queue_stop(&mut output, &mut state, emit_labels, timelapse_inserted);
            } else if timelapse_inserted {
                motion::defer_layer_retraction(&mut state);
            } else {
                motion::end_layer_for_timelapse(&mut output, &mut state);
            }
            append_layer_end_timelapse(
                &mut output,
                &mut state,
                timelapse_inserted,
                traditional_timelapse,
                timelapse_context,
            )?;
            spiral.process_layer(
                &mut output,
                spiral_vase::Layer {
                    start: layer_output_start,
                    enabled: spiral_body_layer,
                    final_layer: object_index + 1 == object_count
                        && layer_index + 1 == object_layer_count,
                    z: f64::from(layer_z),
                    height: f64::from(layer_height),
                },
            );
            cooling.finish_layer(&mut output, layer_output_start);
        }
    }
    let fan_layers_end = output.len();
    let emitted_layer_count = header::finalize_layer_count(&mut output, tags);
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
        processor::ProcessorLimits {
            print_acceleration: machine::first(
                &traversal
                    .resolved
                    .views
                    .full
                    .printer
                    .machine
                    .machine_max_acceleration_extruding,
            ),
            retract_acceleration: machine::first(
                &traversal
                    .resolved
                    .views
                    .full
                    .printer
                    .machine
                    .machine_max_acceleration_retracting,
            ),
            travel_acceleration: machine::first(
                &traversal
                    .resolved
                    .views
                    .full
                    .printer
                    .machine
                    .machine_max_acceleration_travel,
            ),
            gcode_flavor: traversal.resolved.views.full.printer.gcode.gcode_flavor,
            bbl_printer: state.tags.is_bbl(),
            junction_deviation: machine::first(
                &traversal
                    .resolved
                    .views
                    .full
                    .printer
                    .machine
                    .machine_max_junction_deviation,
            ),
        },
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
