//! Owned buffers backing one entry's `LayerGeometry` borrows.
use super::super::motion;
use crate::geometry::ExPolygon;

pub(super) struct EntryGeometry<'a> {
    pub(super) top_surfaces: Vec<&'a ExPolygon>,
    pub(super) lower_boundary_lines: Vec<crate::geometry::Line>,
    pub(super) nearest_penalties:
        Option<&'a crate::project_slice::island_print_order::NearestSeamLayer>,
    pub(super) layer_slices: std::rc::Rc<[ExPolygon]>,
    pub(super) internal_surfaces: &'a [crate::project_slice::region_slices::RegionSurface],
    /// The chunk-wide slice list and cross-object average spacing for the
    /// external motion planner (`get_boundary_external`).
    pub(super) chunk_slices: Vec<ExPolygon>,
    pub(super) chunk_perimeter_spacing: f64,
}

impl EntryGeometry<'_> {
    /// Builds the emit-time geometry view; the distance tree borrows the
    /// owned lines for the duration of one emission.
    pub(super) fn view<'a>(
        &'a self,
        traversal: &'a crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
        object_index: usize,
        layer_index: usize,
        previous_layer_boundary: Option<&'a crate::geometry::LineDistanceTree<'a>>,
    ) -> motion::LayerGeometry<'a> {
        motion::LayerGeometry {
            nearest_seam_penalties: self.nearest_penalties,
            staggered_inner: self.nearest_penalties.is_some()
                && traversal.resolved.objects[object_index]
                    .object
                    .staggered_inner_seams
                    .0,
            internal_surfaces: self.internal_surfaces,
            scale: traversal.scale,
            previous_layer_boundary,
            avoid_crossing: motion::AvoidCrossingGeometry {
                layer_slices: &self.layer_slices,
                perimeter_spacing: traversal.objects[object_index]
                    .perimeter_spacing(layer_index)
                    .unwrap_or_default(),
                external_perimeter_width: traversal.objects[object_index]
                    .external_perimeter_width(layer_index)
                    .unwrap_or_default(),
                top_surfaces: &self.top_surfaces,
                chunk_slices: &self.chunk_slices,
                chunk_perimeter_spacing: self.chunk_perimeter_spacing,
            },
        }
    }
}
