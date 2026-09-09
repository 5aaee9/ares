use crate::ProcessWallDirection;

use super::super::chained_loops::ExtrusionLoop;

pub(in crate::project_slice) fn orient_loop(
    loop_: &mut ExtrusionLoop,
    wall_direction: ProcessWallDirection,
    is_contour: bool,
    reverse_thin_wall_hole: bool,
) {
    let counter_clockwise = match wall_direction {
        ProcessWallDirection::CounterClockwise => true,
        ProcessWallDirection::Clockwise => false,
    };
    let source_wants_counter_clockwise =
        counter_clockwise == (is_contour || reverse_thin_wall_hole);
    if is_counter_clockwise(loop_) != source_wants_counter_clockwise {
        reverse_loop(loop_);
    }
}

pub(in crate::project_slice) fn reverse_loop(loop_: &mut ExtrusionLoop) {
    for path in &mut loop_.paths {
        path.reverse();
    }
    loop_.paths.reverse();
}

pub(in crate::project_slice) fn is_counter_clockwise(loop_: &ExtrusionLoop) -> bool {
    let mut last = None;
    for path in loop_.paths.iter().rev() {
        if path.polyline.points.len() > 1 {
            last = path.polyline.points.get(path.polyline.points.len() - 2);
            break;
        }
    }
    let mut previous = last.expect("an O8 loop polygon has at least one point");
    let mut area = 0.0;
    for path in &loop_.paths {
        for current in path
            .polyline
            .points
            .iter()
            .take(path.polyline.points.len() - 1)
        {
            area += (previous.x as f64 + current.x as f64) * (previous.y as f64 - current.y as f64);
            previous = current;
        }
    }
    -area * 0.5 >= 0.0
}
