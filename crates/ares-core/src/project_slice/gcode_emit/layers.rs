//! GCode.cpp layer-chunk emission, extracted without changing command ordering.
mod boundary;

use super::{
    GenerationMetadata, PreparedPostIslandPrintOrder, SliceError, append_layer_end_timelapse, brim,
    cooling, fan_mover, footprint, island_print_order, layer_boundary_slices, layer_gcode, motion,
    object, skirt, spiral_vase, timelapse, trailing_gcode_xy, value,
};
use crate::geometry::ExPolygon;

pub(super) struct Context<'a> {
    pub metadata: GenerationMetadata,
    pub first_layer_bounds: Option<footprint::FirstLayerBounds>,
    pub start_position: Option<value::Value>,
    pub bed_cache: i32,
    pub extruder_offset: (f64, f64),
    pub brim: &'a Option<brim::BrimPlan>,
    pub skirt: &'a Option<skirt::SkirtPlan>,
}

pub(super) fn append(
    prepared: &mut PreparedPostIslandPrintOrder,
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    context: Context<'_>,
) -> Result<(f64, Option<fan_mover::FanMover>), SliceError> {
    let Context {
        metadata,
        first_layer_bounds,
        start_position,
        bed_cache,
        extruder_offset,
        brim,
        skirt,
    } = context;
    let traversal = &prepared
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
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
    let fan_mover_handle = fan_mover_gate;
    for (object_index, object) in prepared.objects.iter_mut().enumerate() {
        // Each print object extrudes around its own build-item placement:
        // `bbs_3mf.cpp:3554-3560` applies per-instance transforms and
        // `GCode.cpp:5380/5403/5437` calls `set_origin(unscale(
        // instance.shift))` when a print object copy starts, so the
        // emission origin switches with the object instead of keeping the
        // first object's offset for the whole plate.
        let (source_object_index, _) = traversal.objects[object_index]
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object
            .identity();
        if let Some((center_x, center_y)) = footprint::object_center(traversal, source_object_index)
        {
            state.origin = (center_x, center_y);
            state.offset = (center_x - extruder_offset.0, center_y - extruder_offset.1);
        }
        let labels = object::ObjectLabels::from_traversal(traversal, object_index);
        let object_layer_count = object.len();
        let mut precise_layer_z = 0.0;
        let mut previous_layer_z = 0.0_f32;
        for (layer_index, layer) in object.iter_mut().enumerate() {
            if layer_index == 0 {
                layer_gcode::append_print_preamble(
                    output,
                    traversal,
                    metadata,
                    start_position.as_ref(),
                    first_layer_bounds,
                )?;
                // Upstream's writer does NOT know the Z after the start
                // g-code (`GCode.cpp:3139-3140` calls
                // `m_writer.set_current_position_clear(false)`), so
                // `m_pos.z()` stays 0 and `will_move_z` fires for every
                // layer-0 change (`GCode.cpp:5693`). Keep `writer_z` unset
                // to mirror that — scanning the start g-code for a trailing
                // Z would suppress the retract when the purge lines already
                // sit at the first-layer Z.
                // The brim split target (`loop.split_at(last_pos)`) uses
                // the nozzle XY the start g-code left — track it in the
                // live gcode coordinates.
                if let Some((x, y)) = trailing_gcode_xy(output) {
                    state.x = x;
                    state.y = y;
                }
            }
            let layer_output_start = output.len();
            cooling.begin_layer(output, layer_index);
            state.part_fan_speed = cooling.provisional_part_speed();
            let boundary = boundary::append(
                output,
                state,
                &mut spiral,
                boundary::Boundary {
                    traversal,
                    layer_change_template: &layer_change_template,
                    metadata,
                    first_layer_bounds,
                },
                layer_index,
                &mut precise_layer_z,
                &mut previous_layer_z,
                &mut second_layer_done,
                bed_cache,
            )?;
            let layer_z = boundary.layer_z;
            let layer_height = boundary.layer_height;
            let timelapse_context = boundary.timelapse_context;
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
                    output,
                    skirt::SkirtLayer {
                        index: layer_index,
                        height_mm: f64::from(layer_height),
                    },
                    geometry,
                    state,
                );
            }
            if layer_index == 0
                && object_index == 0
                && let Some(plan) = &brim
            {
                plan.emit(output, geometry, state);
            }
            let spiral_body_layer = spiral.is_body_layer(layer, layer_index, f64::from(layer_z));
            state.spiral_vase_layer = spiral_body_layer;
            if let Some(labels) = &labels {
                labels.queue_start(output, state, emit_labels);
            }
            let timelapse_inserted =
                motion::emit_layer(output, layer, geometry, state, |output, state| {
                    timelapse::append_traditional(
                        traditional_interlude,
                        output,
                        state,
                        timelapse_context,
                    )
                })?;
            if let Some(labels) = &labels {
                labels.queue_stop(output, state, emit_labels, timelapse_inserted);
            } else if timelapse_inserted {
                motion::defer_layer_retraction(state);
            } else {
                motion::end_layer_for_timelapse(output, state);
            }
            append_layer_end_timelapse(
                output,
                state,
                timelapse_inserted,
                traditional_timelapse,
                timelapse_context,
            )?;
            spiral.process_layer(
                output,
                spiral_vase::Layer {
                    start: layer_output_start,
                    enabled: spiral_body_layer,
                    final_layer: object_index + 1 == object_count
                        && layer_index + 1 == object_layer_count,
                    z: f64::from(layer_z),
                    height: f64::from(layer_height),
                },
            );
            cooling.finish_layer(output, layer_output_start);
        }
    }
    Ok((max_layer_z, fan_mover_handle))
}
