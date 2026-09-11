//! Inner-outer-inner sandwich reordering for the arachne generator
//! (`PerimeterGenerator.cpp:2374-2464`): after the topological sort,
//! bring inset-0 contours to the front, reorder by proximity, then
//! resequence each island as [deep internals (innermost first),
//! external, first internal].

use super::traverse::PerimeterGeneratorArachneExtrusion;
use crate::geometry::Point;

pub(in crate::project_slice) fn apply_inner_outer_inner(
    mut ordered: Vec<PerimeterGeneratorArachneExtrusion>,
    threshold_external: i64,
    threshold_internal: i64,
) -> Vec<PerimeterGeneratorArachneExtrusion> {
    // `bringContoursToFront` (`PerimeterGenerator.cpp:2082-2087`): a
    // stable partition moving external-perimeter contours to the front.
    ordered.sort_by_key(|item| !(item.is_contour && item.extrusion.inset_index == 0));

    // `reorderPerimetersByProximity` (`:2396`, `:1987-2061`).
    let mut result =
        reorder_perimeters_by_proximity(&ordered, threshold_external, threshold_internal);
    // `reordered_extrusions = ordered_extrusions` — the sandwich reads the
    // copy and writes the resequenced blocks back into the base list.
    let reordered = result.clone();

    // Sandwich scan (`:2398-2456`): `position`, `arr_i`, `arr_j` are the
    // upstream loop indices; `outer`/`first_internal`/`second_internal`
    // use the -1 sentinel, so the scan arithmetic runs in i64.
    let size = reordered.len() as i64;
    let mut position: i64 = 0;
    while position < size {
        let mut outer: i64 = -1;
        let mut first_internal: i64 = -1;
        let mut second_internal: i64 = -1;
        let mut current_perimeter: i64 = -1;
        let mut max_internal: i64 = size - 1;
        let mut arr_i: i64 = position;
        while arr_i < size {
            let inset = reordered[arr_i as usize].extrusion.inset_index;
            match inset {
                0 => {
                    if outer == -1 {
                        outer = arr_i;
                    }
                }
                1 => {
                    if first_internal == -1 && arr_i > outer && outer != -1 {
                        first_internal = arr_i;
                    }
                }
                2 => {
                    if second_internal == -1 && arr_i > first_internal && outer != -1 {
                        second_internal = arr_i;
                    }
                }
                _ => {}
            }
            // A new external perimeter after the first internal starts a
            // new island (`:2427-2432`): step back one perimeter and bound
            // the resequencing block there.
            if outer > -1
                && first_internal > -1
                && reordered[arr_i as usize].extrusion.inset_index == 0
            {
                arr_i -= 1;
                max_internal = arr_i;
                break;
            }
            arr_i += 1;
        }
        if outer > -1 && first_internal > -1 && second_internal > -1 {
            let mut inner_outer: Vec<Option<PerimeterGeneratorArachneExtrusion>> =
                vec![None; (max_internal - position + 1) as usize];
            // Inside out: the deep internals from `max_internal` down to
            // `second_internal` fill the front of the block (`:2439-2444`).
            let mut arr_j: i64 = max_internal;
            while arr_j >= position {
                if arr_j >= second_internal {
                    inner_outer[(max_internal - arr_j) as usize] =
                        Some(reordered[arr_j as usize].clone());
                    current_perimeter += 1;
                }
                arr_j -= 1;
            }
            // Outside in: the external and first-internal walls keep their
            // order after the deep internals (`:2446-2449`).
            arr_j = position;
            while arr_j < second_internal {
                current_perimeter += 1;
                inner_outer[current_perimeter as usize] = Some(reordered[arr_j as usize].clone());
                arr_j += 1;
            }
            arr_j = position;
            while arr_j <= max_internal {
                result[arr_j as usize] = inner_outer[(arr_j - position) as usize]
                    .clone()
                    .expect("sandwich block fully assigned");
                arr_j += 1;
            }
        } else {
            break;
        }
        position = arr_i + 1;
    }
    result
}

/// `reorderPerimetersByProximity` (`PerimeterGenerator.cpp:1987-2061`):
/// level-by-level BFS from each external reference — first-level touching
/// perimeters insert largest-last, deeper levels insert largest-first.
fn reorder_perimeters_by_proximity(
    entities: &[PerimeterGeneratorArachneExtrusion],
    threshold_external: i64,
    threshold_internal: i64,
) -> Vec<PerimeterGeneratorArachneExtrusion> {
    let mut reordered = Vec::with_capacity(entities.len());
    let mut included = vec![false; entities.len()];

    for reference_index in 0..entities.len() {
        if entities[reference_index].extrusion.inset_index != 0 || included[reference_index] {
            continue;
        }
        // `reorderFromReference` (`:1993-2044`).
        let mut first_level_touching = find_all_touching_perimeters(
            entities,
            &[reference_index],
            threshold_external,
            threshold_internal,
            1,
        );
        // Bring the largest first-level perimeter to the end
        // (`:1998-2003`): `iter_swap(maxIt, end() - 1)`.
        if !first_level_touching.is_empty() {
            let mut max_position = 0;
            for (position, index) in first_level_touching.iter().enumerate() {
                if entities[*index].extrusion.length()
                    > entities[first_level_touching[max_position]]
                        .extrusion
                        .length()
                {
                    max_position = position;
                }
            }
            let last = first_level_touching.len() - 1;
            first_level_touching.swap(max_position, last);
        }
        reordered.push(entities[reference_index].clone());
        included[reference_index] = true;
        for index in &first_level_touching {
            if !included[*index] {
                reordered.push(entities[*index].clone());
                included[*index] = true;
            }
        }
        // Levels 2+ (`:2006-2040`): each level searches from the previous
        // level's members; already-included perimeters drop out and an
        // empty unfiltered result ends the walk.
        let mut current_inset_index = 2;
        loop {
            let current_level_touching = find_all_touching_perimeters(
                entities,
                &first_level_touching,
                threshold_external,
                threshold_internal,
                current_inset_index,
            );
            if current_level_touching.is_empty() {
                break;
            }
            let mut current_level_touching: Vec<usize> = current_level_touching
                .into_iter()
                .filter(|index| !included[*index])
                .collect();
            // Bring the largest current-level perimeter to the front
            // (`:2024-2029`): `iter_swap(maxIt, begin())`.
            if !current_level_touching.is_empty() {
                let mut max_position = 0;
                for (position, index) in current_level_touching.iter().enumerate() {
                    if entities[*index].extrusion.length()
                        > entities[current_level_touching[max_position]]
                            .extrusion
                            .length()
                    {
                        max_position = position;
                    }
                }
                current_level_touching.swap(max_position, 0);
            }
            for index in &current_level_touching {
                if !included[*index] {
                    reordered.push(entities[*index].clone());
                    included[*index] = true;
                }
            }
            first_level_touching = current_level_touching;
            current_inset_index += 1;
        }
    }
    // Leftovers that no external reference reached (`:2053-2058`).
    for (index, entity) in entities.iter().enumerate() {
        if !included[index] {
            reordered.push(entity.clone());
        }
    }
    reordered
}

/// `findAllTouchingPerimeters` (`PerimeterGenerator.cpp:1943-1973`): for
/// each reference, the candidates at `considered_inset_idx` whose minimum
/// polyline distance falls under the reference's threshold (external
/// references use `threshold_external`, internal ones
/// `threshold_internal`). Inset-0 candidates never match.
fn find_all_touching_perimeters(
    entities: &[PerimeterGeneratorArachneExtrusion],
    reference_indices: &[usize],
    threshold_external: i64,
    threshold_internal: i64,
    considered_inset_idx: usize,
) -> Vec<usize> {
    let mut touching = Vec::new();
    let mut seen = vec![false; entities.len()];
    for reference_index in reference_indices {
        let reference_entity = &entities[*reference_index];
        let reference_points = junction_points(&reference_entity.extrusion);
        for (index, entity) in entities.iter().enumerate() {
            if reference_indices.contains(&index) {
                continue;
            }
            if entity.extrusion.inset_index == 0 {
                continue;
            }
            if entity.extrusion.inset_index != considered_inset_idx {
                continue;
            }
            let points = junction_points(&entity.extrusion);
            let distance = minimum_distance_between_lines(&reference_points, &points);
            let threshold = if reference_entity.extrusion.inset_index == 0 {
                threshold_external
            } else {
                threshold_internal
            };
            if distance <= threshold as f64 && !seen[index] {
                seen[index] = true;
                touching.push(index);
            }
        }
    }
    touching
}

fn junction_points(line: &crate::arachne::ExtrusionLine) -> Vec<Point> {
    line.junctions
        .iter()
        .map(|junction| junction.point)
        .collect()
}

/// `MultiPoint::minimumDistanceBetweenLinesDefinedByPoints`
/// (`MultiPoint.cpp:403-423`): the minimum over segments of A × points of
/// B and segments of B × points of A, `sqrt` per pair.
fn minimum_distance_between_lines(a: &[Point], b: &[Point]) -> f64 {
    let mut min_distance = f64::INFINITY;
    for pair in a.windows(2) {
        for point in b {
            let distance = squared_distance_to_segment(*point, pair[0], pair[1]);
            min_distance = min_distance.min(distance.sqrt());
        }
    }
    for pair in b.windows(2) {
        for point in a {
            let distance = squared_distance_to_segment(*point, pair[0], pair[1]);
            min_distance = min_distance.min(distance.sqrt());
        }
    }
    min_distance
}

/// `MultiPoint::squaredDistanceToLineSegment` (`MultiPoint.cpp:382-395`):
/// integer segment norm and dot product, clamped double projection, and
/// the projection lands on the lattice through the rounding
/// `Point(double, double)` constructor (`Point.hpp:197`).
fn squared_distance_to_segment(point: Point, v: Point, w: Point) -> f64 {
    let segment = (w.x() - v.x(), w.y() - v.y());
    let squared_norm =
        (segment.0 as i128 * segment.0 as i128 + segment.1 as i128 * segment.1 as i128) as f64;
    let offset = (point.x() - v.x(), point.y() - v.y());
    if squared_norm == 0.0 {
        return (offset.0 as i128 * offset.0 as i128 + offset.1 as i128 * offset.1 as i128) as f64;
    }
    let dot = offset.0 as i128 * segment.0 as i128 + offset.1 as i128 * segment.1 as i128;
    let t = ((dot as f64) / squared_norm).clamp(0.0, 1.0);
    let projection = Point::new(
        (v.x() as f64 + t * segment.0 as f64).round() as i64,
        (v.y() as f64 + t * segment.1 as f64).round() as i64,
    );
    let delta = (point.x() - projection.x(), point.y() - projection.y());
    (delta.0 as i128 * delta.0 as i128 + delta.1 as i128 * delta.1 as i128) as f64
}
