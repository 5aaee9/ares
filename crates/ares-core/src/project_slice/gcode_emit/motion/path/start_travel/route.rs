//! `GCode::travel_to` routing and retraction-distance planning.
use super::super::avoid_crossing;
use crate::project_slice::gcode_emit::motion::{EmitState, LayerGeometry, arc};

/// Plan the travel route (detour waypoints plus the destination), routing
/// through the avoid-crossing boundary when armed (`GCode.cpp:7415-7434`).
pub(super) fn plan_route(
    state: &mut EmitState,
    geometry: &LayerGeometry<'_>,
    feature: &str,
    first_x: f64,
    first_y: f64,
) -> Vec<arc::Point> {
    // Upstream gates routing on `is_current_position_clear()`
    // (`GCode.cpp:7420`); the rectangle shell keeps its layer gate so the
    // dormant default matches the previously verified output.
    let routing = avoid_crossing::routing_active();
    let route_gate = if routing {
        state.positioned
    } else {
        state.layer_index > 0
    };
    // Upstream disables the avoid-crossing once after the first-layer
    // skirt (`disable_once`, `GCode.cpp:4448-4450`) so the travel to the
    // first object point is straight. The flag survives the wipe
    // re-plan within one travel (upstream resets it only after the
    // emitted travel, `reset_once_modifiers` `GCode.cpp:7431`).
    let avoid_disabled = state.avoid_crossing_disabled_once;
    // `GCode::travel_to` starts its polyline at last_pos(), not the writer's
    // inward-move endpoint (GCode.cpp:7356–7362).
    let cursor = crate::project_slice::gcode_emit::motion::local_cursor(state, *geometry);
    let start = arc::Point {
        x: geometry.scale.unscale(cursor.x()) + state.offset.0,
        y: geometry.scale.unscale(cursor.y()) + state.offset.1,
    };
    let after_skirt = false;
    let mut route = if state.options.reduce_crossing_wall
        && route_gate
        && !avoid_disabled
        && !matches!(feature, "Skirt" | "Brim")
    {
        if state.avoid_boundary.is_none() {
            state.avoid_boundary = avoid_crossing::build_boundary(geometry).map(std::rc::Rc::new);
        }
        avoid_crossing::route(
            avoid_crossing::Request {
                start,
                end: arc::Point {
                    x: first_x,
                    y: first_y,
                },
                geometry: *geometry,
                offset: state.offset,
                inset: state.options.crossing_boundary_inset,
                after_skirt,
            },
            state.avoid_boundary.as_mut().map(std::rc::Rc::make_mut),
        )
        .unwrap_or_else(|| {
            avoid_crossing::rectangle_route(avoid_crossing::Request {
                start,
                end: arc::Point {
                    x: first_x,
                    y: first_y,
                },
                geometry: *geometry,
                offset: state.offset,
                inset: state.options.crossing_boundary_inset,
                after_skirt,
            })
        })
    } else {
        Vec::new()
    };
    route.push(arc::Point {
        x: first_x,
        y: first_y,
    });
    route
}

/// Total polyline length from the current position through the route
/// (`Polyline::length` in `needs_retraction`, `GCode.cpp:7530`).
pub(super) fn polyline_length(x: f64, y: f64, route: &[arc::Point]) -> f64 {
    let mut length = 0.0;
    let mut current = (x, y);
    for point in route {
        length += (point.x - current.0).hypot(point.y - current.1);
        current = (point.x, point.y);
    }
    length
}
