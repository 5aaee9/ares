//! `GCodeProcessor` axis limits, junction deviation and logical position.

use super::{GCodeFlavor, MotionState, word};

impl MotionState {
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

    pub(super) fn update_axis_limits(&mut self, code: &str, acceleration: bool) {
        let limits = if acceleration {
            &mut self.max_acceleration
        } else {
            &mut self.max_feedrate
        };
        for (axis, letter) in ['X', 'Y', 'Z', 'E'].into_iter().enumerate() {
            limits[axis] = word(code, letter).unwrap_or(limits[axis]);
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
