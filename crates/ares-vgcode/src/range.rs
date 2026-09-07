// Ports rendering-neutral data from AGPL-licensed OrcaSlicer `src/libvgcode/src/Range.hpp`, `src/Range.cpp`, `src/ViewRange.hpp`, and `src/ViewRange.cpp`.

use crate::Interval;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Range {
    range: Interval,
}

impl Range {
    pub fn new(min: usize, max: usize) -> Self {
        let mut range = Self::default();
        range.set(min, max);
        range
    }

    pub const fn get(&self) -> Interval {
        self.range
    }

    pub fn set_range(&mut self, other: &Range) {
        self.range = other.range;
    }

    pub fn set_interval(&mut self, range: Interval) {
        self.set(range[0], range[1]);
    }

    pub fn set(&mut self, min: usize, max: usize) {
        if max < min {
            self.range = [max, min];
        } else {
            self.range = [min, max];
        }
    }

    pub const fn get_min(&self) -> usize {
        self.range[0]
    }

    pub fn set_min(&mut self, min: usize) {
        self.set(min, self.range[1]);
    }

    pub const fn get_max(&self) -> usize {
        self.range[1]
    }

    pub fn set_max(&mut self, max: usize) {
        self.set(self.range[0], max);
    }

    pub fn clamp(&self, other: &mut Range) {
        other.range[0] = other.range[0].clamp(self.range[0], self.range[1]);
        other.range[1] = other.range[1].clamp(self.range[0], self.range[1]);
    }

    pub fn reset(&mut self) {
        self.range = [0, 0];
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ViewRange {
    full: Range,
    enabled: Range,
    visible: Range,
}

impl ViewRange {
    pub const fn get_full(&self) -> Interval {
        self.full.get()
    }

    pub fn set_full_range(&mut self, other: &Range) {
        self.set_full_interval(other.get());
    }

    pub fn set_full_interval(&mut self, range: Interval) {
        self.set_full(range[0], range[1]);
    }

    pub fn set_full(&mut self, min: usize, max: usize) {
        self.full.set(min, max);
        self.full.clamp(&mut self.enabled);
        self.enabled.clamp(&mut self.visible);
    }

    pub const fn get_enabled(&self) -> Interval {
        self.enabled.get()
    }

    pub fn set_enabled_range(&mut self, other: &Range) {
        self.set_enabled_interval(other.get());
    }

    pub fn set_enabled_interval(&mut self, range: Interval) {
        self.set_enabled(range[0], range[1]);
    }

    pub fn set_enabled(&mut self, min: usize, max: usize) {
        self.enabled.set(min, max);
        self.enabled.clamp(&mut self.visible);
    }

    pub const fn get_visible(&self) -> Interval {
        self.visible.get()
    }

    pub fn set_visible_range(&mut self, other: &Range) {
        self.set_visible_interval(other.get());
    }

    pub fn set_visible_interval(&mut self, range: Interval) {
        self.set_visible(range[0], range[1]);
    }

    pub fn set_visible(&mut self, min: usize, max: usize) {
        self.visible.set(min, max);
        self.enabled.clamp(&mut self.visible);
    }

    pub fn reset(&mut self) {
        self.full.reset();
        self.enabled.reset();
        self.visible.reset();
    }
}

#[cfg(test)]
mod tests;
