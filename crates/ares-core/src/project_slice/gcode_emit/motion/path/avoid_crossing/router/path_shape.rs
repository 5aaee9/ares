//! Contour detour geometry and simplification (`AvoidCrossingPerimeters.cpp:299–479`).
use crate::geometry::EdgeGrid;

#[cfg(test)]
mod shortest_direction_tests;
#[cfg(test)]
mod tests;

use super::{
    Boundary, Coord, Intersection, Point, SCALED_EPSILON, TravelPoint, segment_intersection,
};

/// `get_shortest_direction` (:390-423): which way around the contour is
/// shorter between the two intersections.
pub(super) fn shortest_direction_is_forward(
    boundary: &Boundary,
    first: Intersection,
    second: Intersection,
) -> bool {
    let contour = boundary.contour(first.contour);
    let lengths = boundary.lengths(first.contour);
    let total = *lengths.last().expect("closed contour") as f32;
    let mut dist_first = first.distance as f32;
    let mut dist_second = second.distance as f32;
    let mut reversed = false;
    if dist_first > dist_second {
        std::mem::swap(&mut dist_first, &mut dist_second);
        reversed = true;
    }
    let mut forward = dist_second - dist_first;
    let mut backward = dist_first + total - dist_second;
    if reversed {
        std::mem::swap(&mut forward, &mut backward);
    }
    // Upstream casts the integer difference before the float squared norm/sqrt.
    let norm = |end: Point, start: Point| {
        let dx = (end.x() - start.x()) as f32;
        let dy = (end.y() - start.y()) as f32;
        (dx * dx + dy * dy).sqrt()
    };
    forward -= norm(first.point, contour[first.segment]);
    backward -= norm(contour[(first.segment + 1) % contour.len()], first.point);
    forward -= norm(contour[(second.segment + 1) % contour.len()], second.point);
    backward -= norm(second.point, contour[second.segment]);
    forward < backward
}

/// `get_polygon_vertex_offset` (:335-338): cast displacement before integer addition.
pub(super) fn vertex_offset(boundary: &Boundary, contour: usize, point_index: usize) -> Point {
    let points = boundary.contour(contour);
    let middle = points[point_index];
    let left = previous_different(
        points,
        (point_index + points.len() - 1) % points.len(),
        middle,
    );
    let right = next_different(points, (point_index + 1) % points.len(), middle);
    let normal = three_points_inward_normal(left, middle, right);
    Point::new(
        middle.x() + (normal.0 * SCALED_EPSILON) as Coord,
        middle.y() + (normal.1 * SCALED_EPSILON) as Coord,
    )
}

/// `get_middle_point_offset` (:341-346): offset an intersection point inward
/// based on its neighbouring contour vertices.
pub(super) fn middle_point_offset(
    boundary: &Boundary,
    contour: usize,
    left_index: usize,
    right_index: usize,
    middle: Point,
) -> Point {
    let points = boundary.contour(contour);
    let left = previous_different(points, left_index, middle);
    let right = next_different(points, right_index, middle);
    let normal = three_points_inward_normal(left, middle, right);
    Point::new(
        middle.x() + (normal.0 * SCALED_EPSILON) as Coord,
        middle.y() + (normal.1 * SCALED_EPSILON) as Coord,
    )
}

/// `find_first_different_vertex<false>` (:299–315) starts at index+1 even
/// when scanning backward. Vertex normals pass the prior index (:325–330).
fn previous_different(points: &[Point], index: usize, point: Point) -> Point {
    if points[index] != point {
        return points[index];
    }
    let mut line_index = (index + 1) % points.len();
    while points[line_index] == point && line_index != index {
        line_index = (line_index + points.len() - 1) % points.len();
    }
    points[line_index]
}

/// `find_first_different_vertex<true>` (:299–315).
fn next_different(points: &[Point], index: usize, point: Point) -> Point {
    if points[index] != point {
        return points[index];
    }
    let mut line_index = (index + 1) % points.len();
    while points[line_index] == point && line_index != index {
        line_index = (line_index + 1) % points.len();
    }
    points[line_index]
}

/// `three_points_inward_normal` (:335-339).
fn three_points_inward_normal(left: Point, middle: Point, right: Point) -> (f64, f64) {
    let first = (
        -(middle.y() as f64 - left.y() as f64),
        middle.x() as f64 - left.x() as f64,
    );
    let second = (
        -(right.y() as f64 - middle.y() as f64),
        right.x() as f64 - middle.x() as f64,
    );
    let first_length = first.0.hypot(first.1);
    let second_length = second.0.hypot(second.1);
    let first = (first.0 / first_length, first.1 / first_length);
    let second = (second.0 / second_length, second.1 / second_length);
    let sum = (first.0 + second.0, first.1 + second.1);
    let length = sum.0.hypot(sum.1);
    (sum.0 / length, sum.1 / length)
}

/// `simplify_travel` (:437-479): drop path points whose removal does not
/// cross a boundary.
pub(super) fn simplify_travel(boundary: &Boundary, travel: &[TravelPoint]) -> Vec<TravelPoint> {
    let mut simplified = Vec::with_capacity(travel.len());
    simplified.push(travel[0]);
    let mut point_index = 1;
    while point_index < travel.len() {
        let current_point = travel[point_index - 1].point;
        let mut next = travel[point_index];
        if !next.do_not_remove {
            let furthest = furthest_skippable(boundary, travel, point_index, current_point);
            next = travel[furthest];
            point_index = furthest;
        }
        simplified.push(next);
        point_index += 1;
    }
    simplified
}

/// The furthest later point whose direct segment from `current_point`
/// crosses no boundary, stopping at `do_not_remove` markers (:445-466).
fn furthest_skippable(
    boundary: &Boundary,
    travel: &[TravelPoint],
    from: usize,
    current_point: Point,
) -> usize {
    let mut best = from;
    for (probe, candidate) in travel.iter().enumerate().skip(from + 1) {
        if candidate.do_not_remove {
            break;
        }
        if candidate.point == current_point {
            best = probe;
            continue;
        }
        if !crosses_boundary(&boundary.grid, current_point, candidate.point) {
            best = probe;
        }
    }
    best
}

fn crosses_boundary(grid: &EdgeGrid, start: Point, end: Point) -> bool {
    let mut crosses = false;
    let _ = grid.visit_cells_intersecting_line(start, end, |_, _, edges| {
        for &edge in edges {
            let (segment_start, segment_end) = grid.segment(edge);
            if segment_intersection(start, end, segment_start, segment_end).is_some() {
                crosses = true;
                return false;
            }
        }
        true
    });
    crosses
}
