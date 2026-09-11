use super::{Movement, format_axis, format_e, set_word, word_value};
use crate::{
    Point2,
    gcode_spiral_vase::{distance, distance_f32, nearest_point_on_polyline},
};

pub(super) struct SmoothRequest<'a> {
    pub(super) normal: String,
    pub(super) movement: Movement,
    pub(super) progress: f64,
    pub(super) previous_layer: &'a [Point2],
    pub(super) maximum_distance: f64,
    pub(super) minimum_segment_length: f64,
}

pub(super) fn smooth_line(
    request: SmoothRequest<'_>,
    last_emitted: &mut Option<Point2>,
    current_layer: &mut Vec<Point2>,
) -> Option<String> {
    let SmoothRequest {
        mut normal,
        movement,
        progress,
        previous_layer,
        maximum_distance,
        minimum_segment_length,
    } = request;
    let original = Point2::new(movement.target_x, movement.target_y);
    current_layer.push(original);
    // Upstream `SpiralVaseHelpers` computes the projection, distance gate,
    // and interpolation entirely in float before the X/Y words format.
    let original_f = [movement.target_x as f32, movement.target_y as f32];
    let previous_layer_f = previous_layer
        .iter()
        .map(|point| [point.x() as f32, point.y() as f32])
        .collect::<Vec<_>>();
    let Some(nearest) = nearest_point_on_polyline(&previous_layer_f, original_f) else {
        *last_emitted = Some(original);
        return Some(normal);
    };
    if f64::from(distance_f32(nearest, original_f)) >= maximum_distance {
        *last_emitted = Some(original);
        return Some(normal);
    }
    let factor = progress as f32;
    let target = Point2::new(
        f64::from(nearest[0] * (1.0 - factor) + original_f[0] * factor),
        f64::from(nearest[1] * (1.0 - factor) + original_f[1] * factor),
    );
    let adjusted_length = distance(last_emitted.unwrap_or(original), target);
    if adjusted_length < minimum_segment_length {
        return None;
    }
    normal = set_word(&normal, 'X', format_axis(target.x()));
    normal = set_word(&normal, 'Y', format_axis(target.y()));
    if let Some(e) = word_value(&normal, 'E') {
        normal = set_word(
            &normal,
            'E',
            format_e(e * adjusted_length / movement.xy_distance),
        );
    }
    *last_emitted = Some(target);
    Some(normal)
}
