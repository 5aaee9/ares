//! `TimeBlock::calculate_trapezoid()` + `time()` port
//! (GCodeProcessor.cpp:127-143, 255-274; GCodeProcessor.hpp:436-476).
use super::PlannedBlock;

// `GCodeProcessor.cpp:128-142`: `speed_from_distance` clamps its radicand
// so numerical error can never produce a negative square root.
fn speed_from_distance(initial_feedrate: f32, distance: f32, acceleration: f32) -> f32 {
    (initial_feedrate * initial_feedrate + 2.0 * acceleration * distance)
        .max(0.0)
        .sqrt()
}

fn acceleration_time_from_distance(initial_feedrate: f32, distance: f32, acceleration: f32) -> f32 {
    if acceleration != 0.0 {
        (speed_from_distance(initial_feedrate, distance, acceleration) - initial_feedrate)
            / acceleration
    } else {
        0.0
    }
}

// `GCodeProcessor.cpp:255-274` `TimeBlock::calculate_trapezoid` plus
// `GCodeProcessor.hpp:439-446,473-476` `time()`: every phase time derives
// from an accel/decel DISTANCE, and a distance whose velocity difference
// points the wrong way clamps to zero, so it contributes zero time, never a
// negative one.
pub(super) fn block_time(block: PlannedBlock) -> f32 {
    let acceleration = block.acceleration;
    let accelerate_distance = if acceleration == 0.0 {
        0.0
    } else {
        ((block.cruise * block.cruise - block.entry * block.entry) / (2.0 * acceleration)).max(0.0)
    };
    let decelerate_distance = if acceleration == 0.0 {
        0.0
    } else {
        ((block.cruise * block.cruise - block.exit * block.exit) / (2.0 * acceleration)).max(0.0)
    };
    let cruise_distance = block.distance - accelerate_distance - decelerate_distance;
    let (accelerate_until, cruise_feedrate, decelerate_after) = if cruise_distance < 0.0 {
        // Not enough distance to reach cruise: abort acceleration at the
        // accel/decel intersection.
        let accelerate_until = if acceleration == 0.0 {
            0.0
        } else {
            ((2.0 * acceleration * block.distance - block.entry * block.entry
                + block.exit * block.exit)
                / (4.0 * acceleration))
                .clamp(0.0, block.distance)
        };
        (
            accelerate_until,
            speed_from_distance(block.entry, accelerate_until, acceleration),
            accelerate_until,
        )
    } else {
        (
            accelerate_distance,
            block.cruise,
            accelerate_distance + cruise_distance,
        )
    };
    acceleration_time_from_distance(block.entry, accelerate_until, acceleration)
        + if cruise_feedrate != 0.0 {
            (decelerate_after - accelerate_until) / cruise_feedrate
        } else {
            0.0
        }
        + acceleration_time_from_distance(
            cruise_feedrate,
            block.distance - decelerate_after,
            -acceleration,
        )
}
