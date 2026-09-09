use super::super::{
    chained_loops::ExtrusionLoop,
    materialize::{ExtrusionPath, Point3},
    traversal::PreparedPostClassicTraversal,
};

pub(in crate::project_slice) struct PreparedPostClassicEntityCollections {
    pub(in crate::project_slice) predecessor: Box<PreparedPostClassicTraversal>,
    pub(in crate::project_slice) objects: Vec<PreparedEntityCollectionObject>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PreparedEntityCollectionObject {
    pub(in crate::project_slice) records: Vec<Option<PreparedEntityCollectionRecord>>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PreparedEntityCollectionRecord {
    pub(in crate::project_slice) surfaces: Vec<PreparedEntityCollectionSurface>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PreparedEntityCollectionSurface {
    pub(in crate::project_slice) source_index: usize,
    pub(in crate::project_slice) collection: ExtrusionEntityCollection,
}

#[derive(Debug, Default, PartialEq)]
pub(in crate::project_slice) struct ExtrusionEntityCollection {
    pub(in crate::project_slice) entities: Vec<ExtrusionEntity>,
    pub(in crate::project_slice) source_order: usize,
}

/// Collection element vocabulary shared by classic and arachne wall
/// generators (OrcaSlicer `ExtrusionEntity.hpp`): classic orders loops
/// (`PerimeterGenerator.cpp:270`) while arachne `traverse_extrusions`
/// appends loops and open variable-width multi-paths into the same
/// collection (`PerimeterGenerator.cpp:518-566`).
#[derive(Debug, PartialEq)]
pub(in crate::project_slice) enum ExtrusionEntity {
    Loop(OrderedExtrusionLoop),
    MultiPath(ExtrusionMultiPath),
}

impl ExtrusionEntity {
    /// `ExtrusionEntity::inset_idx` base default is -1 (`ExtrusionEntity.hpp`);
    /// only classic loops set it from loop depth (`PerimeterGenerator.cpp:270`).
    pub(in crate::project_slice) const fn inset_idx(&self) -> i32 {
        match self {
            Self::Loop(loop_) => loop_.inset_idx,
            Self::MultiPath(_) => -1,
        }
    }

    /// `ExtrusionEntity::first_point` (`ExtrusionEntity.hpp`): the first
    /// point of the first sub-path, identical for loops and multi-paths.
    pub(in crate::project_slice) fn first_point3(&self) -> &Point3 {
        match self {
            Self::Loop(loop_) => &loop_.extrusion_loop.paths[0].polyline.points[0],
            Self::MultiPath(multi_path) => &multi_path.paths[0].polyline.points[0],
        }
    }
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct OrderedExtrusionLoop {
    pub(in crate::project_slice) extrusion_loop: ExtrusionLoop,
    pub(in crate::project_slice) inset_idx: i32,
}

/// Single continuous extrusion path, possibly with varying extrusion
/// thickness, extrusion height or bridging / non bridging (OrcaSlicer
/// `ExtrusionEntity.hpp` `ExtrusionMultiPath`). Arachne's `traverse_extrusions`
/// builds these from open wall lines whose per-junction widths clip into
/// constant-width sub-paths (`PerimeterGenerator.cpp:553-566`).
#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct ExtrusionMultiPath {
    pub(in crate::project_slice) paths: Vec<ExtrusionPath>,
}
