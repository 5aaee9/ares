use crate::{
    SliceError,
    geometry::{ExPolygon, intersection_ex},
    project_slice::{
        perimeters::{
            classic::PostClassicTraversalPrintObject,
            layer_region::{
                PreparedLayerRegionPerimeterObject, PreparedLayerRegionPerimeterRecord,
                PreparedPostLayerRegionPerimeters,
            },
            types::{PerimeterInputRecord, PostPerimeterInputPrintObject},
        },
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

use super::{
    cracks, extra_bridge,
    geometry::{
        GeometryStep, expolygons, fresh, geometry_error, internal, observe, opening,
        opening_offset, paths, safety_difference,
    },
    preflight,
    types::StagedRecord,
};

pub(super) struct StagedObject {
    pub(super) records: Vec<Option<StagedRecord>>,
}

#[derive(Clone, Copy)]
struct StageRecordContext {
    external_width: i64,
    bottom_kind: RegionSurfaceKind,
    bottom_shell_layers: usize,
    layer_count: usize,
    spiral_mode: bool,
}

pub(super) fn project(
    prepared: &PreparedPostLayerRegionPerimeters,
) -> Result<Vec<StagedObject>, SliceError> {
    let classic = prepared.predecessor.as_classic();
    assert_eq!(prepared.objects.len(), classic.objects.len());
    prepared
        .objects
        .iter()
        .zip(&classic.objects)
        .map(|(object, traversal)| stage_object(prepared, object, traversal))
        .collect()
}

fn stage_object(
    prepared: &PreparedPostLayerRegionPerimeters,
    object: &PreparedLayerRegionPerimeterObject,
    traversal: &PostClassicTraversalPrintObject,
) -> Result<StagedObject, SliceError> {
    let prelude = &traversal.predecessor.predecessor.predecessor.predecessor;
    let input_object = &prelude.object;
    assert_eq!(object.records.len(), input_object.records.len());
    assert_eq!(object.records.len(), prelude.records.len());
    let source_index = input_object.identity().0;
    let options = &prepared
        .predecessor
        .as_classic()
        .resolved
        .objects
        .iter()
        .find(|resolved| resolved.source_object_index == source_index)
        .expect("O17 object retains its resolved source")
        .object;
    let bottom_kind = preflight::bottom_kind(options);
    let spiral_mode = prepared
        .predecessor
        .as_classic()
        .resolved
        .views
        .full
        .process
        .print
        .spiral_mode
        .0;
    let layer_count = input_object
        .records
        .iter()
        .flatten()
        .map(|record| record.layer_id + 1)
        .max()
        .unwrap_or(0);
    let mut records = object
        .records
        .iter()
        .zip(&input_object.records)
        .zip(&prelude.records)
        .map(
            |((output, input), prelude)| match (output, input, prelude) {
                (Some(output), Some(input), Some(prelude)) => {
                    let bottom_shell_layers =
                        usize::try_from(input_object.region_options(input).bottom_shell_layers.0)
                            .expect("normalized bottom_shell_layers is non-negative");
                    stage_record(
                        input_object,
                        input,
                        output,
                        StageRecordContext {
                            external_width: prelude.external_width,
                            bottom_kind,
                            bottom_shell_layers,
                            layer_count,
                            spiral_mode,
                        },
                    )
                    .map(Some)
                }
                (None, None, None) => Ok(None),
                _ => unreachable!("O17 slots remain aligned with O16 and the Classic prelude"),
            },
        )
        .collect::<Result<Vec<_>, _>>()?;
    extra_bridge::apply(
        &mut records,
        &input_object.records,
        input_object,
        options.enable_extra_bridge_layer,
        prepared.predecessor.as_classic().scale,
    )?;
    for (staged, output) in records.iter_mut().zip(&object.records) {
        match (staged, output) {
            (Some(staged), Some(output)) => {
                staged.fill_surfaces = clipped_fill(&staged.slices, &output.fill_expolygons)?;
            }
            (None, None) => {}
            _ => unreachable!("external extra-bridge records remain aligned"),
        }
    }
    Ok(StagedObject { records })
}

fn stage_record(
    object: &PostPerimeterInputPrintObject,
    input: &PerimeterInputRecord,
    output: &PreparedLayerRegionPerimeterRecord,
    context: StageRecordContext,
) -> Result<StagedRecord, SliceError> {
    let source = object.current_surfaces(input);
    let mut slices = classify_slices(
        source,
        object.upper_slices(input),
        object.lower_slices(input),
        context.external_width,
        context.bottom_kind,
    )?;
    apply_spiral_surface_types(
        &mut slices,
        context.spiral_mode,
        input.layer_id,
        context.bottom_shell_layers,
        context.layer_count,
    );
    let fill_surfaces = clipped_fill(&slices, &output.fill_expolygons)?;
    Ok(StagedRecord {
        slices,
        fill_surfaces,
    })
}

pub(super) fn apply_spiral_surface_types(
    surfaces: &mut [RegionSurface],
    spiral_mode: bool,
    layer_id: usize,
    bottom_shell_layers: usize,
    layer_count: usize,
) {
    if !spiral_mode {
        return;
    }
    let base_layer_count = bottom_shell_layers.min(layer_count);
    let kind = if base_layer_count > 1 && layer_id + 1 == base_layer_count {
        Some(RegionSurfaceKind::Top)
    } else if layer_id >= base_layer_count {
        Some(RegionSurfaceKind::Internal)
    } else {
        None
    };
    if let Some(kind) = kind {
        for surface in surfaces {
            surface.retag(kind);
        }
    }
}

pub(super) fn classify_slices(
    source: &[RegionSurface],
    upper: Option<&[ExPolygon]>,
    lower: Option<&[ExPolygon]>,
    external_width: i64,
    bottom_kind: RegionSurfaceKind,
) -> Result<Vec<RegionSurface>, SliceError> {
    let previous = expolygons(source);
    let offset = opening_offset(external_width);

    let mut top = if let Some(upper) = upper {
        fresh(
            RegionSurfaceKind::Top,
            opening(
                &safety_difference(&previous, upper, GeometryStep::TopSafetyDifference)?,
                offset,
                GeometryStep::TopShrink,
                GeometryStep::TopExpand,
            )?,
        )
    } else {
        source
            .iter()
            .map(|surface| surface.clone_with_kind(RegionSurfaceKind::Top))
            .collect()
    };
    let mut bottom = if let Some(lower) = lower {
        fresh(
            bottom_kind,
            opening(
                &safety_difference(&previous, lower, GeometryStep::BottomSafetyDifference)?,
                offset,
                GeometryStep::BottomShrink,
                GeometryStep::BottomExpand,
            )?,
        )
    } else {
        source
            .iter()
            .map(|surface| surface.clone_with_kind(RegionSurfaceKind::Bottom))
            .collect()
    };

    cracks::resolve(&mut top, &mut bottom, external_width, lower.is_some())?;
    let mut top_bottom = paths(&top);
    top_bottom.extend(paths(&bottom));
    let mut slices = internal(&previous, &top_bottom)?;
    slices.extend(top);
    slices.extend(bottom);
    Ok(slices)
}

pub(super) fn clipped_fill(
    slices: &[RegionSurface],
    boundaries: &[ExPolygon],
) -> Result<Vec<RegionSurface>, SliceError> {
    let mut output = Vec::new();
    for (kind, step) in [
        (RegionSurfaceKind::Top, GeometryStep::FillTopIntersection),
        (
            RegionSurfaceKind::Bottom,
            GeometryStep::FillBottomIntersection,
        ),
        (
            RegionSurfaceKind::BottomBridge,
            GeometryStep::FillBottomBridgeIntersection,
        ),
        (
            RegionSurfaceKind::Internal,
            GeometryStep::FillInternalIntersection,
        ),
    ] {
        let group = slices
            .iter()
            .filter(|surface| surface.as_parts().0 == kind)
            .map(|surface| surface.as_parts().1.clone())
            .collect::<Vec<_>>();
        if !group.is_empty() {
            observe(step)?;
            let clipped = intersection_ex(&group, boundaries).map_err(geometry_error)?;
            output.extend(fresh(kind, clipped));
        }
    }
    Ok(output)
}
