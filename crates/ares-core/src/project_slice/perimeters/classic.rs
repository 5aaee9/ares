// Source boundary: OrcaSlicer v2.4.2 `LayerRegion::make_perimeters` and the
// classic-perimeter prefix through pre-medial gap-domain simplification.

pub(in crate::project_slice) mod chained_loops;
pub(in crate::project_slice) mod entity_collections;
mod fuzzy_skin;
pub(in crate::project_slice) mod gap_domain;
pub(in crate::project_slice) mod gap_extrusion;
pub(in crate::project_slice) mod hierarchy;
pub(in crate::project_slice) mod infill_boundary;
pub(in crate::project_slice) mod materialize;
pub(in crate::project_slice) mod medial_gap;
pub(in crate::project_slice) mod onion;
pub(in crate::project_slice) mod perimeter_append;
mod preflight;
pub(in crate::project_slice) mod prelude;
pub(in crate::project_slice) mod shortest_path;
pub(in crate::project_slice) mod top_split;
pub(in crate::project_slice) mod traversal;
mod types;

pub(in crate::project_slice) use chained_loops::PreparedPostClassicChainedLoops;
pub(in crate::project_slice) use entity_collections::PreparedPostClassicEntityCollections;
pub(in crate::project_slice) use gap_domain::PreparedPostClassicGapDomain;
pub(in crate::project_slice) use gap_extrusion::PreparedPostClassicGapExtrusion;
pub(in crate::project_slice) use hierarchy::{
    PostClassicHierarchyPrintObject, PreparedPostClassicHierarchy,
};
pub(in crate::project_slice) use infill_boundary::PreparedPostClassicInfillBoundary;
pub(in crate::project_slice) use materialize::PreparedPostClassicRawPaths;
pub(in crate::project_slice) use medial_gap::PreparedPostClassicMedialGap;
pub(in crate::project_slice) use onion::{PostClassicOnionPrintObject, PreparedPostClassicOnion};
pub(in crate::project_slice) use perimeter_append::PreparedPostClassicPerimeterAppend;
pub(in crate::project_slice) use top_split::PreparedPostClassicTopSplit;
pub(in crate::project_slice) use top_split::{
    ClassicTopSplitRecord, PostClassicTopSplitPrintObject,
};
pub(in crate::project_slice) use traversal::{
    PostClassicTraversalPrintObject, PreparedPostClassicTraversal,
};
pub(in crate::project_slice) use types::{ClassicPreludeRecord, PreparedPostClassicPrelude};

use crate::SliceError;

use super::PreparedPostPerimeterInputs;
use preflight::ClassicValidationContext;
use types::PostClassicPreludePrintObject;

pub(super) fn finish_classic_prelude(
    prepared: PreparedPostPerimeterInputs,
) -> Result<PreparedPostClassicPrelude, SliceError> {
    let enable_arc_fitting = prepared
        .resolved
        .views
        .full
        .process
        .gcode
        .enable_arc_fitting
        .0;
    let resolution = prepared.resolved.views.full.process.print.resolution.0;
    let validated = preflight::validate_project(
        &prepared.objects,
        ClassicValidationContext {
            resolved_objects: &prepared.resolved.objects,
            enable_arc_fitting,
            resolution,
            nozzle_diameters: &prepared.resolved.views.full.project.print.nozzle_diameter,
            scale: prepared.scale,
        },
    )?;
    let objects = prepared
        .objects
        .into_iter()
        .zip(validated)
        .map(|(object, validated)| {
            let records = prelude::prepare_object(&object, validated, prepared.scale)?;
            Ok(PostClassicPreludePrintObject { object, records })
        })
        .collect::<Result<Vec<_>, SliceError>>()?;

    Ok(PreparedPostClassicPrelude {
        project: prepared.project,
        resolved: Box::new(prepared.resolved),
        config_block: prepared.config_block,
        scale: prepared.scale,
        objects,
    })
}

pub(super) fn finish_classic_top_split(
    prepared: PreparedPostClassicPrelude,
) -> Result<PreparedPostClassicTopSplit, SliceError> {
    top_split::finish(prepared)
}

pub(super) fn finish_classic_onion(
    prepared: PreparedPostClassicTopSplit,
) -> Result<PreparedPostClassicOnion, SliceError> {
    onion::finish(prepared)
}

pub(super) fn finish_classic_hierarchy(
    prepared: PreparedPostClassicOnion,
) -> PreparedPostClassicHierarchy {
    hierarchy::finish(prepared)
}

pub(super) fn finish_classic_traversal(
    prepared: PreparedPostClassicHierarchy,
) -> PreparedPostClassicTraversal {
    traversal::finish(prepared)
}

pub(super) fn finish_classic_raw_paths(
    prepared: Box<PreparedPostClassicTraversal>,
) -> Result<PreparedPostClassicRawPaths, SliceError> {
    materialize::finish(prepared)
}

pub(super) fn finish_classic_chained_loops(
    prepared: PreparedPostClassicRawPaths,
) -> PreparedPostClassicChainedLoops {
    chained_loops::finish(prepared)
}

pub(super) fn finish_classic_entity_collections(
    prepared: PreparedPostClassicChainedLoops,
) -> PreparedPostClassicEntityCollections {
    entity_collections::finish(prepared)
}

pub(super) fn finish_classic_perimeter_append(
    prepared: PreparedPostClassicEntityCollections,
) -> PreparedPostClassicPerimeterAppend {
    perimeter_append::finish(prepared)
}

pub(super) fn finish_classic_gap_domain(
    prepared: PreparedPostClassicPerimeterAppend,
) -> Result<PreparedPostClassicGapDomain, SliceError> {
    gap_domain::finish(prepared)
}

pub(super) fn finish_classic_medial_gap(
    prepared: PreparedPostClassicGapDomain,
) -> Result<PreparedPostClassicMedialGap, SliceError> {
    medial_gap::finish(prepared)
}

pub(super) fn finish_classic_gap_extrusion(
    prepared: PreparedPostClassicMedialGap,
) -> Result<PreparedPostClassicGapExtrusion, SliceError> {
    gap_extrusion::finish(prepared)
}

pub(super) fn finish_classic_infill_boundary(
    prepared: PreparedPostClassicGapExtrusion,
) -> Result<PreparedPostClassicInfillBoundary, SliceError> {
    infill_boundary::finish(prepared)
}
