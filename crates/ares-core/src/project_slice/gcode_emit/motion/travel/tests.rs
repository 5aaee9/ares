use super::super::{LiftMode, MotionOptions};
use super::{
    EmitState, emit_pending_lift, flush_pending_retract_lift, inside_internal_surfaces,
    lift_is_allowed_at, retract_and_lift, retract_for_print_end, wipe_moves,
};
use crate::{
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::{gcode_emit::motion::arc, region_slices::RegionSurface},
};

fn internal_square() -> RegionSurface {
    RegionSurface::internal(ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(10_000_000, 0),
            Point::new(10_000_000, 10_000_000),
            Point::new(0, 10_000_000),
        ]),
        Vec::new(),
    ))
}

#[test]
fn bottom_only_lift_does_not_lift_later_layers() {
    let mut state = EmitState {
        layer_index: 1,
        options: MotionOptions {
            retraction_length: 1.0,
            retraction_feedrate: 3_600.0,
            z_hop: 0.4,
            retract_lift_enforce: crate::RetractLiftEnforce::BottomOnly,
            use_relative_e_distances: true,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };
    let mut output = Vec::new();

    retract_and_lift(&mut output, &mut state);

    assert_eq!(output, b"G1 E-1 F3600\n");
    assert!(state.retracted);
    assert_eq!(state.pending_lift, None);
}

#[test]
fn top_only_lift_requires_a_top_or_ironing_feature() {
    let mut state = EmitState {
        layer_z: 0.2,
        options: MotionOptions {
            z_hop: 0.4,
            retract_lift_enforce: crate::RetractLiftEnforce::TopOnly,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };

    assert!(!lift_is_allowed_at(&state, state.layer_z));
    state.last_feature = Some("Top surface");
    assert!(lift_is_allowed_at(&state, state.layer_z));
}

#[test]
fn deferred_bottom_only_lift_uses_the_target_layer_index() {
    let mut state = EmitState {
        layer_index: 1,
        pending_layer_retract: true,
        options: MotionOptions {
            z_hop: 0.4,
            retract_lift_enforce: crate::RetractLiftEnforce::BottomOnly,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };

    let writer_z = state.layer_z;
    flush_pending_retract_lift(&mut Vec::new(), &mut state, writer_z);

    assert_eq!(state.pending_lift, None);
    assert!(state.retracted);
}

#[test]
fn layer_change_hop_cannot_raise_above_destination_is_dropped() {
    // `GCodeWriter::travel_to_xyz` (`GCodeWriter.cpp:701-710`): a deferred
    // `m_to_lift` only raises the travel destination while
    // `m_to_lift + m_pos(2) > point(2)`; when the layer step already reaches
    // the hop height (z_hop == layer height, e.g. LH Stinger), the hop is
    // dropped and the travel moves straight to the layer z.
    let mut state = EmitState {
        layer_z: 0.4,
        pending_layer_retract: true,
        options: MotionOptions {
            retraction_length: 1.0,
            z_hop: 0.2,
            z_hop_type: crate::ZHopType::Auto,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };

    flush_pending_retract_lift(&mut Vec::new(), &mut state, 0.2);

    assert_eq!(state.pending_lift, None);
    assert!(state.retracted);
}

#[test]
fn slope_type_schedules_a_slope_lift() {
    let mut state = EmitState {
        layer_z: 0.4,
        options: MotionOptions {
            // `maybe_zlift` requires `needs_lift` (retraction_length > 0,
            // GCode.cpp:7678-7681).
            retraction_length: 1.0,
            z_hop: 0.4,
            z_hop_type: crate::ZHopType::Slope,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };

    retract_and_lift(&mut Vec::new(), &mut state);

    assert_eq!(state.pending_lift, Some(LiftMode::Slope));
}

#[test]
fn lift_above_gate_keeps_retraction_without_lifting() {
    let mut state = EmitState {
        layer_z: 0.4,
        options: MotionOptions {
            retraction_length: 1.0,
            retraction_feedrate: 3_600.0,
            z_hop: 0.4,
            retract_lift_above: 0.5,
            use_relative_e_distances: true,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };
    let mut output = Vec::new();

    retract_and_lift(&mut output, &mut state);

    assert_eq!(output, b"G1 E-1 F3600\n");
    assert_eq!(state.pending_lift, None);
}

#[test]
fn degenerate_slope_uses_the_raised_target_without_a_preliminary_move() {
    let mut state = EmitState {
        layer_z: 0.4,
        pending_lift: Some(LiftMode::Slope),
        options: MotionOptions {
            z_hop: 0.4,
            travel_slope_radians: 0.0,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };
    let mut output = Vec::new();

    assert!(!emit_pending_lift(
        &mut output,
        arc::Point { x: 1.0, y: 1.0 },
        &mut state,
    ));
    assert!(output.is_empty());
    assert!(state.lifted);
}

#[test]
fn print_end_retracts_without_a_layer_change_lift() {
    let mut state = EmitState {
        options: MotionOptions {
            retraction_length: 1.0,
            retraction_feedrate: 3_600.0,
            use_relative_e_distances: true,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };
    let mut output = Vec::new();

    retract_for_print_end(&mut output, &mut state);

    assert_eq!(output, b"G1 E-1 F3600\n");
    assert!(state.retracted);
    assert!(!state.lifted);
}

#[test]
fn travel_fully_inside_internal_surface_avoids_retraction() {
    assert!(inside_internal_surfaces(
        &[internal_square()],
        arc::Point { x: 1.0, y: 1.0 },
        arc::Point { x: 9.0, y: 9.0 },
        CoordinateScale::Normal,
        (0.0, 0.0),
    ));
}

#[test]
fn travel_leaving_internal_surface_requires_retraction() {
    assert!(!inside_internal_surfaces(
        &[internal_square()],
        arc::Point { x: 1.0, y: 1.0 },
        arc::Point { x: 11.0, y: 9.0 },
        CoordinateScale::Normal,
        (0.0, 0.0),
    ));
}

#[test]
fn pending_spiral_lift_uses_resolution_based_linear_segments() {
    let mut state = EmitState {
        x: 102.379,
        y: 108.302,
        layer_z: 0.4,
        travel_feedrate: 12_000.0,
        pending_lift: Some(LiftMode::Spiral),
        options: MotionOptions {
            z_hop: 0.4,
            travel_slope_radians: 3.0_f64.to_radians(),
            arc_fitting_tolerance: 0.012,
            travel_feedrate: 12_000.0,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };
    let mut output = Vec::new();

    assert!(emit_pending_lift(
        &mut output,
        arc::Point {
            x: 108.991,
            y: 108.991,
        },
        &mut state,
    ));

    assert_eq!(
        output,
        b"G1 F12000\n\
G1 X103.278 Y108.856 Z0.457143\n\
G1 X103.405 Y109.905 Z0.514286\n\
G1 X102.664 Y110.658 Z0.571429\n\
G1 X101.614 Y110.548 Z0.628571\n\
G1 X101.045 Y109.659 Z0.685714\n\
G1 X101.385 Y108.659 Z0.742857\n\
G1 X102.379 Y108.302 Z0.8\n"
    );
}

#[test]
fn wipe_distribution_uses_configured_distance_when_clipping_rounds_long() {
    let state = EmitState {
        scale_factor: 0.000_001,
        options: MotionOptions {
            wipe: true,
            wipe_distance: 1.0,
            ..MotionOptions::default()
        },
        wipe_start: Some(arc::Point { x: 0.0, y: 0.0 }),
        wipe_path: vec![
            arc::Point { x: 0.0, y: 0.0 },
            arc::Point { x: 0.6, y: 0.2 },
            arc::Point { x: -1.0, y: 0.4 },
        ],
        ..EmitState::default()
    };

    let path = wipe_moves(&state);
    let clipped_distance = path.segments.iter().map(|(_, length)| length).sum::<f64>();

    assert!(clipped_distance > 1_000_000.0);
    assert_eq!(path.retraction_distance, 1.0);
    assert_eq!(path.distribution_distance, 1_000_000.0);
}
