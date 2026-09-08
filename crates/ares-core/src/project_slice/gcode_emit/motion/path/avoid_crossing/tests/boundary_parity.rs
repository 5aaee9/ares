//! Captured `get_boundary` / `avoid_perimeters_inner` output, Orca 2.4.2.
use super::*;

fn square() -> Vec<ExPolygon> {
    vec![ExPolygon::new(
        Polygon::new(vec![
            Point::new(5_000_000, 5_000_000),
            Point::new(-5_000_000, 5_000_000),
            Point::new(-5_000_000, -5_000_000),
            Point::new(5_000_000, -5_000_000),
        ]),
        Vec::new(),
    )]
}

fn crossing_geometry(slices: &[ExPolygon]) -> AvoidCrossingGeometry<'_> {
    AvoidCrossingGeometry {
        external_perimeter_width: 0.22,
        layer_slices: slices,
        perimeter_spacing: 0.189_955_74,
        top_surfaces: &[],
    }
}

#[test]
fn get_boundary_keeps_scaled_spacing_fraction_and_plain_union_contour() {
    let slices = square();
    let super::super::boundary::BuildResult::Ready(boundary) =
        super::super::boundary::Boundary::build(
            &crossing_geometry(&slices),
            CoordinateScale::Normal,
            [
                Point::new(-3_559_422, 4_728_540),
                Point::new(4_890_001, 4_890_001),
            ],
        )
        .unwrap()
    else {
        panic!("a square builds a boundary");
    };
    assert_eq!(boundary.scaled_spacing, 189_955.0);
    assert_eq!(
        boundary.contours,
        [vec![
            Point::new(4_715_068, 4_715_068),
            Point::new(-4_715_068, 4_715_068),
            Point::new(-4_715_068, -4_715_068),
            Point::new(4_715_068, -4_715_068),
        ]]
    );
}

#[test]
fn avoid_perimeters_inner_captured_outer_wall_approach() {
    assert_route(
        (-3_559_422, 4_728_540),
        (4_890_001, 4_890_001),
        &[(-3_559_522, 4_714_968), (4_714_998, 4_714_998)],
    );
}

#[test]
fn avoid_perimeters_inner_captured_retry_retains_backward_route() {
    assert_route(
        (4_890_001, 4_860_001),
        (4_814_021, -2_328_698),
        &[(4_714_998, 4_714_998), (4_714_968, -2_328_798)],
    );
}

fn assert_route(start: (i64, i64), end: (i64, i64), expected: &[(i64, i64)]) {
    let slices = square();
    let geometry = LayerGeometry {
        avoid_crossing: crossing_geometry(&slices),
        ..geometry()
    };
    let point = |(x, y)| MotionPoint {
        x: geometry.scale.unscale(x),
        y: geometry.scale.unscale(y),
    };
    let mut boundary = super::super::build_boundary(&geometry).unwrap();
    let route = super::super::route(
        Request {
            start: point(start),
            end: point(end),
            geometry,
            offset: (0.0, 0.0),
            inset: 0.0,
            after_skirt: false,
        },
        Some(&mut boundary),
    )
    .unwrap();
    assert_eq!(
        route,
        expected.iter().copied().map(point).collect::<Vec<_>>()
    );
}
