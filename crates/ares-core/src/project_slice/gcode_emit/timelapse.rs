use crate::{
    GenerationMetadata, SliceError,
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

use super::{footprint, template, value};

#[derive(Clone, Copy)]
pub(super) struct TimelapseLayer {
    pub(super) index: usize,
    pub(super) z: f64,
    pub(super) max_z: f64,
}

#[derive(Clone, Copy)]
pub(super) struct Context<'a> {
    pub(super) traversal: &'a PreparedPostClassicTraversal,
    pub(super) layer: TimelapseLayer,
    pub(super) metadata: GenerationMetadata,
    pub(super) first_layer_bounds: Option<footprint::FirstLayerBounds>,
}

pub(super) fn append_and_track(
    output: &mut Vec<u8>,
    state: &mut super::motion::EmitState,
    context: Context<'_>,
) -> Result<(), SliceError> {
    if let Some(z) = append(
        output,
        context.traversal,
        context.layer,
        context.metadata,
        context.first_layer_bounds,
    )? {
        state.lifted = z > context.layer.z + f64::EPSILON;
        state.template_lifted = state.lifted;
        // The timelapse g-code parks the head; upstream marks the writer
        // position unclear (`GCode.cpp:5171-5178`) so the next travel takes
        // the separate first-position form.
        state.positioned = false;
    }
    Ok(())
}

pub(super) fn append_traditional(
    enabled: bool,
    output: &mut Vec<u8>,
    state: &mut super::motion::EmitState,
    context: Context<'_>,
) -> Result<bool, SliceError> {
    if !enabled {
        return Ok(false);
    }
    super::motion::prepare_traditional_timelapse(output, state);
    append_and_track(output, state, context)?;
    Ok(true)
}

fn append(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    layer: TimelapseLayer,
    metadata: GenerationMetadata,
    first_layer_bounds: Option<footprint::FirstLayerBounds>,
) -> Result<Option<f64>, SliceError> {
    let runtime = &traversal.resolved.views.runtime_gcode;
    let source = &runtime.time_lapse_gcode.0;
    if source.is_empty() {
        return Ok(None);
    }

    let mut config = super::placeholders::base_config(traversal, metadata, first_layer_bounds)?;
    config.insert("layer_num", value::Value::number(layer.index as f64));
    config.insert("layer_z", value::Value::number(layer.z));
    config.insert("max_layer_z", value::Value::number(layer.max_z));
    if let Some((min_x, min_y, size_x, size_y)) = first_layer_bounds {
        config.insert(
            "first_layer_center_no_wipe_tower",
            value::Value::List(vec![
                value::Value::number(min_x + 0.5 * size_x),
                value::Value::number(min_y + 0.5 * size_y),
            ]),
        );
    }
    // `ToolOrdering::cal_most_used_extruder` (`ToolOrdering.cpp:955-981`):
    // the most-used PHYSICAL extruder is `filament_map[f] - 1` over the
    // used filaments, counted once per layer; ties resolve to the HIGHEST
    // index. With every configured filament used each layer, each distinct
    // physical extruder weighs one layer-count, so the tie-to-highest rule
    // decides. The placeholder id is then
    // `physical_extruder_map.get_at(most_used)` (`GCode.cpp:5162`).
    let full = &traversal.resolved.views.full;
    let map = &full.project.gcode.filament_map.0;
    let most_used = map
        .iter()
        .filter(|entry| entry.0 >= 1)
        .map(|entry| (entry.0 - 1) as usize)
        .max();
    let physical_extruder = match most_used {
        Some(index) => runtime
            .physical_extruder_map
            .0
            .get(index)
            .or_else(|| runtime.physical_extruder_map.0.first())
            .map_or(0, |value| value.0),
        // An all-zero/empty map degenerates to the first physical entry
        // (`physical_extruder_map.get_at(0)`).
        None => runtime
            .physical_extruder_map
            .0
            .first()
            .map_or(0, |value| value.0),
    };
    config.insert(
        "most_used_physical_extruder_id",
        value::Value::number(physical_extruder as f64),
    );
    config.insert(
        "curr_physical_extruder_id",
        value::Value::number(physical_extruder as f64),
    );
    let position = safe_position(traversal);
    config.insert(
        "has_timelapse_safe_pos",
        value::Value::option_bool(position.is_some()),
    );
    let (x, y) = position.unwrap_or_default();
    config.insert("timelapse_pos_x", value::Value::number(f64::from(x)));
    config.insert("timelapse_pos_y", value::Value::number(f64::from(y)));

    let rendered = template::render(source, &mut config).map_err(|error| {
        SliceError::InvalidInput(format!(
            "invalid project timelapse G-code template: {error}"
        ))
    })?;
    let last_z = last_motion_z(&rendered);
    output.extend_from_slice(rendered.as_bytes());
    output.push(b'\n');
    Ok(last_z)
}

fn last_motion_z(gcode: &str) -> Option<f64> {
    gcode.lines().rev().find_map(|line| {
        let code = line.split_once(';').map_or(line, |(code, _)| code).trim();
        matches!(
            code.split_ascii_whitespace().next(),
            Some("G0" | "G1" | "G2" | "G3")
        )
        .then(|| {
            code.split_ascii_whitespace()
                .skip(1)
                .find_map(|word| word.strip_prefix('Z')?.parse().ok())
        })
        .flatten()
    })
}

fn safe_position(traversal: &PreparedPostClassicTraversal) -> Option<(i32, i32)> {
    let (object_min_x, object_min_y, object_max_x, object_max_y) =
        footprint::model_bounds(traversal)?;
    let printer = &traversal.resolved.views.full.printer.remaining;
    let first = printer.printable_area.0.first()?;
    let (bed_min_x, bed_min_y, bed_max_x, bed_max_y) =
        printer.printable_area.0.iter().skip(1).fold(
            (first.x, first.y, first.x, first.y),
            |bounds, point| {
                (
                    bounds.0.min(point.x),
                    bounds.1.min(point.y),
                    bounds.2.max(point.x),
                    bounds.3.max(point.y),
                )
            },
        );
    let camera_clearance = printer.extruder_clearance_radius.0 * std::f64::consts::FRAC_1_SQRT_2;
    let y = object_max_y + camera_clearance;
    if y < bed_min_y || y > bed_max_y {
        return None;
    }
    let line_start = (object_min_x - camera_clearance).max(bed_min_x);
    let line_end = (object_max_x + camera_clearance).min(bed_max_x);
    if line_start > line_end {
        return None;
    }

    let current_x = (object_min_x + object_max_x) * 0.5;
    let current_y = (object_min_y + object_max_y) * 0.5;
    let steps = ((line_end - line_start) / 5.0).floor() as usize;
    let mut best = line_start;
    let mut best_penalty = f64::MAX;
    for step in 0..=steps {
        let candidate = line_start + step as f64 * 5.0;
        let penalty = (current_x - candidate).abs() + (current_y - y).abs()
            - (candidate.abs() + y.abs()) / 3.0;
        if penalty < best_penalty {
            best_penalty = penalty;
            best = candidate;
        }
    }
    Some((best as i32, y as i32))
}
