//! The merged layer-chunk schedule: `collect_layers_to_print(Print)`
//! (`GCode.cpp:1835-1870`) plus the per-chunk chained object order
//! (`ShortestPath.cpp:2015-2042`, seeded and reversed per
//! `GCode.cpp:5114-5131`).
use super::object::ObjectLabels;
use super::object_order;
use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;

pub(super) struct Schedule {
    /// Cumulative print z per object per layer.
    pub(super) per_object_z: Vec<Vec<f32>>,
    /// `(z, object_index, layer_index)` for every object layer, sorted by
    /// print z (`collect_layers_to_print`'s ordering).
    pub(super) merged: Vec<(f32, usize, usize)>,
    /// Chained print order over the print objects, reversed
    /// (`GCode.cpp:5120-5125`).
    pub(super) print_position: Vec<usize>,
    pub(super) labels: Vec<Option<ObjectLabels>>,
    pub(super) object_layer_counts: Vec<usize>,
    pub(super) last_entry: Option<(usize, usize)>,
}

pub(super) fn build(
    traversal: &PreparedPostClassicTraversal,
    objects: &[Vec<crate::project_slice::island_print_order::OrderedExtrusionLayer>],
) -> Schedule {
    let object_count = objects.len();
    let per_object_z: Vec<Vec<f32>> = traversal
        .objects
        .iter()
        .map(|object| {
            object
                .records
                .iter()
                .filter_map(|record| record.as_ref())
                .scan(0.0_f64, |precise, record| {
                    *precise += record.layer_height;
                    Some(*precise as f32)
                })
                .collect()
        })
        .collect();
    let mut merged: Vec<(f32, usize, usize)> = objects
        .iter()
        .enumerate()
        .flat_map(|(object_index, object)| {
            let z = &per_object_z[object_index];
            (0..object.len()).map(move |layer_index| {
                (
                    z.get(layer_index).copied().unwrap_or(f32::MAX),
                    object_index,
                    layer_index,
                )
            })
        })
        .collect();
    merged.sort_by(|&(a, _, _), &(b, _, _)| a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal));
    let centers: Vec<(f64, f64)> = (0..object_count)
        .map(|object_index| {
            let (source_object_index, _) = traversal.objects[object_index]
                .predecessor
                .predecessor
                .predecessor
                .predecessor
                .object
                .identity();
            super::footprint::object_center(traversal, source_object_index).unwrap_or((0.0, 0.0))
        })
        .collect();
    // `GCode.cpp:5114-5125`: the per-layer instance ordering chains from
    // the wipe-tower point and REVERSES the path. The Point constructor
    // truncates the millimetre config values into scaled integers, so the
    // seed sits a few microns from the origin for every sane config
    // (`GCode.cpp:5115`, defaults `wipe_tower_x = 15`, `wipe_tower_y =
    // 220`).
    let mut print_order = object_order::chain_instance_order(&centers, (15, 220));
    print_order.reverse();
    let print_position = {
        let mut position = vec![0_usize; object_count];
        for (rank, &object_index) in print_order.iter().enumerate() {
            position[object_index] = rank;
        }
        position
    };
    let labels: Vec<Option<ObjectLabels>> = (0..object_count)
        .map(|object_index| ObjectLabels::from_traversal(traversal, object_index))
        .collect();
    let object_layer_counts: Vec<usize> = objects.iter().map(|object| object.len()).collect();
    let last_entry = merged
        .last()
        .map(|&(_, object_index, layer_index)| (object_index, layer_index));
    Schedule {
        per_object_z,
        merged,
        print_position,
        labels,
        object_layer_counts,
        last_entry,
    }
}
