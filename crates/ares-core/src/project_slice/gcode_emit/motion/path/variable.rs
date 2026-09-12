use super::{EmitState, PathProperties, SOURCE_EPSILON_MM, arc, extrusion, fan, overhang};

pub(super) struct Emission<'a> {
    pub(super) output: &'a mut Vec<u8>,
    pub(super) points: &'a [(f64, f64)],
    pub(super) wipe_points: &'a [(f64, f64)],
    pub(super) processed: &'a [overhang::ProcessedPoint],
    pub(super) original_speed: f64,
    pub(super) properties: PathProperties<'a>,
    pub(super) state: &'a mut EmitState,
}

pub(super) fn emit(command: Emission<'_>) {
    let Emission {
        output,
        points,
        wipe_points,
        processed,
        original_speed,
        properties,
        state,
    } = command;
    let original_feedrate = original_speed * 60.0;
    let mut last_feedrate = processed[0].speed * 60.0;
    let mut previous = points[0];
    for index in 1..points.len() {
        let end = arc::Point {
            x: points[index].0,
            y: points[index].1,
        };
        fan::update_for_variable_segment(
            output,
            properties,
            processed[index - 1],
            processed[index],
            state,
        );
        let length = ((end.x - previous.0) * (end.x - previous.0)
            + (end.y - previous.1) * (end.y - previous.1))
            .sqrt();
        if length < SOURCE_EPSILON_MM {
            continue;
        }
        let feedrate = processed[index - 1].speed * 60.0;
        if (last_feedrate - feedrate).abs() > 60.0 {
            extrusion::speed(output, feedrate, properties);
            state.current_feedrate = feedrate;
            last_feedrate = feedrate;
        } else if (original_feedrate - feedrate).abs() <= 60.0 {
            extrusion::speed(output, original_feedrate, properties);
            state.current_feedrate = original_feedrate;
            last_feedrate = original_feedrate;
        }
        extrusion::linear_segment(output, end, length, properties, state);
        previous = (end.x, end.y);
    }
    state.extrusion_feedrate = last_feedrate;
    state.wipe_start = wipe_points.last().map(|&(x, y)| arc::Point {
        x: x + state.origin.0 - state.extruder_offset.0,
        y: y + state.origin.1 - state.extruder_offset.1,
    });
    state.wipe_path = wipe_points
        .iter()
        .rev()
        .map(|&(x, y)| arc::Point {
            x: x + state.origin.0 - state.extruder_offset.0,
            y: y + state.origin.1 - state.extruder_offset.1,
        })
        .collect();
    if let Ok(path) = std::env::var("ARES_DUMP_PATH") {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = write!(file, "VP");
            for point in &state.wipe_path {
                let scaled = super::super::travel::scaled_position(*point, state);
                let _ = write!(file, " ({},{})", scaled.0, scaled.1);
            }
            let _ = writeln!(file);
        }
    }
}
