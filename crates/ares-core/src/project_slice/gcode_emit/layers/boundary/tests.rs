//! Ordering regression for the change-layer state advance
//! (`layers/boundary.rs::advance_for_layer`).
use super::super::motion::{EmitState, MotionOptions};
use crate::project_slice::gcode_emit::layers::boundary::advance_for_layer;

fn top_and_bottom_state() -> EmitState {
    EmitState {
        layer_index: 0,
        writer_z: Some(0.2),
        options: MotionOptions {
            retraction_length: 1.0,
            z_hop: 0.2,
            z_hop_type: crate::ZHopType::Auto,
            retract_lift_enforce: crate::RetractLiftEnforce::TopAndBottom,
            ..Default::default()
        },
        ..EmitState::default()
    }
}

/// The 0 -> 1 layer transition must defer NO hop under `Top and Bottom`:
/// `change_layer` increments `m_layer_index` before its retract
/// (`GCode.cpp:5690-5696`), so the enforce gate sees index 1 and its
/// bottom clause cannot hold. Reverting the advance/defer ordering
/// inside `advance_for_layer` would evaluate the gate at index 0 and
/// wrongly defer a hop, failing this test.
#[test]
fn zero_to_one_transition_advances_index_before_deferring() {
    let mut state = top_and_bottom_state();

    advance_for_layer(&mut state, 0.4, 0.4, 1, true);

    assert_eq!(state.layer_index, 1);
    assert_eq!(state.pending_lift, None);
}

/// The print's first change-layer (entering index 0) still defers.
#[test]
fn first_layer_transition_still_defers_the_hop() {
    let mut state = top_and_bottom_state();

    advance_for_layer(&mut state, 0.2, 0.2, 0, true);

    assert_eq!(state.layer_index, 0);
    assert!(state.pending_lift.is_some());
}
