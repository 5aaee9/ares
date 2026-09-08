//! `AvoidCrossingPerimeters::init_layer` containment gate, before lazy routing initialization.
use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, JoinType, Point, offset_expolygons,
};
use crate::project_slice::gcode_emit::motion::state::AvoidCrossingGeometry;

const MITER_LIMIT: f64 = 2.0;

/// `init_layer` safe zone (`AvoidCrossingPerimeters.cpp:1324-1327`): the
/// layer slices inset by external_perimeter_width × coeff, trying
/// 0.6/0.5/0.45 until non-empty.
pub(super) fn build(
    geometry: &AvoidCrossingGeometry<'_>,
    scale: CoordinateScale,
) -> Result<Vec<crate::geometry::ExPolygon>, ClipperError> {
    for coeff in [0.6_f32, 0.5, 0.45] {
        let Some(inset) = scale.checked_scale(f64::from(geometry.external_perimeter_width * coeff))
        else {
            continue;
        };
        let offset = offset_expolygons(
            &geometry
                .layer_slices
                .iter()
                .map(|expolygon| (*expolygon).clone())
                .collect::<Vec<_>>(),
            -(inset as f32),
            JoinType::Miter,
            MITER_LIMIT,
        )?;
        if !offset.is_empty() {
            return Ok(offset);
        }
    }
    Ok(Vec::new())
}

/// Both endpoints inside the contour (outside every hole) and no contour
/// edge crossing the segment.
pub(super) fn contains(start: Point, end: Point, expolygon: &ExPolygon) -> bool {
    // Upstream `any_expolygon_contains` (AvoidCrossingPerimeters.cpp:
    // 716-736): with no grid-cell edge intersection along the line, the
    // test is `bbox.contains(a) && bbox.contains(b) &&
    // ex_polygon.contains(travel.a)` — ONLY the START point must lie
    // inside the polygon (both inside the bbox). A travel starting
    // inside and ending outside without crossing edges is SAFE.
    let contour = expolygon.contour();
    if !contour.contains(&start) {
        return false;
    }
    for hole in expolygon.holes() {
        if hole.contains(&start) {
            return false;
        }
    }
    let travel = crate::geometry::Line::new(start, end);
    let crossing = contour
        .lines()
        .into_iter()
        .chain(expolygon.holes().iter().flat_map(|hole| hole.lines()))
        .any(|edge| travel.intersection(edge).is_some());
    !crossing
}
