use super::{MotionBlock, MotionKind};
mod trapezoid;
use trapezoid::block_time;

#[derive(Clone, Copy)]
struct PlannedBlock {
    distance: f32,
    acceleration: f32,
    cruise: f32,
    entry: f32,
    exit: f32,
    max_entry: f32,
    safe: f32,
    time: f32,
    nominal_length: bool,
    recalculate: bool,
    direction: [f32; 4],
    axis_feedrate: [f32; 4],
}

struct JunctionInput {
    direction: [f32; 4],
    jerk: [f32; 4],
    cruise: f32,
    axis_feedrate: [f32; 4],
    safe: f32,
}

pub(crate) struct RollingPlanner {
    blocks: Vec<PlannedBlock>,
    previous: Option<PlannedBlock>,
}

impl RollingPlanner {
    pub(crate) fn new() -> Self {
        Self {
            blocks: Vec::new(),
            previous: None,
        }
    }

    pub(crate) fn push(&mut self, block: &MotionBlock) -> Vec<f64> {
        let planned = prepare(block, self.previous, !self.blocks.is_empty());
        self.previous = Some(planned);
        self.blocks.push(planned);
        if self.blocks.len() > 256 {
            self.process(64)
        } else {
            Vec::new()
        }
    }

    pub(crate) fn flush(&mut self) -> Vec<f64> {
        if self.blocks.len() < 2 {
            Vec::new()
        } else {
            self.process(0)
        }
    }

    pub(crate) fn finish(&mut self) -> Vec<f64> {
        if self.blocks.is_empty() {
            Vec::new()
        } else {
            self.process(0)
        }
    }

    fn process(&mut self, keep: usize) -> Vec<f64> {
        plan(&mut self.blocks);
        let process_len = self.blocks.len() - keep;
        let times = self.blocks[..process_len]
            .iter()
            .map(|block| f64::from(block.time))
            .collect();
        self.blocks.drain(..process_len);
        if let Some(first) = self.blocks.first_mut() {
            first.max_entry = first.entry;
        }
        times
    }
}

#[cfg(test)]
pub(super) fn planned_times(blocks: &[MotionBlock]) -> Vec<f64> {
    let mut planner = RollingPlanner::new();
    let mut times = Vec::with_capacity(blocks.len());
    for block in blocks {
        times.extend(planner.push(block));
    }
    times.extend(planner.finish());
    times
}

/// `TimeBlock::calculate_trapezoid()` + `time()` over one fixed profile
/// (GCodeProcessor.cpp:255-274, GCodeProcessor.hpp:473-476).
#[cfg(test)]
pub(super) fn planned_trapezoid_time(
    distance: f32,
    cruise: f32,
    entry: f32,
    exit: f32,
    acceleration: f32,
) -> f32 {
    block_time(PlannedBlock {
        distance,
        acceleration,
        cruise,
        entry,
        exit,
        max_entry: entry,
        safe: exit,
        time: 0.0,
        nominal_length: false,
        recalculate: false,
        direction: [0.0; 4],
        axis_feedrate: [0.0; 4],
    })
}

fn plan(blocks: &mut [PlannedBlock]) {
    if blocks.is_empty() {
        return;
    }

    for index in (0..blocks.len().saturating_sub(1)).rev() {
        let next = blocks[index + 1];
        let current = &mut blocks[index];
        if current.entry != current.max_entry || next.recalculate {
            let entry = if current.nominal_length {
                current.max_entry
            } else {
                current.max_entry.min(max_allowable_speed(
                    current.acceleration,
                    next.entry,
                    current.distance,
                ))
            };
            if current.entry != entry {
                current.entry = entry;
                current.recalculate = true;
            }
        }
    }
    for index in 1..blocks.len() {
        let previous = blocks[index - 1];
        let current = &mut blocks[index];
        if !previous.nominal_length && previous.entry < current.entry {
            let entry =
                max_allowable_speed(previous.acceleration, previous.entry, previous.distance);
            if entry < current.entry {
                current.entry = entry;
                current.recalculate = true;
            }
        }
    }
    for index in 0..blocks.len().saturating_sub(1) {
        if blocks[index].recalculate || blocks[index + 1].recalculate {
            blocks[index].exit = blocks[index + 1].entry;
            blocks[index].time = block_time(blocks[index]);
            blocks[index].recalculate = false;
        }
    }
    let last = blocks.len() - 1;
    blocks[last].exit = blocks[last].safe;
    blocks[last].time = block_time(blocks[last]);
    blocks[last].recalculate = false;
}

fn prepare(
    block: &MotionBlock,
    previous: Option<PlannedBlock>,
    has_junction: bool,
) -> PlannedBlock {
    if block.kind == MotionKind::ToolChange {
        return PlannedBlock {
            distance: 0.0,
            acceleration: 0.0,
            cruise: 0.0,
            entry: 0.0,
            exit: 0.0,
            max_entry: 0.0,
            safe: 0.0,
            time: 0.0,
            nominal_length: false,
            recalculate: false,
            direction: [0.0; 4],
            axis_feedrate: [0.0; 4],
        };
    }
    let distance = block.distance as f32;
    let acceleration = block.acceleration as f32;
    let centripetal_acceleration = block.centripetal_acceleration as f32;
    let direction = block.direction.map(|component| component as f32);
    let jerk = block.jerk.map(|component| component as f32);
    let mut cruise = block.speed as f32;
    if let Some(previous) = previous {
        let previous_xy = xy_unit(previous.direction);
        let current_xy = xy_unit(direction);
        if let (Some(previous_xy), Some(current_xy)) = (previous_xy, current_xy) {
            let difference = (current_xy[0] - previous_xy[0]).hypot(current_xy[1] - previous_xy[1]);
            if difference < 0.5 && difference > 0.000_01 {
                let dot = previous_xy[0] * current_xy[0] + previous_xy[1] * current_xy[1];
                let cross = previous_xy[0] * current_xy[1] - previous_xy[1] * current_xy[0];
                let angle = cross.atan2(dot);
                let sin_half = ((1.0 - angle.cos()) * 0.5).sqrt();
                let xy_distance = distance * direction[0].hypot(direction[1]);
                let radius = xy_distance * 0.5 / sin_half;
                cruise = cruise.min((centripetal_acceleration * radius).sqrt());
            }
        }
    }

    let axis_feedrate = direction.map(|direction| cruise * direction);
    let safe = axis_feedrate
        .iter()
        .zip(jerk)
        .filter(|(feedrate, jerk)| feedrate.abs() > *jerk)
        .fold(cruise, |safe, (_, jerk)| safe.min(jerk));
    let max_entry = if has_junction {
        junction_speed(
            previous.expect("queued block has a predecessor"),
            JunctionInput {
                direction,
                jerk,
                cruise,
                axis_feedrate,
                safe,
            },
        )
    } else {
        safe
    };
    let allowable = max_allowable_speed(acceleration, safe, distance);
    let entry = max_entry.min(allowable);

    let mut planned = PlannedBlock {
        distance,
        acceleration,
        cruise,
        entry,
        exit: safe,
        max_entry,
        safe,
        time: 0.0,
        nominal_length: cruise <= allowable,
        recalculate: true,
        direction,
        axis_feedrate,
    };
    planned.time = block_time(planned);
    planned
}

fn junction_speed(previous: PlannedBlock, current: JunctionInput) -> f32 {
    let JunctionInput {
        direction,
        jerk,
        cruise,
        axis_feedrate,
        safe,
    } = current;
    if previous.cruise <= 0.000_1 {
        return safe;
    }
    let previous_larger = previous.cruise > cruise;
    let smaller_factor = if previous_larger {
        cruise / previous.cruise
    } else {
        previous.cruise / cruise
    };
    let mut maximum = previous.cruise.min(cruise);
    let mut factor = 1.0;
    let mut limited = false;

    let mut exit = previous
        .direction
        .map(|direction| previous.cruise * direction);
    if previous_larger {
        exit.iter_mut().for_each(|value| *value *= smaller_factor);
    }
    let mut difference = [
        (cruise * direction[0] - exit[0]).abs(),
        (cruise * direction[1] - exit[1]).abs(),
        (cruise * direction[2] - exit[2]).abs(),
    ];
    for axis in 0..3 {
        if difference[axis] > jerk[axis] {
            factor *= jerk[axis] / difference[axis];
            difference.iter_mut().for_each(|value| *value *= factor);
            limited = true;
        }
    }

    let mut e_exit = previous.axis_feedrate[3];
    let mut e_entry = axis_feedrate[3];
    if previous_larger {
        e_exit *= smaller_factor;
    }
    if limited {
        e_exit *= factor;
        e_entry *= factor;
    }
    let e_jerk = axis_jerk(e_exit, e_entry);
    if e_jerk > jerk[3] {
        factor *= jerk[3] / e_jerk;
        limited = true;
    }
    if limited {
        maximum *= factor;
    }
    if previous.safe > maximum * 0.99 && safe > maximum * 0.99 {
        safe
    } else {
        maximum
    }
}

fn axis_jerk(exit: f32, entry: f32) -> f32 {
    if exit > entry {
        if entry > 0.0 || exit < 0.0 {
            exit - entry
        } else {
            exit.max(-entry)
        }
    } else if entry < 0.0 || exit > 0.0 {
        entry - exit
    } else {
        (-exit).max(entry)
    }
}

fn xy_unit(direction: [f32; 4]) -> Option<[f32; 2]> {
    let norm = direction[0].hypot(direction[1]);
    (norm > 0.0).then(|| [direction[0] / norm, direction[1] / norm])
}

fn max_allowable_speed(acceleration: f32, target: f32, distance: f32) -> f32 {
    (target * target + 2.0 * acceleration * distance).sqrt()
}
