//! Detour routing along the boundary — a source-cited port of
//! `avoid_perimeters` / `avoid_perimeters_inner` / `simplify_travel` and
//! helpers (`GCode/AvoidCrossingPerimeters.cpp:437-705, 714-858`).

use crate::geometry::{ClipperError, Coord, Point};

use super::boundary::Boundary;

mod closest_lines;
mod path_shape;

use closest_lines::{closest_lines_in_radius, extend_for_closest_lines};
use path_shape::{
    middle_point_offset, shortest_direction_is_forward, simplify_travel, vertex_offset,
};

/// Upstream `#define SCALED_EPSILON scale_(EPSILON)` — 1e-4 mm in scaled
/// coordinates (SCALING_FACTOR 1e-6 → 100 units).
const SCALED_EPSILON: f64 = 100.0;

#[derive(Clone, Copy, Debug)]
struct Intersection {
    point: Point,
    contour: usize,
    segment: usize,
    distance: f64,
    do_not_remove: bool,
}

#[derive(Clone, Copy, Debug)]
struct TravelPoint {
    point: Point,
    #[expect(dead_code, reason = "kept for the upstream TravelPoint layout")]
    contour: i32,
    do_not_remove: bool,
}

/// `avoid_perimeters` (:690-705): route `start`→`end` around the boundary
/// contours; returns the path (including both endpoints) and the number of
/// boundary intersections on the straight line.
pub(super) fn avoid_perimeters(
    boundary: &Boundary,
    start: Point,
    end: Point,
) -> Result<(Vec<Point>, usize), ClipperError> {
    let mut start = start;
    let mut end = end;
    let mut intersections = collect_intersections(boundary, start, end)?;
    if intersections.is_empty() {
        // The inner-offset boundary may not touch a short travel; nudge the
        // endpoints toward the closest boundary lines and retry
        // (:560-584).
        let radius = 1.5 * f64::from(boundary.scaled_spacing);
        let start_lines = closest_lines_in_radius(boundary, start, radius);
        let end_lines = closest_lines_in_radius(boundary, end, radius);
        if !(start_lines.is_empty() && end_lines.is_empty()) {
            let new_start = start_lines.first().map_or(start, |line| line.point);
            let new_end = end_lines.first().map_or(end, |line| line.point);
            let direction = (
                new_end.x() as f64 - new_start.x() as f64,
                new_end.y() as f64 - new_start.y() as f64,
            );
            let length = direction.0.hypot(direction.1);
            if length > 0.0 {
                let nudge = SCALED_EPSILON;
                let unit = (direction.0 / length, direction.1 / length);
                let nudged_start = Point::new(
                    (new_start.x() as f64 - unit.0 * nudge) as Coord,
                    (new_start.y() as f64 - unit.1 * nudge) as Coord,
                );
                let nudged_end = Point::new(
                    (new_end.x() as f64 + unit.0 * nudge) as Coord,
                    (new_end.y() as f64 + unit.1 * nudge) as Coord,
                );
                let retry = collect_intersections(boundary, nudged_start, nudged_end)?;
                (start, end, intersections) = nudged_retry(
                    (start, end, intersections),
                    (nudged_start, nudged_end, retry),
                );
            }
        }
    }
    if !intersections.is_empty() {
        let search_radius = f64::from(2.0 * boundary.scaled_spacing);
        intersections =
            extend_for_closest_lines(boundary, intersections, start, end, search_radius);
    }

    let mut result = vec![TravelPoint {
        point: start,
        contour: -1,
        do_not_remove: false,
    }];
    let mut index = 0;
    while index < intersections.len() {
        let first = intersections[index];
        let left = first.segment;
        let right = (first.segment + 1) % boundary.contour(first.contour).len();
        result.push(TravelPoint {
            point: middle_point_offset(boundary, first.contour, left, right, first.point),
            contour: first.contour as i32,
            do_not_remove: first.do_not_remove,
        });
        // The farthest later intersection on the same contour is the exit.
        let exit = intersections[index + 1..]
            .iter()
            .rposition(|intersection| intersection.contour == first.contour)
            .map(|offset| index + 1 + offset);
        if let Some(exit_index) = exit {
            let second = intersections[exit_index];
            let forward = shortest_direction_is_forward(boundary, first, second);
            let contour = boundary.contour(first.contour);
            let around = if forward {
                forward_vertices(first.segment, second.segment, contour.len())
            } else {
                backward_vertices(first.segment, second.segment, contour.len())
            };
            for vertex in around {
                result.push(TravelPoint {
                    point: vertex_offset(boundary, first.contour, vertex),
                    contour: first.contour as i32,
                    do_not_remove: false,
                });
            }
            let left = second.segment;
            let right = (second.segment + 1) % boundary.contour(second.contour).len();
            result.push(TravelPoint {
                point: middle_point_offset(boundary, second.contour, left, right, second.point),
                contour: second.contour as i32,
                do_not_remove: second.do_not_remove,
            });
            index = exit_index;
        }
        index += 1;
    }
    result.push(TravelPoint {
        point: end,
        contour: -1,
        do_not_remove: false,
    });

    let count = intersections.len();
    if count > 0 {
        result = simplify_travel(boundary, &result);
    }
    // `to_polyline` (:348–355) appends through `Polyline.hpp:59–65`:
    // only an integer-equal current last point prevents an append.
    let mut polyline = Vec::with_capacity(result.len());
    for travel_point in result {
        if polyline.last() != Some(&travel_point.point) {
            polyline.push(travel_point.point);
        }
    }
    Ok((polyline, count))
}

/// The contour vertices between the entry and exit segments, walking
/// forward (`line_idx + 1` per upstream) or backward from the entry.
fn forward_vertices(entry: usize, exit: usize, len: usize) -> Vec<usize> {
    let mut vertices = Vec::new();
    let mut line = entry;
    while line != exit {
        line = (line + 1) % len;
        vertices.push(line);
    }
    vertices
}

fn backward_vertices(entry: usize, exit: usize, len: usize) -> Vec<usize> {
    let mut vertices = Vec::new();
    let mut line = entry;
    while line != exit {
        vertices.push(line);
        line = if line == 0 { len - 1 } else { line - 1 };
    }
    vertices
}

fn nudged_retry(
    original: (Point, Point, Vec<Intersection>),
    retry: (Point, Point, Vec<Intersection>),
) -> (Point, Point, Vec<Intersection>) {
    if retry.2.is_empty() { original } else { retry }
}

fn collect_intersections(
    boundary: &Boundary,
    start: Point,
    end: Point,
) -> Result<Vec<Intersection>, ClipperError> {
    let mut raw = Vec::new();
    // Upstream dedups by (contour, segment) — an edge spanning several grid
    // cells is visited once (`AllIntersectionsVisitor`'s `intersection_set`,
    // `AvoidCrossingPerimeters.cpp:76-84`).
    let mut seen = std::collections::HashSet::new();
    boundary
        .grid
        .visit_cells_intersecting_line(start, end, |_, _, edges| {
            for &edge in edges {
                if !seen.insert((edge.contour_index, edge.segment_index)) {
                    continue;
                }
                let (segment_start, segment_end) = boundary.grid.segment(edge);
                if let Some(point) = segment_intersection(start, end, segment_start, segment_end) {
                    raw.push((edge.contour_index, edge.segment_index, point));
                }
            }
            true
        })?;
    let mut intersections = raw
        .into_iter()
        .map(|(contour, segment, point)| {
            let contour_points = boundary.contour(contour);
            let from_line_begin = distance(point, contour_points[segment]);
            let distance = boundary.lengths(contour)[segment] + from_line_begin;
            Intersection {
                point,
                contour,
                segment,
                distance,
                do_not_remove: false,
            }
        })
        .collect::<Vec<_>>();
    if !intersections.is_empty() {
        order_intersections(&mut intersections, start, end);
    }
    Ok(intersections)
}

fn order_intersections(intersections: &mut [Intersection], start: Point, end: Point) {
    let direction = (
        end.x() as f64 - start.x() as f64,
        end.y() as f64 - start.y() as f64,
    );
    intersections.sort_by(|left, right| {
        let along_left = (left.point.x() as f64 - right.point.x() as f64) * direction.0
            + (left.point.y() as f64 - right.point.y() as f64) * direction.1;
        along_left
            .partial_cmp(&0.0)
            .expect("intersection distances are finite")
    });
}

fn distance(first: Point, second: Point) -> f64 {
    let dx = second.x() as f64 - first.x() as f64;
    let dy = second.y() as f64 - first.y() as f64;
    dx.hypot(dy)
}

fn segment_intersection(
    line_start: Point,
    line_end: Point,
    segment_start: Point,
    segment_end: Point,
) -> Option<Point> {
    let line = (
        line_end.x() as f64 - line_start.x() as f64,
        line_end.y() as f64 - line_start.y() as f64,
    );
    let segment = (
        segment_end.x() as f64 - segment_start.x() as f64,
        segment_end.y() as f64 - segment_start.y() as f64,
    );
    let denominator = line.0 * segment.1 - line.1 * segment.0;
    if denominator == 0.0 {
        return None;
    }
    let delta = (
        segment_start.x() as f64 - line_start.x() as f64,
        segment_start.y() as f64 - line_start.y() as f64,
    );
    let line_parameter = (delta.0 * segment.1 - delta.1 * segment.0) / denominator;
    let segment_parameter = (delta.0 * line.1 - delta.1 * line.0) / denominator;
    if !(0.0..=1.0).contains(&line_parameter) || !(0.0..=1.0).contains(&segment_parameter) {
        return None;
    }
    Some(Point::new(
        (line_start.x() as f64 + line_parameter * line.0) as Coord,
        (line_start.y() as f64 + line_parameter * line.1) as Coord,
    ))
}
