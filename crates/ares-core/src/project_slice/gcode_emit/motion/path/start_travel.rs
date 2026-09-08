mod route;

use route::{plan_route, polyline_length};

use super::{PathProperties, retraction, travel_emit};
use crate::project_slice::gcode_emit::motion::{
    EmitState, LayerGeometry, LiftMode, append_object_start, arc, begin_path_travel, extrusion,
    format::{axis as format_axis, extrusion as format_extrusion, z as format_z},
    travel,
};

pub(super) struct Request<'a> {
    pub(super) first_scaled: (i64, i64),
    pub(super) first_x: f64,
    pub(super) first_y: f64,
    pub(super) properties: PathProperties<'a>,
    pub(super) geometry: LayerGeometry<'a>,
}

pub(super) fn emit(output: &mut Vec<u8>, state: &mut EmitState, request: Request<'_>) {
    let Request {
        first_scaled,
        first_x,
        first_y,
        properties,
        geometry,
    } = request;
    let first_position = !state.positioned;
    let layer_change_travel = state.layer_change_travel_pending && !first_position;
    let slope_start_z = properties
        .slope
        .filter(|slope| (slope.z_begin - slope.z_end).abs() > super::SOURCE_EPSILON_MM)
        .map(|slope| {
            state.layer_z - f64::from(properties.height)
                + f64::from(properties.height) * slope.z_begin
        });
    let slope_needs_z_travel = slope_start_z.is_some_and(|target| {
        (state.scarf_z.unwrap_or(state.layer_z) - target).abs() >= super::SOURCE_EPSILON_MM
    });
    let target_z = slope_start_z.unwrap_or(state.layer_z);
    let needs_travel =
        first_position || state.last_scaled_position != Some(first_scaled) || slope_needs_z_travel;
    let travel_distance = (first_x - state.x).hypot(first_y - state.y);
    let mut travel_set_layer_z = false;
    let mut eager_lifted_travel = false;
    let mut unclear_position_travel = false;
    if needs_travel {
        begin_path_travel(output, state, properties.feature, travel_distance);
        let inside_internal_surface = travel::inside_internal_surfaces(
            geometry.internal_surfaces,
            arc::Point {
                x: state.x,
                y: state.y,
            },
            arc::Point {
                x: first_x,
                y: first_y,
            },
            geometry.scale,
            state.offset,
        );
        let skip_retraction = super::can_skip_retraction(
            state.options.reduce_infill_retraction,
            state.options.has_sparse_infill,
            state.last_feature,
            properties.is_perimeter,
            inside_internal_surface,
        );
        // Orca decides the travel retraction on the routed polyline length
        // (`GCode.cpp:7424-7425` re-checks `needs_retraction` after
        // `avoid_crossing_perimeters.travel_to`), so route first and then
        // decide; a wiping retract moves the head and the route is planned
        // again from the new position (`GCode.cpp:7436-7443`).
        let mut route = plan_route(state, &geometry, properties.feature, first_x, first_y);
        let routed_length = polyline_length(state.x, state.y, &route);
        let retract = !state.retracted
            && routed_length >= state.options.retraction_minimum_travel
            && !skip_retraction;
        // Upstream `retract()` also runs for a long travel with the
        // extruder already retracted — dE is a no-op but `maybe_zlift`
        // defers, gated on m_lifted == 0 && m_to_lift == 0
        // (GCodeWriter.cpp:637). The lifted_amount model mirrors
        // m_lifted's distance semantics.
        let defer_lift_for_retracted_travel = state.retracted
            && state.lifted_amount == 0.0
            && state.pending_lift.is_none()
            && routed_length >= state.options.retraction_minimum_travel
            && !skip_retraction;
        if retract {
            let head_before = (state.x, state.y);
            travel::retract_and_lift(output, state);
            if (state.x, state.y) != head_before {
                // The wipe moved the head; re-plan the route from here
                // (`GCode.cpp:7436-7443`).
                route = plan_route(state, &geometry, properties.feature, first_x, first_y);
            }
        } else if defer_lift_for_retracted_travel
            && state.options.z_hop > 0.0
            && state.options.retraction_length > 0.0
            && travel::lift_is_allowed_at(state, state.layer_z)
        {
            state.pending_lift = Some(travel::lift_mode_for(state, false));
        }
        let first_travel = route[0];
        let (travel_x, travel_y) = (first_travel.x, first_travel.y);
        append_object_start(output, state);
        // Consume the one-shot disable after the travel is planned
        // (`reset_once_modifiers`, `GCode.cpp:7431`) — only routed
        // travels consume it; the skirt's own travels keep it armed.
        if !matches!(properties.feature, "Skirt" | "Brim") {
            state.avoid_crossing_disabled_once = false;
        }
        // Upstream never schedules a layer-start lift without a retraction
        // (`GCode.cpp:5692-5698`: `change_layer`'s own retract — gated on
        // `retract_when_changing_layer` — is the only deferral source; with
        // it off, `m_to_lift` stays 0 and `travel_to_xyz` leaves the
        // destination at the layer z, `GCodeWriter.cpp:701-710`).
        // `GCodeWriter::travel_to_xyz` (`GCodeWriter.cpp:685-707`) only raises
        // the travel destination for a hop scheduled with this travel
        // (`m_to_lift`); a nozzle already lifted by an earlier sequence (the
        // layer-end timelapse retract, a deferred layer-change lift) travels
        // flat in XY and the unlift descends at the destination.
        let lifted_for_travel = state.pending_lift.is_some();
        // The print's first travel has no known source position, so Orca
        // cannot slope/spiral from it (`GCodeWriter.cpp:travel_to_xyz`
        // skips those branches when `is_current_position_clear()` is
        // false). A pending normal lift still raises before the XY move
        // (`NormalLift` `slop_move`), and both paths end with the separate
        // `_travel_to_z` re-statement from the unclear-position branch.
        let first_travel_lift = if first_position {
            // Upstream: a first-travel lift exists via the travel's own
            // retraction (`GCode.cpp:7440` needs_retraction →
            // `GCodeWriter.cpp:626-648` maybe_zlift defers `m_to_lift`).
            // With the nozzle ALREADY retracted (the start gcode's
            // retract, e.g. Kobra), `retract()` still runs — dE is a
            // no-op but maybe_zlift defers. And with an UNCLEAR source
            // position upstream's travel polyline starts at (0,0), so
            // needs_retraction ALWAYS holds for the first travel — no
            // length check here.
            if !skip_retraction
                && state.pending_lift.is_none()
                && state.options.z_hop > 0.0
                && !state.spiral_vase
                && travel::lift_is_allowed_at(state, state.layer_z)
                && !state.lifted
            {
                state.pending_lift = Some(travel::lift_mode_for(state, false));
            }
            state.pending_lift.take()
        } else {
            travel::emit_pending_lift(
                output,
                arc::Point {
                    x: travel_x,
                    y: travel_y,
                },
                state,
            );
            None
        };
        if state.spiral_vase
            && state.lifted
            && !first_position
            && (layer_change_travel
                || (state.source_layer_z + state.options.z_hop) - state.options.z_hop
                    > state.source_layer_z)
        {
            travel_emit::xyz(
                output,
                travel_x,
                travel_y,
                state.layer_z,
                state.travel_feedrate,
            );
            travel_set_layer_z = true;
        } else if state.template_lifted && state.lifted && !first_position {
            travel_emit::xy(output, travel_x, travel_y, state.travel_feedrate);
            state.template_lifted = false;
        } else if state.lifted {
            eager_lifted_travel = true;
            if !lifted_for_travel {
                travel_emit::xy(output, travel_x, travel_y, state.travel_feedrate);
            } else if (state.current_feedrate - state.travel_feedrate).abs() > f64::EPSILON {
                travel_emit::xyz(
                    output,
                    travel_x,
                    travel_y,
                    state.layer_z + state.options.z_hop,
                    state.travel_feedrate,
                );
            } else {
                output.extend_from_slice(
                    format!(
                        "G1 X{} Y{} Z{}\n",
                        format_axis(travel_x),
                        format_axis(travel_y),
                        format_z(state.layer_z + state.options.z_hop)
                    )
                    .as_bytes(),
                );
            }
        } else if layer_change_travel && state.retracted {
            if state.options.z_hop > 0.0 && retraction::uses_sloped_lift(state.options.z_hop_type) {
                travel_emit::xy(output, travel_x, travel_y, state.travel_feedrate);
            } else if state.lifted {
                output.extend_from_slice(
                    format!(
                        "G1 X{} Y{} Z{}\n",
                        format_axis(travel_x),
                        format_axis(travel_y),
                        format_z(state.layer_z + state.options.z_hop)
                    )
                    .as_bytes(),
                );
            } else {
                travel_emit::xyz(output, travel_x, travel_y, target_z, state.travel_feedrate);
                travel_set_layer_z = true;
            }
        } else if state.retracted
            && first_position
            && state.options.z_hop > 0.0
            && (first_travel_lift.is_some() || travel::lift_is_allowed_at(state, state.layer_z))
        {
            let mode = first_travel_lift.unwrap_or_else(|| travel::lift_mode_for(state, true));
            if mode == LiftMode::Normal {
                let feedrate = travel::lift_z_feedrate(state);
                output.extend_from_slice(
                    format!(
                        "G1 Z{} F{}\n",
                        format_z(state.layer_z + state.options.z_hop),
                        format_axis(feedrate)
                    )
                    .as_bytes(),
                );
                state.current_feedrate = feedrate;
                travel_emit::xy_without_feed(output, travel_x, travel_y);
            } else {
                travel_emit::xy(output, travel_x, travel_y, state.travel_feedrate);
            }
            output.extend_from_slice(
                format!(
                    "G1 Z{} F{}\n",
                    format_z(state.layer_z + state.options.z_hop),
                    format_axis(travel::lift_z_feedrate(state))
                )
                .as_bytes(),
            );
            state.lifted = true;
            state.lifted_amount = state.options.z_hop;
        } else if layer_change_travel {
            // The sloped split only applies when a lift actually exists
            // (`GCodeWriter.cpp:725-757`: slope/spiral moves come from the
            // `m_to_lift` block; with no lift the plain combined XYZ path
            // emits the destination directly).
            let has_lift = state.lifted || state.pending_lift.is_some();
            if has_lift
                && state.options.z_hop > 0.0
                && retraction::uses_sloped_lift(state.options.z_hop_type)
            {
                travel_emit::xy(output, travel_x, travel_y, state.travel_feedrate);
                let z_feedrate = travel::lift_z_feedrate(state);
                output.extend_from_slice(
                    format!("G1 Z{} F{}\n", format_z(target_z), format_axis(z_feedrate)).as_bytes(),
                );
                state.current_feedrate = z_feedrate;
            } else {
                travel_emit::xyz(output, travel_x, travel_y, target_z, state.travel_feedrate);
            }
            travel_set_layer_z = true;
        } else if let Some(z) = slope_start_z {
            travel_emit::xyz(output, travel_x, travel_y, z, state.travel_feedrate);
            travel_set_layer_z = true;
        } else {
            travel_emit::xy(output, travel_x, travel_y, state.travel_feedrate);
            // The print's first travel from an unknown position always
            // splits: the unclear-position branch of `travel_to_xyz`
            // (`GCodeWriter.cpp:754+`) emits XY then `_travel_to_z`
            // separately — `force_z` only merges Z for CLEAR-position
            // travels.
            if first_position {
                let z_feedrate = travel::lift_z_feedrate(state);
                output.extend_from_slice(
                    format!("G1 Z{} F{}\n", format_z(target_z), format_axis(z_feedrate)).as_bytes(),
                );
                state.current_feedrate = z_feedrate;
                unclear_position_travel = true;
            }
        }
        state.x = travel_x;
        state.y = travel_y;
        for point in &route[1..] {
            travel_emit::xy_without_feed(output, point.x, point.y);
            state.x = point.x;
            state.y = point.y;
        }
        state.last_scaled_position = Some(first_scaled);
        state.positioned = true;
        state.current_feedrate = state.travel_feedrate;
    } else if layer_change_travel {
        let z_feedrate = travel::lift_z_feedrate(state);
        output.extend_from_slice(
            format!("G1 Z{} F{}\n", format_z(target_z), format_axis(z_feedrate)).as_bytes(),
        );
        travel_set_layer_z = true;
        state.current_feedrate = z_feedrate;
    }
    if let Some(z) = slope_start_z {
        state.scarf_z = Some(z);
    }
    state.layer_change_travel_pending = false;
    append_object_start(output, state);
    // Orca `_extrude` (`GCode.cpp:6378-6385`): the print's first extrusion
    // re-states Z to sync the writer with the planned layer height
    // (`_last_pos_undefined`). It applies whenever the travel emitted the
    // Z as a separate unclear-position descend — the eager-lift branch or
    // the unknown-source first travel. The lazy-lift branch's descend above
    // already models it.
    if (eager_lifted_travel || unclear_position_travel) && first_position && !travel_set_layer_z {
        let z_feedrate = travel::lift_z_feedrate(state);
        output.extend_from_slice(
            format!(
                "G1 Z{} F{}\n",
                format_z(state.layer_z),
                format_axis(z_feedrate)
            )
            .as_bytes(),
        );
        state.current_feedrate = z_feedrate;
    }
    if state.retracted {
        if first_position
            && state.options.z_hop > 0.0
            && !state.lifted
            && travel::lift_is_allowed_at(state, state.layer_z)
        {
            let z_feedrate = travel::lift_z_feedrate(state);
            output.extend_from_slice(
                format!(
                    "G1 Z{} F{}\n",
                    format_z(state.layer_z + state.options.z_hop),
                    format_axis(z_feedrate)
                )
                .as_bytes(),
            );
            state.current_feedrate = z_feedrate;
            state.lifted = true;
            state.lifted_amount = state.options.z_hop;
        }
        if state.lifted && !travel_set_layer_z {
            let z = slope_start_z.unwrap_or(state.layer_z);
            let z_feedrate = travel::lift_z_feedrate(state);
            output.extend_from_slice(
                format!("G1 Z{} F{}\n", format_z(z), format_axis(z_feedrate)).as_bytes(),
            );
            state.current_feedrate = z_feedrate;
            state.scarf_z = slope_start_z;
        }
        let retraction_length = state.options.retraction_length;
        let unretract = extrusion::coordinate(
            state,
            retraction_length + state.options.retract_restart_extra,
        );
        // `Extruder::unretract()` zeroes `m_retracted` after the extrude;
        // `coordinate` only accumulates it for negative deltas.
        state.retracted_amount = 0.0;
        output.extend_from_slice(
            format!(
                "G1 E{} F{}\n",
                format_extrusion(unretract),
                format_axis(state.options.deretraction_feedrate)
            )
            .as_bytes(),
        );
        state.current_feedrate = state.options.deretraction_feedrate;
        state.retracted = false;
        state.lifted = false;
        state.lifted_amount = 0.0;
        state.template_lifted = false;
    }
}
