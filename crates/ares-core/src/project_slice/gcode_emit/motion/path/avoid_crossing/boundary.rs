//! Avoid-crossing boundary construction — a source-cited port of
//! `GCode/AvoidCrossingPerimeters.cpp` (`inner_offset` :1014-1098,
//! `resample_polygon` :825-840, `init_boundary` :1197-1229,
//! `get_boundary` :1099-1134).

use crate::geometry::{
    ClipperError, Coord, CoordinateScale, EdgeGrid, ExPolygon, JoinType, Point, Polygon,
    difference_ex, offset_expolygons, offset_paths, union_expolygons,
};
use crate::project_slice::elephant_foot::distance::{
    DistanceThresholds, ResampledPoint, filtered_contour_distances,
};
use crate::project_slice::gcode_emit::motion::state::AvoidCrossingGeometry;

const SCALED_EPSILON: f64 = 1.0e-4;
const MITER_LIMIT: f64 = 2.0;

/// The routing boundary: the inner-offset slice union as contours, an edge
/// grid over them, and per-contour cumulative distances.
#[derive(Clone)]
pub(super) struct Boundary {
    pub(super) scaled_spacing: f32,
    pub(super) contours: Vec<Vec<Point>>,
    pub(super) grid: EdgeGrid,
    pub(super) contour_lengths: Vec<Vec<f64>>,
    bounds: (Point, Point),
}

#[expect(clippy::large_enum_variant, reason = "avoid a transient allocation")]
pub(super) enum BuildResult {
    Unavailable,
    Empty,
    Ready(Boundary),
}

impl Boundary {
    /// `get_boundary_external` + the first `init_boundary` overload
    /// (`AvoidCrossingPerimeters.cpp:1137-1189, 1187-1203`): every hole of
    /// every print object at this print z, made CCW and expanded by half
    /// the cross-object average perimeter spacing (miter join), then
    /// reversed so normals point outward; the bbox is padded only by
    /// `SCALED_EPSILON`, so travels clear of every hole stay outside and
    /// route straight.
    pub(in crate::project_slice::gcode_emit) fn build_external(
        chunk_slices: &[ExPolygon],
        perimeter_spacing_mm: f64,
        scale: CoordinateScale,
        endpoints: [Point; 2],
    ) -> Result<BuildResult, ClipperError> {
        let unit = |millimetres: f64| scale.checked_scale(millimetres);
        let Some(scaled_spacing) = unit(perimeter_spacing_mm) else {
            return Ok(BuildResult::Unavailable);
        };
        let scaled_spacing = scaled_spacing as f32;
        // CW holes reversed to CCW before the positive offset
        // (`polygons_reverse(holes_per_obj)`).
        let holes = chunk_slices
            .iter()
            .flat_map(|expolygon| expolygon.holes())
            .map(|hole| {
                let mut points = hole.points().to_vec();
                points.reverse();
                Polygon::new(points)
            })
            .collect::<Vec<_>>();
        if holes.is_empty() {
            return Ok(BuildResult::Empty);
        }
        let expanded = offset_paths(&holes, 0.5 * scaled_spacing, JoinType::Miter, 3.0)?;
        if expanded.is_empty() {
            return Ok(BuildResult::Empty);
        }
        // Reverse every contour so the router's inward vertex offsets
        // steer travels around the hole instead of into it.
        let contours = expanded
            .iter()
            .map(|polygon| {
                let mut points = polygon.points().to_vec();
                points.reverse();
                points
            })
            .collect::<Vec<_>>();
        let (mut min, mut max) = contours_bounds(&contours);
        for point in endpoints {
            min = Point::new(min.x().min(point.x()), min.y().min(point.y()));
            max = Point::new(max.x().max(point.x()), max.y().max(point.y()));
        }
        let epsilon = scale.checked_scale(SCALED_EPSILON).unwrap_or(100) as Coord;
        let padded_min = Point::new(
            min.x().saturating_sub(epsilon),
            min.y().saturating_sub(epsilon),
        );
        let padded_max = Point::new(
            max.x().saturating_add(epsilon),
            max.y().saturating_add(epsilon),
        );
        let grid_resolution = unit(1.0).unwrap_or(1_000_000);
        let grid = EdgeGrid::new_from_contours(
            contours.iter().map(|contour| contour.as_slice()),
            padded_min,
            padded_max,
            grid_resolution,
        )?;
        let contour_lengths = contours
            .iter()
            .map(|contour| cumulative_distances(contour))
            .collect();
        Ok(BuildResult::Ready(Boundary {
            scaled_spacing,
            contours,
            grid,
            contour_lengths,
            bounds: (padded_min, padded_max),
        }))
    }

    pub(super) fn contour(&self, index: usize) -> &[Point] {
        &self.contours[index]
    }

    pub(super) fn lengths(&self, index: usize) -> &[f64] {
        &self.contour_lengths[index]
    }

    pub(super) fn contains(&self, point: Point) -> bool {
        let (min, max) = self.bounds;
        point.x() >= min.x() && point.y() >= min.y() && point.x() <= max.x() && point.y() <= max.y()
    }

    /// `get_boundary` + `init_boundary`: `union_ex(inner_offset(lslices,
    /// 1.5 * perimeter_spacing))`, minus an inset of the top fill surfaces,
    /// gridded at 1 mm cells.
    pub(in crate::project_slice::gcode_emit) fn build(
        geometry: &AvoidCrossingGeometry<'_>,
        scale: CoordinateScale,
        endpoints: [Point; 2],
    ) -> Result<BuildResult, ClipperError> {
        if geometry.layer_slices.is_empty() || geometry.perimeter_spacing <= 0.0 {
            return Ok(BuildResult::Unavailable);
        }
        let unit = |millimetres: f64| scale.checked_scale(millimetres);
        // `Flow::scaled_spacing` truncates before `get_perimeter_spacing`
        // converts to float; retain scaled units for all boundary radii.
        let Some(scaled_spacing) = unit(f64::from(geometry.perimeter_spacing)) else {
            return Ok(BuildResult::Unavailable);
        };
        let scaled_spacing = scaled_spacing as f32;
        let offset_dis = 1.5 * f64::from(scaled_spacing);
        let mut boundary = inner_offset(geometry.layer_slices, offset_dis, scale)?;
        if !geometry.top_surfaces.is_empty() {
            // perimeter_offset = spacing / 2; the diff insets the top
            // surfaces by 1.2 * perimeter_offset.
            let inset_by = 0.6 * f64::from(scaled_spacing);
            let inset = offset_expolygons(
                &geometry
                    .top_surfaces
                    .iter()
                    .map(|expolygon| (*expolygon).clone())
                    .collect::<Vec<_>>(),
                -(inset_by as f32),
                JoinType::Round,
                MITER_LIMIT,
            )?;
            boundary = difference_ex(&boundary, &inset)?;
        }
        if boundary.is_empty() {
            return Ok(BuildResult::Empty);
        }
        let contours = boundary
            .iter()
            .flat_map(|expolygon| {
                std::iter::once(expolygon.contour())
                    .chain(expolygon.holes())
                    .map(|polygon| polygon.points().to_vec())
            })
            .collect::<Vec<_>>();
        let (mut min, mut max) = contours_bounds(&contours);
        for point in endpoints {
            min = Point::new(min.x().min(point.x()), min.y().min(point.y()));
            max = Point::new(max.x().max(point.x()), max.y().max(point.y()));
        }
        // `init_boundary(boundary, polygons, merge_points)` pads the bounds by
        // the bbox radius so travel endpoints outside the contours stay in
        // the grid (`AvoidCrossingPerimeters.cpp:1216-1229`).
        let radius =
            (((max.x() - min.x()) as f64).hypot((max.y() - min.y()) as f64) / 2.0) as Coord;
        let padded_min = Point::new(
            min.x().saturating_sub(radius),
            min.y().saturating_sub(radius),
        );
        let padded_max = Point::new(
            max.x().saturating_add(radius),
            max.y().saturating_add(radius),
        );
        let grid_resolution = unit(1.0).unwrap_or(1_000_000);
        let grid = EdgeGrid::new_from_contours(
            contours.iter().map(|contour| contour.as_slice()),
            padded_min,
            padded_max,
            grid_resolution,
        )?;
        let contour_lengths = contours
            .iter()
            .map(|contour| cumulative_distances(contour))
            .collect();
        Ok(BuildResult::Ready(Boundary {
            scaled_spacing,
            contours,
            grid,
            contour_lengths,
            bounds: (padded_min, padded_max),
        }))
    }
}

fn cumulative_distances(contour: &[Point]) -> Vec<f64> {
    let mut lengths = Vec::with_capacity(contour.len() + 1);
    lengths.push(0.0);
    let mut total = 0.0;
    for pair in contour.windows(2) {
        total += distance(pair[0], pair[1]);
        lengths.push(total);
    }
    if let Some(&last) = contour.last() {
        total += distance(last, contour[0]);
        lengths.push(total);
    }
    lengths
}

fn distance(first: Point, second: Point) -> f64 {
    let dx = second.x() as f64 - first.x() as f64;
    let dy = second.y() as f64 - first.y() as f64;
    dx.hypot(dy)
}

fn contours_bounds(contours: &[Vec<Point>]) -> (Point, Point) {
    let first = contours
        .iter()
        .flatten()
        .copied()
        .next()
        .expect("nonempty contours");
    let mut min = first;
    let mut max = first;
    for &point in contours.iter().flatten() {
        min = Point::new(min.x().min(point.x()), min.y().min(point.y()));
        max = Point::new(max.x().max(point.x()), max.y().max(point.y()));
    }
    (min, max)
}

/// `resample_polygon` (`AvoidCrossingPerimeters.cpp:825-840`): inserts points
/// at `dist_from_vertex` from each vertex and fills longer gaps.
fn resample_polygon(
    polygon: &[Point],
    dist_from_vertex: f64,
    max_allowed_distance: f64,
) -> Result<Vec<Point>, ClipperError> {
    let coordinate = |value: f64| -> Result<Coord, ClipperError> { Ok(value.round() as Coord) };
    let mut resampled = Vec::with_capacity(3 * polygon.len());
    for (point_index, &point) in polygon.iter().enumerate() {
        resampled.push(point);
        let next = polygon[(point_index + 1) % polygon.len()];
        let line = (
            next.x() as f64 - point.x() as f64,
            next.y() as f64 - point.y() as f64,
        );
        let line_length = line.0.hypot(line.1);
        if line_length == 0.0 {
            continue;
        }
        let offset = (
            line.0 / line_length * dist_from_vertex,
            line.1 / line_length * dist_from_vertex,
        );
        let offset_coordinate = (coordinate(offset.0)?, coordinate(offset.1)?);
        let moves = offset_coordinate.0 != 0 || offset_coordinate.1 != 0;
        if line_length > 2.0 * dist_from_vertex && moves {
            resampled.push(Point::new(
                point.x() + offset_coordinate.0,
                point.y() + offset_coordinate.1,
            ));
            let middle = (
                next.x() as f64 - point.x() as f64 - 2.0 * offset.0,
                next.y() as f64 - point.y() as f64 - 2.0 * offset.1,
            );
            let middle_length = middle.0.hypot(middle.1);
            if middle_length > max_allowed_distance {
                let parts = (middle_length / max_allowed_distance).ceil() as usize;
                let anchor = resampled.last().copied().expect("just pushed a point");
                resampled.extend(fill_middle(anchor, middle, parts, coordinate)?);
            }
            resampled.push(Point::new(
                next.x() - offset_coordinate.0,
                next.y() - offset_coordinate.1,
            ));
        }
    }
    Ok(resampled)
}

fn fill_middle(
    anchor: Point,
    middle: (f64, f64),
    parts: usize,
    coordinate: impl Fn(f64) -> Result<Coord, ClipperError>,
) -> Result<Vec<Point>, ClipperError> {
    let mut points = Vec::with_capacity(parts.saturating_sub(1));
    for part in 1..parts {
        let parameter = part as f64 / parts as f64;
        points.push(Point::new(
            coordinate(anchor.x() as f64 + middle.0 * parameter)?,
            coordinate(anchor.y() as f64 + middle.1 * parameter)?,
        ));
    }
    Ok(points)
}

/// `inner_offset` (`AvoidCrossingPerimeters.cpp:1014-1098`): a variable-width
/// inward offset that keeps thin regions connected instead of splitting them.
fn inner_offset(
    expolygons: &[ExPolygon],
    offset_dis: f64,
    scale: CoordinateScale,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let hole_probe = scale.checked_scale(0.1).unwrap_or(100_000) as f32;
    let mut result = Vec::with_capacity(expolygons.len());
    for expolygon in expolygons {
        // Remove too small holes: a 0.1 mm outward offset collapses them.
        let mut ex_poly = expolygon.clone();
        let holes = ex_poly
            .holes()
            .iter()
            .filter(|hole| {
                !offset_paths(&[(*hole).clone()], hole_probe, JoinType::Round, MITER_LIMIT)
                    .unwrap_or_default()
                    .is_empty()
            })
            .cloned()
            .collect::<Vec<_>>();
        if holes.len() != ex_poly.holes().len() {
            ex_poly = ExPolygon::new(ex_poly.contour().clone(), holes);
        }
        let resample = offset_dis / 2.0;
        let tolerance = scale.checked_scale(0.5).unwrap_or(500) as f64;
        let mut ex_poly = resample_expolygon(&ex_poly, resample, tolerance)?;
        // Filter out expolygons smaller than 0.1 mm^2 (bbox estimate).
        let (min, max) = polygon_bounds(ex_poly.contour());
        let filter_extent = scale.checked_scale(0.1).unwrap_or(100_000);
        let width = max.x().saturating_sub(min.x());
        let height = max.y().saturating_sub(min.y());
        if width < filter_extent && height < filter_extent {
            continue;
        }
        let widths = [
            offset_dis / 2.0,
            offset_dis,
            2.0 * offset_dis + SCALED_EPSILON,
        ];
        'widths: for &min_contour_width in &widths {
            let search_radius = 2.0 * (offset_dis + min_contour_width);
            let contours = std::iter::once(ex_poly.contour().points())
                .chain(ex_poly.holes().iter().map(|hole| hole.points()))
                .map(|points| points.to_vec())
                .collect::<Vec<_>>();
            let resolution = (0.7 * search_radius) as Coord;
            if resolution <= 0 {
                continue;
            }
            let (min, max) = contours_bounds(&contours);
            let Ok(grid) = EdgeGrid::new_from_contours(
                contours.iter().map(|contour| contour.as_slice()),
                min,
                max,
                resolution,
            ) else {
                continue;
            };
            let thresholds = DistanceThresholds::new(offset_dis, search_radius, SCALED_EPSILON);
            let mut offsets = Vec::with_capacity(ex_poly.holes().len() + 1);
            for (contour_index, contour) in contours.iter().enumerate() {
                let parameters = own_parameters(contour);
                let mut distances = filtered_contour_distances(
                    &grid,
                    contour_index,
                    contour,
                    &parameters,
                    thresholds,
                )?;
                map_distances(&mut distances, min_contour_width, offset_dis);
                offsets.push(distances);
            }
            let offset_ex_poly =
                crate::geometry::variable_offset_inner_ex(&ex_poly, &offsets, MITER_LIMIT)?;
            // `variable_offset_inner_ex` may split thin artefacts away; keep
            // the largest part per the upstream acceptance cascade.
            if offset_ex_poly.len() == 1 {
                ex_poly = offset_ex_poly.into_iter().next().expect("single result");
                break 'widths;
            } else if offset_ex_poly.len() > 1 {
                let (best, _) = offset_ex_poly
                    .iter()
                    .map(|candidate| candidate.area())
                    .enumerate()
                    .max_by(|left, right| left.1.total_cmp(&right.1))
                    .expect("nonempty result");
                ex_poly = offset_ex_poly.into_iter().nth(best).expect("best exists");
                break 'widths;
            }
        }
        result.push(ex_poly);
    }
    union_expolygons(&result, &[])
}

fn resample_expolygon(
    expolygon: &ExPolygon,
    dist_from_vertex: f64,
    max_allowed: f64,
) -> Result<ExPolygon, ClipperError> {
    let contour = Polygon::new(resample_polygon(
        expolygon.contour().points(),
        dist_from_vertex,
        max_allowed,
    )?);
    let mut holes = Vec::with_capacity(expolygon.holes().len());
    for hole in expolygon.holes() {
        let resampled = resample_polygon(hole.points(), dist_from_vertex, max_allowed)?;
        holes.push(Polygon::new(resampled));
    }
    Ok(ExPolygon::new(contour, holes))
}

/// Parameters for `filtered_contour_distances` when the grid contours are the
/// query contours themselves: every point anchors its own segment with the
/// cumulative length as curve parameter.
fn own_parameters(contour: &[Point]) -> Vec<ResampledPoint> {
    let mut parameters = Vec::with_capacity(contour.len());
    let mut total = 0.0;
    for (index, &point) in contour.iter().enumerate() {
        parameters.push(ResampledPoint {
            source_index: index,
            interpolated: false,
            step_length: 0.0,
            curve_parameter: total,
        });
        let next = contour[(index + 1) % contour.len()];
        total += distance(point, next);
    }
    parameters
}

/// The upstream distance-to-delta mapping (`AvoidCrossingPerimeters.cpp:
/// 1051-1061`), identical to the elephant-foot compensation mapping.
fn map_distances(distances: &mut [f32], min_contour_width: f64, offset_dis: f64) {
    let compensated_width = min_contour_width + 2.0 * offset_dis;
    for distance in distances {
        if f64::from(*distance) < min_contour_width {
            *distance = 0.0;
        } else if f64::from(*distance) > compensated_width {
            *distance = -(offset_dis as f32);
        } else {
            *distance = -(*distance - min_contour_width as f32) / 2.0;
        }
    }
}

fn polygon_bounds(contour: &Polygon) -> (Point, Point) {
    let first = contour.points()[0];
    let mut min = first;
    let mut max = first;
    for &point in contour.points().iter().skip(1) {
        min = Point::new(min.x().min(point.x()), min.y().min(point.y()));
        max = Point::new(max.x().max(point.x()), max.y().max(point.y()));
    }
    (min, max)
}
