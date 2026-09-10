mod arc;
#[cfg(test)]
mod axis_acceleration_tests;
mod limits;
use super::motion_util::{MMMIN_TO_MMSEC, assignment, clamp, clamped_word, norm, scale, word};
use crate::options::GCodeFlavor;
mod planner;
pub(super) use planner::RollingPlanner;
#[cfg(test)]
pub(super) fn planned_times(blocks: &[MotionBlock]) -> Vec<f64> {
    planner::planned_times(blocks)
}
#[cfg(test)]
pub(super) fn planned_trapezoid_time(
    distance: f32,
    cruise: f32,
    entry: f32,
    exit: f32,
    acceleration: f32,
) -> f32 {
    planner::planned_trapezoid_time(distance, cruise, entry, exit, acceleration)
}

use std::f64::consts::PI;

pub(super) struct MotionState {
    pub(super) position: [f64; 3],
    pub(super) e_position: f64,
    pub(super) feedrate: f64,
    pub(super) acceleration: f64,
    pub(super) retract_acceleration: f64,
    pub(super) travel_acceleration: f64,
    pub(super) max_print_acceleration: f64,
    pub(super) max_retract_acceleration: f64,
    pub(super) max_travel_acceleration: f64,
    pub(super) max_acceleration: [f64; 4],
    pub(super) max_feedrate: [f64; 4],
    pub(super) jerk: [f64; 4],
    pub(super) junction_deviation: f64,
    pub(super) relative: bool,
    pub(super) e_relative: bool,
    pub(super) wiping: bool,
    pub(super) gcode_flavor: GCodeFlavor,
}

impl Default for MotionState {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            e_position: 0.0,
            feedrate: 0.0,
            acceleration: 0.0,
            retract_acceleration: 0.0,
            travel_acceleration: 0.0,
            max_print_acceleration: 0.0,
            max_retract_acceleration: 0.0,
            max_travel_acceleration: 0.0,
            max_acceleration: [0.0; 4],
            max_feedrate: [0.0; 4],
            jerk: [9.0, 9.0, 3.0, 2.5],
            junction_deviation: 0.0,
            relative: false,
            e_relative: false,
            gcode_flavor: GCodeFlavor::MarlinLegacy,
            wiping: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum MotionKind {
    Regular,
    Unretract,
    ToolChange,
}

pub(super) struct MotionBlock {
    pub(super) distance: f64,
    pub(super) speed: f64,
    pub(super) acceleration: f64,
    pub(super) centripetal_acceleration: f64,
    pub(super) jerk: [f64; 4],
    pub(super) direction: [f64; 4],
    /// Whether XYZ displacement is zero; cache eligibility also depends on
    /// the upstream move classification and inserted seam vertex.
    pub(super) e_only: bool,
    pub(super) kind: MotionKind,
}

impl MotionState {
    /// `GCodeProcessor.cpp:2110-2122`: the current accelerations are
    /// seeded from the machine limits at configure time (falling back to
    /// the upstream Prusa defaults when a limit is unset), not zero.
    pub(super) fn with_acceleration_limits(print: f64, retract: f64, travel: f64) -> Self {
        Self {
            acceleration: if print > 0.0 { print } else { 1500.0 },
            retract_acceleration: if retract > 0.0 { retract } else { 1500.0 },
            travel_acceleration: if travel > 0.0 { travel } else { 1250.0 },
            max_print_acceleration: print,
            max_retract_acceleration: retract,
            max_travel_acceleration: travel,
            ..Self::default()
        }
    }

    pub(super) fn set_wiping(&mut self, wiping: bool) {
        self.wiping = wiping;
    }

    pub(super) fn motions(&mut self, code: &str) -> Vec<MotionBlock> {
        let start = self.position;
        let start_e = self.e_position;
        let Some(block) = self.motion(code) else {
            return Vec::new();
        };
        let command = code.split_whitespace().next().unwrap_or_default();
        if !matches!(command, "G2" | "G3") {
            return vec![block];
        }
        let Some(deltas) = arc::deltas(
            command,
            code,
            arc::ArcMotion {
                start,
                end: self.position,
                e_delta: self.e_position - start_e,
                feedrate: self.feedrate,
                gcode_flavor: self.gcode_flavor,
            },
        ) else {
            return vec![block];
        };
        deltas
            .into_iter()
            .filter_map(|delta| self.segment_block(delta))
            .collect()
    }

    pub(super) fn motion(&mut self, code: &str) -> Option<MotionBlock> {
        if code == "G90" {
            self.relative = false;
            return None;
        }
        if code == "G91" {
            self.relative = true;
            return None;
        }
        if code == "M82" {
            self.e_relative = false;
            return None;
        }
        if code == "M83" {
            self.e_relative = true;
            return None;
        }
        if code.split_whitespace().next() == Some("G92") {
            self.set_position(code);
            return None;
        }
        if code.starts_with("M201") {
            self.update_axis_limits(code, true);
            return None;
        }
        if code.starts_with("M203") {
            self.update_axis_limits(code, false);
            return None;
        }
        if code.starts_with("M204") {
            if let Some(value) = word(code, 'S') {
                self.acceleration = clamp(value, self.max_print_acceleration);
                self.travel_acceleration = clamp(value, self.max_travel_acceleration);
                self.retract_acceleration = clamped_word(
                    code,
                    'T',
                    self.retract_acceleration,
                    self.max_retract_acceleration,
                );
            } else {
                self.acceleration =
                    clamped_word(code, 'P', self.acceleration, self.max_print_acceleration);
                self.retract_acceleration = clamped_word(
                    code,
                    'R',
                    self.retract_acceleration,
                    self.max_retract_acceleration,
                );
                self.travel_acceleration = clamped_word(
                    code,
                    'T',
                    self.travel_acceleration,
                    self.max_travel_acceleration,
                );
            }
            return None;
        }
        if code.starts_with("M205") {
            for (axis, letter) in ['X', 'Y', 'Z', 'E'].into_iter().enumerate() {
                self.jerk[axis] = word(code, letter).unwrap_or(self.jerk[axis]);
            }
            return None;
        }
        if code.split_whitespace().next() == Some("SET_VELOCITY_LIMIT") {
            // Klipper: ACCEL applies to print and travel moves alike
            // (`GCodeProcessor.cpp:5269-5304`).
            if let Some(value) = assignment(code, "ACCEL=") {
                let value = clamp(value, self.max_print_acceleration);
                self.acceleration = value;
                self.travel_acceleration = clamp(value, self.max_travel_acceleration);
            }
            if let Some(value) = assignment(code, "SQUARE_CORNER_VELOCITY=") {
                self.jerk[0] = value;
                self.jerk[1] = value;
            }
            // Upstream also updates machine_max_speed_x/y from `VELOCITY=`
            // (`GCodeProcessor.cpp:5301-5313`); Ares' planner has no
            // per-axis speed limit yet, so the field is not tracked.
            return None;
        }
        let command = code.split_whitespace().next()?;
        if command == "G28" {
            let has_axis = ['X', 'Y', 'Z'].iter().any(|&axis| code.contains(axis));
            let mut homing = String::from("G1");
            let axes = ['X', 'Y', 'Z']
                .into_iter()
                .filter(|axis| !has_axis || code.contains(*axis));
            for axis in axes {
                homing.push(' ');
                homing.push(axis);
                homing.push('0');
            }
            return self.motion(&homing);
        }
        if !matches!(command, "G0" | "G1" | "G2" | "G3") {
            return None;
        }
        if let Some(value) = word(code, 'F') {
            self.feedrate = f64::from(value as f32 * MMMIN_TO_MMSEC);
        }
        let old = self.position;
        let mut next = old;
        for (axis, letter) in ['X', 'Y', 'Z'].into_iter().enumerate() {
            let Some(value) = word(code, letter) else {
                continue;
            };
            let offset = match self.relative {
                true => value,
                false => value - old[axis],
            };
            next[axis] = old[axis] + offset;
        }
        let old_e = self.e_position;
        let e_delta = word(code, 'E').map_or(0.0, |value| {
            if self.e_relative {
                value
            } else {
                value - old_e
            }
        });
        self.e_position = old_e + e_delta;
        self.position = next;
        // Upstream always tracks the position; a non-positive feedrate only
        // suppresses the motion block (no time contribution).
        if self.feedrate <= 0.0 {
            return None;
        }
        let mut delta = [
            next[0] - old[0],
            next[1] - old[1],
            next[2] - old[2],
            e_delta,
        ];
        let xyz_distance = norm([delta[0], delta[1], delta[2], 0.0]);
        let e_only = xyz_distance <= f64::EPSILON;
        let mut distance = if e_only { e_delta.abs() } else { xyz_distance };
        if matches!(command, "G2" | "G3") {
            let i = word(code, 'I').unwrap_or(0.0);
            let j = word(code, 'J').unwrap_or(0.0);
            let radius = (i * i + j * j).sqrt();
            let same_xy = delta[0].abs() <= f64::EPSILON && delta[1].abs() <= f64::EPSILON;
            let mut sweep = if same_xy {
                0.0
            } else {
                let start = (-j).atan2(-i);
                let end = (delta[1] - j).atan2(delta[0] - i);
                let mut sweep = end - start;
                sweep = match (command, sweep >= 0.0, sweep <= 0.0) {
                    ("G2", true, _) => sweep - 2.0 * PI,
                    ("G3", _, true) => sweep + 2.0 * PI,
                    _ => sweep,
                };
                sweep
            };
            let turns = word(code, 'P').unwrap_or(0.0);
            let turns = if same_xy && turns == 0.0 { 1.0 } else { turns } * 2.0 * PI;
            sweep += if command == "G2" { -turns } else { turns };
            distance = (radius * sweep.abs()).hypot(delta[2]);
            delta[0] = sweep.cos() * radius;
            delta[1] = sweep.sin() * radius;
        }
        self.position = next;
        if distance <= f64::EPSILON {
            return None;
        }
        let has_xy = delta[0] != 0.0 || delta[1] != 0.0;
        let mut acceleration = if e_only {
            self.retract_acceleration
        } else if self.wiping || (e_delta > 0.0 && has_xy) {
            self.acceleration
        } else {
            self.travel_acceleration
        };
        let mut speed = self.feedrate;
        for (axis, delta) in delta.iter().enumerate() {
            let ratio = (delta / distance).abs();
            if ratio == 0.0 {
                continue;
            }
            let max_feedrate = self.max_feedrate[axis];
            if max_feedrate > 0.0 {
                speed = speed.min(max_feedrate / ratio);
            }
            let max_acceleration = self.max_acceleration[axis];
            acceleration = acceleration.min(max_acceleration / ratio);
        }
        Some(MotionBlock {
            distance,
            speed,
            acceleration,
            centripetal_acceleration: self.acceleration.max(1.0),
            jerk: self.effective_jerk(),
            direction: scale(delta, 1.0 / distance),
            kind: if !self.wiping && e_only && e_delta > 0.0 {
                MotionKind::Unretract
            } else {
                MotionKind::Regular
            },
            e_only,
        })
    }

    fn segment_block(&self, delta: [f64; 4]) -> Option<MotionBlock> {
        let xyz_distance = norm([delta[0], delta[1], delta[2], 0.0]);
        let e_only = xyz_distance <= f64::EPSILON;
        let distance = if e_only { delta[3].abs() } else { xyz_distance };
        if distance <= f64::EPSILON {
            return None;
        }
        let has_xy = delta[0] != 0.0 || delta[1] != 0.0;
        let mut acceleration = if e_only {
            self.retract_acceleration
        } else if self.wiping || (delta[3] > 0.0 && has_xy) {
            self.acceleration
        } else {
            self.travel_acceleration
        };
        let mut speed = self.feedrate;
        for (axis, delta) in delta.iter().enumerate() {
            let ratio = (delta / distance).abs();
            if ratio == 0.0 {
                continue;
            }
            let max_feedrate = self.max_feedrate[axis];
            if max_feedrate > 0.0 {
                speed = speed.min(max_feedrate / ratio);
            }
            let max_acceleration = self.max_acceleration[axis];
            acceleration = acceleration.min(max_acceleration / ratio);
        }
        Some(MotionBlock {
            distance,
            speed,
            acceleration,
            centripetal_acceleration: self.acceleration.max(1.0),
            jerk: self.effective_jerk(),
            direction: scale(delta, 1.0 / distance),
            kind: if !self.wiping && e_only && delta[3] > 0.0 {
                MotionKind::Unretract
            } else {
                MotionKind::Regular
            },
            e_only,
        })
    }
}
