// Per-surface arachne wall generation from OrcaSlicer v2.4.2
// `PerimeterGenerator::process_arachne`
// (`PerimeterGenerator.cpp:2111-2176`): the per-island loop-number and
// outer-wall offset preparation feeding `Arachne::WallToolPaths`
// (`WallToolPaths.cpp:472-702`), plus the infill boundary of
// `PerimeterGenerator.cpp:2475-2531`.

use crate::{
    SliceError,
    arachne::{ExtrusionLine, wall_toolpaths},
    geometry::{
        ClipperError, CoordinateScale, ExPolygon, FillRule, JoinType, Polygon,
        append_simplified_expolygon, offset_expolygons, offset2_ex, simplify_expolygon_polygons,
        union_ex,
    },
    project_slice::region_slices::RegionSurface,
};

use super::config::ArachneRecordConfig;

const INSET_OVERLAP_TOLERANCE: f64 = 0.4;
const MITER_LIMIT: f64 = 3.0;

/// One island's generated walls and inner contour
/// (`PerimeterGenerator.cpp:2148-2176`).
pub(in crate::project_slice) struct GeneratedSurfaceWalls {
    pub(in crate::project_slice) toolpaths: Vec<Vec<ExtrusionLine>>,
    pub(in crate::project_slice) infill_contour: Vec<ExPolygon>,
}

/// Scaled flow widths `process_arachne` derives per record
/// (`PerimeterGenerator.cpp:2097-2109`).
#[derive(Clone, Copy)]
pub(in crate::project_slice) struct ArachneSpacings {
    pub(in crate::project_slice) perimeter_spacing: i64,
    pub(in crate::project_slice) ext_perimeter_width: i64,
    pub(in crate::project_slice) ext_perimeter_spacing: i64,
    pub(in crate::project_slice) ext_perimeter_spacing2: i64,
    pub(in crate::project_slice) solid_infill_spacing: i64,
}

impl ArachneSpacings {
    pub(in crate::project_slice) fn new(
        scale: CoordinateScale,
        perimeter_spacing_mm: f32,
        ext_width_mm: f32,
        ext_spacing_mm: f32,
        solid_infill_spacing_mm: f32,
    ) -> Result<Self, SliceError> {
        let scaled = |value: f32| -> Result<i64, SliceError> {
            scale.checked_scale(f64::from(value)).ok_or_else(|| {
                SliceError::InvalidInput(
                    "Arachne perimeter spacing exceeds the coordinate range".to_owned(),
                )
            })
        };
        // `ext_perimeter_spacing2`
        // (`PerimeterGenerator.cpp:2103`): the plain external/internal
        // spacing average.
        let spacing2 = scale
            .checked_scale(0.5 * f64::from(ext_spacing_mm + perimeter_spacing_mm))
            .ok_or_else(|| {
                SliceError::InvalidInput(
                    "Arachne perimeter spacing exceeds the coordinate range".to_owned(),
                )
            })?;
        Ok(Self {
            perimeter_spacing: scaled(perimeter_spacing_mm)?,
            ext_perimeter_width: scaled(ext_width_mm)?,
            ext_perimeter_spacing: scaled(ext_spacing_mm)?,
            ext_perimeter_spacing2: spacing2,
            solid_infill_spacing: scaled(solid_infill_spacing_mm)?,
        })
    }
}

/// Generate one island's walls (`PerimeterGenerator.cpp:2111-2176`).
pub(in crate::project_slice) fn generate_surface_walls(
    surface: &RegionSurface,
    config: &ArachneRecordConfig,
    spacings: &ArachneSpacings,
    wall_loops: i32,
    layer_height: f64,
    scale: CoordinateScale,
) -> Result<GeneratedSurfaceWalls, SliceError> {
    let (_, expolygon, _, _, _, extra_perimeters) = surface.as_parts();
    // 0-indexed loops for this island (`PerimeterGenerator.cpp:2113-2114`).
    let mut loop_number = wall_loops + i32::from(extra_perimeters) - 1;
    // Set the bottommost layer to be one wall (`:2121-2122`).
    if config.is_bottom_layer && config.only_one_wall_first_layer {
        loop_number = 0;
    }
    // Orca: set the topmost layer to be one wall (`:2125-2127`); the
    // top-surface regeneration below topmost layers stays typed-rejected.
    if config.is_topmost_layer && loop_number > 0 && config.only_one_wall_top {
        loop_number = 0;
    }
    let outer_offset = if config.precise_outer_wall {
        -f64::from((spacings.ext_perimeter_width - spacings.ext_perimeter_spacing) as f32)
    } else {
        -(spacings.ext_perimeter_width as f64 / 2.0 - spacings.ext_perimeter_spacing as f64 / 2.0)
    };
    let mut simplified = Vec::new();
    append_simplified_expolygon(
        expolygon.clone(),
        config.surface_simplify_resolution,
        &mut simplified,
    )
    .map_err(geometry_error)?;
    let last = offset_expolygons(
        &simplified,
        outer_offset as f32,
        JoinType::Miter,
        MITER_LIMIT,
    )
    .map_err(geometry_error)?;
    let wall_0_inset = if config.precise_outer_wall {
        -(spacings.ext_perimeter_width / 2 - spacings.ext_perimeter_spacing / 2)
    } else {
        0
    };
    let layer_height = scale
        .checked_scale(layer_height)
        .ok_or_else(|| invalid("Arachne layer height exceeds the coordinate range"))?;
    let params = config.params;
    let generated = wall_toolpaths::generate(
        &to_polygons(&last),
        wall_toolpaths::RawWallToolPathConfig {
            outer_spacing: spacings.ext_perimeter_spacing,
            inner_spacing: spacings.perimeter_spacing,
            inset_count: usize::try_from(loop_number + 1)
                .map_err(|_| invalid("wall_loops underflow"))?,
            outer_wall_inset: wall_0_inset,
            layer_height,
            min_bead_width: params.min_bead_width,
            min_feature_size: params.min_feature_size,
            transition_length: params.wall_transition_length,
            transitioning_angle: params.wall_transition_angle,
            transition_filter_deviation: params.wall_transition_filter_deviation,
            wall_distribution_count: params.wall_distribution_count,
            min_length_factor: params.min_length_factor,
            wall_maximum_resolution: params.wall_maximum_resolution,
            wall_maximum_deviation: params.wall_maximum_deviation,
            is_top_or_bottom_layer: config.is_bottom_layer || config.is_topmost_layer,
            coordinate_scale: scale,
        },
    )
    .map_err(|error| {
        SliceError::InvalidInput(format!("Arachne wall generation failed: {error:?}"))
    })?;
    let infill_contour =
        union_ex(&generated.inner_contour, FillRule::NonZero).map_err(geometry_error)?;
    Ok(GeneratedSurfaceWalls {
        toolpaths: generated.toolpaths,
        infill_contour,
    })
}

/// The infill boundary of one island (`PerimeterGenerator.cpp:2475-2531`).
#[derive(Default)]
pub(in crate::project_slice) struct SurfaceInfillBoundary {
    pub(in crate::project_slice) fill_surfaces: Vec<ExPolygon>,
    pub(in crate::project_slice) fill_no_overlap: Vec<ExPolygon>,
}

#[derive(Clone, Copy)]
pub(in crate::project_slice) struct InfillOverlapPercents {
    pub(in crate::project_slice) infill_wall_overlap: f64,
    pub(in crate::project_slice) top_bottom_infill_wall_overlap: f64,
}

pub(in crate::project_slice) fn surface_infill_boundary(
    infill_contour: Vec<ExPolygon>,
    wall_count: usize,
    loop_number: i32,
    spacings: &ArachneSpacings,
    scaled_resolution: f64,
    overlaps: InfillOverlapPercents,
    top_or_bottom: bool,
) -> Result<SurfaceInfillBoundary, SliceError> {
    // `PerimeterGenerator.cpp:2475-2478`: the too-small infill filter uses
    // the single-wall external spacing average.
    let spacing = if wall_count == 1 {
        spacings.ext_perimeter_spacing2
    } else {
        spacings.perimeter_spacing
    };
    let mut infill_contour = infill_contour;
    if offset_expolygons(
        &infill_contour,
        -((spacing as f64 / 2.0) as f32),
        JoinType::Miter,
        MITER_LIMIT,
    )
    .map_err(geometry_error)?
    .is_empty()
    {
        infill_contour.clear();
    }
    // Inset base by loop count (`PerimeterGenerator.cpp:2480-2493`), then the
    // wall-overlap percentage replaces it.
    let base_inset = if loop_number < 0 {
        0
    } else if loop_number == 0 {
        spacings.ext_perimeter_spacing
    } else {
        spacings.perimeter_spacing
    };
    let overlap_factor = if top_or_bottom {
        overlaps.top_bottom_infill_wall_overlap / 100.0
    } else {
        overlaps.infill_wall_overlap / 100.0
    };
    let inset = overlap_factor * base_inset as f64;
    // Simplify infill contours and collapse narrow areas
    // (`PerimeterGenerator.cpp:2495-2502`).
    let mut polygons = Vec::new();
    for expolygon in &infill_contour {
        polygons.extend(
            simplify_expolygon_polygons(expolygon, scaled_resolution).map_err(geometry_error)?,
        );
    }
    let not_filled = union_ex(&polygons, FillRule::NonZero).map_err(geometry_error)?;
    let min_perimeter_infill_spacing =
        spacings.solid_infill_spacing as f64 * (1.0 - INSET_OVERLAP_TOLERANCE);
    let fill_surfaces = offset2_ex(
        &not_filled,
        (-min_perimeter_infill_spacing / 2.0) as f32,
        (inset + min_perimeter_infill_spacing / 2.0) as f32,
        JoinType::Miter,
        MITER_LIMIT,
    )
    .map_err(geometry_error)?;
    // BBS no-overlap infill expolygons (`PerimeterGenerator.cpp:2522-2531`).
    let fill_no_overlap = offset2_ex(
        &not_filled,
        (-min_perimeter_infill_spacing / 2.0) as f32,
        (min_perimeter_infill_spacing / 2.0) as f32,
        JoinType::Miter,
        MITER_LIMIT,
    )
    .map_err(geometry_error)?;
    Ok(SurfaceInfillBoundary {
        fill_surfaces,
        fill_no_overlap,
    })
}

/// `to_polygons` (`ExPolygons.hpp`): contours and holes as one polygon set.
fn to_polygons(expolygons: &[ExPolygon]) -> Vec<Polygon> {
    let mut polygons = Vec::new();
    for expolygon in expolygons {
        polygons.push(expolygon.contour().clone());
        polygons.extend(expolygon.holes().iter().cloned());
    }
    polygons
}

fn geometry_error(_: ClipperError) -> SliceError {
    SliceError::InvalidInput(
        "Arachne perimeter geometry is outside the supported Clipper range".to_owned(),
    )
}

fn invalid(message: &str) -> SliceError {
    SliceError::InvalidInput(message.to_owned())
}
