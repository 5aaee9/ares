use crate::geometry::{
    ClipperError, JoinType, Point, Polygon, offset_paths, union_even_odd_polygons_paths,
    union_polygons_paths,
};

use super::RawWallToolPathConfig;

pub(super) fn prepare(
    outline: &[Polygon],
    config: RawWallToolPathConfig,
) -> Result<Vec<Polygon>, ClipperError> {
    if config.inset_count == 0 {
        return Ok(Vec::new());
    }
    let normalized = outline
        .iter()
        .cloned()
        .filter_map(normalize_polygon)
        .collect::<Vec<_>>();
    if normalized.is_empty() {
        return Ok(Vec::new());
    }
    let allowed_distance = config.wall_maximum_deviation;
    let epsilon_offset = allowed_distance / 2 - 1;
    let contracted = offset_paths(&normalized, -(epsilon_offset as f32), JoinType::Miter, 3.0)?;
    let expanded = offset_paths(
        &contracted,
        (2 * epsilon_offset) as f32,
        JoinType::Miter,
        3.0,
    )?;
    let restored = offset_paths(&expanded, -(epsilon_offset as f32), JoinType::Miter, 3.0)?;
    let maximum_resolution = config.wall_maximum_resolution;
    let maximum_deviation = config.wall_maximum_deviation;
    let mut simplified = restored
        .into_iter()
        .filter_map(normalize_polygon)
        .filter_map(|polygon| {
            simplify_polygon(
                polygon,
                maximum_resolution * maximum_resolution,
                maximum_deviation * maximum_deviation,
                config.coordinate_scale.checked_scale(0.005).unwrap(),
            )
        })
        .filter_map(clean_degenerate_and_collinear)
        .collect::<Vec<_>>();
    simplified = union_even_odd_polygons_paths(&simplified)?
        .into_iter()
        .filter_map(clean_degenerate_and_collinear)
        .collect();
    let minimum_area = (config.outer_spacing / 2) as f64;
    Ok(union_polygons_paths(&simplified)?
        .into_iter()
        .filter_map(normalize_polygon)
        .filter(|polygon| polygon.area().abs() > minimum_area * minimum_area)
        .collect())
}

fn normalize_polygon(polygon: Polygon) -> Option<Polygon> {
    let mut points = polygon.into_points();
    points.dedup();
    while points.len() > 1 && points.first() == points.last() {
        points.pop();
    }
    (points.len() >= 3).then(|| Polygon::new(points))
}

fn clean_degenerate_and_collinear(polygon: Polygon) -> Option<Polygon> {
    let mut points = polygon.into_points();
    loop {
        if points.len() < 3 {
            return None;
        }
        let removable = (0..points.len()).find(|&index| {
            let previous = points[(index + points.len() - 1) % points.len()];
            let current = points[index];
            let next = points[(index + 1) % points.len()];
            let incoming = (current.x() - previous.x(), current.y() - previous.y());
            let outgoing = (next.x() - current.x(), next.y() - current.y());
            let incoming_length =
                ((incoming.0 as f64).powi(2) + (incoming.1 as f64).powi(2)).sqrt();
            let outgoing_length =
                ((outgoing.0 as f64).powi(2) + (outgoing.1 as f64).powi(2)).sqrt();
            incoming_length == 0.0
                || outgoing_length == 0.0
                || ((incoming.0 as f64 * outgoing.1 as f64 - incoming.1 as f64 * outgoing.0 as f64)
                    .abs()
                    / (incoming_length * outgoing_length))
                    <= 0.005_f64.sin()
        });
        let Some(index) = removable else {
            return Some(Polygon::new(points));
        };
        points.remove(index);
    }
}

fn simplify_polygon(
    polygon: Polygon,
    smallest_segment_squared: i64,
    allowed_error_squared: i64,
    tiny_distance: i64,
) -> Option<Polygon> {
    let points = polygon.into_points();
    if points.len() <= 3 {
        return (points.len() == 3).then(|| Polygon::new(points));
    }
    let tiny_squared = tiny_distance * tiny_distance;
    let mut state = SimplifyState {
        output: Vec::with_capacity(points.len()),
        previous: *points.last().unwrap(),
        previous_previous: points[points.len() - 2],
        accumulated_area: cross(*points.last().unwrap(), points[0]),
    };
    for point_index in 0..points.len() {
        let mut current = points[point_index];
        let next = if point_index + 1 < points.len() {
            points[point_index + 1]
        } else if state.output.len() > 1 {
            state.output[0]
        } else {
            points[0]
        };
        let removed_area_next = cross(current, next);
        state.accumulated_area += removed_area_next;
        let length_squared = squared_distance(current, state.previous);
        if length_squared < tiny_squared {
            continue;
        }
        let removed_area = state.accumulated_area + cross(next, state.previous);
        let base_squared = squared_distance(next, state.previous);
        if base_squared == 0 {
            continue;
        }
        let height_squared =
            (removed_area as f64 * removed_area as f64 / base_squared as f64) as i64;
        if height_squared <= tiny_squared
            && distance_to_infinite(current, state.previous, next) <= tiny_distance as f64
        {
            continue;
        }
        if length_squared < smallest_segment_squared && height_squared <= allowed_error_squared {
            let next_length_squared = squared_distance(current, next);
            if next_length_squared <= 4 * smallest_segment_squared {
                continue;
            }
            let Some(intersection) =
                infinite_intersection(state.previous_previous, state.previous, current, next)
            else {
                state.retain(current, removed_area_next);
                continue;
            };
            if distance_to_infinite(intersection, state.previous, current).powi(2)
                <= allowed_error_squared as f64
                && squared_distance(intersection, state.previous) <= smallest_segment_squared
                && squared_distance(intersection, next) <= smallest_segment_squared
            {
                current = intersection;
                state.replace_previous();
            }
        }
        state.retain(current, removed_area_next);
    }
    (state.output.len() >= 3).then(|| Polygon::new(state.output))
}

struct SimplifyState {
    output: Vec<Point>,
    accumulated_area: i64,
    previous_previous: Point,
    previous: Point,
}

impl SimplifyState {
    fn replace_previous(&mut self) {
        if self.output.pop().is_some() {
            self.previous = self.previous_previous;
        }
    }

    fn retain(&mut self, current: Point, removed_area_next: i64) {
        self.accumulated_area = removed_area_next;
        self.previous_previous = self.previous;
        self.previous = current;
        self.output.push(current);
    }
}

fn cross(left: Point, right: Point) -> i64 {
    left.x() * right.y() - left.y() * right.x()
}

fn squared_distance(left: Point, right: Point) -> i64 {
    let dx = left.x() - right.x();
    let dy = left.y() - right.y();
    dx * dx + dy * dy
}

fn distance_to_infinite(point: Point, start: Point, end: Point) -> f64 {
    let dx = (end.x() - start.x()) as f64;
    let dy = (end.y() - start.y()) as f64;
    let numerator =
        (dy * (point.x() - start.x()) as f64 - dx * (point.y() - start.y()) as f64).abs();
    numerator / (dx * dx + dy * dy).sqrt()
}

fn infinite_intersection(a: Point, b: Point, c: Point, d: Point) -> Option<Point> {
    let ab = ((b.x() - a.x()) as f64, (b.y() - a.y()) as f64);
    let cd = ((d.x() - c.x()) as f64, (d.y() - c.y()) as f64);
    let denominator = ab.0 * cd.1 - ab.1 * cd.0;
    if denominator.abs() < 1.0e-4 {
        return None;
    }
    let ac = ((c.x() - a.x()) as f64, (c.y() - a.y()) as f64);
    let t = (ac.0 * cd.1 - ac.1 * cd.0) / denominator;
    let x = a.x() as f64 + t * ab.0;
    let y = a.y() as f64 + t * ab.1;
    (x.is_finite() && y.is_finite()).then(|| Point::new(x as i64, y as i64))
}
