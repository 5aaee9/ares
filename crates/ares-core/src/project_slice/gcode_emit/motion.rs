mod acceleration;
mod arc;
pub(super) mod clip;
mod extrusion;
mod fan;
mod features;
mod format;
mod jerk;
mod loop_paths;
mod materialized;
mod options;
mod overhang;
mod path;
#[cfg(test)]
#[path = "motion/path/tests.rs"]
mod path_tests;
mod perimeter;
mod scarf;
mod state;
#[cfg(test)]
mod tests;
mod travel;

pub(super) use acceleration::{set_accel_and_jerk, set_layer_acceleration_and_jerk};
pub(in crate::project_slice) use arc::{simplify_linear_points, simplify_points};
pub(super) use state::{
    AvoidCrossingGeometry, EmitState, LayerGeometry, LiftMode, append_exclude_end,
    append_object_start, begin_layer, begin_path_travel, queue_exclude_end, queue_exclude_start,
    queue_object_start, queue_object_stop_label,
};
pub(super) use travel::{
    flush_pending_retract_eager, flush_pending_retract_lift, flush_pending_retract_wipe,
    retract_for_print_end,
};

use features::PathProperties;
pub(in crate::project_slice::gcode_emit) use options::{MotionOptions, first_nullable_float};

use super::super::island_print_order::{IslandPrintEntity, OrderedExtrusionLayer};
use crate::{
    SliceError,
    geometry::Point,
    project_slice::{
        fill_entities::FillExtrusionEntity,
        perimeters::classic::{
            gap_extrusion::GapFillEntity, shortest_path::chain_and_reorder_entities,
        },
    },
};

pub(super) fn prepare_traditional_timelapse(output: &mut Vec<u8>, state: &mut EmitState) {
    travel::retract_for_timelapse(output, state);
}

pub(super) fn defer_layer_retraction(state: &mut EmitState) {
    if state.options.retract_when_changing_layer && state.positioned {
        state.pending_layer_retract = true;
    }
}

/// Defer the layer-change hop: upstream's `change_layer` retract runs
/// `maybe_zlift` (`GCodeWriter.cpp:626-648`) and `m_to_lift` survives into
/// the new layer's first travel, which raises to layer+hop and descends.
pub(super) fn defer_layer_change_lift(state: &mut EmitState) {
    travel::defer_layer_change_lift(state);
}

pub(super) fn end_layer_for_timelapse(output: &mut Vec<u8>, state: &mut EmitState) {
    if state.options.retract_when_changing_layer && state.positioned {
        if state.traditional_timelapse {
            // Traditional timelapse prints (i3 structure or multi-nozzle)
            // retract immediately at the layer end (`GCode.cpp:5527-5546`
            // fires `retract()` before the labels and timelapse template).
            travel::retract_for_timelapse(output, state);
        } else {
            // Core-xy BBL (`GCode.cpp:5693` change_layer retract) and
            // compatible flavors emit the retraction inside the next layer's
            // CHANGE_LAYER block; `flush_pending_retract_wipe`/`_lift` emit
            // it there.
            state.pending_layer_retract = true;
        }
    }
}

/// Configured retraction at the start of the first layer — matches the
/// GCodeWriter `retract()` formatting (`GCodeWriter.cpp`).
pub(super) fn retract_before_layer(output: &mut Vec<u8>, state: &mut EmitState) {
    let length = state.options.retraction_length;
    if length <= 0.0 {
        return;
    }
    let retract = extrusion::coordinate(state, -length);
    output.extend_from_slice(
        format!(
            "G1 E{} F{}\n",
            format::extrusion(retract),
            format::axis(state.options.retraction_feedrate)
        )
        .as_bytes(),
    );
    if !state.options.use_relative_e_distances {
        output.extend_from_slice(b"G92 E0\n");
        state.e_position = 0.0;
    }
    state.current_feedrate = state.options.retraction_feedrate;
    state.retracted = true;
    // The layer-start retract defers the z-hop like upstream `retract()`
    // (needs_lift && can_lift → `lazy_lift` above/below gate at the
    // current writer z, `GCodeWriter.cpp:626-648`).
    travel::defer_layer_change_lift(state);
}

#[derive(Clone, Copy)]
pub(super) struct SkirtLoopFlow {
    pub(super) width: f32,
    pub(super) height: f32,
    pub(super) mm3_per_mm: f64,
}

pub(super) fn emit_brim_loop(
    output: &mut Vec<u8>,
    points: impl Iterator<Item = (i64, i64)>,
    flow: SkirtLoopFlow,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) {
    path::emit(
        output,
        points,
        PathProperties {
            mm3_per_mm: flow.mm3_per_mm,
            width: flow.width,
            height: flow.height,
            feature: "Brim",
            is_perimeter: false,
            end_clip: state.options.seam_gap,
            fitting: &[],
            slope: None,
        },
        geometry,
        state,
    );
}

pub(super) fn emit_skirt_loop(
    output: &mut Vec<u8>,
    points: impl Iterator<Item = (i64, i64)>,
    flow: SkirtLoopFlow,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) {
    path::emit(
        output,
        points,
        PathProperties {
            mm3_per_mm: flow.mm3_per_mm,
            width: flow.width,
            height: flow.height,
            feature: "Skirt",
            is_perimeter: false,
            // `GCode::extrude_loop` clips every loop by the seam gap
            // (`GCode.cpp:5778-5790`); `path::emit` clips in the plate
            // frame to match upstream's origin-shifted coordinates.
            end_clip: state.options.seam_gap,
            fitting: &[],
            slope: None,
        },
        geometry,
        state,
    );
    // `GCode.cpp:5979-5991` stores loop wipe paths forward so the wipe
    // wraps from the loop end back toward the path start.
    state.wipe_path.reverse();
}

pub(super) fn emit_layer<F>(
    output: &mut Vec<u8>,
    layer: &mut OrderedExtrusionLayer,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
    mut before_first_infill: F,
) -> Result<bool, SliceError>
where
    F: FnMut(&mut Vec<u8>, &mut EmitState) -> Result<bool, SliceError>,
{
    let mut interlude_emitted = false;
    for island in &mut layer.islands {
        let mut entities = std::mem::take(&mut island.entities);
        let infill_first = matches!(
            entities.first(),
            Some(
                IslandPrintEntity::Fill(_)
                    | IslandPrintEntity::FillCollection(_)
                    | IslandPrintEntity::Thin(_)
            )
        );
        if infill_first {
            let split = entities
                .iter()
                .position(|entity| matches!(entity, IslandPrintEntity::Perimeter(_)))
                .unwrap_or(entities.len());
            let perimeters = entities.split_off(split);
            emit_infills(output, &mut entities, geometry, state);
            for perimeter in perimeters {
                perimeter::emit_perimeter(output, perimeter, geometry, state);
            }
        } else {
            let split = entities
                .iter()
                .position(|entity| !matches!(entity, IslandPrintEntity::Perimeter(_)))
                .unwrap_or(entities.len());
            for perimeter in entities.drain(..split) {
                perimeter::emit_perimeter(output, perimeter, geometry, state);
            }
            if !interlude_emitted && !entities.is_empty() {
                interlude_emitted = before_first_infill(output, state)?;
            }
            emit_infills(output, &mut entities, geometry, state);
        }
    }
    Ok(interlude_emitted)
}

fn emit_infills(
    output: &mut Vec<u8>,
    entities: &mut Vec<IslandPrintEntity>,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) {
    if entities.is_empty() {
        return;
    }
    let mut ironing = Vec::new();
    let mut index = 0;
    while index < entities.len() {
        if is_ironing_entity(&entities[index]) {
            ironing.push(entities.remove(index));
        } else {
            index += 1;
        }
    }
    chain_and_reorder_entities(entities, local_cursor(state, geometry));
    entities.append(&mut ironing);
    for entity in entities.drain(..) {
        match entity {
            IslandPrintEntity::Fill(entity) => {
                emit_fill_entity(output, &entity, geometry, state);
            }
            IslandPrintEntity::FillCollection(collection) => {
                let collection = collection.chained_path_from(local_cursor(state, geometry));
                for entity in &collection.entities {
                    emit_fill_entity(output, entity, geometry, state);
                }
            }
            IslandPrintEntity::Thin(entity) => {
                emit_variable_width_entity(output, &entity, geometry, state);
            }
            IslandPrintEntity::Perimeter(_) => {
                unreachable!("infill phase contains only infill entities")
            }
        }
    }
}

fn is_ironing_entity(entity: &IslandPrintEntity) -> bool {
    let path_is_ironing = |entity: &FillExtrusionEntity| matches!(entity, FillExtrusionEntity::Path(path) if path.role == crate::ExtrusionRole::Ironing);
    match entity {
        IslandPrintEntity::Fill(entity) => path_is_ironing(entity),
        IslandPrintEntity::FillCollection(collection) => {
            collection.entities.first().is_some_and(path_is_ironing)
        }
        IslandPrintEntity::Perimeter(_) | IslandPrintEntity::Thin(_) => false,
    }
}

fn emit_fill_entity(
    output: &mut Vec<u8>,
    entity: &FillExtrusionEntity,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) {
    match entity {
        FillExtrusionEntity::Path(path) => path::emit(
            output,
            path.polyline
                .points()
                .iter()
                .map(|point| (point.x(), point.y())),
            PathProperties {
                mm3_per_mm: path.mm3_per_mm,
                width: path.width,
                height: path.height,
                feature: features::for_fill(path.role),
                is_perimeter: matches!(
                    path.role,
                    crate::ExtrusionRole::Perimeter
                        | crate::ExtrusionRole::ExternalPerimeter
                        | crate::ExtrusionRole::OverhangPerimeter
                ),
                end_clip: 0.0,
                fitting: &path.fitting,
                slope: None,
            },
            geometry,
            state,
        ),
        FillExtrusionEntity::VariableWidth(entity) => {
            emit_variable_width_entity(output, entity, geometry, state);
        }
    }
}

fn emit_variable_width_entity(
    output: &mut Vec<u8>,
    entity: &GapFillEntity,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) {
    match entity {
        GapFillEntity::Path(path) => materialized::emit_flat(output, path, 0.0, geometry, state),
        GapFillEntity::Loop(paths) => {
            // `variable_width` (VariableWidth.cpp:224-227) closes up loop
            // entities; `GCode::extrude_loop` then splits them at the point
            // nearest to the last position (`GCode.cpp:5771`, the
            // non-perimeter `loop.split_at(last_pos, false)` branch).
            let mut loop_ = crate::project_slice::perimeters::classic::chained_loops::ExtrusionLoop {
                paths: paths.clone(),
                role: crate::project_slice::perimeters::classic::chained_loops::ExtrusionLoopRole::Internal,
            };
            crate::project_slice::seam_placement::place_nearest_projection(
                &mut loop_,
                crate::project_slice::perimeters::classic::materialize::Point3 {
                    x: local_cursor(state, geometry).x(),
                    y: local_cursor(state, geometry).y(),
                    z: 0,
                },
                geometry.scale,
            );
            loop_paths::emit(
                output,
                &loop_.paths,
                crate::project_slice::perimeters::classic::chained_loops::ExtrusionLoopRole::Internal,
                geometry,
                state,
            )
        }
    }
}

fn local_cursor(state: &EmitState, geometry: LayerGeometry<'_>) -> Point {
    // `GCode::extrude_infill` chains from m_last_pos, not GCodeWriter::m_pos.
    if let Some((x, y)) = state.last_scaled_position {
        return Point::new(x, y);
    }
    // Before the first generated path, use the position left by start G-code.
    Point::new(
        geometry
            .scale
            .checked_scale(state.x - state.offset.0)
            .expect("emitted X remains in the coordinate domain"),
        geometry
            .scale
            .checked_scale(state.y - state.offset.1)
            .expect("emitted Y remains in the coordinate domain"),
    )
}
