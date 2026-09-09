//! OrcaSlicer 2.4.2 `Arachne/WallToolPaths.cpp:809-903`, `getRegionOrder`.

use std::collections::{HashMap, HashSet};

use crate::arachne::extrusion_line::{ExtrusionJunction, ExtrusionLine};

#[cfg(test)]
mod tests;

/// Returns (before, after) indices into the distinct, flattened input paths.
/// The set is not a traversal order; `process_arachne` chooses available paths.
pub(crate) fn get_region_order(
    input: &[&ExtrusionLine],
    outer_to_inner: bool,
) -> HashSet<(usize, usize)> {
    let mut requirements = HashSet::new();
    let max_width = input
        .iter()
        .flat_map(|line| &line.junctions)
        .map(|junction| junction.width)
        .max()
        .unwrap_or(0);
    if max_width == 0 {
        return requirements;
    }

    // Preserve source float multiplication and SquareGrid's truncating division,
    // including its double-sized cell at the origin.
    let radius = (max_width as f32 * 1.9_f32) as i64;
    let mut grid: HashMap<(i64, i64), Vec<(usize, &ExtrusionJunction)>> = HashMap::new();
    for (index, line) in input.iter().enumerate() {
        for junction in &line.junctions {
            grid.entry((junction.point.x() / radius, junction.point.y() / radius))
                .or_default()
                .push((index, junction));
        }
    }
    for (index, line) in input.iter().enumerate() {
        for junction in &line.junctions {
            let point = junction.point;
            let min_x = (point.x() - radius) / radius;
            let max_x = (point.x() + radius) / radius;
            let min_y = (point.y() - radius) / radius;
            let max_y = (point.y() + radius) / radius;
            let grid = &grid;
            let nearby = (min_y..=max_y)
                .flat_map(|y| (min_x..=max_x).filter_map(move |x| grid.get(&(x, y))))
                .flatten();
            requirements.extend(nearby.filter_map(|&other| {
                region_constraint(input, (index, junction), other, outer_to_inner)
            }));
        }
    }
    requirements
}

fn region_constraint(
    input: &[&ExtrusionLine],
    (index, junction): (usize, &ExtrusionJunction),
    (other_index, other_junction): (usize, &ExtrusionJunction),
    outer_to_inner: bool,
) -> Option<(usize, usize)> {
    // Each unordered pair has the same constraint in either direction.
    if other_index <= index {
        return None;
    }
    let line = input[index];
    let other = input[other_index];
    if line.inset_index.abs_diff(other.inset_index) != 1 {
        return None;
    }
    let limit = (((junction.width + other_junction.width) / 2) as f32 * 1.9_f32) as i64;
    let dx = i128::from(junction.point.x()) - i128::from(other_junction.point.x());
    let dy = i128::from(junction.point.y()) - i128::from(other_junction.point.y());
    // Point.hpp::shorter_then includes the radius boundary.
    if dx * dx + dy * dy > i128::from(limit).pow(2) {
        return None;
    }
    let (outer, inner) = if line.inset_index < other.inset_index {
        (index, other_index)
    } else {
        (other_index, index)
    };
    if input[outer].is_odd || input[inner].is_odd {
        (!input[outer].is_odd && input[inner].is_odd).then_some((outer, inner))
    } else if outer_to_inner {
        Some((outer, inner))
    } else {
        Some((inner, outer))
    }
}
