use crate::{
    Project, SliceError, geometry::CoordinateScale,
    project::effective_config::types::BoundedResolvedProjectConfig,
};

use super::ProjectBytes;
use super::compensation::{PreparedPostCompensation, prepare_post_compensation};
use context::prepare_perimeter_contexts;

pub(super) mod arachne;
pub(super) mod classic;
pub(super) mod layer_region;
use preflight::preflight_perimeter_flows;
use types::PostPerimeterInputPrintObject;

pub(super) mod context;
pub(super) mod flow;
pub(super) mod preflight;
pub(super) mod types;

pub(super) struct PreparedPostPerimeterInputs {
    pub(super) project: Project,
    pub(super) resolved: BoundedResolvedProjectConfig,
    pub(super) config_block: Option<Vec<u8>>,
    pub(super) scale: CoordinateScale,
    pub(super) objects: Vec<PostPerimeterInputPrintObject>,
}

pub(super) fn prepare_post_perimeter_inputs<'a>(
    project: impl ProjectBytes<'a>,
) -> Result<PreparedPostPerimeterInputs, SliceError> {
    finish_post_perimeter_inputs(prepare_post_compensation(project.into_source())?)
}

pub(super) fn prepare_post_classic_prelude<'a>(
    project: impl ProjectBytes<'a>,
) -> Result<classic::PreparedPostClassicPrelude, SliceError> {
    classic::finish_classic_prelude(prepare_post_perimeter_inputs(project.into_source())?)
}

pub(super) fn prepare_post_classic_top_split<'a>(
    project: impl ProjectBytes<'a>,
) -> Result<classic::PreparedPostClassicTopSplit, SliceError> {
    classic::finish_classic_top_split(prepare_post_classic_prelude(project.into_source())?)
}

pub(super) fn prepare_post_classic_onion<'a>(
    project: impl ProjectBytes<'a>,
) -> Result<Box<classic::PreparedPostClassicOnion>, SliceError> {
    Ok(Box::new(classic::finish_classic_onion(
        prepare_post_classic_top_split(project.into_source())?,
    )?))
}

pub(super) fn prepare_post_classic_hierarchy<'a>(
    project: impl ProjectBytes<'a>,
) -> Result<Box<classic::PreparedPostClassicHierarchy>, SliceError> {
    Ok(Box::new(classic::finish_classic_hierarchy(
        *prepare_post_classic_onion(project.into_source())?,
    )))
}

pub(super) fn prepare_post_classic_traversal<'a>(
    project: impl ProjectBytes<'a>,
) -> Result<Box<classic::PreparedPostClassicTraversal>, SliceError> {
    Ok(Box::new(classic::finish_classic_traversal(
        *prepare_post_classic_hierarchy(project.into_source())?,
    )))
}

pub(super) fn prepare_post_classic_raw_paths<'a>(
    project: impl ProjectBytes<'a>,
) -> Result<classic::PreparedPostClassicRawPaths, SliceError> {
    classic::finish_classic_raw_paths(prepare_post_classic_traversal(project.into_source())?)
}

pub(super) fn prepare_post_classic_chained_loops<'a>(
    project: impl ProjectBytes<'a>,
) -> Result<classic::PreparedPostClassicChainedLoops, SliceError> {
    Ok(classic::finish_classic_chained_loops(
        prepare_post_classic_raw_paths(project.into_source())?,
    ))
}

pub(super) fn prepare_post_classic_entity_collections<'a>(
    project: impl ProjectBytes<'a>,
) -> Result<classic::PreparedPostClassicEntityCollections, SliceError> {
    Ok(classic::finish_classic_entity_collections(
        prepare_post_classic_chained_loops(project.into_source())?,
    ))
}

pub(super) fn prepare_post_classic_perimeter_append<'a>(
    project: impl ProjectBytes<'a>,
) -> Result<classic::PreparedPostClassicPerimeterAppend, SliceError> {
    Ok(classic::finish_classic_perimeter_append(
        prepare_post_classic_entity_collections(project.into_source())?,
    ))
}

pub(super) fn prepare_post_classic_gap_domain<'a>(
    project: impl ProjectBytes<'a>,
) -> Result<classic::PreparedPostClassicGapDomain, SliceError> {
    classic::finish_classic_gap_domain(prepare_post_classic_perimeter_append(
        project.into_source(),
    )?)
}

pub(super) fn prepare_post_classic_medial_gap<'a>(
    project: impl ProjectBytes<'a>,
) -> Result<classic::PreparedPostClassicMedialGap, SliceError> {
    classic::finish_classic_medial_gap(prepare_post_classic_gap_domain(project.into_source())?)
}

pub(super) fn prepare_post_classic_gap_extrusion<'a>(
    project: impl ProjectBytes<'a>,
) -> Result<classic::PreparedPostClassicGapExtrusion, SliceError> {
    classic::finish_classic_gap_extrusion(prepare_post_classic_medial_gap(project.into_source())?)
}

pub(super) fn prepare_post_classic_infill_boundary<'a>(
    project: impl ProjectBytes<'a>,
) -> Result<classic::PreparedPostClassicInfillBoundary, SliceError> {
    classic::finish_classic_infill_boundary(prepare_post_classic_gap_extrusion(
        project.into_source(),
    )?)
}

pub(super) fn prepare_post_layer_region_perimeters<'a>(
    project: impl ProjectBytes<'a>,
) -> Result<layer_region::PreparedPostLayerRegionPerimeters, SliceError> {
    Ok(layer_region::finish(prepare_post_classic_infill_boundary(
        project,
    )?))
}

pub(super) fn finish_post_perimeter_inputs(
    prepared: PreparedPostCompensation,
) -> Result<PreparedPostPerimeterInputs, SliceError> {
    let initial_layer_width = prepared
        .resolved
        .views
        .full
        .process
        .print
        .initial_layer_line_width;
    let nozzle_diameters = &prepared.resolved.views.full.project.print.nozzle_diameter;
    let spiral_mode = prepared.resolved.views.full.process.print.spiral_mode.0;
    let flows = preflight_perimeter_flows(
        &prepared.objects,
        &prepared.resolved.objects,
        initial_layer_width,
        nozzle_diameters,
    )?;
    let PreparedPostCompensation {
        project,
        resolved,
        config_block,
        scale,
        objects,
    } = prepared;
    let objects = prepare_perimeter_contexts(objects, flows, &resolved.objects, spiral_mode);
    Ok(PreparedPostPerimeterInputs {
        project,
        resolved,
        config_block,
        scale,
        objects,
    })
}
