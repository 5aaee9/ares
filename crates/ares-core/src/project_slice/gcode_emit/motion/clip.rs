pub(in crate::project_slice::gcode_emit) fn clip_end(points: &mut Vec<(i64, i64)>, distance: f64) {
    if distance <= 0.0 {
        return;
    }
    let mut remaining = distance;
    while points.len() > 1 {
        let last = points[points.len() - 1];
        let previous = points[points.len() - 2];
        // Upstream `Polyline::clip_end` (`Polyline.cpp:52-72`): the squared
        // comparison and the sqrt division order match exactly; the final
        // `.cast<coord_t>()` on the Vec2d is Eigen's per-component
        // `static_cast<coord_t>` — truncation toward zero, NOT round-half-up.
        let vx = (previous.0 - last.0) as f64;
        let vy = (previous.1 - last.1) as f64;
        let lsqr = vx * vx + vy * vy;
        if lsqr > remaining * remaining {
            let factor = remaining / lsqr.sqrt();
            let endpoint = (
                (last.0 as f64 + vx * factor) as i64,
                (last.1 as f64 + vy * factor) as i64,
            );
            *points.last_mut().expect("the path has an endpoint") = endpoint;
            return;
        }
        points.pop();
        remaining -= lsqr.sqrt();
        if remaining <= f64::EPSILON {
            return;
        }
    }
    points.clear();
}

#[cfg(test)]
mod tests {
    #[test]
    fn clips_a_closed_loop_from_its_end() {
        let mut points = vec![(0, 0), (4, 0), (4, 4), (0, 4), (0, 0)];

        super::clip_end(&mut points, 1.0);

        assert_eq!(points, vec![(0, 0), (4, 0), (4, 4), (0, 4), (0, 1)]);
    }

    #[test]
    fn negative_fractional_endpoint_truncates_toward_zero() {
        // Upstream `Polyline::clip_end` emplaces
        // `(last_point + v * factor).cast<coord_t>()` (`Polyline.cpp:67`) —
        // Eigen's per-component `static_cast` truncates toward zero: raw
        // y = -6892499.659 truncates to -6892499, not -6892500.
        let mut points = vec![(-7_549_495, -6_469_541), (-7_099_181, -6_920_814)];

        super::clip_end(&mut points, 40_000.0);

        assert_eq!(points[1], (-7_127_435, -6_892_499));
    }

    #[test]
    fn clips_across_short_terminal_segments() {
        let mut points = vec![(0, 0), (2, 0), (2, 2), (0, 2), (0, 0)];

        super::clip_end(&mut points, 3.0);

        assert_eq!(points, vec![(0, 0), (2, 0), (2, 2), (1, 2)]);
    }
}
