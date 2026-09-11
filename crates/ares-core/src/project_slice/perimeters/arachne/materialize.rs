// `PerimeterGenerator::process_arachne` materialization from OrcaSlicer
// v2.4.2 `PerimeterGenerator.cpp:2093-2519`: the per-record transaction that
// generates arachne walls, orders them, converts them through
// `traverse_extrusions` (`:370-574`) into the typed entity vocabulary and
// produces the layer-region perimeter record. Classic records are untouched;
// arachne records carry no classic surfaces, so the classic chain computes
// their shared context only and this stage replaces their entities at the
// layer-region boundary.

use crate::{
    SliceError,
    geometry::CoordinateScale,
    project_slice::perimeters::{
        classic::{ClassicPreludeRecord, PreparedPostClassicPrelude},
        layer_region::PreparedLayerRegionPerimeterRecord,
        types::{PerimeterDispatch, PerimeterInputRecord, PostPerimeterInputPrintObject},
    },
};

use super::{
    config::{self},
    order::order_walls,
    traverse::{self, TraverseExtrusionsContext},
    walls::{self, ArachneSpacings, InfillOverlapPercents},
};

/// The arachne half of `PreparedPostLayerRegionPerimeters`: one record slot
/// per classic record, `Some` exactly where the record dispatches arachne.
pub(in crate::project_slice) struct PreparedArachnePerimeters {
    pub(in crate::project_slice) objects: Vec<PreparedArachneObject>,
}

#[derive(Default)]
pub(in crate::project_slice) struct PreparedArachneObject {
    pub(in crate::project_slice) records: Vec<Option<PreparedLayerRegionPerimeterRecord>>,
}

pub(in crate::project_slice) fn finish(
    prelude: &PreparedPostClassicPrelude,
) -> Result<PreparedArachnePerimeters, SliceError> {
    let enable_arc_fitting = prelude
        .resolved
        .views
        .full
        .process
        .gcode
        .enable_arc_fitting
        .0;
    let resolution = prelude.resolved.views.full.process.print.resolution.0;
    let nozzle_diameters = &prelude.resolved.views.full.project.print.nozzle_diameter;
    let objects = prelude
        .objects
        .iter()
        .map(|object| {
            let (source_object_index, _) = object.object.identity();
            let object_options = &prelude
                .resolved
                .objects
                .iter()
                .find(|resolved| resolved.source_object_index == source_object_index)
                .expect("the prelude object retains its resolved source")
                .object;
            let records = object
                .object
                .as_parts()
                .1
                .iter()
                .zip(&object.records)
                .map(|(input, prelude_record)| match (input, prelude_record) {
                    (Some(input), Some(prelude_record))
                        if input.dispatch == PerimeterDispatch::Arachne =>
                    {
                        materialize_record(
                            &object.object,
                            input,
                            prelude_record,
                            object_options,
                            enable_arc_fitting,
                            resolution,
                            nozzle_diameters,
                            prelude.scale,
                        )
                        .map(Some)
                    }
                    _ => Ok(None),
                })
                .collect::<Result<Vec<_>, SliceError>>()?;
            Ok(PreparedArachneObject { records })
        })
        .collect::<Result<Vec<_>, SliceError>>()?;
    Ok(PreparedArachnePerimeters { objects })
}

fn materialize_record(
    object: &PostPerimeterInputPrintObject,
    input: &PerimeterInputRecord,
    prelude_record: &ClassicPreludeRecord,
    object_options: &crate::ObjectOptions,
    enable_arc_fitting: bool,
    resolution: f64,
    nozzle_diameters: &crate::OrcaFloats,
    scale: CoordinateScale,
) -> Result<PreparedLayerRegionPerimeterRecord, SliceError> {
    let region = object.region_options(input);
    let config = config::validate_record(
        region,
        object_options,
        input.layer_id,
        input.upper_layer_index,
        enable_arc_fitting,
        resolution,
        nozzle_diameters,
        scale,
    )?;
    let spacings = ArachneSpacings::new(
        scale,
        input.perimeter_flow.spacing,
        input.ext_perimeter_flow.width,
        input.ext_perimeter_flow.spacing,
        input.solid_infill_flow.spacing,
    )?;
    let overlaps = InfillOverlapPercents {
        infill_wall_overlap: region.infill_wall_overlap.0,
        top_bottom_infill_wall_overlap: region.top_bottom_infill_wall_overlap.0,
    };
    let mut perimeters = Vec::new();
    let mut fill_surfaces = Vec::new();
    let mut fill_no_overlap = Vec::new();
    // we need each island separately (`PerimeterGenerator.cpp:2124-2126`).
    for surface in object.current_surfaces(input) {
        let generated = walls::generate_surface_walls(
            surface,
            &config,
            &spacings,
            region.wall_loops.0,
            input.layer_height,
            scale,
        )?;
        // `loop_number = int(perimeters.size()) - 1`
        // (`PerimeterGenerator.cpp:2466`).
        let loop_number = generated.toolpaths.len() as i32 - 1;
        let ordered = order_walls(&generated.toolpaths, config.outer_wall_first);
        // Inner-outer-inner sandwich reordering (`PerimeterGenerator.cpp:2374-2464`):
        // after the topological sort, gated on layer (skipped on layer 0) and
        // wall count (3 minimum). The proximity thresholds derive from the
        // flow spacings — precise-outer-wall is forced off for this sequence
        // (`PerimeterGenerator.cpp:2409-2417`), so the external threshold is
        // the half-spacing sum.
        let ordered = if region.wall_sequence == crate::ProcessWallSequence::InnerOuterInner
            && input.layer_id > 0
            && ordered.len() > 2
        {
            let threshold_external = (spacings.ext_perimeter_spacing as f64 / 2.0
                + spacings.perimeter_spacing as f64 / 2.0)
                as i64;
            let threshold_internal = spacings.perimeter_spacing;
            super::sandwich::apply_inner_outer_inner(
                ordered,
                threshold_external,
                threshold_internal,
            )
        } else {
            ordered
        };
        let outcome = traverse::traverse_extrusions(
            ordered,
            &TraverseExtrusionsContext {
                layer_id: input.layer_id,
                raft_layers: config.raft_layers,
                detect_overhang_wall: region.detect_overhang_wall.0,
                overhang_reverse: region.overhang_reverse.0,
                wall_direction: region.wall_direction,
                wall_sequence: region.wall_sequence,
                fuzzy_skin: crate::perimeters::FuzzySkinConfig::from_region(region),
                perimeter_flow: input.perimeter_flow,
                ext_perimeter_flow: input.ext_perimeter_flow,
                overhang_flow: input.overhang_flow,
                lower_slices_polygons: &prelude_record.lower_slices_polygons,
                scale,
            },
        )?;
        // `this->loops->append(extrusion_coll)` only when non-empty
        // (`PerimeterGenerator.cpp:2470-2474`).
        if !outcome.collection.entities.is_empty() {
            perimeters.push(outcome.collection);
        }
        let boundary = walls::surface_infill_boundary(
            generated.infill_contour,
            generated.toolpaths.len(),
            loop_number,
            &spacings,
            prelude_record.surface_simplify_resolution,
            overlaps,
            config.is_bottom_layer || config.is_topmost_layer,
        )?;
        fill_surfaces.extend(boundary.fill_surfaces);
        fill_no_overlap.extend(boundary.fill_no_overlap);
    }
    let fill_expolygons = fill_surfaces.clone();
    let fill_surfaces = fill_surfaces
        .into_iter()
        .map(crate::project_slice::region_slices::RegionSurface::internal)
        .collect();
    Ok(PreparedLayerRegionPerimeterRecord {
        perimeters,
        thin_fills: Vec::new(),
        fill_surfaces,
        fill_expolygons,
        fill_no_overlap_expolygons: fill_no_overlap,
    })
}
