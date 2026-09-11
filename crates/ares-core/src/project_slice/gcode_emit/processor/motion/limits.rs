//! `GCodeProcessor` axis limits, junction deviation and logical position.

use super::super::motion_util::MMMIN_TO_MMSEC;
use super::{GCodeFlavor, MotionState, word};

impl MotionState {
    pub(in crate::project_slice::gcode_emit::processor) fn with_limits(
        limits: crate::project_slice::gcode_emit::processor::ProcessorLimits,
    ) -> Self {
        let travel = if limits.gcode_flavor.supports_separate_travel_acceleration() {
            limits.travel_acceleration
        } else {
            0.0
        };
        Self {
            gcode_flavor: limits.gcode_flavor,
            junction_deviation: limits.junction_deviation,
            max_feedrate: limits.max_feedrate,
            max_acceleration: limits.max_acceleration,
            jerk: limits.jerk,
            ..Self::with_acceleration_limits(
                limits.print_acceleration,
                limits.retract_acceleration,
                travel,
            )
        }
    }

    /// `GCodeProcessor.cpp:5766-5791`: for Marlin firmware with a positive
    /// `machine_max_junction_deviation`, the per-axis jerk limit derives
    /// from the junction deviation and the current print acceleration
    /// clamped by the per-axis maximum; classic M205 jerks apply otherwise.
    pub(super) fn effective_jerk(&self) -> [f64; 4] {
        if self.gcode_flavor != GCodeFlavor::MarlinFirmware || self.junction_deviation <= 0.0 {
            return self.jerk;
        }
        let mut jerk = [0.0; 4];
        for (jerk, max_acceleration) in jerk.iter_mut().zip(self.max_acceleration) {
            let effective_acceleration = if max_acceleration > 0.0 {
                self.acceleration.min(max_acceleration)
            } else {
                self.acceleration
            };
            *jerk = if effective_acceleration > 0.0 {
                (self.junction_deviation * effective_acceleration * 2.5).sqrt()
            } else {
                0.0
            };
        }
        jerk
    }

    /// `GCodeProcessor.cpp:5160-5201` (M201) and `:5179-5201` (M203):
    /// acceleration values apply raw; feedrate values apply raw only for
    /// Marlin/Smoothie/Klipper and are converted from mm/min otherwise
    /// (M203 is ignored entirely for Repetier).
    pub(super) fn update_axis_limits(&mut self, code: &str, acceleration: bool) {
        if !acceleration && self.gcode_flavor == GCodeFlavor::Repetier {
            return;
        }
        let factor = if acceleration
            || matches!(
                self.gcode_flavor,
                GCodeFlavor::MarlinLegacy
                    | GCodeFlavor::MarlinFirmware
                    | GCodeFlavor::Klipper
                    | GCodeFlavor::Smoothie
            ) {
            1.0
        } else {
            MMMIN_TO_MMSEC
        };
        let limits = if acceleration {
            &mut self.max_acceleration
        } else {
            &mut self.max_feedrate
        };
        for (axis, letter) in ['X', 'Y', 'Z', 'E'].into_iter().enumerate() {
            limits[axis] =
                word(code, letter).map_or(limits[axis], |value| (value as f32 * factor) as f64);
        }
    }

    /// `GCodeProcessor.cpp:5400-5418`: M566 jerk limits are always
    /// specified in mm/min and converted to mm/s.
    pub(super) fn update_jerk_limits(&mut self, code: &str) {
        for (axis, letter) in ['X', 'Y', 'Z', 'E'].into_iter().enumerate() {
            self.jerk[axis] = word(code, letter).map_or(self.jerk[axis], |value| {
                (value as f32 * MMMIN_TO_MMSEC) as f64
            });
        }
    }

    pub(super) fn set_position(&mut self, code: &str) {
        let e = word(code, 'E');
        let position = ['X', 'Y', 'Z'].map(|letter| word(code, letter));
        if e.is_none() && position.iter().all(Option::is_none) {
            self.position = [0.0; 3];
            self.e_position = 0.0;
            return;
        }
        self.e_position = e.unwrap_or(self.e_position);
        for (axis, value) in position.into_iter().enumerate() {
            self.position[axis] = value.unwrap_or(self.position[axis]);
        }
    }
}
