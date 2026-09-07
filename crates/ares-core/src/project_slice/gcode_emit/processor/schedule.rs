//! `GCodeProcessor::calculate_time` planner flush and delay accounting.

use super::motion::{MotionBlock, MotionKind, RollingPlanner};
use std::collections::VecDeque;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum DelayTarget {
    Any,
    ToolChange,
}

#[derive(Clone, Copy)]
pub(super) struct PendingDelay {
    pub(super) target: DelayTarget,
    pub(super) seconds: f32,
}

pub(super) struct FlushEvent {
    pub(super) block_count: usize,
    pub(super) delay: Option<PendingDelay>,
}

pub(super) fn scheduled_times(blocks: &[MotionBlock], events: &[FlushEvent]) -> (Vec<f64>, f64) {
    let mut planner = RollingPlanner::new();
    let mut times = vec![0.0; blocks.len()];
    let mut pending = VecDeque::new();
    let mut pushed = 0;
    let mut emitted = 0;

    for event in events {
        while pushed < event.block_count {
            let batch = planner.push(&blocks[pushed]);
            pushed += 1;
            record_batch(blocks, &mut times, &mut emitted, batch, &mut pending);
        }
        if let Some(delay) = event.delay.filter(|delay| delay.seconds > 0.0) {
            if let Some(last) = pending
                .back_mut()
                .filter(|last| last.target == delay.target)
            {
                last.seconds += delay.seconds;
            } else {
                pending.push_back(delay);
            }
        }
        let batch = planner.flush();
        record_batch(blocks, &mut times, &mut emitted, batch, &mut pending);
    }
    while pushed < blocks.len() {
        let batch = planner.push(&blocks[pushed]);
        pushed += 1;
        record_batch(blocks, &mut times, &mut emitted, batch, &mut pending);
    }
    let batch = planner.finish();
    record_batch(blocks, &mut times, &mut emitted, batch, &mut pending);

    let trailing_delay = pending
        .into_iter()
        .fold(0.0_f32, |total, delay| total + delay.seconds);
    (times, f64::from(trailing_delay))
}

fn record_batch(
    blocks: &[MotionBlock],
    times: &mut [f64],
    emitted: &mut usize,
    mut batch: Vec<f64>,
    pending: &mut VecDeque<PendingDelay>,
) {
    for (block, time) in blocks[*emitted..].iter().zip(&mut batch) {
        let Some(delay) = pending.front() else {
            break;
        };
        if delay.target == DelayTarget::Any
            || delay.target == DelayTarget::ToolChange && block.kind == MotionKind::ToolChange
        {
            *time = f64::from(*time as f32 + delay.seconds);
            pending.pop_front();
        }
    }
    let end = *emitted + batch.len();
    times[*emitted..end].copy_from_slice(&batch);
    *emitted = end;
}
