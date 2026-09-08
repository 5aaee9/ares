use crate::{
    geometry::CoordinateScale,
    project_slice::{
        island_print_order::{IslandPrintEntity, OrderedExtrusionLayer},
        perimeters::classic::{
            entity_collections::{ExtrusionEntity, ExtrusionEntityCollection},
            materialize::{ExtrusionPath, ExtrusionRole},
            traversal::PreparedPostClassicTraversal,
        },
        seam_candidates::{self, LayerSeamCandidates, SeamPerimeter},
    },
};

use super::{LayerPlan, PerimeterChoice, context, is_better};
use crate::project_slice::seam_placement::{candidate_penalty, visibility};

pub(in crate::project_slice::seam_placement) fn prepare(
    layers: (&[OrderedExtrusionLayer], &[f32]),
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    nozzle_diameter: f32,
    visibility: &visibility::GlobalVisibility,
) -> Vec<LayerPlan> {
    let (layers, layer_zs) = layers;
    let mut plans = layers
        .iter()
        .zip(layer_zs)
        .map(|(layer, &z)| prepare_layer(layer, z, traversal.scale, nozzle_diameter, visibility))
        .collect::<Vec<_>>();
    let input = &traversal.objects[object_index]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object;
    let layer_slices = input.object.as_parts().1;
    context::populate(
        &mut plans,
        &embedding_layers(traversal, object_index, layers.len()),
        layer_slices,
        traversal.scale,
    );
    for plan in &mut plans {
        dump_seam_candidates(plan);
        plan.choices = plan
            .candidates
            .perimeters
            .iter()
            .map(|perimeter| PerimeterChoice {
                seam_index: best_candidate(perimeter, plan),
                final_position: None,
                finalized: false,
            })
            .collect();
    }
    plans
}

fn dump_seam_candidates(plan: &LayerPlan) {
    let Ok(path) = std::env::var("ARES_DUMP_SEAM") else {
        return;
    };
    use std::io::Write;
    let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    else {
        return;
    };
    for perimeter in &plan.candidates.perimeters {
        let _ = writeln!(
            file,
            "PERIM first={} n={}",
            perimeter.start_index,
            perimeter.end_index - perimeter.start_index
        );
        for index in perimeter.start_index..perimeter.end_index {
            let position = plan.positions[index];
            let _ = writeln!(
                file,
                "C {:.6} {:.6} {:.6} vis={:.8} overhang={:.8}",
                position.x, position.y, position.z, plan.scores[index], plan.overhangs[index]
            );
        }
    }
}

fn embedding_layers(
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    layer_count: usize,
) -> Vec<bool> {
    let object = &traversal.objects[object_index];
    let input = &object
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object;
    let mut region_counts = vec![0_u16; layer_count];
    for (input, perimeters) in input.records.iter().zip(&object.records) {
        let (Some(input), Some(perimeters)) = (input, perimeters) else {
            continue;
        };
        if perimeters
            .surfaces
            .iter()
            .any(|surface| !surface.roots.is_empty())
        {
            region_counts[input.planned_layer_index] += 1;
        }
    }
    region_counts.into_iter().map(|count| count > 1).collect()
}

fn prepare_layer(
    layer: &OrderedExtrusionLayer,
    z: f32,
    scale: CoordinateScale,
    nozzle_diameter: f32,
    visibility: &visibility::GlobalVisibility,
) -> LayerPlan {
    let collections = layer
        .islands
        .iter()
        .flat_map(|island| &island.entities)
        .filter_map(|entity| match entity {
            IslandPrintEntity::Perimeter(collection) => Some(collection),
            IslandPrintEntity::Fill(_)
            | IslandPrintEntity::FillCollection(_)
            | IslandPrintEntity::Thin(_) => None,
        })
        .collect::<Vec<_>>();
    let mut source_order = (0..collections.len()).collect::<Vec<_>>();
    source_order.sort_unstable_by_key(|&index| collections[index].source_order);
    if let Ok(path) = std::env::var("ARES_DUMP_SEAM") {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(file, "LAYER z={z}");
            for collection in &collections {
                let _ = writeln!(
                    file,
                    "COLL n={} roles={}",
                    collection.entities.len(),
                    collection
                        .entities
                        .iter()
                        .map(|entity| {
                            let first_path = match entity {
                                ExtrusionEntity::Loop(ordered) => {
                                    ordered.extrusion_loop.paths.first()
                                }
                                ExtrusionEntity::MultiPath(multi_path) => multi_path.paths.first(),
                            };
                            format!(
                                "{:?}/{}",
                                first_path.map(|path| path.role),
                                first_path.map(|path| path.width).unwrap_or_default()
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(",")
                );
            }
        }
    }
    let regions = source_order
        .iter()
        .map(|&index| {
            let collection = collections[index];
            let external_flow_width = collection
                .entities
                .iter()
                .flat_map(|entity| match entity {
                    ExtrusionEntity::Loop(ordered) => &ordered.extrusion_loop.paths,
                    ExtrusionEntity::MultiPath(multi_path) => &multi_path.paths,
                })
                .find(|path| path.role == ExtrusionRole::ExternalPerimeter)
                .map_or(0.0, |path| path.width);
            seam_candidates::RegionPerimeters {
                collection: Some(collection),
                external_flow_width,
            }
        })
        .collect::<Vec<_>>();
    let candidates = seam_candidates::generate_regions(&regions, z, scale, nozzle_diameter);
    let candidate_count = candidates.points.len();
    let scores = candidates
        .points
        .iter()
        .map(|candidate| candidate_penalty(candidate, visibility))
        .collect::<Vec<_>>();
    let positions = candidates
        .points
        .iter()
        .map(|candidate| {
            let position = candidate.position;
            super::Vec3::new(position.x, position.y, position.z)
        })
        .collect::<Vec<_>>();
    let point_tree = super::PointKdTree::new(&positions);
    let association = PerimeterAssociation {
        candidates: &candidates,
        positions: &positions,
        point_tree: &point_tree,
        z,
        scale,
    };
    let collection_perimeters = collections
        .iter()
        .map(|collection| association.collection_map(collection))
        .collect();
    LayerPlan {
        candidates,
        choices: Vec::new(),
        collection_perimeters,
        scores,
        z,
        overhangs: vec![0.0; candidate_count],
        embedded_distances: vec![0.0; candidate_count],
        positions,
        point_tree,
    }
}

struct PerimeterAssociation<'a> {
    candidates: &'a LayerSeamCandidates,
    positions: &'a [super::Vec3],
    point_tree: &'a super::PointKdTree,
    z: f32,
    scale: CoordinateScale,
}

impl PerimeterAssociation<'_> {
    fn collection_map(&self, collection: &ExtrusionEntityCollection) -> Vec<usize> {
        collection
            .entities
            .iter()
            .map(|entity| {
                let paths = match entity {
                    ExtrusionEntity::Loop(ordered) => &ordered.extrusion_loop.paths,
                    ExtrusionEntity::MultiPath(multi_path) => &multi_path.paths,
                };
                self.closest_perimeter(paths)
            })
            .collect()
    }

    fn closest_perimeter(&self, paths: &[ExtrusionPath]) -> usize {
        let point_count = paths.iter().map(|path| path.polyline.points.len()).sum();
        let mut path_index = 0;
        let mut point_index = 0;
        let mut candidate_index = 0;
        let mut closest_perimeter = None;
        for _ in 0..point_count {
            let point = paths[path_index].polyline.points[point_index];
            candidate_index = self.point_tree.closest(
                self.positions,
                super::Vec3::new(
                    self.scale.unscale(point.x) as f32,
                    self.scale.unscale(point.y) as f32,
                    self.z,
                ),
            );
            let perimeter_index = self.candidates.points[candidate_index].perimeter_index;
            if closest_perimeter == Some(perimeter_index) {
                break;
            }
            closest_perimeter = Some(perimeter_index);
            point_index += 1;
            if point_index == paths[path_index].polyline.points.len() {
                path_index = (path_index + 1) % paths.len();
                point_index = 0;
            }
        }
        self.candidates.points[candidate_index].perimeter_index
    }
}

fn best_candidate(perimeter: &SeamPerimeter, layer: &LayerPlan) -> usize {
    let mut best = perimeter.start_index;
    for index in perimeter.start_index + 1..perimeter.end_index {
        if is_better(layer, index, best) {
            best = index;
        }
    }
    best
}
