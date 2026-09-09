use std::collections::HashSet;

use super::get_region_order;
use crate::{
    arachne::{ExtrusionJunction, ExtrusionLine},
    geometry::Point,
};

fn line(inset: usize, odd: bool, points: &[(i64, i64, i64)]) -> ExtrusionLine {
    let mut line = ExtrusionLine::new(inset, odd);
    line.junctions = points
        .iter()
        .map(|&(x, y, width)| ExtrusionJunction::new(Point::new(x, y), width, inset))
        .collect();
    line
}

fn requirements(lines: &[ExtrusionLine], outer_first: bool) -> HashSet<(usize, usize)> {
    get_region_order(&lines.iter().collect::<Vec<_>>(), outer_first)
}

#[test]
fn adjacent_even_walls_follow_selected_direction_not_input_order() {
    let lines = [
        line(2, false, &[(0, 800, 400)]),
        line(0, false, &[(0, 0, 400)]),
        line(1, false, &[(0, 400, 400)]),
    ];
    assert_eq!(requirements(&lines, true), HashSet::from([(1, 2), (2, 0)]));
    assert_eq!(requirements(&lines, false), HashSet::from([(0, 2), (2, 1)]));
}

#[test]
fn odd_center_wall_is_blocked_only_by_adjacent_even_outer_wall() {
    let lines = [
        line(0, false, &[(0, 0, 400)]),
        line(1, true, &[(0, 400, 400)]),
        line(2, false, &[(0, 800, 400)]),
    ];
    for outer_first in [true, false] {
        assert_eq!(requirements(&lines, outer_first), HashSet::from([(0, 1)]));
    }
    let odd = [
        line(0, true, &[(0, 0, 400)]),
        line(1, true, &[(0, 400, 400)]),
    ];
    assert!(requirements(&odd, true).is_empty());
}

#[test]
fn diagonal_extension_uses_integer_half_sum_then_float_and_inclusive_distance() {
    // (100 + 101) / 2 * 1.9f truncates to 190, not 190.95.
    let at_radius = [
        line(0, false, &[(0, 0, 100)]),
        line(1, false, &[(114, 152, 101)]),
    ];
    assert_eq!(requirements(&at_radius, true), HashSet::from([(0, 1)]));
    let beyond_radius = [
        line(0, false, &[(0, 0, 100)]),
        line(1, false, &[(114, 153, 101)]),
    ];
    assert!(requirements(&beyond_radius, true).is_empty());
    let truncation = [
        line(0, false, &[(0, 0, 100)]),
        line(1, false, &[(190, 1, 101)]),
    ];
    assert!(requirements(&truncation, true).is_empty());
}

#[test]
fn only_neighboring_vertices_constrain_paths_and_duplicates_do_not_count_twice() {
    let lines = [
        line(
            0,
            false,
            &[(-1000, -1000, 400), (1000, -1000, 400), (-1000, -1000, 400)],
        ),
        line(1, false, &[(-1000, -600, 400), (1000, -600, 400)]),
        line(1, false, &[(0, -900, 400)]),
        line(2, false, &[(-1000, -1000, 400)]),
    ];
    assert_eq!(requirements(&lines, true), HashSet::from([(0, 1), (1, 3)]));
}

#[test]
fn empty_and_zero_width_inputs_have_no_constraints() {
    assert!(get_region_order(&[], true).is_empty());
    let lines = [line(0, false, &[]), line(1, false, &[(0, 0, 0)])];
    assert!(requirements(&lines, false).is_empty());
}

#[test]
fn variable_width_nearby_threshold_depends_on_each_junction() {
    let narrow = line(0, false, &[(-1000, 0, 100), (1000, 0, 800)]);
    let lines = [
        narrow,
        line(1, false, &[(-1000, 500, 100)]),
        line(1, false, &[(1000, 500, 100)]),
    ];
    assert_eq!(requirements(&lines, true), HashSet::from([(0, 2)]));
}
