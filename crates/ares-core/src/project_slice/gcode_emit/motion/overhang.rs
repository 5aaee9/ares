#[cfg(test)]
mod tests;

use super::{LayerGeometry, MotionOptions, features::PathProperties};
use crate::{FloatOrPercent, geometry::Point};

const OVERLAPS: [f64; 6] = [90.0, 75.0, 50.0, 25.0, 13.0, 0.0];
const INTERSECTION_EPSILON_MM: f64 = 1e-4;

pub(super) struct EstimateRequest<'a> {
    pub(super) points: &'a [(f64, f64)],
    pub(super) properties: PathProperties<'a>,
    pub(super) geometry: LayerGeometry<'a>,
    pub(super) options: &'a MotionOptions,
    pub(super) layer_index: usize,
    pub(super) original_speed: f64,
}

#[derive(Clone, Copy)]
pub(super) struct ProcessedPoint {
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) speed: f64,
    pub(super) overlap: f64,
}

#[derive(Clone, Copy)]
struct ExtendedPoint {
    x: f64,
    y: f64,
    distance: f32,
}

struct BoundaryContext<'a> {
    tree: &'a crate::geometry::LineDistanceTree<'a>,
    scale: crate::geometry::CoordinateScale,
    offset: f32,
    minimum_spacing: f64,
}

pub(super) fn estimate(request: EstimateRequest<'_>) -> Option<Vec<ProcessedPoint>> {
    let boundary = request.geometry.previous_layer_boundary?;
    if !request.options.enable_overhang_speed
        || request.layer_index == 0
        || !matches!(
            request.properties.feature,
            "Inner wall" | "Outer wall" | "Overhang wall" | "Bridge" | "Internal Bridge"
        )
        || request.points.len() < 2
    {
        return None;
    }

    // The reference speed caps against the same effective `_mm3_per_mm` as
    // the extrusion speed cap (`GCode.cpp:6658-6665`), and only while
    // `filament_max_volumetric_speed > 0`.
    let role_speed = if matches!(request.properties.feature, "Outer wall" | "Overhang wall") {
        request.options.outer_wall_speed
    } else {
        request.options.inner_wall_speed
    };
    let reference_speed = if request.options.max_volumetric_speed > 0.0 {
        role_speed.min(
            request.options.max_volumetric_speed
                / (request.properties.mm3_per_mm
                    * request.options.print_flow_ratio
                    * request.options.filament_flow_ratio),
        )
    } else {
        role_speed
    };
    let reference_speed = request
        .properties
        .slope
        .map_or(reference_speed, |slope| reference_speed.min(slope.speed));
    let sections = speed_sections(request.properties.width, reference_speed, request.options);
    let original_speed = request.original_speed as f32;
    let minimum_slowdown_distance = sections
        .iter()
        .filter(|section| section.1 <= original_speed)
        .map(|section| section.0)
        .reduce(f32::min)
        .unwrap_or(-1.0);
    let context = BoundaryContext {
        tree: boundary,
        scale: request.geometry.scale,
        offset: 0.5_f32 * request.properties.width,
        minimum_spacing: f64::from(request.properties.width) * 0.25,
    };
    let extended = context.add_boundary_intersections(request.points);
    let extended = context.add_segmentation_points(&extended, minimum_slowdown_distance);

    let mut processed = Vec::with_capacity(extended.len());
    let mut variable = false;
    for index in 0..extended.len() {
        let current = extended[index];
        let next = extended.get(index + 1).copied().unwrap_or(current);
        let speed = speed_for_distance(current.distance, &sections, original_speed)
            .min(speed_for_distance(next.distance, &sections, original_speed))
            .min(original_speed);
        variable |= (f64::from(speed) - request.original_speed).abs() > 1.0;
        let width_inverse = 1.0_f32 / request.properties.width;
        processed.push(ProcessedPoint {
            x: context.scale.unscale(scale_trunc(current.x, context.scale)),
            y: context.scale.unscale(scale_trunc(current.y, context.scale)),
            speed: f64::from(speed),
            overlap: f64::from(
                (1.0_f32 - current.distance * width_inverse)
                    .min(1.0_f32 - next.distance * width_inverse),
            ),
        });
    }
    variable.then_some(processed)
}

fn speed_sections(width: f32, reference_speed: f64, options: &MotionOptions) -> Vec<(f32, f32)> {
    let reference_speed_f32 = reference_speed as f32;
    let band_speeds = options.overhang_speed_bands.map(|value| {
        value
            .map(|value| absolute(value, reference_speed))
            .filter(|speed| *speed >= 0.5)
            .map(|speed| source_dynamic_speed(speed, reference_speed, reference_speed_f32))
            .unwrap_or(reference_speed_f32)
    });
    let severe_speed = if options.slowdown_for_curled_perimeters {
        band_speeds[3]
    } else {
        source_dynamic_speed(options.bridge_speed, reference_speed, reference_speed_f32)
    };
    let speeds = [
        reference_speed_f32,
        band_speeds[0],
        band_speeds[1],
        band_speeds[2],
        band_speeds[3],
        severe_speed,
    ];
    let mut sections = OVERLAPS
        .into_iter()
        .zip(speeds)
        .map(|(overlap, speed)| ((f64::from(width) * (1.0 - overlap / 100.0)) as f32, speed))
        .collect::<Vec<_>>();
    sections.sort_by(|left, right| {
        left.0
            .total_cmp(&right.0)
            .then_with(|| right.1.total_cmp(&left.1))
    });
    for index in 1..sections.len() {
        if sections[index].0 == sections[index - 1].0 {
            sections[index].1 = sections[index - 1].1;
        }
    }
    sections
}

impl BoundaryContext<'_> {
    fn add_boundary_intersections(&self, points: &[(f64, f64)]) -> Vec<ExtendedPoint> {
        let mut result = Vec::with_capacity(points.len() * 2);
        result.push(self.extended(points[0]));
        for &point in &points[1..] {
            let next = self.extended(point);
            let previous = *result.last().unwrap();
            self.append_intersections(&mut result, previous, next);
            result.push(next);
        }
        result
    }

    fn append_intersections(
        &self,
        result: &mut Vec<ExtendedPoint>,
        previous: ExtendedPoint,
        next: ExtendedPoint,
    ) {
        let threshold = f64::from(self.offset) + INTERSECTION_EPSILON_MM;
        let previous_outside = f64::from(previous.distance) > threshold;
        let next_outside = f64::from(next.distance) > threshold;
        if previous_outside == next_outside {
            return;
        }
        let line = crate::geometry::Line::new(self.scaled(previous), self.scaled(next));
        for (intersection, _) in self.tree.intersections_sorted(line) {
            let intersection = ExtendedPoint {
                x: self.scale.unscale(intersection.x()),
                y: self.scale.unscale(intersection.y()),
                distance: self.offset,
            };
            if distance(previous, intersection) > self.minimum_spacing
                && distance(intersection, next) > self.minimum_spacing
            {
                result.push(intersection);
            }
        }
    }

    fn add_segmentation_points(
        &self,
        points: &[ExtendedPoint],
        minimum_slowdown_distance: f32,
    ) -> Vec<ExtendedPoint> {
        let mut result = Vec::with_capacity(points.len() * 2);
        result.push(points[0]);
        for pair in points.windows(2) {
            let current = pair[0];
            let next = pair[1];
            let line_length = distance(current, next);
            let near_boundary = |point: ExtendedPoint| {
                point.distance > -self.offset && point.distance < self.offset + 2.0
            };
            let needs_slowdown = current.distance.abs() > minimum_slowdown_distance
                || next.distance.abs() > minimum_slowdown_distance;
            let should_segment = (near_boundary(current) || near_boundary(next))
                && ((minimum_slowdown_distance > 0.0 && needs_slowdown && line_length >= 2.0)
                    || (minimum_slowdown_distance <= 0.0 && line_length > 4.0));
            if should_segment {
                let a0 = (f64::from(current.distance + 3.0_f32 * self.offset) / line_length)
                    .clamp(0.0, 1.0);
                let a1 = (1.0 - f64::from(next.distance + 3.0_f32 * self.offset) / line_length)
                    .clamp(0.0, 1.0);
                self.append_segmentation_candidate(
                    &mut result,
                    [current, next],
                    a0.min(a1),
                    minimum_slowdown_distance,
                );
                self.append_segmentation_candidate(
                    &mut result,
                    [current, next],
                    a0.max(a1),
                    minimum_slowdown_distance,
                );
            }
            result.push(next);
        }
        result
    }

    fn append_segmentation_candidate(
        &self,
        result: &mut Vec<ExtendedPoint>,
        endpoints: [ExtendedPoint; 2],
        factor: f64,
        minimum_slowdown_distance: f32,
    ) {
        let [current, next] = endpoints;
        if factor <= 0.0 || factor >= 1.0 {
            return;
        }
        let candidate = interpolate(current, next, factor);
        let raw_distance = self.signed_distance((candidate.x, candidate.y));
        let candidate = ExtendedPoint {
            distance: raw_distance + self.offset,
            ..candidate
        };
        if raw_distance.abs() > minimum_slowdown_distance
            && distance(current, candidate) > self.minimum_spacing
            && distance(candidate, next) > self.minimum_spacing
        {
            result.push(candidate);
        }
    }

    fn extended(&self, point: (f64, f64)) -> ExtendedPoint {
        let raw_distance = self.signed_distance(point);
        ExtendedPoint {
            x: point.0,
            y: point.1,
            distance: raw_distance + self.offset,
        }
    }

    fn signed_distance(&self, point: (f64, f64)) -> f32 {
        let nearest = self
            .tree
            .nearest_f32([point.0 as f32, point.1 as f32], self.scale)
            .expect("a nonempty boundary has a nearest line");
        let scaled = Point::new(
            scale_trunc(point.0, self.scale),
            scale_trunc(point.1, self.scale),
        );
        self.tree.outside(scaled) as f32 * nearest.squared_distance.sqrt()
    }

    fn scaled(&self, point: ExtendedPoint) -> Point {
        Point::new(
            scale_trunc(point.x, self.scale),
            scale_trunc(point.y, self.scale),
        )
    }
}

fn speed_for_distance(distance: f32, sections: &[(f32, f32)], original_speed: f32) -> f32 {
    if distance <= sections[0].0 {
        return original_speed.round();
    }
    if distance >= sections[sections.len() - 1].0 {
        return sections[sections.len() - 1].1.round();
    }
    let upper = sections.partition_point(|section| distance > section.0);
    let lower = upper - 1;
    let ratio = (distance - sections[lower].0) / (sections[upper].0 - sections[lower].0);
    ((1.0_f32 - ratio) * sections[lower].1 + ratio * sections[upper].1).round()
}

fn source_dynamic_speed(speed: f64, reference_speed: f64, reference_speed_f32: f32) -> f32 {
    (f64::from(reference_speed_f32) * (speed * 100.0 / reference_speed) / 100.0) as f32
}

fn absolute(value: FloatOrPercent, base: f64) -> f64 {
    match value {
        FloatOrPercent::Float(value) => value,
        FloatOrPercent::Percent(value) => base * value.0 / 100.0,
    }
}

fn scale_trunc(value: f64, scale: crate::geometry::CoordinateScale) -> i64 {
    (value / scale.factor()) as i64
}

fn interpolate(first: ExtendedPoint, second: ExtendedPoint, factor: f64) -> ExtendedPoint {
    ExtendedPoint {
        x: first.x + factor * (second.x - first.x),
        y: first.y + factor * (second.y - first.y),
        distance: 0.0,
    }
}

fn distance(first: ExtendedPoint, second: ExtendedPoint) -> f64 {
    (second.x - first.x).hypot(second.y - first.y)
}
