mod role_end;

#[tokio::test]
async fn task22o174_layer_cooling_matches_orca_slowdown_feedrate() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let lines = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .collect::<Vec<_>>();
    let layer = lines
        .iter()
        .position(|line| *line == "; Z_HEIGHT: 19.2")
        .unwrap();
    let inner_wall = lines[layer..]
        .iter()
        .position(|line| *line == "; FEATURE: Inner wall")
        .map(|offset| layer + offset)
        .unwrap();
    let feedrate = lines[inner_wall..]
        .iter()
        .find(|line| line.starts_with("G1 F"))
        .copied();

    assert_eq!(feedrate, Some("G1 F12997"));
}

#[test]
fn redundant_feed_only_move_is_dropped() {
    let mut output = b"G1 F9000\nG1 F9000\n".to_vec();
    let mut state = super::feedrate::State::new(
        super::feedrate::Config {
            enabled: false,
            target_time: 0.0,
            minimum_speed: 0.0,
            keep_outer_wall_speed: false,
            relative_e: true,
        },
        150.0,
    );

    super::feedrate::rewrite_layer(&mut output, 0, &mut state);

    // Upstream skips the complete zero-length F-collision line including
    // its newline (`CoolingBuffer.cpp:910-913`).
    assert_eq!(output, b"G1 F9000\n");
}

#[test]
fn fan_mover_suppresses_only_the_redundant_initial_zero_state() {
    assert!(!super::should_emit_initial_part_fan(0, true, true, 0));
    assert!(super::should_emit_initial_part_fan(0, true, false, 0));
    assert!(super::should_emit_initial_part_fan(0, true, true, 20));
    assert!(!super::should_emit_initial_part_fan(1, true, false, 0));
    assert!(!super::should_emit_initial_part_fan(0, false, false, 0));
}
