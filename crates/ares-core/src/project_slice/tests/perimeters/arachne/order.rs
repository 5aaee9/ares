// `process_arachne` wall-line ordering tests (OrcaSlicer
// `PerimeterGenerator.cpp:2269-2440`).

use crate::{
    arachne::{ExtrusionJunction, ExtrusionLine},
    geometry::Point,
    project_slice::perimeters::arachne::order::order_walls,
};

/// Clockwise square so `ExtrusionLine::is_contour` reports a contour
/// (`ExtrusionLine.cpp:263-270`).
fn closed_square(inset_index: usize, offset: i64) -> ExtrusionLine {
    let mut line = ExtrusionLine::new(inset_index, false);
    line.is_closed = true;
    let side = 1_000_000 + offset;
    for (x, y) in [
        (offset, offset),
        (offset, side),
        (side, side),
        (side, offset),
        (offset, offset),
    ] {
        line.push(ExtrusionJunction::new(
            Point::new(x, y),
            420_000,
            inset_index,
        ));
    }
    line
}

fn open_segment(inset_index: usize, x: i64, y: i64) -> ExtrusionLine {
    let mut line = ExtrusionLine::new(inset_index, false);
    line.push(ExtrusionJunction::new(
        Point::new(x, y),
        420_000,
        inset_index,
    ));
    line.push(ExtrusionJunction::new(
        Point::new(x + 100_000, y),
        420_000,
        inset_index,
    ));
    line
}

#[test]
fn task22w1_orders_walls_inner_to_outer_by_default() {
    let perimeters = vec![
        vec![closed_square(0, 0)],
        vec![closed_square(1, -400_000)],
        vec![closed_square(2, -800_000)],
    ];

    let ordered = order_walls(&perimeters, false);

    let insets = ordered
        .iter()
        .map(|extrusion| extrusion.extrusion.inset_index)
        .collect::<Vec<_>>();
    // `PerimeterGenerator.cpp:2269-2271`: start at the innermost index.
    assert_eq!(insets, [2, 1, 0]);
}

#[test]
fn task22w2_orders_walls_outer_to_inner_when_outer_first() {
    let perimeters = vec![vec![closed_square(0, 0)], vec![closed_square(1, -400_000)]];

    let ordered = order_walls(&perimeters, true);

    let insets = ordered
        .iter()
        .map(|extrusion| extrusion.extrusion.inset_index)
        .collect::<Vec<_>>();
    assert_eq!(insets, [0, 1]);
}

#[test]
fn task22w3_marks_contours_with_the_source_is_contour_flag() {
    let perimeters = vec![vec![closed_square(0, 0)]];

    let ordered = order_walls(&perimeters, false);

    assert!(ordered[0].is_contour);
}

#[test]
fn task22w4_prefers_closed_walls_over_open_lines_at_equal_distance() {
    let perimeters = vec![vec![closed_square(0, 0), open_segment(0, 2_000_000, 0)]];

    let ordered = order_walls(&perimeters, false);

    // `PerimeterGenerator.cpp:2312-2315`: closed walls sort first.
    assert!(ordered[0].extrusion.is_closed);
    assert!(!ordered[1].extrusion.is_closed);
}

#[test]
fn task22w5_constraints_take_precedence_over_the_closed_first_preference() {
    // Nested insets 0.4mm apart: `getRegionOrder` blocks the outer wall until
    // its inner neighbor finishes (`WallToolPaths.cpp:809-903`), so the
    // closed-first sort (`PerimeterGenerator.cpp:2312-2315`) cannot promote
    // the closed outer wall over the open inner line.
    let perimeters = vec![
        vec![closed_square(0, 0)],
        vec![open_segment(1, -400_000, -400_000)],
    ];

    let ordered = order_walls(&perimeters, false);

    assert_eq!(
        ordered
            .iter()
            .map(|extrusion| extrusion.extrusion.inset_index)
            .collect::<Vec<_>>(),
        [1, 0]
    );
}
