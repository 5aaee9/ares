//! Closest boundary hits and endpoint extension (`AvoidCrossingPerimeters.cpp:150–294`).
use super::{Boundary, Coord, Intersection, Point, distance};

#[derive(Clone, Copy)]
pub(super) struct ClosestLine {
    pub(super) point: Point,
    contour: usize,
    segment: usize,
}

/// `get_closest_lines_in_radius` (:150-165) over the boundary grid.
pub(super) fn closest_lines_in_radius(
    boundary: &Boundary,
    center: Point,
    radius: f64,
) -> Vec<ClosestLine> {
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
pub(super) fn extend_for_closest_lines(
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
        if let Some(line) = get_closer(&start_lines, new_intersections[0], start, search_radius) {
            new_intersections[0] = intersection_of(&line, distance_of);
        } else {
            let line = shared_contour_line(&start_lines, &new_intersections, true)
                .unwrap_or(start_lines[0]);
            new_intersections.insert(0, intersection_of(&line, distance_of));
        }
    }
    if !end_lines.is_empty() {
        let last = new_intersections.len() - 1;
        if let Some(line) = get_closer(&end_lines, new_intersections[last], end, search_radius) {
            new_intersections[last] = intersection_of(&line, distance_of);
        } else {
            let line =
                shared_contour_line(&end_lines, &new_intersections, false).unwrap_or(end_lines[0]);
            new_intersections.push(intersection_of(&line, distance_of));
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

/// `get_closer` (:219–231): replacement requires the existing crossing to be
/// within the radius. Upstream casts coordinate differences to float before
/// squaring; failure means append/prepend, not replacement by another hit.
fn get_closer(
    lines: &[ClosestLine],
    current: Intersection,
    close_to: Point,
    search_radius: f64,
) -> Option<ClosestLine> {
    let squared_distance = |point: Point| {
        let dx = (close_to.x() - point.x()) as f32;
        let dy = (close_to.y() - point.y()) as f32;
        dx * dx + dy * dy
    };
    let old_distance = squared_distance(current.point);
    let radius = search_radius as f32;
    lines.iter().copied().find(|line| {
        line.contour == current.contour
            && old_distance <= radius * radius
            && squared_distance(line.point) < old_distance
    })
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
