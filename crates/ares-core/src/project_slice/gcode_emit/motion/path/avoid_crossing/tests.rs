mod boundary_parity;
mod empty_boundary;

use super::{Request, rectangle_route};
use crate::{
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::{
        gcode_emit::motion::{
            LayerGeometry, arc::Point as MotionPoint, state::AvoidCrossingGeometry,
        },
        region_slices::RegionSurface,
    },
};

fn geometry() -> LayerGeometry<'static> {
    let surface = RegionSurface::internal(ExPolygon::new(
        Polygon::new(vec![
            Point::new(-5_000_000, -5_000_000),
            Point::new(5_000_000, -5_000_000),
            Point::new(5_000_000, 5_000_000),
            Point::new(-5_000_000, 5_000_000),
        ]),
        Vec::new(),
    ));
    LayerGeometry {
        nearest_seam_penalties: None,
        staggered_inner: false,
        internal_surfaces: Box::leak(vec![surface].into_boxed_slice()),
        scale: CoordinateScale::Normal,
        previous_layer_boundary: None,
        avoid_crossing: AvoidCrossingGeometry {
            external_perimeter_width: 0.42,
            layer_slices: &[],
            perimeter_spacing: 0.0,
            top_surfaces: &[],
            chunk_slices: &[],
            chunk_perimeter_spacing: 0.0,
        },
    }
}

/// The rectangle shell keeps the routed-corner behavior for crossing travels
/// (`after_skirt` projection and excessive-detour clamps) that the
/// printers passing today depend on.
#[test]
fn rectangle_shell_routes_crossing_travels_along_inset_boundary() {
    let geometry = geometry();
    let inset = 0.610_62;
    let offset = (110.0, 110.0);

    assert_eq!(
        rectangle_route(Request {
            start: MotionPoint {
                x: 105.674,
                y: 117.189,
            },
            end: MotionPoint {
                x: 114.325,
                y: 114.325,
            },
            geometry,
            offset,
            inset,
            after_skirt: true,
            use_external: false,
        }),
        [MotionPoint {
            x: 114.389_38,
            y: 114.325,
        }]
    );
    assert_eq!(
        rectangle_route(Request {
            start: MotionPoint {
                x: 113.775,
                y: 114.739,
            },
            end: MotionPoint {
                x: 112.604,
                y: 113.918,
            },
            geometry,
            offset,
            inset,
            after_skirt: false,
            use_external: false,
        }),
        [
            MotionPoint {
                x: 113.775,
                y: 114.389_38,
            },
            MotionPoint {
                x: 112.604,
                y: 114.389_38,
            },
        ]
    );
    assert_eq!(
        rectangle_route(Request {
            start: MotionPoint {
                x: 121.29,
                y: 122.254,
            },
            end: MotionPoint {
                x: 120.187,
                y: 121.005,
            },
            geometry,
            offset: (117.5, 117.5),
            inset,
            after_skirt: false,
            use_external: false,
        }),
        [MotionPoint {
            x: 121.29,
            y: 121.889_38,
        }]
    );
}

/// A single square slice builds a boundary whose contour keeps the layer's
/// shape inset by the variable-width inner offset.
#[test]
fn boundary_builds_from_layer_slices() {
    let layer = Box::leak(
        vec![ExPolygon::new(
            Polygon::new(vec![
                Point::new(-4_000_000, -4_000_000),
                Point::new(4_000_000, -4_000_000),
                Point::new(4_000_000, 4_000_000),
                Point::new(-4_000_000, 4_000_000),
            ]),
            Vec::new(),
        )]
        .into_boxed_slice(),
    );
    let geometry = LayerGeometry {
        nearest_seam_penalties: None,
        staggered_inner: false,
        internal_surfaces: &[],
        scale: CoordinateScale::Normal,
        previous_layer_boundary: None,
        avoid_crossing: AvoidCrossingGeometry {
            external_perimeter_width: 0.42,
            layer_slices: layer,
            perimeter_spacing: 0.45,
            top_surfaces: &[],
            chunk_slices: &[],
            chunk_perimeter_spacing: 0.0,
        },
    };
    let super::boundary::BuildResult::Ready(boundary) = super::boundary::Boundary::build(
        &geometry.avoid_crossing,
        geometry.scale,
        [Point::new(0, 0); 2],
    )
    .unwrap() else {
        panic!("a square builds a boundary");
    };
    assert!(!boundary.contours.is_empty());
    let length: f64 = boundary
        .contours
        .iter()
        .map(|contour| {
            contour
                .windows(2)
                .map(|pair| {
                    (pair[1].x() as f64 - pair[0].x() as f64)
                        .hypot(pair[1].y() as f64 - pair[0].y() as f64)
                })
                .sum::<f64>()
        })
        .sum();
    // The 8 mm square inset by roughly 1.5 × spacing keeps most of its
    // perimeter (the variable offset shaves thin margins).
    assert!(length > 4.0 * 1_000_000.0, "perimeter {length}");
}
