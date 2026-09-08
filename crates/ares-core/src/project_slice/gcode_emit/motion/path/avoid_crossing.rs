//! Reduce-crossing-wall travel routing — a source-cited port of
//! `AvoidCrossingPerimeters::travel_to` (`GCode/AvoidCrossingPerimeters.cpp:
//! 1233-1312`) over the layer boundary.

mod boundary;
mod router;
mod safe_zone;

#[cfg(test)]
mod tests;

use crate::project_slice::gcode_emit::motion::arc;

/// `init_layer`'s safe zone and lazily initialized `m_internal` boundary.
#[derive(Clone)]
pub(in crate::project_slice::gcode_emit) struct Boundary {
    safe_zone: Vec<crate::geometry::ExPolygon>,
    internal: Option<boundary::Boundary>,
}
pub(super) mod rectangle;
pub(in crate::project_slice::gcode_emit) use rectangle::route as rectangle_route;

use super::super::state::LayerGeometry;

pub(super) struct Request<'a> {
    pub(super) start: arc::Point,
    pub(super) end: arc::Point,
    pub(super) geometry: LayerGeometry<'a>,
    pub(super) offset: (f64, f64),
    pub(super) inset: f64,
    pub(super) after_skirt: bool,
}

/// Initialize the layer safe zone; routing geometry is built only after a
/// travel fails containment (`AvoidCrossingPerimeters.cpp:1250-1258`).
pub(in crate::project_slice::gcode_emit) fn build_boundary(
    geometry: &LayerGeometry<'_>,
) -> Option<Boundary> {
    Some(Boundary {
        safe_zone: safe_zone::build(&geometry.avoid_crossing, geometry.scale).ok()?,
        internal: None,
    })
}

/// Route `start`→`end` along the boundary contours, mirroring
/// `AvoidCrossingPerimeters::travel_to`. Rebuild the cached internal boundary
/// when either endpoint leaves its original (pre-EdgeGrid expansion) bounds.
/// Return interior waypoints in G-code millimetres; the caller owns endpoints.
/// `None` still reaches the existing temporary rectangle shell.
pub(super) fn route(
    request: Request<'_>,
    boundary: Option<&mut Boundary>,
) -> Option<Vec<arc::Point>> {
    // Detour waypoints require the multi-point ramp emission branch of
    // `GCodeWriter::travel_to_xyz` (`GCode.cpp:7486-7505`); until it lands,
    // all routing goes through the rectangle shell.
    if !detour_emission_ready() {
        return None;
    }
    let Request {
        start,
        end,
        geometry,
        offset,
        inset: _,
        after_skirt: _,
    } = request;
    let boundary = boundary?;
    let scale = geometry.scale;
    let to_scaled = |point: arc::Point| -> Option<crate::geometry::Point> {
        Some(crate::geometry::Point::new(
            scale.checked_scale(point.x - offset.0)?,
            scale.checked_scale(point.y - offset.1)?,
        ))
    };
    let scaled_start = to_scaled(start)?;
    let scaled_end = to_scaled(end)?;
    // Travels fully inside the lslices safe zone never route
    // (`any_expolygon_contains(m_lslices_offset, ...)`,
    // `AvoidCrossingPerimeters.cpp:1255`).
    if boundary.safe_zone.is_empty()
        || boundary
            .safe_zone
            .iter()
            .any(|polygon| safe_zone::contains(scaled_start, scaled_end, polygon))
    {
        return Some(Vec::new());
    }
    if boundary
        .internal
        .as_ref()
        .is_none_or(|internal| !internal.contains(scaled_start) || !internal.contains(scaled_end))
    {
        boundary.internal = match boundary::Boundary::build(
            &geometry.avoid_crossing,
            scale,
            [scaled_start, scaled_end],
        )
        .ok()?
        {
            boundary::BuildResult::Unavailable => None,
            // `travel_to` (:1259–1264, :1285–1288) skips an empty internal
            // boundary and returns {start, end}, not a rectangle detour.
            boundary::BuildResult::Empty => {
                boundary.internal = None;
                return Some(Vec::new());
            }
            boundary::BuildResult::Ready(internal) => Some(internal),
        };
    }
    let boundary = boundary.internal.as_ref()?;
    let (path, _intersections) =
        router::avoid_perimeters(boundary, scaled_start, scaled_end).ok()?;
    let mut output = Vec::with_capacity(path.len());
    // Upstream restores the original endpoints after routing (which may nudge
    // them), then emits each interior point, even if formatting rounds two
    // distinct points to the same command. The caller appends the destination.
    // Polyline::append suppresses equal integer endpoints, yielding a singleton.
    let interior_count = path.len().saturating_sub(2);
    for point in path.into_iter().skip(1).take(interior_count) {
        output.push(arc::Point {
            x: scale.unscale(point.x()) + offset.0,
            y: scale.unscale(point.y()) + offset.1,
        });
    }
    Some(output)
}

fn detour_emission_ready() -> bool {
    true
}

pub(super) fn routing_active() -> bool {
    detour_emission_ready()
}
