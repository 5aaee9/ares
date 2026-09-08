mod feedrate;

#[cfg(test)]
mod tests;

use crate::{
    options::{PartCoolingFanRamp, PartCoolingFanRampConfig},
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

const PART_FAN_MARKER: &[u8] = b";__ARES_PART_FAN_STATE__\n";
const ROLE_FAN_MARKER_PREFIX: &str = ";__ARES_ROLE_FAN_";

pub(super) enum DeferredRoleFan {
    Baseline,
    Conditional { speed: u8, force: bool },
    Fixed(u8),
}

pub(super) fn append_deferred_role_fan(output: &mut Vec<u8>, target: DeferredRoleFan) {
    let marker = match target {
        DeferredRoleFan::Baseline => format!("{ROLE_FAN_MARKER_PREFIX}BASE__\n"),
        DeferredRoleFan::Conditional { speed, force } => {
            let force = if force { "FORCE_" } else { "" };
            format!("{ROLE_FAN_MARKER_PREFIX}CONDITIONAL_{force}{speed}__\n")
        }
        DeferredRoleFan::Fixed(speed) => {
            format!("{ROLE_FAN_MARKER_PREFIX}FIXED_{speed}__\n")
        }
    };
    output.extend_from_slice(marker.as_bytes());
}

pub(super) struct CoolingState {
    part_speed: u8,
    physical_part_speed: u8,
    provisional_part_speed: u8,
    pending_layer_index: Option<usize>,
    part_fan_ramp: PartCoolingFanRamp,
    additional_speed: u8,
    additional_fan_speed: u8,
    auxiliary_fan: bool,
    part_cooling_fan_min_pwm: u8,
    /// Non-BBL printers emit the initial fan state at the first layer
    /// boundary; BBL machines carry it inside their start sequence.
    emit_initial_fan: bool,
    fan_mover_enabled: bool,
    feedrate: feedrate::State,
}

impl CoolingState {
    pub(super) fn from_traversal(traversal: &PreparedPostClassicTraversal) -> Self {
        let full = &traversal.resolved.views.full;
        let filament = &full.filament.print;
        let runtime = &traversal.resolved.views.runtime_gcode;
        let first_bool = |values: &[crate::OrcaBool]| values.first().is_some_and(|value| value.0);
        let max_speed = first_percent(&filament.fan_max_speed.0);
        let slow_down_layer_time = filament
            .slow_down_layer_time
            .0
            .first()
            .map_or(0.0, |value| value.0);
        let part_fan_ramp = PartCoolingFanRamp::new(PartCoolingFanRampConfig {
            min_speed: first_percent(&filament.fan_min_speed.0),
            max_speed,
            full_speed_layer: first_non_negative(&filament.full_fan_speed_layer.0) as u32,
            close_fan_first_layers: first_non_negative(&filament.close_fan_the_first_x_layers.0)
                as u32,
            layer_times_s: [
                slow_down_layer_time,
                filament
                    .fan_cooling_layer_time
                    .0
                    .first()
                    .map_or(0.0, |value| value.0),
            ],
            fan_kickstart_s: runtime.fan_kickstart.0,
            reduce_fan_stop_start_freq: first_bool(&filament.reduce_fan_stop_start_freq.0),
        });
        Self {
            part_speed: 0,
            physical_part_speed: 0,
            provisional_part_speed: 0,
            pending_layer_index: None,
            part_fan_ramp,
            additional_speed: 0,
            additional_fan_speed: first_percent_int(&filament.additional_cooling_fan_speed.0),
            auxiliary_fan: runtime.auxiliary_fan.0,
            part_cooling_fan_min_pwm: runtime.part_cooling_fan_min_pwm.0.clamp(0, 100) as u8,
            emit_initial_fan: !super::tags::Tags::of(traversal).is_bbl(),
            fan_mover_enabled: runtime.fan_speedup_time.0 != 0.0 || runtime.fan_kickstart.0 > 0.0,
            feedrate: feedrate::State::new(
                feedrate::Config {
                    enabled: first_bool(&filament.slow_down_for_layer_cooling.0),
                    target_time: slow_down_layer_time as f32,
                    minimum_speed: filament
                        .slow_down_min_speed
                        .0
                        .first()
                        .map_or(0.0, |value| value.0 as f32),
                    keep_outer_wall_speed: first_bool(&filament.dont_slow_down_outer_wall.0),
                    relative_e: runtime.use_relative_e_distances.0,
                },
                runtime.travel_speed.0,
            ),
        }
    }

    pub(super) fn begin_layer(&mut self, output: &mut Vec<u8>, layer_index: usize) {
        self.pending_layer_index = Some(layer_index);
        self.provisional_part_speed = self
            .part_fan_ramp
            .speed_for_layer_time(layer_index, Some(0.0))
            .unwrap_or(0);
        output.extend_from_slice(PART_FAN_MARKER);

        let additional_speed = if layer_index < self.part_fan_ramp.close_fan_first_layers() as usize
        {
            0
        } else {
            self.additional_fan_speed
        };
        // Orca emits the initial auxiliary-fan state unconditionally at the
        // first layer boundary (print-start block, before LAYER_CHANGE), not
        // only when the speed changes.
        if self.auxiliary_fan
            && (additional_speed != self.additional_speed
                || (layer_index == 0 && self.emit_initial_fan))
        {
            self.additional_speed = additional_speed;
            output.extend_from_slice(
                format!("M106 P2 S{}\n", additional_fan_pwm(additional_speed)).as_bytes(),
            );
        }
    }

    pub(super) const fn provisional_part_speed(&self) -> u8 {
        self.provisional_part_speed
    }

    pub(super) fn finish_layer(&mut self, output: &mut Vec<u8>, layer_start: usize) {
        let layer_time = feedrate::rewrite_layer(output, layer_start, &mut self.feedrate);
        let layer_index = self.pending_layer_index.take().unwrap();
        let part_speed = self
            .part_fan_ramp
            .speed_for_layer_time(layer_index, Some(f64::from(layer_time)))
            .unwrap_or(0);
        let initial = should_emit_initial_part_fan(
            layer_index,
            self.emit_initial_fan,
            self.fan_mover_enabled,
            part_speed,
        );
        let replacement = if part_speed != self.part_speed || initial {
            let speed = clamped_part_speed(part_speed, self.part_cooling_fan_min_pwm);
            format!("M106 S{}\n", part_fan_pwm(speed)).into_bytes()
        } else {
            Vec::new()
        };
        let marker_start = layer_start
            + output[layer_start..]
                .windows(PART_FAN_MARKER.len())
                .position(|window| window == PART_FAN_MARKER)
                .unwrap();
        if !replacement.is_empty() {
            self.physical_part_speed = part_speed;
        }
        output.splice(
            marker_start..marker_start + PART_FAN_MARKER.len(),
            replacement,
        );
        self.resolve_role_fans(output, layer_start, part_speed);
        self.part_speed = part_speed;
        self.provisional_part_speed = part_speed;
    }

    fn resolve_role_fans(&mut self, output: &mut Vec<u8>, layer_start: usize, baseline: u8) {
        while let Some(offset) = output[layer_start..]
            .windows(ROLE_FAN_MARKER_PREFIX.len())
            .position(|window| window == ROLE_FAN_MARKER_PREFIX.as_bytes())
        {
            let start = layer_start + offset;
            let end = output[start..]
                .iter()
                .position(|byte| *byte == b'\n')
                .map_or(output.len(), |offset| start + offset + 1);
            let marker = std::str::from_utf8(&output[start..end]).unwrap();
            let (target, force) = if marker.contains("_BASE__") {
                (baseline, true)
            } else if let Some(value) = marker
                .strip_prefix(";__ARES_ROLE_FAN_CONDITIONAL_")
                .and_then(|value| value.strip_suffix("__\n"))
            {
                let (value, force) = value
                    .strip_prefix("FORCE_")
                    .map_or((value, false), |value| (value, true));
                let requested = value.parse::<u8>().unwrap();
                (requested.max(baseline), force || requested > baseline)
            } else {
                (
                    marker
                        .strip_prefix(";__ARES_ROLE_FAN_FIXED_")
                        .and_then(|value| value.strip_suffix("__\n"))
                        .unwrap()
                        .parse::<u8>()
                        .unwrap(),
                    true,
                )
            };
            let replacement = if force || target != self.physical_part_speed {
                self.physical_part_speed = target;
                let emitted = clamped_part_speed(target, self.part_cooling_fan_min_pwm);
                format!("M106 S{}\n", part_fan_pwm(emitted)).into_bytes()
            } else {
                Vec::new()
            };
            output.splice(start..end, replacement);
        }
    }
}

fn should_emit_initial_part_fan(
    layer_index: usize,
    emit_initial_fan: bool,
    fan_mover_enabled: bool,
    speed: u8,
) -> bool {
    layer_index == 0 && emit_initial_fan && !(fan_mover_enabled && speed == 0)
}

fn first_percent(values: &[crate::OrcaFloat]) -> f64 {
    values
        .first()
        .map_or(0.0, |value| value.0.clamp(0.0, 100.0))
}

fn first_percent_int(values: &[crate::OrcaInt]) -> u8 {
    values
        .first()
        .map_or(0, |value| value.0.clamp(0, 100) as u8)
}

fn first_non_negative(values: &[crate::OrcaInt]) -> usize {
    values.first().map_or(0, |value| value.0.max(0) as usize)
}

fn clamped_part_speed(speed: u8, minimum: u8) -> u8 {
    if speed > 0 && speed < minimum {
        minimum
    } else {
        speed
    }
}

fn part_fan_pwm(speed: u8) -> u32 {
    (255.5 * f64::from(speed) / 100.0).floor() as u32
}

fn additional_fan_pwm(speed: u8) -> u32 {
    (255.0 * f64::from(speed) / 100.0).floor() as u32
}
