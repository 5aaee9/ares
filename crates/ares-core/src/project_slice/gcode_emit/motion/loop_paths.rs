#[cfg(test)]
mod tests;

use super::{
    EmitState, LayerGeometry,
    format::{axis as format_axis, z as format_z},
};
use crate::project_slice::perimeters::classic::{
    chained_loops::ExtrusionLoopRole,
    materialize::{ExtrusionPath, ExtrusionRole, Point3},
};

pub(super) fn emit(
    output: &mut Vec<u8>,
    paths: &[ExtrusionPath],
    loop_role: ExtrusionLoopRole,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) {
    append_wipe_before_external(output, paths, loop_role, geometry, state);
    if !state.spiral_vase
        && let Some(scarf) = super::scarf::build(
            paths,
            loop_role,
            geometry,
            &state.options,
            state.layer_index,
        )
    {
        for path in &scarf.paths {
            super::materialized::emit(
                output,
                state,
                super::materialized::Emission {
                    path: &path.path,
                    end_clip: 0.0,
                    slope: path.slope,
                    geometry,
                },
            );
        }
        state.wipe_path = wipe_points(&scarf.wipe_paths, geometry, state.offset);
        append_inward_move(output, &scarf.wipe_paths, loop_role, geometry, state);
        return;
    }
    // `GCode.cpp:4596-4612,5796-5804`: active vase layers keep complete
    // loops for the full-layer SpiralVase filter.
    let mut remaining_clip = if state.spiral_vase_layer {
        0.0
    } else {
        state.options.seam_gap
    };
    let mut path_count = paths.len();
    while path_count > 0 {
        let length = path_length(&paths[path_count - 1], geometry);
        if length > remaining_clip {
            break;
        }

        path_count -= 1;
        remaining_clip -= length;
    }
    let mut emitted_loop_path = Vec::new();
    for (index, path) in paths[..path_count].iter().enumerate() {
        let end_clip = if index + 1 == path_count {
            remaining_clip
        } else {
            0.0
        };
        super::materialized::emit_flat(output, path, end_clip, geometry, state);
        if emitted_loop_path.is_empty() {
            emitted_loop_path.extend(state.wipe_path.iter().rev().copied());
        } else {
            emitted_loop_path.extend(state.wipe_path.iter().rev().copied().skip(1));
        }
    }
    state.wipe_path = emitted_loop_path;
    append_inward_move(output, paths, loop_role, geometry, state);
}

fn wipe_points(
    paths: &[ExtrusionPath],
    geometry: LayerGeometry<'_>,
    offset: (f64, f64),
) -> Vec<super::arc::Point> {
    let mut points = Vec::new();
    for path in paths {
        for point in &path.polyline.points {
            let point = super::arc::Point {
                x: geometry.scale.unscale(point.x) + offset.0,
                y: geometry.scale.unscale(point.y) + offset.1,
            };
            if points.last() != Some(&point) {
                points.push(point);
            }
        }
    }
    points
}

fn append_wipe_before_external(
    output: &mut Vec<u8>,
    paths: &[ExtrusionPath],
    loop_role: ExtrusionLoopRole,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) {
    let Some(first) = paths.first() else {
        return;
    };
    let last = paths.last().unwrap();
    if !state.options.wipe_before_external_loop
        || first.role != ExtrusionRole::ExternalPerimeter
        || state.options.wall_loops <= 1
        || first.polyline.points.len() < 2
        || last.polyline.points.len() < 2
    {
        return;
    }
    let current = first.polyline.points[0];
    let previous = first.polyline.points[1];
    let next = clipped_loop_end(paths, state.options.seam_gap, geometry);
    let mut a = next;
    let mut b = previous;
    let is_hole = loop_role == ExtrusionLoopRole::Hole;
    let counter_clockwise = loop_is_counter_clockwise(paths);
    if is_hole != counter_clockwise {
        std::mem::swap(&mut a, &mut b);
    }
    let mut angle = ccw_angle(current, a, b) / 3.0;
    if is_hole != counter_clockwise {
        angle = -angle;
    }
    let dx = (next.x - current.x) as f64;
    let dy = (next.y - current.y) as f64;
    let length = dx.hypot(dy);
    if length == 0.0 {
        return;
    }
    let maximum =
        0.5 * state.options.nozzle_diameter.min(f64::from(first.width)) / geometry.scale.factor();
    let base_x = (current.x as f64 + dx * (2.0 * maximum / length)) as i64 as f64;
    let base_y = (current.y as f64 + dy * (2.0 * maximum / length)) as i64 as f64;
    let (sine, cosine) = angle.sin_cos();
    let x = current.x as f64 + cosine * (base_x - current.x as f64)
        - sine * (base_y - current.y as f64);
    let y = current.y as f64
        + sine * (base_x - current.x as f64)
        + cosine * (base_y - current.y as f64);
    let point = crate::geometry::Point::new(x.round() as i64, y.round() as i64);
    let x = geometry.scale.unscale(point.x()) + state.offset.0;
    let y = geometry.scale.unscale(point.y()) + state.offset.1;
    // Orca routes this hop through `travel_to` inside the fake
    // extrusion path (`GCode.cpp:5884-5893`), so the travel-class
    // accel/jerk setup fires for it exactly like any other path-start
    // travel (the GT trace shows the `role=2` travel_to for it).
    super::state::begin_path_travel(
        output,
        state,
        "Outer wall",
        (x - state.x).hypot(y - state.y),
    );
    // The wipe hop is a `travel_to` (`GCode.cpp:5884-5893`): a lift
    // deferred by the preceding retract (e.g. the start gcode's) is
    // consumed HERE. With the nozzle already retracted and no lift
    // deferred yet, upstream's `retract()` (needs_retraction always
    // holds from the unclear start position) defers one — same here.
    let lift_here = state.pending_lift.is_some()
        || (state.retracted
            && state.options.z_hop > 0.0
            && state.options.retraction_length > 0.0
            && !state.lifted
            && super::travel::lift_is_allowed_at(state, state.layer_z));
    if lift_here {
        let raised = state.layer_z + state.options.z_hop;
        output.extend_from_slice(
            format!(
                "G1 X{} Y{} Z{} F{}\n",
                format_axis(x),
                format_axis(y),
                format_z(raised),
                format_axis(state.travel_feedrate)
            )
            .as_bytes(),
        );
        output.extend_from_slice(
            format!(
                "G1 Z{} F{}\n",
                format_z(state.layer_z),
                format_axis(super::travel::lift_z_feedrate(state))
            )
            .as_bytes(),
        );
        state.lifted = false;
        state.lifted_amount = 0.0;
        state.pending_lift = None;
    } else {
        output.extend_from_slice(
            format!(
                "G1 X{} Y{} F{}\n",
                format_axis(x),
                format_axis(y),
                format_axis(state.travel_feedrate)
            )
            .as_bytes(),
        );
    }
    state.x = x;
    state.y = y;
    state.current_feedrate = state.travel_feedrate;
    state.last_scaled_position = Some((current.x, current.y));
    state.pending_wipe_before_external_target = Some(super::arc::Point {
        x: geometry.scale.unscale(current.x) + state.offset.0,
        y: geometry.scale.unscale(current.y) + state.offset.1,
    });
}

fn clipped_loop_end(
    paths: &[ExtrusionPath],
    mut distance: f64,
    geometry: LayerGeometry<'_>,
) -> Point3 {
    for path in paths.iter().rev() {
        for segment in path.polyline.points.windows(2).rev() {
            let dx = geometry.scale.unscale(segment[1].x - segment[0].x);
            let dy = geometry.scale.unscale(segment[1].y - segment[0].y);
            let length = dx.hypot(dy);
            if length > distance {
                let ratio = distance / length;
                return Point3 {
                    x: (segment[1].x as f64 + (segment[0].x - segment[1].x) as f64 * ratio) as i64,
                    y: (segment[1].y as f64 + (segment[0].y - segment[1].y) as f64 * ratio) as i64,
                    z: segment[1].z,
                };
            }
            distance -= length;
        }
    }
    paths[0].polyline.points[0]
}

fn append_inward_move(
    output: &mut Vec<u8>,
    paths: &[ExtrusionPath],
    loop_role: ExtrusionLoopRole,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) {
    let Some(first) = paths.first() else {
        return;
    };
    let last = paths.last().unwrap();
    if !state.options.wipe_on_loops
        || last.role != ExtrusionRole::ExternalPerimeter
        || state.options.wall_loops <= 1
        || first.polyline.points.len() < 2
        || last.polyline.points.len() < 3
    {
        return;
    }

    let point = inward_point(
        first.polyline.points[0],
        first.polyline.points[1],
        last.polyline.points[last.polyline.points.len() - 3],
        (
            loop_role == ExtrusionLoopRole::Hole,
            loop_is_counter_clockwise(paths),
        ),
        state.options.nozzle_diameter / geometry.scale.factor(),
    );
    let x = geometry.scale.unscale(point.x()) + state.offset.0;
    let y = geometry.scale.unscale(point.y()) + state.offset.1;
    output.extend_from_slice(format!("G1 X{} Y{}\n", format_axis(x), format_axis(y)).as_bytes());
    state.x = x;
    state.y = y;
    // `GCode::extrude_loop` (GCode.cpp:6032) moves only the writer here;
    // m_last_pos remains the clipped extrusion endpoint for subsequent chaining.
}

fn inward_point(
    center: Point3,
    mut first: Point3,
    mut second: Point3,
    orientation: (bool, bool),
    nozzle: f64,
) -> crate::geometry::Point {
    let (is_hole, counter_clockwise) = orientation;
    if is_hole == counter_clockwise {
        std::mem::swap(&mut first, &mut second);
    }
    let mut angle = ccw_angle(center, first, second) / 3.0;
    if is_hole == counter_clockwise {
        angle = -angle;
    }
    let dx = (first.x - center.x) as f64;
    let dy = (first.y - center.y) as f64;
    let length_squared = dx * dx + dy * dy;
    let factor = if nozzle * nozzle < length_squared {
        0.2 * nozzle / length_squared.sqrt()
    } else {
        0.2
    };
    let x = center.x as f64 + dx * factor;
    let y = center.y as f64 + dy * factor;
    let (sine, cosine) = angle.sin_cos();
    let rotated_x = center.x as f64 + cosine * (x - center.x as f64) - sine * (y - center.y as f64);
    let rotated_y = center.y as f64 + sine * (x - center.x as f64) + cosine * (y - center.y as f64);
    crate::geometry::Point::new(
        (rotated_x + 0.5).floor() as i64,
        (rotated_y + 0.5).floor() as i64,
    )
}

fn loop_is_counter_clockwise(paths: &[ExtrusionPath]) -> bool {
    let points = paths
        .iter()
        .flat_map(|path| path.polyline.points.iter())
        .collect::<Vec<_>>();
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .map(|(left, right)| left.x as i128 * right.y as i128 - right.x as i128 * left.y as i128)
        .sum::<i128>()
        > 0
}

fn ccw_angle(center: Point3, first: Point3, second: Point3) -> f64 {
    let angle = ((first.x - center.x) as f64).atan2((first.y - center.y) as f64)
        - ((second.x - center.x) as f64).atan2((second.y - center.y) as f64);
    if angle <= 0.0 {
        angle + std::f64::consts::TAU
    } else {
        angle
    }
}

fn path_length(path: &ExtrusionPath, geometry: LayerGeometry<'_>) -> f64 {
    path.polyline
        .points
        .windows(2)
        .map(|segment| {
            geometry
                .scale
                .unscale(segment[1].x - segment[0].x)
                .hypot(geometry.scale.unscale(segment[1].y - segment[0].y))
        })
        .sum()
}
