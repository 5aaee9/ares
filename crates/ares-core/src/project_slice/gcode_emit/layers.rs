//! GCode.cpp layer-chunk emission, extracted without changing command ordering.
mod boundary;
mod entry;
mod object_order;
mod schedule;

use entry::EntryGeometry;
use schedule::Schedule;

use super::{
    GenerationMetadata, PreparedPostIslandPrintOrder, SliceError, append_layer_end_timelapse, brim,
    cooling, fan_mover, footprint, island_print_order, layer_boundary_slices_rc, layer_gcode,
    motion, object, skirt, spiral_vase, timelapse, trailing_gcode_xy, value,
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
    // Disjoint field borrows: `objects` stays mutable while the geometry
    // buffers (top surfaces, seam plans, the traversal chain) are shared.
    let objects = &mut prepared.objects;
    let top_surfaces = &prepared.top_surfaces;
    let nearest_seam_plans = &prepared.nearest_seam_plans;
    let island_predecessor = &prepared.predecessor;
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
    let object_count = objects.len();
    // Avoid-crossing boundaries are built per slice_z across every object
    // (`Layer::lslices` covers all instances of the print object;
    // `AvoidCrossingPerimeters.cpp:1100`). Copies of one source share the
    // layer layout, so records are matched by slice_z.
    let mut layer_boundary_cache: std::collections::HashMap<
        (usize, usize),
        std::rc::Rc<[ExPolygon]>,
    > = std::collections::HashMap::new();
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

    // `collect_layers_to_print(Print)` (`GCode.cpp:1835-1870`): every print
    // The merged layer-chunk schedule (`layers/schedule.rs`): every print
    // object's layers merged by print z, each chunk's objects in the
    // chained instance order (`collect_layers_to_print` /
    // `chain_print_object_instances`, applied per chunk through
    // `sort_print_object_instances`, `GCode.cpp:4118-4167`). Each print
    // object extrudes around its own build-item placement
    // (`bbs_3mf.cpp:3554-3560`, `GCode.cpp:5380/5403/5437`
    // `set_origin(unscale(instance.shift))`).
    let schedule = schedule::build(traversal, objects);
    let Schedule {
        per_object_z,
        merged,
        print_position,
        labels,
        object_layer_counts,
        last_entry,
        ..
    } = schedule;

    let mut entry_geometry = |object_index: usize, layer_index: usize| -> EntryGeometry<'_> {
        let lower_boundary_lines = traversal.objects[object_index]
            .lower_slices(layer_index)
            .into_iter()
            .flatten()
            .flat_map(ExPolygon::lines)
            .collect::<Vec<_>>();
        EntryGeometry {
            top_surfaces: top_surfaces[object_index]
                .get(layer_index)
                .map(|expolygons| expolygons.iter().collect::<Vec<_>>())
                .unwrap_or_default(),
            lower_boundary_lines,
            nearest_penalties: nearest_seam_plans
                .get(object_index)
                .and_then(|plans| plans.get(layer_index)),
            layer_slices: layer_boundary_slices_rc(
                traversal,
                object_index,
                layer_index,
                &mut layer_boundary_cache,
            ),
            internal_surfaces: island_print_order::internal_surfaces(
                island_predecessor,
                object_index,
                layer_index,
            ),
        }
    };

    let mut group_start = 0;
    let mut first_group = true;
    while group_start < merged.len() {
        let group_z = merged[group_start].0;
        let mut group_end = group_start;
        while group_end < merged.len() && merged[group_end].0 == group_z {
            group_end += 1;
        }
        let mut entries: Vec<(usize, usize)> = merged[group_start..group_end]
            .iter()
            .map(|&(_, object_index, layer_index)| (object_index, layer_index))
            .collect();
        entries.sort_by_key(|(object_index, _)| print_position[*object_index]);
        group_start = group_end;
        let (first_object, first_layer) = entries[0];
        if first_group && first_layer == 0 {
            layer_gcode::append_print_preamble(
                output,
                traversal,
                metadata,
                start_position.as_ref(),
                first_layer_bounds,
            )?;
            // Upstream's writer does NOT know the Z after the start g-code
            // (`GCode.cpp:3139-3140`), so `writer_z` stays unset
            // (`GCode.cpp:5693`). The brim split target uses the nozzle XY
            // the start g-code left — track it in live gcode coordinates.
            if let Some((x, y)) = trailing_gcode_xy(output) {
                state.x = x;
                state.y = y;
            }
            first_group = false;
        }
        // The cooling rewrite window opens after the preamble (the start
        // g-code is not part of the layer's buffered extrusions).
        let layer_output_start = output.len();
        cooling.begin_layer(output, first_layer);
        state.part_fan_speed = cooling.provisional_part_speed();
        // The change-layer block uses the merged chunk's leading entry
        // (`change_layer` runs once per layer chunk, `GCode.cpp:5685`).
        let previous_layer_z = if first_layer > 0 {
            per_object_z[first_object]
                .get(first_layer - 1)
                .copied()
                .unwrap_or(0.0)
        } else {
            0.0
        };
        let layer_z = group_z;
        let layer_height = layer_z - previous_layer_z;
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
            first_layer,
            previous_layer_z,
            layer_z,
            layer_height,
            &mut second_layer_done,
            bed_cache,
        )?;
        let timelapse_context = boundary.timelapse_context;
        // The skirt prints once per layer before any object content
        // (`GCode.cpp:4388+`), on the layers it covers.
        if let (Some(&(_, skirt_layer)), Some(plan)) = (
            entries.iter().find(|&&(object_index, _)| object_index == 0),
            &skirt,
        ) {
            let geometry = entry_geometry(0, skirt_layer);
            let lower_boundary = (!geometry.lower_boundary_lines.is_empty())
                .then(|| crate::geometry::LineDistanceTree::new(&geometry.lower_boundary_lines));
            let motion_geometry = geometry.view(traversal, 0, skirt_layer, lower_boundary.as_ref());
            plan.emit(
                output,
                skirt::SkirtLayer {
                    index: skirt_layer,
                    height_mm: f64::from(layer_height),
                },
                motion_geometry,
                state,
            );
        }
        for (entry_position, &(object_index, layer_index)) in entries.iter().enumerate() {
            let is_group_end = entry_position + 1 == entries.len();
            let (source_object_index, _) = traversal.objects[object_index]
                .predecessor
                .predecessor
                .predecessor
                .predecessor
                .object
                .identity();
            if let Some((center_x, center_y)) =
                footprint::object_center(traversal, source_object_index)
            {
                state.origin = (center_x, center_y);
                state.offset = (center_x - extruder_offset.0, center_y - extruder_offset.1);
            }
            // Upstream re-initializes the avoid-crossing boundaries for
            // every instance's layer (`init_layer(*m_layer)` per
            // `instance_to_print`, `GCode.cpp:5343-5345`); drop the cached
            // boundary so each entry rebuilds it from its own slices.
            state.avoid_boundary = None;
            if object_index == 0
                && layer_index == 0
                && let Some(plan) = &brim
            {
                let geometry = entry_geometry(object_index, layer_index);
                let lower_boundary = (!geometry.lower_boundary_lines.is_empty()).then(|| {
                    crate::geometry::LineDistanceTree::new(&geometry.lower_boundary_lines)
                });
                plan.emit(
                    output,
                    geometry.view(
                        traversal,
                        object_index,
                        layer_index,
                        lower_boundary.as_ref(),
                    ),
                    state,
                );
            }
            let object = &mut objects[object_index];
            let Some(layer) = object.get_mut(layer_index) else {
                continue;
            };
            let spiral_body_layer = spiral.is_body_layer(layer, layer_index, f64::from(layer_z));
            state.spiral_vase_layer = spiral_body_layer;
            let geometry = entry_geometry(object_index, layer_index);
            let lower_boundary = (!geometry.lower_boundary_lines.is_empty())
                .then(|| crate::geometry::LineDistanceTree::new(&geometry.lower_boundary_lines));
            let motion_geometry = geometry.view(
                traversal,
                object_index,
                layer_index,
                lower_boundary.as_ref(),
            );
            let entry_output_start = output.len();
            if let Some(labels) = &labels[object_index] {
                labels.queue_start(output, state, emit_labels);
            }
            let timelapse_inserted =
                motion::emit_layer(output, layer, motion_geometry, state, |output, state| {
                    timelapse::append_traditional(
                        traditional_interlude,
                        output,
                        state,
                        timelapse_context,
                    )
                })?;
            let is_last_entry = Some((object_index, layer_index)) == last_entry;
            if let Some(labels) = &labels[object_index] {
                labels.queue_stop(
                    output,
                    state,
                    emit_labels,
                    timelapse_inserted && is_group_end,
                );
            }
            if is_group_end {
                // The layer-end timelapse sequence closes every layer chunk
                // (`GCode.cpp:5527-5546`), after all of the chunk's objects.
                if labels[object_index].is_none() && timelapse_inserted {
                    motion::defer_layer_retraction(state);
                } else if labels[object_index].is_none() {
                    motion::end_layer_for_timelapse(output, state);
                }
                append_layer_end_timelapse(
                    output,
                    state,
                    timelapse_inserted,
                    traditional_timelapse,
                    timelapse_context,
                )?;
            }
            spiral.process_layer(
                output,
                spiral_vase::Layer {
                    start: entry_output_start,
                    enabled: spiral_body_layer,
                    final_layer: is_last_entry
                        && layer_index + 1 == object_layer_counts[object_index],
                    z: f64::from(layer_z),
                    height: f64::from(layer_height),
                },
            );
        }
        cooling.finish_layer(output, layer_output_start);
    }
    Ok((max_layer_z, fan_mover_handle))
}
