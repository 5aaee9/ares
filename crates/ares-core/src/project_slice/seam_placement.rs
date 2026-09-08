mod alignment;
mod fitting;
mod mesh;
mod runtime;
mod sampling;
mod spatial;
mod spline;
mod visibility;

use crate::{
    ProcessSeamPosition,
    geometry::CoordinateScale,
    project_slice::{
        island_print_order::{
            IslandPrintEntity, NearestSeamLayer, OrderedExtrusionLayer,
            PreparedPostIslandPrintOrder,
        },
        perimeters::classic::{
            chained_loops::ExtrusionLoop,
            entity_collections::{ExtrusionEntity, ExtrusionEntityCollection},
            materialize::{ExtrusionPath, ExtrusionRole, Point3},
            traversal::PreparedPostClassicTraversal,
        },
        seam_candidates::SeamCandidate,
    },
};

const VISIBILITY_SAMPLE_COUNT: usize = 30_000;
const ANGLE_IMPORTANCE_ALIGNED: f32 = 0.6;
/// `SeamPlacer.hpp:126` — angle weight for the spNearest comparator.
const ANGLE_IMPORTANCE_NEAREST: f32 = 1.0;

/// `SeamPlacer.cpp:44-49` — `gauss(value, mean, falloff)` used for the
/// emit-time distance penalty (`cpp:784-785`).
fn gauss_penalty(value: f32, falloff_speed: f32) -> f32 {
    let denominator = falloff_speed * value * value + 1.0;
    let exponent = 1.0 / denominator;
    (std::f32::consts::E.exp() - 1.0).recip() * (exponent.exp() - 1.0)
}
const SEAM_VERTEX_SNAP_MM: f64 = 0.0015;

#[cfg(test)]
use runtime::is_closed_axis_rectangle;
pub(in crate::project_slice) use runtime::{
    place_nearest, place_nearest_penalized, place_nearest_projection,
};

pub(in crate::project_slice) fn apply(prepared: &mut PreparedPostIslandPrintOrder) {
    let predecessor = &prepared.predecessor;
    let traversal = &predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let placements = runtime::placement_modes(&traversal.resolved.objects, &prepared.objects);
    if !placements.iter().any(Option::is_some) {
        return;
    }

    let mesh = mesh::TriangleMesh::from_project(&traversal.project);
    if mesh.triangles.is_empty() {
        return;
    }
    let visibility = visibility::GlobalVisibility::from_mesh(mesh, VISIBILITY_SAMPLE_COUNT);
    let nozzle_diameter = traversal
        .resolved
        .views
        .full
        .project
        .print
        .nozzle_diameter
        .0
        .first()
        .map_or(0.4, |value| value.0) as f32;
    let mut nearest_seam_plans = std::mem::take(&mut prepared.nearest_seam_plans);
    nearest_seam_plans.resize_with(placements.len(), Vec::new);
    apply_objects(
        &mut prepared.objects,
        traversal,
        &placements,
        &visibility,
        nozzle_diameter,
        &mut nearest_seam_plans,
    );
    prepared.nearest_seam_plans = nearest_seam_plans;
}

fn apply_objects(
    objects: &mut [Vec<OrderedExtrusionLayer>],
    traversal: &PreparedPostClassicTraversal,
    placements: &[Option<ProcessSeamPosition>],
    visibility: &visibility::GlobalVisibility,
    nozzle_diameter: f32,
    nearest_plans: &mut [Vec<NearestSeamLayer>],
) {
    for (object_index, layers) in objects.iter_mut().enumerate() {
        let Some(placement) = placements[object_index] else {
            continue;
        };
        let layer_zs = runtime::layer_mid_zs(&traversal.objects[object_index].records);
        let mut plans = alignment::prepare(
            (layers, &layer_zs),
            traversal,
            object_index,
            nozzle_diameter,
            visibility,
        );
        match placement {
            ProcessSeamPosition::Aligned => alignment::align(&mut plans),
            ProcessSeamPosition::Random => alignment::randomize(&mut plans),
            ProcessSeamPosition::Nearest => {
                // `SeamPlacer.cpp:1465` skips `pick_seam_point` for spNearest —
                // the seam is picked at emit with actual nozzle position
                // (`place_seam` cpp:1500-1560 via
                // `pick_nearest_seam_point_index` cpp:930-940). Store the
                // penalty data for the emit-time selection; no loop splitting
                // here.
                nearest_plans[object_index] = nearest_seam_data(&plans, visibility);
                continue;
            }
            _ => unreachable!("only active seam placement modes are prepared"),
        }
        let staggered = traversal.resolved.objects[object_index]
            .object
            .staggered_inner_seams
            .0;
        for (layer, plan) in layers.iter_mut().zip(&plans) {
            place_layer(layer, plan, staggered, traversal.scale);
        }
    }
}

fn nearest_seam_data(
    plans: &[alignment::LayerPlan],
    visibility: &visibility::GlobalVisibility,
) -> Vec<NearestSeamLayer> {
    plans
        .iter()
        .map(|plan| {
            let scores = plan
                .candidates
                .points
                .iter()
                .map(|candidate| {
                    let position = mesh::Vec3::new(
                        candidate.position.x,
                        candidate.position.y,
                        candidate.position.z,
                    );
                    visibility.at(position)
                        + ANGLE_IMPORTANCE_NEAREST * angle_penalty(candidate.local_ccw_angle)
                })
                .collect();
            nearest_seam_layer(plan, scores)
        })
        .collect()
}

fn nearest_seam_layer(plan: &alignment::LayerPlan, scores: Vec<f32>) -> NearestSeamLayer {
    NearestSeamLayer {
        positions: plan
            .candidates
            .points
            .iter()
            .map(|candidate| (candidate.position.x, candidate.position.y))
            .collect(),
        scores,
        overhangs: plan.overhangs.clone(),
        perimeter_ranges: plan
            .candidates
            .perimeters
            .iter()
            .map(|perimeter| (perimeter.start_index, perimeter.end_index))
            .collect(),
        perimeter_of_candidate: plan
            .candidates
            .points
            .iter()
            .map(|candidate| candidate.perimeter_index)
            .collect(),
        ccw_angles: plan
            .candidates
            .points
            .iter()
            .map(|candidate| candidate.local_ccw_angle)
            .collect(),
    }
}

fn place_layer(
    layer: &mut OrderedExtrusionLayer,
    plan: &alignment::LayerPlan,
    staggered: bool,
    scale: CoordinateScale,
) {
    let collections = layer
        .islands
        .iter_mut()
        .flat_map(|island| &mut island.entities)
        .filter_map(|entity| match entity {
            IslandPrintEntity::Perimeter(collection) => Some(collection),
            IslandPrintEntity::Fill(_)
            | IslandPrintEntity::FillCollection(_)
            | IslandPrintEntity::Thin(_) => None,
        });
    for (collection, perimeter_indices) in collections.zip(&plan.collection_perimeters) {
        place_collection(collection, perimeter_indices, plan, staggered, scale);
    }
}

fn place_collection(
    collection: &mut ExtrusionEntityCollection,
    perimeter_indices: &[usize],
    plan: &alignment::LayerPlan,
    staggered: bool,
    scale: CoordinateScale,
) {
    for (entity, &perimeter_index) in collection.entities.iter_mut().zip(perimeter_indices) {
        let ExtrusionEntity::Loop(ordered) = entity else {
            // Aligned-seam placement targets loops (`SeamPlacer.cpp:1500`);
            // arachne wall generation stays typed-rejected before
            // materialization, so no multi-path can reach placement.
            unreachable!("multi-path seam placement lands with the arachne materialization seam");
        };
        let perimeter = &plan.candidates.perimeters[perimeter_index];
        let choice = &plan.choices[perimeter_index];
        let selected = choice.seam_index;
        let previous = if selected == perimeter.start_index {
            perimeter.end_index - 1
        } else {
            selected - 1
        };
        let next = if selected + 1 == perimeter.end_index {
            perimeter.start_index
        } else {
            selected + 1
        };
        let selected_candidate = &plan.candidates.points[selected];
        place_loop(
            &mut ordered.extrusion_loop,
            Placement {
                selected: selected_candidate,
                previous: &plan.candidates.points[previous],
                next: &plan.candidates.points[next],
                position: choice.final_position.unwrap_or_else(|| {
                    mesh::Vec3::new(
                        selected_candidate.position.x,
                        selected_candidate.position.y,
                        selected_candidate.position.z,
                    )
                }),
            },
            staggered,
            scale,
        );
    }
}

#[derive(Clone, Copy)]
struct Placement<'a> {
    selected: &'a SeamCandidate,
    previous: &'a SeamCandidate,
    next: &'a SeamCandidate,
    position: mesh::Vec3,
}

fn candidate_penalty(candidate: &SeamCandidate, visibility: &visibility::GlobalVisibility) -> f32 {
    let position = mesh::Vec3::new(
        candidate.position.x,
        candidate.position.y,
        candidate.position.z,
    );
    visibility.at(position) + ANGLE_IMPORTANCE_ALIGNED * angle_penalty(candidate.local_ccw_angle)
}

fn angle_penalty(angle: f32) -> f32 {
    let denominator = 3.0 * angle * angle + 1.0;
    let gaussian = ((1.0 / denominator).exp() - 1.0) / (1.0_f32.exp() - 1.0);
    gaussian + 1.0 / (2.0 + (-angle).exp())
}

#[expect(
    clippy::approx_constant,
    reason = "OrcaSlicer uses the literal 1.4142 seam-depth coefficient"
)]
fn place_loop(
    loop_: &mut ExtrusionLoop,
    placement: Placement<'_>,
    staggered: bool,
    scale: CoordinateScale,
) {
    let selected = placement.selected.position;
    let mut seam = scale_position((placement.position.x, placement.position.y), scale);
    if loop_.paths[0].role == ExtrusionRole::Perimeter {
        let mut projection = closest_projection(&loop_.paths, seam);
        let mut depth = scale
            .unscale(seam.0 - projection.x)
            .hypot(scale.unscale(seam.1 - projection.y)) as f32;
        let angle = placement.selected.local_ccw_angle;
        let displacement = (
            placement.position.x - selected.x,
            placement.position.y - selected.y,
            placement.position.z - selected.z,
        );
        let displacement_squared = displacement.0.mul_add(
            displacement.0,
            displacement
                .1
                .mul_add(displacement.1, displacement.2 * displacement.2),
        );
        if displacement_squared < depth && angle < -f32::EPSILON {
            let previous = placement.previous.position;
            let next = placement.next.position;
            let to_previous = normalized((selected.x - previous.x, selected.y - previous.y));
            let to_next = normalized((selected.x - next.x, selected.y - next.y));
            let direction = (
                0.5 * (to_previous.0 + to_next.0),
                0.5 * (to_previous.1 + to_next.1),
            );
            depth = (1.4142 * f64::from(depth) / f64::from((angle * 0.5).cos())) as f32;
            seam = scale_position(
                (
                    selected.x + depth * direction.0,
                    selected.y + depth * direction.1,
                ),
                scale,
            );
            projection = closest_projection(&loop_.paths, seam);
        } else {
            // Convex corner: the perpendicular depth, not the distance to
            // the nearest point (upstream place_loop, SeamPlacer.cpp:1594).
            depth *= (angle * 0.5).cos() / 1.4142;
        }
        seam = (projection.x, projection.y);
        // Stagger inner seams inside the placement pass — upstream walks the
        // loop points from the projected position (SeamPlacer.cpp:1607-1620),
        // not a separate re-split pass.
        if staggered {
            let mut point = projection;
            depth = loop_.paths[point.path].width.max(depth);
            while depth > 0.0 {
                let mut next = next_loop_point(loop_, &point);
                let a = (point.x, point.y);
                let b = (next.x, next.y);
                let dist = scale.unscale(a.0 - b.0).hypot(scale.unscale(a.1 - b.1)) as f32;
                if dist > depth {
                    let ratio = depth / dist;
                    next.x = (a.0 as f64 + f64::from(ratio) * (b.0 - a.0) as f64) as i64;
                    next.y = (a.1 as f64 + f64::from(ratio) * (b.1 - a.1) as f64) as i64;
                }
                depth -= dist;
                point = next;
            }
            seam = (point.x, point.y);
        }
    }
    if let Ok(path) = std::env::var("ARES_DUMP_SEAMPT") {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(
                file,
                "SPT {} {}",
                scale.unscale(seam.0),
                scale.unscale(seam.1)
            );
        }
    }
    split_at(loop_, seam, scale);
}

fn scale_position(position: (f32, f32), scale: CoordinateScale) -> (i64, i64) {
    (
        (f64::from(position.0) / scale.factor()) as i64,
        (f64::from(position.1) / scale.factor()) as i64,
    )
}

fn normalized(vector: (f32, f32)) -> (f32, f32) {
    let length = vector.0.hypot(vector.1);
    (vector.0 / length, vector.1 / length)
}

struct Projection {
    path: usize,
    segment: usize,
    x: i64,
    y: i64,
}

/// `get_next_loop_point` (SeamPlacer.cpp:1511-1519): advance to the next
/// polyline point, wrapping across paths.
fn next_loop_point(loop_: &ExtrusionLoop, mut current: &Projection) -> Projection {
    let mut segment = current.segment + 1;
    let mut path = current.path;
    if segment >= loop_.paths[path].polyline.points.len() {
        path = (path + 1) % loop_.paths.len();
        segment = 0;
    }
    let point = loop_.paths[path].polyline.points[segment];
    Projection {
        path,
        segment,
        x: point.x,
        y: point.y,
    }
}

fn closest_projection(paths: &[ExtrusionPath], target: (i64, i64)) -> Projection {
    let mut best = None::<(Projection, f64)>;
    for (path_index, path) in paths.iter().enumerate() {
        for (segment_index, segment) in path.polyline.points.windows(2).enumerate() {
            let a = (segment[0].x, segment[0].y);
            let b = (segment[1].x, segment[1].y);
            let (x, y) = project_onto_segment(a, b, target);
            let distance = squared_distance((x, y), target);
            if best.as_ref().is_none_or(|(_, best)| distance < *best) {
                best = Some((
                    Projection {
                        path: path_index,
                        segment: segment_index,
                        x,
                        y,
                    },
                    distance,
                ));
            }
        }
    }
    best.expect("an extrusion loop has a segment").0
}

fn project_onto_segment(a: (i64, i64), b: (i64, i64), target: (i64, i64)) -> (i64, i64) {
    let lx = (b.0 - a.0) as f64;
    let ly = (b.1 - a.1) as f64;
    let denominator = lx.mul_add(lx, ly * ly);
    if denominator == 0.0 {
        return a;
    }
    let theta = (((b.0 - target.0) as f64) * lx + ((b.1 - target.1) as f64) * ly) / denominator;
    if !(0.0..=1.0).contains(&theta) {
        return if squared_distance(a, target) < squared_distance(b, target) {
            a
        } else {
            b
        };
    }
    (
        (theta * a.0 as f64 + (1.0 - theta) * b.0 as f64) as i64,
        (theta * a.1 as f64 + (1.0 - theta) * b.1 as f64) as i64,
    )
}

fn squared_distance(left: (i64, i64), right: (i64, i64)) -> f64 {
    let dx = (left.0 - right.0) as f64;
    let dy = (left.1 - right.1) as f64;
    dx.mul_add(dx, dy * dy)
}

fn split_at(loop_: &mut ExtrusionLoop, seam: (i64, i64), scale: CoordinateScale) {
    let snap_distance = SEAM_VERTEX_SNAP_MM / scale.factor();
    let snap_distance_squared = snap_distance * snap_distance;
    let seam = loop_
        .paths
        .iter()
        .flat_map(|path| &path.polyline.points)
        .find(|point| squared_distance((point.x, point.y), seam) < snap_distance_squared)
        .map_or(seam, |point| (point.x, point.y));
    let projection = closest_projection(&loop_.paths, seam);
    let single_path = loop_.paths.len() == 1;
    let mut paths = std::mem::take(&mut loop_.paths);
    let following = paths.split_off(projection.path + 1);
    let target = paths.pop().expect("projected path exists");
    let split = projected_parts(target, &projection, scale);
    if single_path {
        match split {
            (Some(mut suffix), Some(prefix)) => {
                fitting::append_polyline(&mut suffix.polyline, prefix.polyline);
                loop_.paths.push(suffix);
            }
            (Some(path), None) | (None, Some(path)) => loop_.paths.push(path),
            (None, None) => unreachable!("a projected loop path has a valid side"),
        }
        return;
    }
    loop_.paths.extend(split.0);
    loop_.paths.extend(following);
    loop_.paths.extend(paths);
    loop_.paths.extend(split.1);
}

fn projected_parts(
    path: ExtrusionPath,
    projection: &Projection,
    scale: CoordinateScale,
) -> (Option<ExtrusionPath>, Option<ExtrusionPath>) {
    let point = Point3 {
        x: projection.x,
        y: projection.y,
        z: path.polyline.points[projection.segment].z,
    };
    let start = path.polyline.points[projection.segment];
    let end = path.polyline.points[projection.segment + 1];
    let (suffix, prefix) = if point.x == start.x && point.y == start.y {
        let (prefix, suffix) = fitting::split_at_index(&path.polyline, projection.segment, scale);
        (suffix, prefix)
    } else if point.x == end.x && point.y == end.y {
        let (prefix, suffix) =
            fitting::split_at_index(&path.polyline, projection.segment + 1, scale);
        (suffix, prefix)
    } else {
        let (mut prefix, _) = fitting::split_at_index(&path.polyline, projection.segment, scale);
        fitting::append(&mut prefix, point);
        let (_, mut suffix) =
            fitting::split_at_index(&path.polyline, projection.segment + 1, scale);
        fitting::prepend(&mut suffix, point);
        (suffix, prefix)
    };
    let make = |polyline: crate::project_slice::perimeters::classic::materialize::Polyline3| {
        (polyline.points.len() >= 2).then_some(ExtrusionPath {
            polyline,
            role: path.role,
            can_reverse: path.can_reverse,
            mm3_per_mm: path.mm3_per_mm,
            width: path.width,
            height: path.height,
        })
    };
    (make(suffix), make(prefix))
}

#[cfg(test)]
mod tests;
