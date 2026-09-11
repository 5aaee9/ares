use crate::geometry::{Line, Point};

pub(super) fn closed_length(points: &[Point]) -> f64 {
    (0..points.len())
        .map(|index| Line::new(points[index], points[(index + 1) % points.len()]).length())
        .sum()
}

/// `Geometry/ConvexHull.cpp:11-43`, counter-clockwise from the smallest point.
pub(in crate::project_slice::gcode_emit) fn convex_hull(points: &[Point]) -> Vec<Point> {
    let mut sorted = points.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    let count = sorted.len();
    if count < 3 {
        return Vec::new();
    }
    let ccw = |a: Point, b: Point, c: Point| {
        let cross = (i128::from(b.x()) - i128::from(a.x()))
            * (i128::from(c.y()) - i128::from(a.y()))
            - (i128::from(b.y()) - i128::from(a.y())) * (i128::from(c.x()) - i128::from(a.x()));
        cross > 0
    };
    let mut hull = Vec::with_capacity(2 * count);
    for point in &sorted {
        while hull.len() >= 2 && !ccw(hull[hull.len() - 2], hull[hull.len() - 1], *point) {
            hull.pop();
        }
        hull.push(*point);
    }
    let lower_end = hull.len() + 1;
    for point in sorted[..count - 1].iter().rev() {
        while hull.len() >= lower_end && !ccw(hull[hull.len() - 2], hull[hull.len() - 1], *point) {
            hull.pop();
        }
        hull.push(*point);
    }
    hull.pop();
    hull
}

/// `GCode.cpp:4334-4359`, target on the loop bounding-circle radius.
pub(super) fn find_start_point(points: &[Point], start_angle_deg: f64) -> Point {
    let mut min_x = i64::MAX;
    let mut max_x = i64::MIN;
    let mut min_y = i64::MAX;
    let mut max_y = i64::MIN;
    for point in points {
        let x = point.x();
        let y = point.y();
        if x < min_x {
            min_x = x;
        } else if x > max_x {
            max_x = x;
        }
        if y < min_y {
            min_y = y;
        } else if y > max_y {
            max_y = y;
        }
    }
    // `Point center((min + max) / 2.)` constructs Point from doubles —
    // the `Point(double, double)` ctor applies `std::round` (half away from
    // zero), not truncation (`Point.hpp:197`). The same rounding applies to
    // the returned angle target.
    let center = Point::new(
        ((min_x + max_x) as f64 / 2.0).round() as i64,
        ((min_y + max_y) as f64 / 2.0).round() as i64,
    );
    let center_x = center.x() as f64;
    let center_y = center.y() as f64;
    let radius = ((center_x - min_x as f64).powi(2) + (center_y - min_y as f64).powi(2)).sqrt();
    // Upstream: `deg = start_angle * PI / 180` (multiply, then divide) —
    // not `to_radians()`'s single multiply by the precomputed constant.
    let radians = start_angle_deg * std::f64::consts::PI / 180.0;
    Point::new(
        (center_x + radius * radians.cos()).round() as i64,
        (center_y + radius * radians.sin()).round() as i64,
    )
}

/// `ExtrusionLoop::split_at` with `Point::projection_onto` endpoint clamping.
pub(in crate::project_slice::gcode_emit) fn split_at_nearest(
    points: &[Point],
    target: Point,
) -> Vec<Point> {
    split_at_nearest_for_brim(points, target)
}

pub(in crate::project_slice::gcode_emit) fn split_at_nearest_for_brim(
    points: &[Point],
    target: Point,
) -> Vec<Point> {
    if points.len() < 2 {
        return points.to_vec();
    }
    let mut best_distance = f64::MAX;
    let mut seam = points[0];
    let mut seam_index = 0usize;
    for (index, pair) in points.windows(2).enumerate() {
        let foot = projection_onto(pair[0], pair[1], target);
        let dx = (foot.x() - target.x()) as f64;
        let dy = (foot.y() - target.y()) as f64;
        let distance = dx * dx + dy * dy;
        if distance < best_distance {
            best_distance = distance;
            seam = foot;
            seam_index = index;
        }
    }
    let mut output = Vec::with_capacity(points.len() + 1);
    if seam == points[seam_index + 1] {
        output.extend_from_slice(&points[seam_index + 1..]);
        output.extend_from_slice(&points[..=seam_index]);
        output.push(seam);
    } else {
        output.push(seam);
        output.extend_from_slice(&points[seam_index + 1..]);
        output.extend_from_slice(&points[..=seam_index]);
        output.push(seam);
    }
    output
}

fn projection_onto(a: Point, b: Point, point: Point) -> Point {
    if a == b {
        return a;
    }
    let lx = (b.x() - a.x()) as f64;
    let ly = (b.y() - a.y()) as f64;
    // Disassembly of the AppImage `Point::projection_onto(Line)` confirms
    // GCC emits plain `mulsd`/`addsd`/`divsd` here — no FMA contraction —
    // and `cvttsd2si` truncation for the interpolated point.
    let theta =
        ((b.x() - point.x()) as f64 * lx + (b.y() - point.y()) as f64 * ly) / (lx * lx + ly * ly);
    if (0.0..=1.0).contains(&theta) {
        return Point::new(
            (theta * a.x() as f64 + (1.0 - theta) * b.x() as f64) as i64,
            (theta * a.y() as f64 + (1.0 - theta) * b.y() as f64) as i64,
        );
    }
    let da = (a.x() - point.x()).pow(2) + (a.y() - point.y()).pow(2);
    let db = (b.x() - point.x()).pow(2) + (b.y() - point.y()).pow(2);
    if da < db { a } else { b }
}
