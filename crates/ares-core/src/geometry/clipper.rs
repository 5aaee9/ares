// BSL-1.0 rewrite of the Clipper 6.4.2 state machine vendored by OrcaSlicer at
// fixed commit 8500fcdccaa10b5099ac20d252af3a7c560046f1.

mod active_edges;
mod boolean_ex;
mod boolean_paths;
mod bounds;
mod engine;
mod horizontals;
mod input;
mod intersections;
mod minima;
mod offset;
pub(crate) mod ordering;
mod output;
mod point_in_polygon;
mod polyline;
mod polytree;
mod predicates;
mod simplify;
mod strictly_simple;
mod types;
mod variable_offset;
mod winding;
pub(in crate::geometry) mod z;
pub(crate) mod z_clip;

#[cfg(test)]
pub(super) use boolean_ex::safety_offset_configuration_for_test;
pub(crate) use boolean_ex::{
    SAFETY_OFFSET, difference_ex, difference_ex_polygons,
    difference_ex_polygons_with_safety_offset, difference_ex_with_safety_offset,
    difference_polygons_ex, difference_polygons_ex_with_safety_offset, intersection_ex,
    intersection_ex_with_safety_offset, intersection_polygons_ex,
    intersection_polygons_polygons_ex, union_expolygons, union_safety_offset_ex,
    union_safety_offset_expolygons, xor_ex,
};
#[cfg(test)]
pub(crate) use boolean_paths::safety_offset_clip_paths_for_test;
pub(crate) use boolean_paths::{
    difference_polygons_paths, intersection_polygons_paths,
    intersection_polygons_paths_with_safety_offset, union_even_odd_polygons_paths,
    union_polygons_paths, union_safety_offset_polygons,
};
#[cfg(test)]
pub(crate) use bounds::{IntBounds, negative_outer};
#[cfg(test)]
pub(in crate::geometry) use input::reverse_horizontal_for_test;
#[cfg(test)]
pub(in crate::geometry) use intersections::top_updates_for_test;
pub(crate) use offset::offset_paths_tree;
#[cfg(test)]
pub(in crate::geometry) use offset::opening_path_configurations_for_test;
#[cfg(test)]
pub(crate) use offset::opening_paths_with_interstage;
pub(crate) use offset::{
    ClipperOffset, JoinType, closing_ex, offset_expolygon, offset_expolygon_refs_paths,
    offset_expolygons, offset_expolygons_paths, offset_open_paths, offset_paths, offset2_ex,
    offset2_ex_with_interstage, opening_ex, opening_paths, raw_offset_paths,
};
#[cfg(test)]
pub(crate) use offset::{offset_expolygons_raw, raw_offset_open_paths};
pub(crate) use point_in_polygon::point_in_polygon;
#[cfg(test)]
pub(crate) use polyline::recombine_polylines;
pub(crate) use polyline::{
    diff_pl, difference_open_polylines, intersection_open_polylines, intersection_pl,
    open_polyline_inside_expolygon,
};
#[cfg(test)]
pub(crate) use polytree::PolyNode;
pub(crate) use polytree::{PolyTree, union_contours, union_ex};
pub(crate) use predicates::{fixed_round, slopes_equal};
pub(super) use simplify::simplify_polygons;
#[cfg(test)]
pub(crate) use strictly_simple::MaximaCursor;
pub(crate) use variable_offset::variable_offset_inner_ex;

use std::collections::BinaryHeap;

use super::{Point, Polygon, Polyline};
use predicates::area;
use types::{
    EdgeArena, EdgeId, GhostJoin, IntersectionNode, Join, LocalMinimum, OutPointArena, OutRec,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PathRole {
    Subject,
    Clip,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ClipOperation {
    Intersection,
    Union,
    Difference,
    Xor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FillRule {
    EvenOdd,
    NonZero,
    Positive,
    Negative,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ClipperOptions {
    pub(crate) reverse_solution: bool,
    pub(crate) preserve_collinear: bool,
    pub(crate) strictly_simple: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ClipperError {
    CoordinateOutOfRange,
    OpenPathMustBeSubject,
    OpenPathsRequirePolyTree,
}

pub(crate) fn orientation(polygon: &Polygon) -> bool {
    area(polygon.points()) >= 0.0
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SimpleRepair {
    FirstLefts1,
    FirstLefts2,
}

pub(crate) struct Clipper {
    options: ClipperOptions,
    using_polytree: bool,
    edges: EdgeArena,
    minima: Vec<LocalMinimum>,
    use_full_range: bool,
    scanbeam: BinaryHeap<i64>,
    active_edges: Option<EdgeId>,
    sorted_edges: Option<EdgeId>,
    out_recs: Vec<OutRec>,
    out_points: OutPointArena,
    joins: Vec<Join>,
    ghost_joins: Vec<GhostJoin>,
    intersections: Vec<IntersectionNode>,
    maxima: Vec<i64>,
    has_open_paths: bool,
    z_intersections: Option<Vec<(i64, i64)>>,
    #[cfg(test)]
    collected_maxima_for_test: Vec<i64>,
    #[cfg(test)]
    simple_repairs_for_test: Vec<SimpleRepair>,
}

impl Clipper {
    pub(crate) fn new(options: ClipperOptions) -> Self {
        Self {
            options,
            using_polytree: false,
            edges: EdgeArena::default(),
            minima: Vec::new(),
            use_full_range: false,
            scanbeam: BinaryHeap::new(),
            active_edges: None,
            sorted_edges: None,
            out_recs: Vec::new(),
            out_points: OutPointArena::default(),
            joins: Vec::new(),
            ghost_joins: Vec::new(),
            intersections: Vec::new(),
            maxima: Vec::new(),
            has_open_paths: false,
            z_intersections: None,
            #[cfg(test)]
            collected_maxima_for_test: Vec::new(),
            #[cfg(test)]
            simple_repairs_for_test: Vec::new(),
        }
    }
}

const _: [PathRole; 2] = [PathRole::Subject, PathRole::Clip];
const _: [ClipOperation; 4] = [
    ClipOperation::Intersection,
    ClipOperation::Union,
    ClipOperation::Difference,
    ClipOperation::Xor,
];
const _: [FillRule; 4] = [
    FillRule::EvenOdd,
    FillRule::NonZero,
    FillRule::Positive,
    FillRule::Negative,
];
const _: [JoinType; 3] = [JoinType::Square, JoinType::Round, JoinType::Miter];
const _: fn() -> ClipperOffset = ClipperOffset::default;
const _: fn(ClipperOptions) -> Clipper = Clipper::new;
const _: fn(&mut Clipper, &Polygon, PathRole) -> Result<bool, ClipperError> =
    Clipper::add_closed_path;
const _: fn(&mut Clipper, &[Polygon], PathRole) -> Result<bool, ClipperError> =
    Clipper::add_closed_paths;
const _: fn(&mut Clipper, &Polyline, PathRole) -> Result<bool, ClipperError> =
    Clipper::add_open_path;
const _: fn(&mut Clipper, &[Polyline], PathRole) -> Result<bool, ClipperError> =
    Clipper::add_open_paths;
const _: fn(&mut Clipper, ClipOperation, FillRule, FillRule) -> PolyTree =
    Clipper::execute_polytree;
const _: fn(&[Polygon], FillRule) -> Result<Vec<super::ExPolygon>, ClipperError> = union_ex;
type OpenPolylineDifferenceFn = fn(&[Polyline], &[Polygon]) -> Result<Vec<Polyline>, ClipperError>;
type OpenPolylineIntersectionFn =
    fn(&[Polyline], &[Polygon]) -> Result<Vec<Polyline>, ClipperError>;
const _: OpenPolylineDifferenceFn = difference_open_polylines;
const _: OpenPolylineIntersectionFn = intersection_open_polylines;
const _: fn(&[Polygon]) -> Result<Vec<Polygon>, ClipperError> = simplify_polygons;
const _: fn(&mut Clipper) = Clipper::clear;
const _: fn(f64) -> i64 = fixed_round;
const _: fn(i64, i64, i64, i64, bool) -> bool = slopes_equal;
const _: fn(Point, &[Point]) -> i32 = point_in_polygon;
const _: fn(&[Point]) -> f64 = area;
const _: fn(&EdgeArena, types::EdgeId) -> PathRole = EdgeArena::role;
