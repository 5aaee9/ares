//! `AvoidCrossingPerimeters::travel_to` (:1259–1264, :1285–1288): an empty
//! internal boundary yields the direct polyline. Captured last-layer Z10.4 travel.
use super::*;

fn slices() -> Vec<ExPolygon> {
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

fn top_surface() -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(4_842_513, 4_842_513),
            Point::new(-4_842_510, 4_842_513),
            Point::new(-4_842_510, -4_842_510),
            Point::new(4_842_513, -4_842_510),
        ]),
        Vec::new(),
    )
}

fn request(geometry: LayerGeometry<'_>) -> Request<'_> {
    Request {
        start: MotionPoint {
            x: 156.572_598,
            y: 171.735_383,
        },
        end: MotionPoint {
            x: 164.890_001,
            y: 179.890_001,
        },
        geometry,
        offset: (160.0, 175.0),
        inset: 0.284_932_5,
        after_skirt: false,
    }
}

#[test]
fn travel_to_empty_internal_boundary_after_top_subtraction_is_direct() {
    let slices = slices();
    let top = top_surface();
    let geometry = LayerGeometry {
        avoid_crossing: AvoidCrossingGeometry {
            external_perimeter_width: 0.22,
            layer_slices: &slices,
            perimeter_spacing: 0.189_955_74,
            top_surfaces: &[&top],
        },
        ..geometry()
    };
    assert!(matches!(
        super::super::boundary::Boundary::build(
            &geometry.avoid_crossing,
            geometry.scale,
            [
                Point::new(-3_427_402, -3_264_616),
                Point::new(4_890_001, 4_890_001)
            ],
        )
        .unwrap(),
        super::super::boundary::BuildResult::Empty
    ));
    let mut boundary = super::super::build_boundary(&geometry).unwrap();
    assert!(!boundary.safe_zone.is_empty());
    assert!(
        !boundary
            .safe_zone
            .iter()
            .any(|polygon| super::super::safe_zone::contains(
                Point::new(-3_427_402, -3_264_616),
                Point::new(4_890_001, 4_890_001),
                polygon,
            ))
    );
    for _ in 0..2 {
        // The interior-waypoint API leaves the start/end polyline to its caller.
        assert_eq!(
            super::super::route(request(geometry), Some(&mut boundary)),
            Some(Vec::new())
        );
    }
}

#[test]
fn travel_to_unavailable_internal_boundary_still_requests_rectangle_shell() {
    let slices = slices();
    let geometry = LayerGeometry {
        avoid_crossing: AvoidCrossingGeometry {
            external_perimeter_width: 0.22,
            layer_slices: &slices,
            perimeter_spacing: 0.0,
            top_surfaces: &[],
        },
        ..geometry()
    };
    let mut boundary = super::super::build_boundary(&geometry).unwrap();
    assert!(super::super::route(request(geometry), Some(&mut boundary)).is_none());
}
