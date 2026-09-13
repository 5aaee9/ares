//! Temporary rectangle router — a simplified stand-in for the ported
//! `AvoidCrossingPerimeters::travel_to` used while the boundary router's
//! multi-point emission branch (`GCodeWriter::travel_to_xyz` ramp over
//! detour points, `GCode.cpp:7486-7505`) is completed. Replaced by
//! `router::avoid_perimeters` once the detour waypoints match upstream.

use crate::project_slice::gcode_emit::motion::{LayerGeometry, arc};

use super::Request;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Edge {
    Right,
    Left,
    Top,
    Bottom,
}

#[derive(Clone, Copy)]
struct Rect {
    left: f64,
    right: f64,
    bottom: f64,
    top: f64,
}

pub(in crate::project_slice::gcode_emit) fn route(request: Request<'_>) -> Vec<arc::Point> {
    let Request {
        start,
        end,
        geometry,
        offset,
        inset,
        after_skirt,
        use_external: _,
    } = request;
    let Some(rect) = rectangle(&geometry, offset, inset) else {
        return Vec::new();
    };
    let start_inside = rect.contains(start);
    let end_inside = rect.contains(end);
    if start_inside == end_inside {
        return Vec::new();
    }
    let (start_projection, start_edge) = rect.project(start);
    let (end_projection, end_edge) = rect.project(end);
    let mut output;
    if start_inside {
        output = rect.boundary_path(start_projection, start_edge, end_projection, end_edge);
    } else {
        if after_skirt {
            output = vec![end_projection];
        } else {
            let boundary =
                rect.boundary_path(start_projection, start_edge, end_projection, end_edge);
            output = if boundary_detour_is_excessive(&boundary, end) {
                vec![start_projection]
            } else {
                boundary
            };
        }
    }
    output.dedup();
    output
}

impl Rect {
    fn contains(self, point: arc::Point) -> bool {
        (self.left..=self.right).contains(&point.x) && (self.bottom..=self.top).contains(&point.y)
    }

    fn project(self, point: arc::Point) -> (arc::Point, Edge) {
        if !self.contains(point) {
            let x = point.x.clamp(self.left, self.right);
            let y = point.y.clamp(self.bottom, self.top);
            let edge = if point.x > self.right {
                Edge::Right
            } else if point.x < self.left {
                Edge::Left
            } else if point.y > self.top {
                Edge::Top
            } else {
                Edge::Bottom
            };
            return (arc::Point { x, y }, edge);
        }
        let choices = [
            (self.right - point.x, Edge::Right),
            (point.x - self.left, Edge::Left),
            (self.top - point.y, Edge::Top),
            (point.y - self.bottom, Edge::Bottom),
        ];
        let (_, edge) = choices
            .into_iter()
            .min_by(|left, right| left.0.total_cmp(&right.0))
            .unwrap();
        let projected = match edge {
            Edge::Right => arc::Point {
                x: self.right,
                y: point.y,
            },
            Edge::Left => arc::Point {
                x: self.left,
                y: point.y,
            },
            Edge::Top => arc::Point {
                x: point.x,
                y: self.top,
            },
            Edge::Bottom => arc::Point {
                x: point.x,
                y: self.bottom,
            },
        };
        (projected, edge)
    }

    fn boundary_path(
        self,
        start: arc::Point,
        start_edge: Edge,
        end: arc::Point,
        end_edge: Edge,
    ) -> Vec<arc::Point> {
        if start_edge == end_edge {
            return vec![start, end];
        }
        let corner = match (start_edge, end_edge) {
            (Edge::Top, Edge::Right) | (Edge::Right, Edge::Top) => arc::Point {
                x: self.right,
                y: self.top,
            },
            (Edge::Top, Edge::Left) | (Edge::Left, Edge::Top) => arc::Point {
                x: self.left,
                y: self.top,
            },
            (Edge::Bottom, Edge::Right) | (Edge::Right, Edge::Bottom) => arc::Point {
                x: self.right,
                y: self.bottom,
            },
            (Edge::Bottom, Edge::Left) | (Edge::Left, Edge::Bottom) => arc::Point {
                x: self.left,
                y: self.bottom,
            },
            _ => return vec![start, end],
        };
        vec![start, corner, end]
    }
}

fn boundary_detour_is_excessive(boundary: &[arc::Point], end: arc::Point) -> bool {
    let direct = distance(boundary[0], end);
    let detour = boundary
        .windows(2)
        .map(|points| distance(points[0], points[1]))
        .sum::<f64>()
        + distance(*boundary.last().expect("nonempty"), end);
    detour > 1.4 * direct
}

fn distance(first: arc::Point, second: arc::Point) -> f64 {
    (second.x - first.x).hypot(second.y - first.y)
}

fn rectangle(geometry: &LayerGeometry<'_>, offset: (f64, f64), inset: f64) -> Option<Rect> {
    let [surface] = geometry.internal_surfaces else {
        return None;
    };
    let (_, expolygon, _, _, _, _) = surface.as_parts();
    if !expolygon.holes().is_empty() || expolygon.contour().points().len() != 4 {
        return None;
    }
    let points = expolygon.contour().points();
    if !points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(4)
        .all(|(first, second)| first.x() == second.x() || first.y() == second.y())
    {
        return None;
    }
    let left = points.iter().map(|point| point.x()).min()?;
    let right = points.iter().map(|point| point.x()).max()?;
    let bottom = points.iter().map(|point| point.y()).min()?;
    let top = points.iter().map(|point| point.y()).max()?;
    Some(Rect {
        left: geometry.scale.unscale(left) + offset.0 + inset,
        right: geometry.scale.unscale(right) + offset.0 - inset,
        bottom: geometry.scale.unscale(bottom) + offset.1 + inset,
        top: geometry.scale.unscale(top) + offset.1 - inset,
    })
}
