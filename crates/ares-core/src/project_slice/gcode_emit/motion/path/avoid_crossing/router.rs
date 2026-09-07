//! Detour routing along the boundary — a source-cited port of
//! `avoid_perimeters` / `avoid_perimeters_inner` / `simplify_travel` and
//! helpers (`GCode/AvoidCrossingPerimeters.cpp:437-705, 714-858`).

use crate::geometry::{ClipperError, Coord, EdgeGrid, Point};

use super::boundary::Boundary;

/// Upstream `#define SCALED_EPSILON scale_(EPSILON)` — 1e-4 mm in scaled
/// coordinates (SCALING_FACTOR 1e-6 → 100 units).
const SCALED_EPSILON: f64 = 100.0;

#[derive(Clone, Copy, Debug)]
struct Intersection {
    point: Point,
    contour: usize,
    segment: usize,
    #[expect(dead_code, reason = "kept for the upstream Intersection layout")]
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
    search_radius: f64,
) -> Result<(Vec<Point>, usize), ClipperError> {
    let mut start = start;
    let mut end = end;
    let mut intersections = collect_intersections(boundary, start, end)?;
    if intersections.is_empty() {
        // The inner-offset boundary may not touch a short travel; nudge the
        // endpoints toward the closest boundary lines and retry
        // (:560-584).
        let radius = 1.5 * search_radius;
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
        if let Ok(path) = std::env::var("ARES_DUMP_PRESIMPLIFY") {
            use std::io::Write;
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                let _ = write!(
                    file,
                    "PRE start={},{} end={},{} n={count} path=",
                    start.x(),
                    start.y(),
                    end.x(),
                    end.y()
                );
                for point in &result {
                    let _ = write!(file, "({},{})", point.point.x(), point.point.y());
                }
                let _ = writeln!(file);
            }
        }
        result = simplify_travel(boundary, &result);
    }
    if let Ok(path) = std::env::var("ARES_DUMP_ROUTE") {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = write!(
                file,
                "AP start={},{} end={},{} n={count} path=",
                start.x(),
                start.y(),
                end.x(),
                end.y()
            );
            for point in &result {
                let _ = write!(file, "({},{})", point.point.x(), point.point.y());
            }
            let _ = writeln!(file);
        }
    }
    Ok((result.into_iter().map(|point| point.point).collect(), count))
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

#[derive(Clone, Copy)]
struct ClosestLine {
    point: Point,
    contour: usize,
    segment: usize,
}

/// `get_closest_lines_in_radius` (:150-165) over the boundary grid.
fn closest_lines_in_radius(boundary: &Boundary, center: Point, radius: f64) -> Vec<ClosestLine> {
    let radius_coordinate = radius.round() as Coord;
    let query_min = Point::new(
        center.x().saturating_sub(radius_coordinate),
        center.y().saturating_sub(radius_coordinate),
    );
    let query_max = Point::new(
        center.x().saturating_add(radius_coordinate),
        center.y().saturating_add(radius_coordinate),
    );
    let mut lines = Vec::new();
    boundary
        .grid
        .visit_cells_intersecting_box(query_min, query_max, |_, _, edges| {
            for &edge in edges {
                let (segment_start, segment_end) = boundary.grid.segment(edge);
                let closest = project_on_segment(center, segment_start, segment_end);
                if distance(center, closest) <= radius {
                    lines.push(ClosestLine {
                        point: closest,
                        contour: edge.contour_index,
                        segment: edge.segment_index,
                    });
                }
            }
            true
        });
    lines.sort_by(|left, right| {
        distance(center, left.point)
            .partial_cmp(&distance(center, right.point))
            .expect("finite distances")
    });
    lines
}

fn project_on_segment(point: Point, start: Point, end: Point) -> Point {
    let segment = (
        end.x() as f64 - start.x() as f64,
        end.y() as f64 - start.y() as f64,
    );
    let from_start = (
        point.x() as f64 - start.x() as f64,
        point.y() as f64 - start.y() as f64,
    );
    let length_squared = segment.0 * segment.0 + segment.1 * segment.1;
    let parameter = if length_squared == 0.0 {
        0.0
    } else {
        ((from_start.0 * segment.0 + from_start.1 * segment.1) / length_squared).clamp(0.0, 1.0)
    };
    Point::new(
        (start.x() as f64 + parameter * segment.0) as Coord,
        (start.y() as f64 + parameter * segment.1) as Coord,
    )
}

/// `extend_for_closest_lines` (:171-290): when the offset swallowed the
/// intersections, splice closest-line hits at the travel endpoints.
fn extend_for_closest_lines(
    boundary: &Boundary,
    intersections: Vec<Intersection>,
    start: Point,
    end: Point,
    search_radius: f64,
) -> Vec<Intersection> {
    let start_lines = closest_lines_in_radius(boundary, start, search_radius);
    let end_lines = closest_lines_in_radius(boundary, end, search_radius);
    let distance_of = |line: &ClosestLine| -> f64 {
        let contour = boundary.contour(line.contour);
        boundary.lengths(line.contour)[line.segment] + distance(line.point, contour[line.segment])
    };
    // If both endpoints are close to the same boundary, the whole detour is
    // on one contour.
    let mut new_intersections = intersections;
    let start_shared = start_lines
        .first()
        .is_some_and(|line| end_lines.iter().any(|other| other.contour == line.contour));
    if start_shared {
        let start_line = start_lines.first().copied().expect("checked above");
        let end_line = end_lines
            .iter()
            .find(|line| line.contour == start_line.contour)
            .copied()
            .expect("checked above");
        return vec![
            intersection_of(&start_line, distance_of),
            intersection_of(&end_line, distance_of),
        ];
    }
    if !start_lines.is_empty() {
        let replacement = replacement_line(EndpointQuery {
            lines: &start_lines,
            current: new_intersections.first().copied(),
            close_to: start,
            reverse: true,
            intersections: &new_intersections,
            search_radius,
        });
        match (replacement, new_intersections.first_mut()) {
            (Some(line), Some(first)) => *first = intersection_of(&line, distance_of),
            (Some(line), None) => {
                new_intersections.insert(0, intersection_of(&line, distance_of));
            }
            (None, _) => {}
        }
    }
    if !end_lines.is_empty() {
        let replacement = replacement_line(EndpointQuery {
            lines: &end_lines,
            current: new_intersections.last().copied(),
            close_to: end,
            reverse: false,
            intersections: &new_intersections,
            search_radius,
        });
        match (replacement, new_intersections.last_mut()) {
            (Some(line), Some(last)) => *last = intersection_of(&line, distance_of),
            (Some(line), None) => {
                new_intersections.push(intersection_of(&line, distance_of));
            }
            (None, _) => {}
        }
    }
    new_intersections
}

fn intersection_of(line: &ClosestLine, distance_of: impl Fn(&ClosestLine) -> f64) -> Intersection {
    Intersection {
        point: line.point,
        contour: line.contour,
        segment: line.segment,
        distance: distance_of(line),
        do_not_remove: true,
    }
}

/// `get_closer` + `find_closest_line_with_same_boundary_idx` (:236-278):
/// a closest line nearer than the endpoint's existing intersection, else one
/// sharing a contour with any intersection (reverse scan on the start side),
/// else the closest line overall.
fn replacement_line(query: EndpointQuery<'_>) -> Option<ClosestLine> {
    let EndpointQuery {
        lines,
        current,
        close_to,
        reverse,
        intersections,
        search_radius,
    } = query;
    let first = lines.first().copied()?;
    let Some(current) = current else {
        return Some(first);
    };
    let closer = lines.iter().copied().find(|line| {
        line.contour == current.contour
            && distance(close_to, line.point) < distance(close_to, current.point)
            && distance(close_to, line.point) <= search_radius * search_radius
    });
    if closer.is_some() {
        return closer;
    }
    let shared_contour = shared_contour_line(lines, intersections, reverse);
    Some(shared_contour.unwrap_or(first))
}

struct EndpointQuery<'a> {
    lines: &'a [ClosestLine],
    current: Option<Intersection>,
    close_to: Point,
    reverse: bool,
    intersections: &'a [Intersection],
    search_radius: f64,
}

fn shared_contour_line(
    lines: &[ClosestLine],
    intersections: &[Intersection],
    reverse: bool,
) -> Option<ClosestLine> {
    let contours = lines
        .iter()
        .map(|line| line.contour)
        .collect::<std::collections::HashSet<_>>();
    let found = if reverse {
        intersections
            .iter()
            .rev()
            .find(|intersection| contours.contains(&intersection.contour))
    } else {
        intersections
            .iter()
            .find(|intersection| contours.contains(&intersection.contour))
    };
    let shared = found?;
    lines
        .iter()
        .copied()
        .find(|line| line.contour == shared.contour)
}

/// `get_shortest_direction` (:390-423): which way around the contour is
/// shorter between the two intersections.
fn shortest_direction_is_forward(
    boundary: &Boundary,
    first: Intersection,
    second: Intersection,
) -> bool {
    let contour = boundary.contour(first.contour);
    let lengths = boundary.lengths(first.contour);
    let total = lengths.last().copied().unwrap_or(0.0);
    let mut dist_first = lengths[first.segment] + distance(contour[first.segment], first.point);
    let mut dist_second = lengths[second.segment] + distance(contour[second.segment], second.point);
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
    forward -= distance(contour[first.segment], first.point);
    backward -= distance(
        contour[first.segment],
        contour[(first.segment + 1) % contour.len()],
    ) - distance(first.point, contour[(first.segment + 1) % contour.len()])
        + distance(first.point, contour[(first.segment + 1) % contour.len()])
        - distance(first.point, contour[(first.segment + 1) % contour.len()]);
    forward -= distance(
        contour[second.segment],
        contour[(second.segment + 1) % contour.len()],
    ) - distance(second.point, contour[(second.segment + 1) % contour.len()]);
    backward -= distance(second.point, contour[second.segment]);
    forward < backward
}

/// `get_polygon_vertex_offset` (:349-352): offset a contour vertex inward.
fn vertex_offset(boundary: &Boundary, contour: usize, point_index: usize) -> Point {
    let points = boundary.contour(contour);
    let middle = points[point_index];
    let left = previous_different(points, point_index, middle);
    let right = next_different(points, point_index, middle);
    let normal = three_points_inward_normal(left, middle, right);
    Point::new(
        (middle.x() as f64 + normal.0 * SCALED_EPSILON) as Coord,
        (middle.y() as f64 + normal.1 * SCALED_EPSILON) as Coord,
    )
}

/// `get_middle_point_offset` (:354-360): offset an intersection point inward
/// based on its neighbouring contour vertices.
fn middle_point_offset(
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
        (middle.x() as f64 + normal.0 * SCALED_EPSILON) as Coord,
        (middle.y() as f64 + normal.1 * SCALED_EPSILON) as Coord,
    )
}

fn previous_different(points: &[Point], index: usize, point: Point) -> Point {
    if points[index] != point {
        return points[index];
    }
    let mut index = (index + points.len() - 1) % points.len();
    while points[index] == point {
        index = (index + points.len() - 1) % points.len();
    }
    points[index]
}

fn next_different(points: &[Point], index: usize, point: Point) -> Point {
    if points[index] != point {
        return points[index];
    }
    let mut index = (index + 1) % points.len();
    while points[index] == point {
        index = (index + 1) % points.len();
    }
    points[index]
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
fn simplify_travel(boundary: &Boundary, travel: &[TravelPoint]) -> Vec<TravelPoint> {
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
