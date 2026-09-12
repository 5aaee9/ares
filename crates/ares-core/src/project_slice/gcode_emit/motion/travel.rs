// Source boundary: OrcaSlicer v2.4.2 `GCode.cpp:310-358, 7400-7448` and
// `GCodeWriter.cpp` retract, wipe, and spiral-lift motion.

mod lift;
#[cfg(test)]
mod tests;

use super::{
    EmitState, arc, extrusion,
    format::{axis as format_axis, extrusion as format_extrusion},
};
pub(super) use lift::{
    emit_pending as emit_pending_lift, is_allowed_at as lift_is_allowed_at,
    mode_for as lift_mode_for, z_feedrate as lift_z_feedrate,
};

pub(super) fn retract_and_lift(output: &mut Vec<u8>, state: &mut EmitState) {
    retract_and_wipe(output, state);
    if state.spiral_vase {
        lift::append_spiral_vase(output, state);
    } else {
        lift::schedule(state, false);
    }
    state.retracted = true;
}

pub(in crate::project_slice::gcode_emit) fn retract_for_print_end(
    output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    if !state.retracted {
        retract_and_wipe_with(output, state, false);
        state.retracted = true;
    }
    if state.spiral_vase {
        lift::append_spiral_vase(output, state);
    }
    state.pending_layer_retract = false;
}

/// `GCode.cpp:470-475`: the last wipe of the print emits no `;_WIPE` cooling
/// marker — no later cooling pass runs to clean it (print-end retraction
/// happens after the final layer flushes).
pub(super) fn retract_for_timelapse(output: &mut Vec<u8>, state: &mut EmitState) {
    if state.retracted {
        return;
    }
    retract_and_wipe(output, state);
    lift::append_eager(output, state);
    state.retracted = true;
}

/// Non-BBL printers defer the layer-change retraction to after the next
/// layer marker block (`GCode.cpp` change_layer: the wipe lands between
/// the rendered `before_layer_change_gcode` and the cooling fan marker,
/// the lift after it).
pub(in crate::project_slice::gcode_emit) fn flush_pending_retract_wipe(
    output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    if state.pending_layer_retract && !state.retracted {
        retract_and_wipe(output, state);
    }
}

pub(in crate::project_slice::gcode_emit) fn flush_pending_retract_lift(
    _output: &mut Vec<u8>,
    state: &mut EmitState,
    writer_z: f64,
) {
    if state.pending_layer_retract && !state.retracted {
        lift::schedule_at(state, true, writer_z);
        state.retracted = true;
    }
    state.pending_layer_retract = false;
}

/// Core-xy BBL change-layer retraction: the wipe lands inside the new
/// layer's CHANGE_LAYER block and the lift is emitted eagerly right after
/// (`GCode.cpp:5693` change_layer retract, `GCode.cpp:5205-5210` second
/// retract whose E half is a no-op once retracted).
pub(in crate::project_slice::gcode_emit) fn flush_pending_retract_eager(
    output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    if state.pending_layer_retract && !state.retracted {
        retract_and_wipe(output, state);
        state.retracted = true;
    }
    state.pending_layer_retract = false;
    // The eager lift supersedes any lift that the deferred path scheduled
    // for the layer's first travel.
    state.pending_lift = None;
    if state.retracted {
        lift::append_eager(output, state);
    }
}

fn retract_and_wipe(output: &mut Vec<u8>, state: &mut EmitState) {
    retract_and_wipe_with(output, state, true);
}

fn retract_and_wipe_with(output: &mut Vec<u8>, state: &mut EmitState, cooling_marker: bool) {
    let WipePath {
        segments,
        retraction_distance,
        distribution_distance,
    } = wipe_moves(state);
    let requested_before = state.options.retraction_length * state.options.retract_before_wipe;
    let remaining = state.options.retraction_length - requested_before;
    let wipe_speed = if state.options.role_based_wipe_speed {
        state.extrusion_feedrate / 60.0
    } else {
        state.options.wipe_speed
    }
    .max(10.0);
    let during = (state.options.retraction_feedrate / 60.0 * retraction_distance / wipe_speed)
        .min(remaining);
    let before = state.options.retraction_length - during;
    if before > f64::EPSILON {
        let retract = extrusion::coordinate(state, -before);
        output.extend_from_slice(
            format!(
                "G1 E{} F{}\n",
                format_extrusion(retract),
                format_axis(state.options.retraction_feedrate)
            )
            .as_bytes(),
        );
        state.current_feedrate = state.options.retraction_feedrate;
    }
    if !segments.is_empty() {
        let wipe_start = if state.tags.is_bbl() {
            "; WIPE_START\n"
        } else {
            ";WIPE_START\n"
        };
        output.extend_from_slice(wipe_start.as_bytes());
        let speed_line = if cooling_marker {
            format!("G1 F{};_WIPE\n", format_axis(wipe_speed * 60.0))
        } else {
            format!("G1 F{}\n", format_axis(wipe_speed * 60.0))
        };
        output.extend_from_slice(speed_line.as_bytes());
        state.current_feedrate = wipe_speed * 60.0;
        for (point, segment_length) in segments {
            let retraction = during * (segment_length / distribution_distance);
            // Orca's `extrude_to_xy` omits the E word when the wipe carries
            // no retraction (`retract_before_wipe` = 100% leaves nothing to
            // distribute during the wipe).
            let line = if retraction > f64::EPSILON {
                let retract = extrusion::coordinate(state, -retraction);
                format!(
                    "G1 X{} Y{} E{}\n",
                    format_axis(point.x),
                    format_axis(point.y),
                    format_extrusion(retract)
                )
            } else {
                format!("G1 X{} Y{}\n", format_axis(point.x), format_axis(point.y))
            };
            output.extend_from_slice(line.as_bytes());
            state.x = point.x;
            state.y = point.y;
            state.last_scaled_position = Some((
                ((point.x - state.offset.0) / state.scale_factor).round() as i64,
                ((point.y - state.offset.1) / state.scale_factor).round() as i64,
            ));
            state.wipe_start = Some(point);
        }
        let wipe_end = if state.tags.is_bbl() {
            "; WIPE_END\n"
        } else {
            ";WIPE_END\n"
        };
        output.extend_from_slice(wipe_end.as_bytes());
    }
    if !state.options.use_relative_e_distances && state.e_position.abs() > f64::EPSILON {
        output.extend_from_slice(b"G92 E0\n");
        state.e_position = 0.0;
    }
}

#[derive(Default)]
struct WipePath {
    segments: Vec<(arc::Point, f64)>,
    retraction_distance: f64,
    distribution_distance: f64,
}

fn wipe_moves(state: &EmitState) -> WipePath {
    if !state.options.wipe || state.options.wipe_distance <= 0.0 {
        return WipePath::default();
    }
    let start = state.wipe_start.unwrap_or(arc::Point {
        x: state.x,
        y: state.y,
    });
    let mut points = Vec::with_capacity(state.wipe_path.len());
    points.push(scaled_position(start, state));
    points.extend(
        state
            .wipe_path
            .iter()
            .skip(1)
            .map(|&point| scaled_position(point, state)),
    );
    let total_length = points
        .windows(2)
        .map(|segment| scaled_distance(segment[0], segment[1]))
        .sum::<f64>();
    let configured_distance = state.options.wipe_distance / state.scale_factor;
    let retraction_distance = total_length.min(configured_distance) * state.scale_factor;
    let mut clip = total_length - configured_distance;
    // Upstream `Polyline::clip_end` math: squared comparison, sqrt divisor,
    // truncating cast (`Polyline.cpp:52-72`).
    while clip > 0.0 {
        let last = points.pop().unwrap();
        let previous = *points.last().unwrap();
        let vx = (previous.0 - last.0) as f64;
        let vy = (previous.1 - last.1) as f64;
        let lsqr = vx * vx + vy * vy;
        if lsqr > clip * clip {
            let factor = clip / lsqr.sqrt();
            points.push((
                (last.0 as f64 + vx * factor) as i64,
                (last.1 as f64 + vy * factor) as i64,
            ));
            break;
        }
        clip -= lsqr.sqrt();
    }
    if let Ok(path) = std::env::var("ARES_DUMP_WIPE") {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = write!(file, "WP");
            for point in &points {
                let _ = write!(file, " ({},{})", point.0, point.1);
            }
            let _ = writeln!(file);
        }
    }
    let segments = points
        .windows(2)
        .filter_map(|segment| {
            let length = scaled_distance(segment[0], segment[1]);
            (length > f64::EPSILON).then(|| (unscaled_position(segment[1], state), length))
        })
        .collect::<Vec<_>>();
    let distribution_distance = segments
        .iter()
        .map(|(_, length)| length)
        .sum::<f64>()
        .min(configured_distance);
    if std::env::var_os("ARES_WIPE_DEBUG").is_some() {
        eprintln!(
            "WIPE_PATH_DEBUG points={points:?} total={total_length:.17} cfg={configured_distance:.17} clip_left={clip:.17} start_mm=({:.6},{:.6})",
            start.x, start.y
        );
    }
    WipePath {
        segments,
        retraction_distance,
        distribution_distance,
    }
}

fn scaled_distance(left: (i64, i64), right: (i64, i64)) -> f64 {
    let dx = (left.0 - right.0) as f64;
    let dy = (left.1 - right.1) as f64;
    (dx * dx + dy * dy).sqrt()
}

pub(in crate::project_slice::gcode_emit::motion) fn scaled_position(
    point: arc::Point,
    state: &EmitState,
) -> (i64, i64) {
    (
        ((point.x - state.offset.0) / state.scale_factor).round() as i64,
        ((point.y - state.offset.1) / state.scale_factor).round() as i64,
    )
}

fn unscaled_position(point: (i64, i64), state: &EmitState) -> arc::Point {
    arc::Point {
        x: point.0 as f64 * state.scale_factor + state.origin.0 - state.extruder_offset.0,
        y: point.1 as f64 * state.scale_factor + state.origin.1 - state.extruder_offset.1,
    }
}

pub(super) fn inside_internal_surfaces(
    surfaces: &[crate::project_slice::region_slices::RegionSurface],
    start: arc::Point,
    end: arc::Point,
    scale: crate::geometry::CoordinateScale,
    offset: (f64, f64),
) -> bool {
    use crate::project_slice::region_slices::RegionSurfaceKind;

    let point = |point: arc::Point| {
        crate::geometry::Point::new(
            scale.checked_scale(point.x - offset.0).unwrap(),
            scale.checked_scale(point.y - offset.1).unwrap(),
        )
    };
    let travel = crate::geometry::Polyline::new(vec![point(start), point(end)]);
    surfaces.iter().any(|surface| {
        let (kind, region, ..) = surface.as_parts();
        matches!(
            kind,
            RegionSurfaceKind::Internal
                | RegionSurfaceKind::InternalSolid
                | RegionSurfaceKind::InternalBridge
                | RegionSurfaceKind::InternalVoid
        ) && crate::geometry::open_polyline_inside_expolygon(&travel, region).unwrap()
    })
}

/// Defer the layer-change hop: upstream's `change_layer` retract runs
/// `maybe_zlift` (`GCodeWriter.cpp:626-648`) and `m_to_lift` survives into
/// the new layer's first travel, which raises to layer+hop and descends.
pub(in crate::project_slice::gcode_emit) fn defer_layer_change_lift(state: &mut EmitState) {
    // Upstream `change_layer` retracts while the writer still sits at the
    // previous layer's z (`GCode.cpp:5693-5706` before the z move), so the
    // `retract_lift_above/below` gate evaluates at that z, not the new
    // layer's (`GCodeWriter.cpp:633-639`).
    let writer_z = state.writer_z.map_or(state.layer_z, f64::from);
    lift::schedule_at(state, true, writer_z);
}
