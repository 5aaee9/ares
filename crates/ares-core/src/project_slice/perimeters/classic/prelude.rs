use crate::{
    SliceError,
    geometry::{
        ClipperError, CoordinateScale, ExPolygon, JoinType, Polygon, append_simplified_expolygon,
        chain_expolygons_order, offset_expolygons_paths,
    },
};

use super::{
    preflight::{ValidatedClassicConfig, ValidatedClassicObject},
    types::{ClassicPreludeRecord, PreparedClassicSurface},
};
use crate::project_slice::perimeters::types::{
    Flow, PerimeterDispatch, PerimeterInputRecord, PostPerimeterInputPrintObject,
};

const INSET_OVERLAP_TOLERANCE: f64 = 0.4;
const SMALLER_EXTERNAL_INSET_OVERLAP_TOLERANCE: f64 = 0.22;
const OVERHANG_SAMPLING_NUMBER: f32 = 6.0;
const MITER_LIMIT: f64 = 3.0;

pub(super) fn prepare_object(
    object: &PostPerimeterInputPrintObject,
    validated: ValidatedClassicObject,
    scale: CoordinateScale,
) -> Result<Vec<Option<ClassicPreludeRecord>>, SliceError> {
    object
        .as_parts()
        .1
        .iter()
        .zip(validated.records)
        .map(|(record, config)| match (record, config) {
            (Some(record), Some(config)) => prepare_record(object, record, config, scale).map(Some),
            (None, None) => Ok(None),
            _ => unreachable!("Classic validation slots must remain aligned with Task 22N"),
        })
        .collect()
}

fn prepare_record(
    object: &PostPerimeterInputPrintObject,
    record: &PerimeterInputRecord,
    config: ValidatedClassicConfig,
    scale: CoordinateScale,
) -> Result<ClassicPreludeRecord, SliceError> {
    let perimeter_width = scaled_flow(scale, record.perimeter_flow.width)?;
    let perimeter_spacing = scaled_flow(scale, record.perimeter_flow.spacing)?;
    let external_width = scaled_flow(scale, record.ext_perimeter_flow.width)?;
    let external_spacing = scaled_flow(scale, record.ext_perimeter_flow.spacing)?;
    let solid_infill_spacing = scaled_flow(scale, record.solid_infill_flow.spacing)?;
    let external_to_internal_spacing = if config.precise_outer_wall {
        scaled_f32(
            scale,
            0.5_f32 * (record.ext_perimeter_flow.width + record.perimeter_flow.width),
        )?
    } else {
        scaled_f32(
            scale,
            0.5_f32 * (record.ext_perimeter_flow.spacing + record.perimeter_flow.spacing),
        )?
    };
    let minimum_spacing = (perimeter_spacing as f64 * (1.0 - INSET_OVERLAP_TOLERANCE)) as i64;
    let external_minimum_spacing =
        (external_spacing as f64 * (1.0 - INSET_OVERLAP_TOLERANCE)) as i64;
    let smaller_external_minimum_spacing =
        (external_spacing as f64 * (1.0 - SMALLER_EXTERNAL_INSET_OVERLAP_TOLERANCE)) as i64;
    let smaller_width = (scale.factor()
        * (external_width as f64
            - 0.5 * SMALLER_EXTERNAL_INSET_OVERLAP_TOLERANCE * external_spacing as f64))
        as f32;
    let smaller_external_flow = record.ext_perimeter_flow.with_width(smaller_width)?;

    let lower = object.lower_slices(record);
    let lower_slices_polygons = if config.detect_overhang_wall {
        lower
            .map(|lower| offset_paths(lower, 0.5 * config.support_nozzle_diameter, scale))
            .transpose()?
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let nozzle_diameter = record.ext_perimeter_flow.nozzle_diameter;
    let lower_polygons_series = lower_series(lower, record.perimeter_flow, nozzle_diameter, scale)?;
    let external_lower_polygons_series = if external_width == perimeter_width {
        lower_polygons_series.clone()
    } else {
        lower_series(lower, record.ext_perimeter_flow, nozzle_diameter, scale)?
    };
    let smaller_external_lower_polygons_series =
        lower_series(lower, smaller_external_flow, nozzle_diameter, scale)?;
    let surfaces = prepare_surfaces(object, record, config)?;
    Ok(ClassicPreludeRecord {
        perimeter_width,
        perimeter_spacing,
        external_width,
        external_spacing,
        external_to_internal_spacing,
        solid_infill_spacing,
        minimum_spacing,
        external_minimum_spacing,
        smaller_external_minimum_spacing,
        has_gap_fill: config.gap_infill_speed > 0.0,
        smaller_external_flow,
        lower_slices_polygons,
        lower_polygons_series,
        external_lower_polygons_series,
        smaller_external_lower_polygons_series,
        surface_simplify_resolution: config.surface_simplify_resolution,
        surfaces,
    })
}

fn prepare_surfaces(
    object: &PostPerimeterInputPrintObject,
    record: &PerimeterInputRecord,
    config: ValidatedClassicConfig,
) -> Result<Vec<PreparedClassicSurface>, SliceError> {
    // Arachne records carry no classic surfaces: `process_arachne` owns
    // their wall generation and materializes entities at the layer-region
    // boundary. The classic chain still prepares their shared context
    // (spacing, lower slices, staging inputs) from this record.
    if record.dispatch == PerimeterDispatch::Arachne {
        return Ok(Vec::new());
    }
    let sources = object.current_surfaces(record);
    let geometry = sources
        .iter()
        .map(|surface| surface.as_parts().1.clone())
        .collect::<Vec<ExPolygon>>();
    let order = chain_expolygons_order(&geometry);
    order
        .into_iter()
        .map(|source_index| {
            let (kind, expolygon, thickness, thickness_layers, bridge_angle, extra_perimeters) =
                sources[source_index].as_parts();
            let mut polygons = Vec::new();
            append_simplified_expolygon(
                expolygon.clone(),
                config.surface_simplify_resolution,
                &mut polygons,
            )
            .map_err(geometry_error)?;
            let mut loop_number = config.wall_loops + i32::from(extra_perimeters) - 1;
            if loop_number > 0 && config.only_one_wall_top && record.upper_layer_index.is_none() {
                loop_number = 0;
            }
            Ok(PreparedClassicSurface {
                source_index,
                kind,
                thickness,
                thickness_layers,
                bridge_angle,
                extra_perimeters,
                loop_number,
                polygons,
            })
        })
        .collect()
}

fn lower_series(
    lower: Option<&[ExPolygon]>,
    flow: Flow,
    nozzle_diameter: f32,
    scale: CoordinateScale,
) -> Result<Vec<Vec<Polygon>>, SliceError> {
    let Some(lower) = lower else {
        return Ok(Vec::new());
    };
    let offsets = lower_sample_offsets(flow.width, nozzle_diameter);
    offsets
        .into_iter()
        .map(|offset| offset_paths(lower, f64::from(offset), scale))
        .collect()
}

pub(in crate::project_slice) fn lower_sample_offsets(width: f32, nozzle_diameter: f32) -> [f32; 2] {
    let start = -0.5_f32 * width;
    let end = 0.5_f32 * nozzle_diameter;
    let difference = end - start;
    let first = (f64::from(start)
        + 0.5 * f64::from(difference) / f64::from(OVERHANG_SAMPLING_NUMBER - 1.0))
        as f32;
    [first, end]
}

fn offset_paths(
    input: &[ExPolygon],
    offset_mm: f64,
    scale: CoordinateScale,
) -> Result<Vec<Polygon>, SliceError> {
    scale.checked_scale(offset_mm).ok_or_else(|| {
        SliceError::InvalidInput(
            "Classic perimeter prelude coordinate is outside the supported range".to_owned(),
        )
    })?;
    let offset = (offset_mm / scale.factor()) as f32;
    offset_expolygons_paths(input, offset, JoinType::Miter, MITER_LIMIT).map_err(geometry_error)
}

fn scaled_flow(scale: CoordinateScale, value: f32) -> Result<i64, SliceError> {
    scale.checked_scale(f64::from(value)).ok_or_else(|| {
        SliceError::InvalidInput(
            "Classic perimeter prelude coordinate is outside the supported range".to_owned(),
        )
    })?;
    Ok((f64::from(value) / scale.factor()) as i64)
}

fn scaled_f32(scale: CoordinateScale, value: f32) -> Result<i64, SliceError> {
    scale.checked_scale(f64::from(value)).ok_or_else(|| {
        SliceError::InvalidInput(
            "Classic perimeter prelude coordinate is outside the supported range".to_owned(),
        )
    })?;
    Ok((value / scale.factor() as f32) as i64)
}

fn geometry_error(_: ClipperError) -> SliceError {
    SliceError::InvalidInput(
        "Classic perimeter prelude geometry is outside the supported Clipper range".to_owned(),
    )
}
