//! Arc traversal time attribution to the arc line's own `g1_times_cache` id
//! (`GCodeProcessor.cpp:574` cache push, `:1466` arc lookup,
//! `:771-810` `update` internal-line skip).
use super::ProcessorLimits;
use super::estimate::Estimate;
use super::motion::MotionState;

fn nonbinding_axis_limits() -> ProcessorLimits {
    ProcessorLimits {
        max_acceleration: [f64::MAX; 4],
        ..ProcessorLimits::default()
    }
}

#[test]
fn arc_traversal_time_is_attributed_to_the_arc_lines_own_marker() {
    // From (0,0): a Z hop (id 1), then a G3 half circle (radius 1) that
    // discretizes into 10 internal segments (ids 2..=11), then a G1 (id 12).
    // Each internal segment consumes its own g1 line id
    // (`GCodeProcessor.cpp:3868` `++m_g1_line_id` per discretized G1), and
    // `process_line_move(g1_lines_counter + internal_g1_lines_counter)`
    // (`GCodeProcessor.cpp:1466`) reads the 9th segment's cache entry at the
    // arc line itself; only the last segment's time lands on the next line.
    let lines = [
        "M204 S1000",
        "G1 Z0.2 F600",
        "G3 X0 Y2 I0 J1 F600",
        "G1 X1 F600",
    ]
    .map(str::to_owned);

    assert_eq!(
        MotionState::default().motions("G3 X0 Y2 I0 J1 F600").len(),
        10
    );
    let estimate = Estimate::from_lines(&lines, 0.0, nonbinding_axis_limits());

    let at_arc = estimate
        .elapsed_at(3)
        .expect("arc line exports the 9th segment's cumulative time");
    let after_arc = estimate
        .elapsed_at(4)
        .expect("trailing arc segment lands on the next motion line");

    assert!(
        at_arc > 0.8 * after_arc,
        "arc marker {at_arc} should carry most of the traversal time {after_arc}"
    );
    assert!(after_arc > at_arc);
}
