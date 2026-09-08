use super::{split_at, squared_distance};
use crate::{
    ProcessSeamPosition,
    geometry::CoordinateScale,
    project::effective_config::types::ResolvedProjectObject,
    project_slice::{
        island_print_order::{IslandPrintEntity, OrderedExtrusionLayer},
        perimeters::classic::{
            chained_loops::ExtrusionLoop,
            entity_collections::ExtrusionEntity,
            materialize::{ExtrusionRole, Point3},
            traversal::ClassicTraversalRecord,
        },
    },
};

pub(super) fn placement_modes(
    objects: &[ResolvedProjectObject],
    prepared: &[Vec<OrderedExtrusionLayer>],
) -> Vec<Option<ProcessSeamPosition>> {
    objects
        .iter()
        .enumerate()
        .map(|(index, object)| match object.object.seam_position {
            ProcessSeamPosition::Aligned
                if object.layer_candidates[0].model_parts[0]
                    .region
                    .wall_direction
                    == crate::ProcessWallDirection::Clockwise
                    && prepared_cw_rectangles_have_source_seams(&prepared[index]) =>
            {
                None
            }
            ProcessSeamPosition::Aligned | ProcessSeamPosition::Random => {
                Some(object.object.seam_position)
            }
            ProcessSeamPosition::Nearest => Some(ProcessSeamPosition::Nearest),
            _ => None,
        })
        .collect()
}

pub(super) fn layer_mid_zs(records: &[Option<ClassicTraversalRecord>]) -> Vec<f32> {
    records
        .iter()
        .scan(0.0_f64, |print_z, record| {
            let height = record.as_ref().map_or(0.0, |record| record.layer_height);
            *print_z += height;
            Some((*print_z - 0.5 * height) as f32)
        })
        .collect()
}

pub(super) fn stagger_inner_seams(
    layers: &mut [OrderedExtrusionLayer],
    enabled: bool,
    scale: CoordinateScale,
) {
    if !enabled {
        return;
    }
    for loop_ in layers
        .iter_mut()
        .flat_map(|layer| &mut layer.islands)
        .flat_map(|island| &mut island.entities)
        .filter_map(|entity| match entity {
            IslandPrintEntity::Perimeter(collection) => Some(collection),
            _ => None,
        })
        .flat_map(|collection| &mut collection.entities)
        .map(|entity| match entity {
            // Staggered inner-wall seam splitting targets loops; arachne wall
            // generation stays typed-rejected before materialization, so no
            // multi-path can reach staggering.
            ExtrusionEntity::Loop(ordered) => &mut ordered.extrusion_loop,
            ExtrusionEntity::MultiPath(_) => unreachable!(
                "multi-path seam staggering lands with the arachne materialization seam"
            ),
        })
    {
        if let Some(target) = stagger_target(loop_, scale) {
            split_at(loop_, target, scale);
        }
    }
}

fn stagger_target(loop_: &ExtrusionLoop, scale: CoordinateScale) -> Option<(i64, i64)> {
    let first = loop_.paths.first()?;
    if first.role != ExtrusionRole::Perimeter {
        return None;
    }
    let mut remaining = f64::from(first.width) / scale.factor();
    for segment in loop_
        .paths
        .iter()
        .flat_map(|path| path.polyline.points.windows(2))
    {
        let dx = (segment[1].x - segment[0].x) as f64;
        let dy = (segment[1].y - segment[0].y) as f64;
        let length = dx.hypot(dy);
        if remaining <= length {
            let ratio = remaining / length;
            return Some((
                (segment[0].x as f64 + dx * ratio) as i64,
                (segment[0].y as f64 + dy * ratio) as i64,
            ));
        }
        remaining -= length;
    }
    None
}

pub(super) fn prepared_cw_rectangles_have_source_seams(layers: &[OrderedExtrusionLayer]) -> bool {
    let mut found = false;
    for loop_ in layers
        .iter()
        .flat_map(|layer| &layer.islands)
        .flat_map(|island| &island.entities)
        .filter_map(|entity| match entity {
            IslandPrintEntity::Perimeter(collection) => Some(collection),
            _ => None,
        })
        .flat_map(|collection| &collection.entities)
        .map(|entity| match entity {
            ExtrusionEntity::Loop(ordered) => &ordered.extrusion_loop,
            ExtrusionEntity::MultiPath(_) => unreachable!(
                "multi-path seam staggering lands with the arachne materialization seam"
            ),
        })
    {
        found = true;
        if !is_closed_axis_rectangle(loop_) {
            return false;
        }
    }
    found
}

pub(super) fn is_closed_axis_rectangle(loop_: &ExtrusionLoop) -> bool {
    if loop_.paths.len() != 1 {
        return false;
    }
    let points = &loop_.paths[0].polyline.points;
    points.len() == 5
        && points.first() == points.last()
        && points
            .windows(2)
            .all(|segment| segment[0].x == segment[1].x || segment[0].y == segment[1].y)
}

pub(in crate::project_slice) fn place_nearest(
    loop_: &mut ExtrusionLoop,
    cursor: Point3,
    scale: CoordinateScale,
) {
    let cursor = (cursor.x, cursor.y);
    let seam = loop_
        .paths
        .iter()
        .flat_map(|path| &path.polyline.points)
        .min_by(|left, right| {
            squared_distance((left.x, left.y), cursor)
                .total_cmp(&squared_distance((right.x, right.y), cursor))
        })
        .map(|point| (point.x, point.y));
    if let Some(seam) = seam {
        split_at(loop_, seam, scale);
    }
}

/// Penalty-aware Nearest seam selection (`SeamPlacer.cpp:1500-1560`):
/// geometrically match the loop to its perimeter via the closest
/// candidate, then minimize
/// `overhang + visibility + 1.0 * angle_penalty + distance_penalty`
/// over that perimeter's candidates
/// (`pick_nearest_seam_point_index` cpp:930-940), and split at the loop
/// vertex nearest the winning candidate.
pub(in crate::project_slice) fn place_nearest_penalized(
    loop_: &mut ExtrusionLoop,
    cursor: Point3,
    layer: &crate::project_slice::island_print_order::NearestSeamLayer,
    staggered: bool,
    scale: CoordinateScale,
) {
    let Some(first) = loop_
        .paths
        .first()
        .and_then(|path| path.polyline.points.first())
    else {
        return;
    };
    // Scaled-loop start vertex → unscaled mm query against candidates.
    let query = (scale.unscale(first.x) as f32, scale.unscale(first.y) as f32);
    let mut closest = usize::MAX;
    let mut closest_distance = f32::INFINITY;
    for (index, &(x, y)) in layer.positions.iter().enumerate() {
        let distance = (x - query.0).mul_add(x - query.0, (y - query.1) * (y - query.1));
        if distance < closest_distance {
            closest_distance = distance;
            closest = index;
        }
    }
    if closest == usize::MAX {
        return;
    }
    let Some(&(start, end)) = layer
        .perimeter_of_candidate
        .get(closest)
        .and_then(|&perimeter| layer.perimeter_ranges.get(perimeter))
    else {
        return;
    };
    let cursor = (
        scale.unscale(cursor.x) as f32,
        scale.unscale(cursor.y) as f32,
    );
    let mut best = usize::MAX;
    let mut best_penalty = f32::INFINITY;
    for index in start..end {
        let (x, y) = layer.positions[index];
        let distance = ((x - cursor.0) * (x - cursor.0) + (y - cursor.1) * (y - cursor.1)).sqrt();
        // `SeamPlacer.cpp:784-785`: 1 - gauss(dist, 0, 1, 0.005)
        let distance_penalty =
            1.0 - crate::project_slice::seam_placement::gauss_penalty(distance, 0.005);
        let penalty = layer.overhangs[index] + layer.scores[index] + distance_penalty;
        if penalty < best_penalty {
            best_penalty = penalty;
            best = index;
        }
    }
    let (seam_x, seam_y) = layer.positions[best];
    let (Some(seam_x), Some(seam_y)) = (
        scale.checked_scale(seam_x as f64),
        scale.checked_scale(seam_y as f64),
    ) else {
        return;
    };
    let seam_query = (seam_x, seam_y);
    // External loops split directly at the candidate; inner loops project
    // the candidate onto the loop and stagger it (`SeamPlacer.cpp:1562-1618`).
    if loop_
        .paths
        .iter()
        .all(|path| path.role != ExtrusionRole::ExternalPerimeter)
    {
        let projection = super::closest_projection(&loop_.paths, seam_query);
        let depth = squared_distance((projection.x, projection.y), seam_query).sqrt();
        let angle = layer.ccw_angles[best];
        let beta = (angle / 2.0).cos();
        let depth = if beta.abs() > f32::EPSILON {
            depth * f64::from(beta) / std::f64::consts::SQRT_2
        } else {
            depth
        };
        // `SeamPlacer.cpp:1602` — the stagger depth is at least the path
        // width (it is sometimes strongly underestimated).
        let width = f64::from(loop_.paths[projection.path].width) / scale.factor();
        let depth = depth.max(width);
        let target = if staggered {
            walk_along(loop_, &projection, depth)
        } else {
            (projection.x, projection.y)
        };
        split_at(loop_, target, scale);
        return;
    }
    let seam = loop_
        .paths
        .iter()
        .flat_map(|path| &path.polyline.points)
        .min_by(|left, right| {
            squared_distance((left.x, left.y), seam_query)
                .total_cmp(&squared_distance((right.x, right.y), seam_query))
        })
        .map(|point| (point.x, point.y));
    if let Some(seam) = seam {
        split_at(loop_, seam, scale);
    }
}

/// Walk `distance` (scaled units) forward along the loop from `from`,
/// wrapping around (`SeamPlacer.cpp:1601-1617`).
fn walk_along(loop_: &ExtrusionLoop, from: &super::Projection, distance: f64) -> (i64, i64) {
    let mut remaining = distance;
    let mut position = (from.x, from.y);
    let mut started = false;
    for pass in 0..2 {
        for (path_index, path) in loop_.paths.iter().enumerate() {
            for (segment_index, segment) in path.polyline.points.windows(2).enumerate() {
                if !started {
                    if path_index == from.path && segment_index == from.segment {
                        started = true;
                    } else {
                        continue;
                    }
                }
                let end = (segment[1].x, segment[1].y);
                let length = squared_distance(position, end).sqrt();
                if remaining <= length {
                    let ratio = if length == 0.0 {
                        0.0
                    } else {
                        remaining / length
                    };
                    return (
                        (position.0 as f64 + (end.0 - position.0) as f64 * ratio) as i64,
                        (position.1 as f64 + (end.1 - position.1) as f64 * ratio) as i64,
                    );
                }
                remaining -= length;
                position = end;
            }
        }
        if pass == 1 {
            break;
        }
    }
    position
}

pub(in crate::project_slice) fn place_nearest_projection(
    loop_: &mut ExtrusionLoop,
    cursor: Point3,
    scale: CoordinateScale,
) {
    split_at(loop_, (cursor.x, cursor.y), scale);
}
