use crate::{
    geometry::ExPolygon,
    project_slice::{
        perimeters::classic::{
            entity_collections::ExtrusionEntityCollection, gap_extrusion::GapFillEntity,
            traversal::PreparedPostClassicTraversal,
        },
        region_slices::RegionSurface,
    },
};

/// Common typed predecessor envelope crossing the post-perimeter boundary.
/// Classic wall generation carries its prepared traversal; an arachne variant
/// joins when `PerimeterGenerator::process_arachne`
/// (OrcaSlicer `PerimeterGenerator.cpp:2093`) materializes. Consumers drill
/// the owning generator's hierarchy; a classic hierarchy is never fabricated
/// for arachne results.
pub(in crate::project_slice) enum PostPerimeterPredecessor {
    Classic(Box<PreparedPostClassicTraversal>),
}

impl PostPerimeterPredecessor {
    pub(in crate::project_slice) fn as_classic(&self) -> &PreparedPostClassicTraversal {
        match self {
            Self::Classic(traversal) => traversal,
        }
    }

    #[cfg(test)]
    pub(in crate::project_slice) fn as_classic_mut(&mut self) -> &mut PreparedPostClassicTraversal {
        match self {
            Self::Classic(traversal) => traversal,
        }
    }

    pub(in crate::project_slice) fn into_classic(self) -> Box<PreparedPostClassicTraversal> {
        match self {
            Self::Classic(traversal) => traversal,
        }
    }
}

pub(in crate::project_slice) struct PreparedPostLayerRegionPerimeters {
    pub(in crate::project_slice) predecessor: PostPerimeterPredecessor,
    pub(in crate::project_slice) objects: Vec<PreparedLayerRegionPerimeterObject>,
}

pub(in crate::project_slice) struct PreparedLayerRegionPerimeterObject {
    pub(in crate::project_slice) records: Vec<Option<PreparedLayerRegionPerimeterRecord>>,
}

pub(in crate::project_slice) struct PreparedLayerRegionPerimeterRecord {
    pub(in crate::project_slice) perimeters: Vec<ExtrusionEntityCollection>,
    pub(in crate::project_slice) thin_fills: Vec<GapFillEntity>,
    pub(in crate::project_slice) fill_surfaces: Vec<RegionSurface>,
    pub(in crate::project_slice) fill_expolygons: Vec<ExPolygon>,
    pub(in crate::project_slice) fill_no_overlap_expolygons: Vec<ExPolygon>,
}
