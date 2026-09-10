mod avoid_crossing;
mod constant;
mod retraction;
mod start_travel;
mod travel_emit;
mod variable;

pub(in crate::project_slice::gcode_emit) use avoid_crossing::Boundary;
#[allow(unused_imports)]
pub(super) use avoid_crossing::build_boundary;
pub(super) use retraction::can_skip_retraction;

use super::{
    EmitState, LayerGeometry, arc, clip, extrusion, fan, features::PathProperties,
    format::axis as format_axis, overhang, set_accel_and_jerk,
};

pub(super) const SOURCE_EPSILON_MM: f64 = 1e-4;

pub(super) fn emit(
    output: &mut Vec<u8>,
    points: impl Iterator<Item = (i64, i64)>,
    properties: PathProperties<'_>,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) {
    let mut scaled_points = points.collect::<Vec<_>>();
    let source_length = scaled_points
        .windows(2)
        .map(|pair| {
            let dx = (pair[1].0 - pair[0].0) as f64;
            let dy = (pair[1].1 - pair[0].1) as f64;
            (dx * dx + dy * dy).sqrt() * geometry.scale.factor()
        })
        .sum();
    // Upstream clips every loop tail in the origin-shifted (plate) frame
    // (`set_origin(unscaled(offset))`, `GCode.cpp:4424`): the truncating
    // cast inside `clip_end` rounds opposite directions for negative local
    // coordinates, shifting interpolated endpoints by one lattice unit.
    // Shift into the plate frame for the clip and restore after.
    let plate_offset = (
        (state.offset.0 / geometry.scale.factor()).round() as i64,
        (state.offset.1 / geometry.scale.factor()).round() as i64,
    );
    for point in &mut scaled_points {
        point.0 += plate_offset.0;
        point.1 += plate_offset.1;
    }
    clip::clip_end(
        &mut scaled_points,
        properties.end_clip / geometry.scale.factor(),
    );
    for point in &mut scaled_points {
        point.0 -= plate_offset.0;
        point.1 -= plate_offset.1;
    }
    let Some((&first_scaled, &last_scaled)) = scaled_points.first().zip(scaled_points.last())
    else {
        return;
    };
    // Upstream computes segment lengths from scaled integer coordinates
    // (`Line::length() * SCALING_FACTOR`, `GCode.cpp:6997-6998`), not from
    // the unscaled mm coordinates. Computing from mm loses precision through
    // the double rounding of `unscale` before the distance.
    let segment_lengths = scaled_points
        .windows(2)
        .map(|pair| {
            let dx = (pair[1].0 - pair[0].0) as f64;
            let dy = (pair[1].1 - pair[0].1) as f64;
            (dx * dx + dy * dy).sqrt() * geometry.scale.factor()
        })
        .collect::<Vec<_>>();
    let mut local_points = scaled_points
        .into_iter()
        .map(|(x, y)| (geometry.scale.unscale(x), geometry.scale.unscale(y)))
        .collect::<Vec<_>>();
    let mut fitting = properties.fitting.to_vec();
    if properties.end_clip > 0.0 {
        arc::clip_fitting_end(&mut local_points, &mut fitting, geometry.scale);
    }
    let (acceleration, configured_speed) =
        properties.kinematics(&state.options, state.layer_index, source_length);
    // `_mm3_per_mm` carries the effective flow (`GCode.cpp:6469-6471`:
    // geometric mm3_per_mm scaled by print and filament flow ratios, with
    // role ratios pre-baked into `PathProperties::mm3_per_mm`), and the cap
    // only applies while `filament_max_volumetric_speed > 0`
    // (`GCode.cpp:6615-6617`).
    let original_speed = if state.options.max_volumetric_speed > 0.0 {
        configured_speed.min(
            state.options.max_volumetric_speed
                / (properties.mm3_per_mm
                    * state.options.print_flow_ratio
                    * state.options.filament_flow_ratio),
        )
    } else {
        configured_speed
    };
    let processed = overhang::estimate(overhang::EstimateRequest {
        points: &local_points,
        properties,
        geometry,
        options: &state.options,
        layer_index: state.layer_index,
        original_speed,
    });
    let points = processed.as_ref().map_or_else(
        || {
            local_points
                .iter()
                .map(|&(x, y)| (x + state.offset.0, y + state.offset.1))
                .collect::<Vec<_>>()
        },
        |points| {
            points
                .iter()
                .map(|point| {
                    (
                        travel_emit::quantize_axis(point.x + state.offset.0),
                        travel_emit::quantize_axis(point.y + state.offset.1),
                    )
                })
                .collect::<Vec<_>>()
        },
    );
    // Source travels to the raw path start before overhang processing quantizes its points.
    let Some(&(first_local_x, first_local_y)) = local_points.first() else {
        return;
    };
    let first_x = first_local_x + state.offset.0;
    let first_y = first_local_y + state.offset.1;
    start_travel::emit(
        output,
        state,
        start_travel::Request {
            first_scaled,
            first_x,
            first_y,
            properties,
            geometry,
        },
    );
    let jerk = properties.jerk(&state.options, state.layer_index);
    set_accel_and_jerk(output, state, acceleration, jerk, false);
    state.extrusion_feedrate = processed
        .as_ref()
        .map_or(original_speed, |points| points[0].speed)
        * 60.0;
    if state.last_feature != Some(properties.feature) {
        let feature = state.tags.feature(properties.feature) + "\n";
        output.extend_from_slice(feature.as_bytes());
        state.last_feature = Some(properties.feature);
        // Arm the one-shot routing disable after the first-layer skirt
        // (`disable_once`, `GCode.cpp:4448-4450`).
        if properties.feature == "Skirt" && state.layer_index == 0 {
            state.avoid_crossing_disabled_once = true;
        }
    }
    if state.last_width != Some(properties.width) {
        output.extend_from_slice(
            format!(
                "{}\n",
                state
                    .tags
                    .width(&super::super::format_processor_float(f64::from(
                        properties.width
                    )))
            )
            .as_bytes(),
        );
        state.last_width = Some(properties.width);
    }
    if state
        .last_height
        .is_none_or(|height| (height - properties.height).abs() > 0.000_1)
    {
        output.extend_from_slice(
            format!(
                "{}\n",
                state
                    .tags
                    .height(&super::super::format_processor_float(f64::from(
                        properties.height
                    )))
            )
            .as_bytes(),
        );
        state.last_height = Some(properties.height);
    }
    extrusion::speed(output, state.extrusion_feedrate, properties);
    state.current_feedrate = state.extrusion_feedrate;
    if let Some(target) = state.pending_wipe_before_external_target.take() {
        output.extend_from_slice(
            format!("G1 X{} Y{}\n", format_axis(target.x), format_axis(target.y)).as_bytes(),
        );
        state.x = target.x;
        state.y = target.y;
        state.wipe_start = Some(target);
        // Upstream wraps the fake wipe path in its own
        // `;_EXTRUDE_SET_SPEED..;_EXTRUDE_END` block (`GCode.cpp:5884-5893` —
        // `extrude_path` on the force-no-extrusion polyline), then the wall
        // loop opens a fresh block; the cooling buffer therefore attributes
        // the wipe move's time to its own entry instead of the wall block.
        output.extend_from_slice(b";_EXTRUDE_END\n");
        extrusion::speed(output, state.extrusion_feedrate, properties);
        state.current_feedrate = state.extrusion_feedrate;
    }
    if let Some(slope) = properties.slope {
        let wipe_points = local_points
            .iter()
            .map(|&(x, y)| arc::Point {
                x: x + state.offset.0,
                y: y + state.offset.1,
            })
            .collect::<Vec<_>>();
        fan::update_for_constant_path(output, properties, state);
        super::scarf::emit_segments(output, &points, slope, properties, state);
        output.extend_from_slice(b";_EXTRUDE_END\n");
        state.wipe_path = wipe_points.into_iter().rev().collect();
        state.last_scaled_position = Some(last_scaled);
        return;
    }
    if let Some(processed) = processed {
        variable::emit(variable::Emission {
            output,
            points: &points,
            wipe_points: &local_points,
            processed: &processed,
            original_speed,
            properties,
            state,
        });
        state.last_scaled_position = Some(last_scaled);
        state.scarf_z = Some(state.layer_z);
        output.extend_from_slice(b";_EXTRUDE_END\n");
        return;
    }
    constant::emit(constant::Emission {
        output,
        points: &points,
        local_points: &local_points,
        segment_lengths: &segment_lengths,
        fitting: &fitting,
        last_scaled,
        properties,
        state,
    });
}
