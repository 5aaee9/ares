use super::super::{
    EmitState, LiftMode, arc,
    format::{axis as format_axis, offset as format_offset, z as format_z},
};

pub(super) fn schedule(state: &mut EmitState, layer_change: bool) {
    schedule_at(state, layer_change, state.layer_z);
}

/// Upstream `lazy_lift` gates the hop on the writer's *current* z
/// (`GCodeWriter.cpp:633-639` `retract_lift_above/below` vs `m_pos.z()`);
/// `change_layer` fires its retract before the layer z move, so the
/// deferred layer-change lift must evaluate the gate at the previous
/// layer's z, not the new one.
pub(super) fn schedule_at(state: &mut EmitState, layer_change: bool, writer_z: f64) {
    // `GCode.cpp:7678-7681` — `needs_lift` requires a non-zero retraction
    // length (or firmware retraction / toolchange): with retraction
    // disabled (nil length, e.g. Wanhao), `maybe_zlift` never runs and
    // no hop is ever deferred.
    if state.options.retraction_length <= 0.0 && !layer_change {
        return;
    }
    if state.options.z_hop <= 0.0 || !is_allowed_at(state, writer_z) {
        return;
    }
    state.pending_lift = Some(mode_for(state, layer_change));
}

/// The `LiftMode` a retract at this state schedules; shared with the
/// first-travel emission, which may run after a retraction that fired
/// before any lift could be scheduled.
pub(in crate::project_slice::gcode_emit) fn mode_for(
    state: &EmitState,
    layer_change: bool,
) -> LiftMode {
    match state.options.z_hop_type {
        crate::ZHopType::Auto if layer_change => LiftMode::Spiral,
        crate::ZHopType::Auto | crate::ZHopType::Slope => LiftMode::Slope,
        crate::ZHopType::Normal => LiftMode::Normal,
        crate::ZHopType::Spiral => LiftMode::Spiral,
    }
}

fn height_is_allowed_at(state: &EmitState, layer_z: f64) -> bool {
    const EPSILON: f64 = 1.0e-4;

    layer_z >= state.options.retract_lift_above - EPSILON
        && (state.options.retract_lift_below == 0.0
            || layer_z <= state.options.retract_lift_below + EPSILON)
}

fn height_is_allowed(state: &EmitState) -> bool {
    height_is_allowed_at(state, state.layer_z)
}

pub(in crate::project_slice::gcode_emit::motion) fn is_allowed_at(
    state: &EmitState,
    writer_z: f64,
) -> bool {
    use crate::RetractLiftEnforce;

    if !height_is_allowed_at(state, writer_z) {
        return false;
    }
    let bottom = state.layer_index == 0;
    let top = matches!(state.last_feature, Some("Top surface" | "Ironing"));
    match state.options.retract_lift_enforce {
        RetractLiftEnforce::AllSurfaces => true,
        RetractLiftEnforce::TopOnly => top,
        RetractLiftEnforce::BottomOnly => bottom,
        RetractLiftEnforce::TopAndBottom => top || bottom,
    }
}

/// Upstream `_travel_to_z` feedrate (`GCodeWriter.cpp:832-842`):
/// `travel_speed_z` when non-zero, else the first-layer or normal travel
/// speed per `m_is_first_layer`. Every standalone `G1 Z` travel carries it.
pub(in crate::project_slice::gcode_emit) fn z_feedrate(state: &EmitState) -> f64 {
    if state.options.z_travel_feedrate > 0.0 {
        state.options.z_travel_feedrate
    } else if state.layer_index == 0 {
        state.options.first_layer_travel_feedrate
    } else {
        state.travel_feedrate
    }
}

pub(in crate::project_slice::gcode_emit::motion) fn emit_pending(
    output: &mut Vec<u8>,
    target: arc::Point,
    state: &mut EmitState,
) -> bool {
    let Some(mode) = state.pending_lift.take() else {
        return false;
    };
    let raised_z = state.layer_z + state.options.z_hop;
    let dx = target.x - state.x;
    let dy = target.y - state.y;
    // `GCodeWriter::travel_to_xyz` measures the XY distance with Eigen
    // `Vec2d::norm()` (`sqrt(dx*dx + dy*dy)`, `GCodeWriter.cpp:718`), not
    // `hypot`; the spiral points land on 6-significant-digit rounding
    // boundaries where the extra `hypot` precision flips the last digit
    // (qVMcEP X91.7173 vs upstream X91.7174).
    let travel_distance = (dx * dx + dy * dy).sqrt();
    let emitted = match mode {
        LiftMode::Normal => {
            let feedrate = z_feedrate(state);
            output.extend_from_slice(
                format!("G1 Z{} F{}\n", format_z(raised_z), format_axis(feedrate)).as_bytes(),
            );
            state.current_feedrate = feedrate;
            true
        }
        LiftMode::Spiral => {
            let slope = state.options.travel_slope_radians;
            if slope > 0.0 && travel_distance > f64::EPSILON {
                let radius = state.options.z_hop / (std::f64::consts::TAU * slope.atan());
                append_spiral(
                    output,
                    state,
                    raised_z,
                    -dy / travel_distance * radius,
                    dx / travel_distance * radius,
                );
                state.current_feedrate = state.travel_feedrate;
                true
            } else {
                false
            }
        }
        LiftMode::Slope => {
            let slope = state.options.travel_slope_radians;
            if travel_distance > f64::EPSILON && state.options.z_hop.atan2(travel_distance) < slope
            {
                let slope_distance = state.options.z_hop / slope.tan();
                let x = state.x + dx * slope_distance / travel_distance;
                let y = state.y + dy * slope_distance / travel_distance;
                output.extend_from_slice(
                    format!(
                        "G1 X{} Y{} Z{} F{}\n",
                        format_axis(x),
                        format_axis(y),
                        format_z(raised_z),
                        format_axis(state.travel_feedrate)
                    )
                    .as_bytes(),
                );
                state.current_feedrate = state.travel_feedrate;
                true
            } else {
                false
            }
        }
    };
    state.lifted = true;
    state.lifted_amount = state.options.z_hop;
    emitted
}

pub(super) fn append_eager(output: &mut Vec<u8>, state: &mut EmitState) {
    if state.options.z_hop <= 0.0 || !height_is_allowed(state) {
        return;
    }
    let raised_z = state.layer_z + state.options.z_hop;
    // `GCodeWriter::eager_lift`: the spiral form requires a known-clear
    // position; an unknown position (right after the start g-code) falls
    // back to the normal Z-only lift.
    if matches!(
        state.options.z_hop_type,
        crate::ZHopType::Spiral | crate::ZHopType::Auto
    ) && state.options.travel_slope_radians > 0.0
        && state.positioned
    {
        let radius = state.options.z_hop
            / (std::f64::consts::TAU * state.options.travel_slope_radians.atan());
        append_spiral(output, state, raised_z, radius, 0.0);
        state.current_feedrate = state.travel_feedrate;
    } else {
        let feedrate = z_feedrate(state);
        output.extend_from_slice(
            format!("G1 Z{} F{}\n", format_z(raised_z), format_axis(feedrate)).as_bytes(),
        );
        state.current_feedrate = feedrate;
    }
    state.lifted = true;
    state.lifted_amount = state.options.z_hop;
}

pub(super) fn append_spiral_vase(output: &mut Vec<u8>, state: &mut EmitState) {
    if state.options.z_hop <= 0.0 || !is_allowed_at(state, state.layer_z) {
        return;
    }
    let feedrate = z_feedrate(state);
    output.extend_from_slice(
        format!(
            "G1 Z{} F{}\n",
            format_z(state.layer_z + state.options.z_hop),
            format_axis(feedrate)
        )
        .as_bytes(),
    );
    state.current_feedrate = feedrate;
    state.lifted = true;
    state.lifted_amount = state.options.z_hop;
}

fn append_spiral(output: &mut Vec<u8>, state: &EmitState, raised_z: f64, i: f64, j: f64) {
    if std::env::var_os("ARES_DEBUG_SPIRAL").is_some() {
        eprintln!(
            "SPIRAL layer_z={} raised={} hop={} writer_z={:?} source_layer_z={} retracted={} lifted={} pending={:?}",
            state.layer_z,
            raised_z,
            state.options.z_hop,
            state.writer_z,
            state.source_layer_z,
            state.retracted,
            state.lifted,
            state.pending_lift
        );
    }
    if state.options.enable_arc_fitting {
        output.extend_from_slice(b"G17\n");
        output.extend_from_slice(
            format!(
                "G3 Z{} I{} J{} P1  F{}\n",
                format_z(raised_z),
                format_offset(i),
                format_offset(j),
                format_axis(state.travel_feedrate)
            )
            .as_bytes(),
        );
        return;
    }

    output.extend_from_slice(format!("G1 F{}\n", format_axis(state.travel_feedrate)).as_bytes());
    let segments = (8.0 * (0.01 / state.options.arc_fitting_tolerance))
        .round()
        .clamp(4.0, 16.0) as usize;
    let center_x = state.x + i;
    let center_y = state.y + j;
    // `_spiral_travel_to_z` derives the circle radius from
    // `ij_offset.norm()` (`GCodeWriter.cpp:876`) — `sqrt(i*i + j*j)` with
    // its double rounding, not `hypot`.
    let radius = (i * i + j * j).sqrt();
    let start_angle = (state.y - center_y).atan2(state.x - center_x);
    for index in 1..segments {
        let progress = index as f64 / segments as f64;
        let angle = start_angle + std::f64::consts::TAU * progress;
        output.extend_from_slice(
            format!(
                "G1 X{} Y{} Z{}\n",
                super::super::super::format_processor_float(center_x + radius * angle.cos()),
                super::super::super::format_processor_float(center_y + radius * angle.sin()),
                super::super::super::format_processor_float(
                    state.layer_z + (raised_z - state.layer_z) * progress
                )
            )
            .as_bytes(),
        );
    }
    output.extend_from_slice(
        format!(
            "G1 X{} Y{} Z{}\n",
            super::super::super::format_processor_float(state.x),
            super::super::super::format_processor_float(state.y),
            super::super::super::format_processor_float(raised_z)
        )
        .as_bytes(),
    );
}
