//! `AvoidCrossingPerimeters.cpp:299–331` neighbor selection, not coordinate deduplication.
use super::*;

#[test]
fn find_first_different_vertex_backward_probes_next_before_scanning_back() {
    let points = [Point::new(0, 10), Point::new(0, 0), Point::new(10, 0)];
    assert_eq!(previous_different(&points, 1, points[1]), points[2]);
}

#[test]
fn find_first_different_vertex_backward_initial_probe_wraps() {
    let points = [Point::new(10, 0), Point::new(0, 10), Point::new(0, 0)];
    assert_eq!(previous_different(&points, 2, points[2]), points[0]);
}

#[test]
fn find_first_different_vertex_returns_different_passed_vertex_in_both_directions() {
    let points = [Point::new(0, 10), Point::new(0, 0), Point::new(10, 0)];
    assert_eq!(previous_different(&points, 0, points[1]), points[0]);
    assert_eq!(next_different(&points, 2, points[1]), points[2]);
}

#[test]
fn find_first_different_vertex_forward_skips_equal_run_and_wraps() {
    let point = Point::new(0, 0);
    let points = [point, Point::new(10, 0), point, point];
    assert_eq!(next_different(&points, 2, point), points[1]);
}
