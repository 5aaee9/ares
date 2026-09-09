// Wall-line ordering from OrcaSlicer v2.4.2
// `PerimeterGenerator::process_arachne`
// (`PerimeterGenerator.cpp:2269-2440`): flattens generated walls inner-to-
// outer (or outer-to-inner), applies the `WallToolPaths::getRegionOrder`
// constraints (`WallToolPaths.cpp:809-903`) and greedily chains the remaining
// candidates by distance, preferring closed walls. The inner-outer-inner
// sandwich reordering (`:2404-2464`) stays a typed-rejected scope.

use crate::{
    arachne::{ExtrusionLine, wall_toolpaths::region_order::get_region_order},
    geometry::Point,
};

use super::traverse::PerimeterGeneratorArachneExtrusion;

pub(in crate::project_slice) fn order_walls(
    perimeters: &[Vec<ExtrusionLine>],
    outer_wall_first: bool,
) -> Vec<PerimeterGeneratorArachneExtrusion> {
    // `PerimeterGenerator.cpp:2269-2281`: default inner-to-outer walk.
    let indexes = if outer_wall_first {
        (0..perimeters.len()).collect::<Vec<_>>()
    } else {
        (0..perimeters.len()).rev().collect::<Vec<_>>()
    };
    let all_extrusions = indexes
        .into_iter()
        .filter(|perimeter_idx| !perimeters[*perimeter_idx].is_empty())
        .flat_map(|perimeter_idx| perimeters[perimeter_idx].iter().cloned())
        .collect::<Vec<_>>();

    // Blocked/blocking constraint graph from `extrusions_constrains`
    // (`PerimeterGenerator.cpp:2283-2299`).
    let references = all_extrusions.iter().collect::<Vec<_>>();
    let mut blocked = vec![0_usize; all_extrusions.len()];
    let mut blocking = vec![Vec::new(); all_extrusions.len()];
    for (before, after) in get_region_order(&references, outer_wall_first) {
        blocked[after] += 1;
        blocking[before].push(after);
    }

    // Greedy candidate selection (`PerimeterGenerator.cpp:2301-2340`).
    let mut processed = vec![false; all_extrusions.len()];
    let mut current_position = all_extrusions
        .first()
        .and_then(|line| line.junctions.first())
        .map_or_else(|| Point::new(0, 0), |junction| junction.point);
    let mut ordered = Vec::with_capacity(all_extrusions.len());
    while ordered.len() < all_extrusions.len() {
        let mut best_candidate = 0;
        let mut best_distance = f64::INFINITY;
        let mut is_best_closed = false;

        let mut available = (0..all_extrusions.len())
            .filter(|candidate| !processed[*candidate] && blocked[*candidate] == 0)
            .collect::<Vec<_>>();
        // Closed walls first (`PerimeterGenerator.cpp:2312-2315`).
        available.sort_by_key(|candidate| !all_extrusions[*candidate].is_closed);

        for candidate in available {
            let path = &all_extrusions[candidate];
            // No vertices: plan it in last (`:2318-2322`).
            let empty = path.junctions.is_empty();
            if empty && best_distance == f64::INFINITY {
                best_candidate = candidate;
                is_best_closed = path.is_closed;
            }
            if empty {
                continue;
            }
            // `distance_sqr` is a plain distance upstream
            // (`:2324-2327`, `.norm()` of the offset).
            let dx = current_position.x() - path.junctions[0].point.x();
            let dy = current_position.y() - path.junctions[0].point.y();
            let distance = (dx as f64 * dx as f64 + dy as f64 * dy as f64).sqrt();
            if distance >= best_distance {
                continue;
            }
            if path.is_closed || best_distance != f64::INFINITY || !is_best_closed {
                best_candidate = candidate;
                best_distance = distance;
                is_best_closed = path.is_closed;
            }
        }

        // `PerimeterGenerator.cpp:2342-2348`.
        let best_path = &all_extrusions[best_candidate];
        ordered.push(PerimeterGeneratorArachneExtrusion {
            extrusion: best_path.clone(),
            is_contour: best_path.is_contour(),
        });
        processed[best_candidate] = true;
        for unlocked in &blocking[best_candidate] {
            blocked[*unlocked] -= 1;
        }
        if let Some(junction) = best_path
            .junctions
            .first()
            .filter(|_| best_path.is_closed)
            .or_else(|| best_path.junctions.last())
        {
            current_position = junction.point;
        }
    }
    ordered
}
