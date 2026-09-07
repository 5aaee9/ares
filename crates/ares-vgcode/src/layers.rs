// Ports rendering-neutral data from AGPL-licensed OrcaSlicer `src/libvgcode/src/Layers.hpp` and `src/Layers.cpp`.

use crate::{GCodeExtrusionRole, Interval, MoveType, PathVertex, Range, TimeMode};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layers {
    items: Vec<Item>,
    view_range: Range,
}

impl Layers {
    pub fn update(&mut self, vertex: &PathVertex, vertex_id: u32) {
        if self.items.is_empty() || vertex.layer_id as usize == self.items.len() {
            assert_eq!(vertex.layer_id as usize, self.items.len());
            let mut item = Item::default();
            if is_z_capturing_extrusion(vertex) {
                item.z = vertex.position[2];
            }
            item.range.set(vertex_id as usize, vertex_id as usize);
            item.times = vertex.times;
            item.contains_colorprint_options |= is_colorprint_option(vertex);
            self.items.push(item);
        } else {
            let item = self.items.last_mut().expect("layers has a last item");
            if is_z_capturing_extrusion(vertex) && item.z != vertex.position[2] {
                item.z = vertex.position[2];
            }
            item.range.set_max(vertex_id as usize);
            for i in 0..TimeMode::COUNT {
                item.times[i] += vertex.times[i];
            }
            item.contains_colorprint_options |= is_colorprint_option(vertex);
        }
    }

    pub fn reset(&mut self) {
        self.items.clear();
        self.view_range.reset();
    }

    pub fn empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn count(&self) -> usize {
        self.items.len()
    }

    pub fn get_times(&self, mode: TimeMode) -> Vec<f32> {
        self.items
            .iter()
            .map(|item| item.times[mode.index()])
            .collect()
    }

    pub fn get_zs(&self) -> Vec<f32> {
        self.items.iter().map(|item| item.z).collect()
    }

    pub fn get_layer_time(&self, mode: TimeMode, layer_id: usize) -> f32 {
        self.items
            .get(layer_id)
            .map_or(0.0, |item| item.times[mode.index()])
    }

    pub fn get_layer_z(&self, layer_id: usize) -> f32 {
        self.items.get(layer_id).map_or(0.0, |item| item.z)
    }

    pub fn get_layer_id_at(&self, z: f32) -> usize {
        let mut first = 0;
        let mut count = self.items.len();
        while count > 0 {
            let step = count / 2;
            let index = first + step;
            if self.items[index].z < z {
                count = step;
            } else {
                first = index + 1;
                count -= step + 1;
            }
        }
        first
    }

    pub fn get_view_range(&self) -> Interval {
        self.view_range.get()
    }

    pub fn set_view_range_interval(&mut self, range: Interval) {
        self.set_view_range(range[0], range[1]);
    }

    pub fn set_view_range(&mut self, min: usize, max: usize) {
        self.view_range.set(min, max);
    }

    pub fn layer_contains_colorprint_options(&self, layer_id: usize) -> bool {
        self.items
            .get(layer_id)
            .is_some_and(|item| item.contains_colorprint_options)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Item {
    z: f32,
    range: Range,
    times: [f32; TimeMode::COUNT],
    contains_colorprint_options: bool,
}

fn is_z_capturing_extrusion(vertex: &PathVertex) -> bool {
    vertex.move_type == MoveType::Extrude && vertex.role != GCodeExtrusionRole::Custom
}

fn is_colorprint_option(vertex: &PathVertex) -> bool {
    matches!(
        vertex.move_type,
        MoveType::PausePrint | MoveType::CustomGCode
    )
}

#[cfg(test)]
mod tests;
