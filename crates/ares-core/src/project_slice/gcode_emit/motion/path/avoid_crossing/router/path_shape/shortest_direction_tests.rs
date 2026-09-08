//! `AvoidCrossingPerimeters.cpp:390–420::get_shortest_direction`, pinned8500fcdc.
use super::*;
use crate::{
    geometry::{CoordinateScale, ExPolygon, Polygon},
    project_slice::gcode_emit::motion::{
        path::avoid_crossing::boundary::BuildResult, state::AvoidCrossingGeometry,
    },
};

fn captured_boundary() -> Boundary {
    let slices = [ExPolygon::new(
        Polygon::new(vec![
            Point::new(5_000_000, 5_000_000),
            Point::new(-5_000_000, 5_000_000),
            Point::new(-5_000_000, -5_000_000),
            Point::new(5_000_000, -5_000_000),
        ]),
        Vec::new(),
    )];
    let BuildResult::Ready(boundary) = Boundary::build(
        &AvoidCrossingGeometry {
            external_perimeter_width: 0.22,
            layer_slices: &slices,
            perimeter_spacing: 0.189_955_74,
            top_surfaces: &[],
        },
        CoordinateScale::Normal,
        [
            Point::new(-4_728_537, -3_848_256),
            Point::new(4_890_001, 4_890_001),
        ],
    )
    .unwrap() else {
        panic!("a square builds a boundary");
    };
    boundary
}

fn intersection(segment: usize, point: Point, distance: f64) -> Intersection {
    Intersection {
        contour: 0,
        segment,
        point,
        distance,
        do_not_remove: true,
    }
}

#[test]
fn get_shortest_direction_captured_segment_one_to_zero_subtracts_segment_ends() {
    let boundary = captured_boundary();
    // Qualified Z3.26 trace: F1733624 / B17126648, not F11163760 / B9430136.
    let first = intersection(1, Point::new(-4_715_068, -3_848_256), 17_993_460.0);
    let second = intersection(0, Point::new(4_715_068, 4_715_068), 0.0);
    assert!(shortest_direction_is_forward(&boundary, first, second));
    assert!(!shortest_direction_is_forward(&boundary, second, first));
}

#[test]
fn get_shortest_direction_captured_rust_one_unit_drift_still_goes_forward() {
    let boundary = captured_boundary();
    let first = intersection(1, Point::new(-4_715_068, -3_848_255), 17_993_459.0);
    let second = intersection(0, Point::new(4_715_068, 4_715_068), 0.0);
    assert!(shortest_direction_is_forward(&boundary, first, second));
}

#[test]
fn get_shortest_direction_equal_costs_choose_backward() {
    let boundary = captured_boundary();
    let first = intersection(0, Point::new(4_715_068, 4_715_068), 0.0);
    let second = intersection(2, Point::new(-4_715_068, -4_715_068), 18_860_272.0);
    assert!(!shortest_direction_is_forward(&boundary, first, second));
}
