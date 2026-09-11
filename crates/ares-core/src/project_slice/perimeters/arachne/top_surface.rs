//! Top-surface single-wall regeneration for the arachne generator
//! (`PerimeterGenerator.cpp:2160-2248`): after the main single-wall pass,
//! the not-top areas regrow their inner walls.

use crate::{
    SliceError,
    arachne::{ExtrusionLine, wall_toolpaths},
    geometry::{
        BoundingBox, CoordinateScale, ExPolygon, FillRule, JoinType,
        clip_clipper_expolygons_with_subject_bbox, difference_ex, difference_ex_polygons,
        intersection_ex, offset_expolygons, offset2_ex, union_ex, union_expolygons,
    },
    project_slice::region_slices::RegionSurface,
};

use super::{
    config::{ArachneRecordConfig, ArachneWallParams},
    walls::{ArachneSpacings, GeneratedSurfaceWalls, MITER_LIMIT},
};

/// Inputs for the top-surface single-wall regeneration
/// (`PerimeterGenerator.cpp:2160-2248`).
pub(in crate::project_slice) struct TopSurfaceInputs<'a> {
    /// `*upper_slices` — the full upper-layer slices.
    pub(in crate::project_slice) upper_slices: &'a [ExPolygon],
    /// `upper_slices_same_region` when `interface_shells` is set.
    pub(in crate::project_slice) upper_same_region: Option<&'a [RegionSurface]>,
    /// `*lower_slices` for bridge removal.
    pub(in crate::project_slice) lower_slices: Option<&'a [ExPolygon]>,
    /// `object_config->interface_shells`.
    pub(in crate::project_slice) interface_shells: bool,
    /// `config->min_width_top_surface` resolved over the perimeter width
    /// (mm).
    pub(in crate::project_slice) min_width_top_surface_mm: f64,
    /// `perimeter_flow.scaled_width()`.
    pub(in crate::project_slice) perimeter_width: i64,
}

/// The regeneration pass (`PerimeterGenerator.cpp:2177-2247`). Takes the
/// main-pass single-wall output and regrows the inner walls on the not-top
/// areas (or regenerates fully when no top surface exists on this island).
pub(in crate::project_slice) fn regenerate(
    toolpaths: Vec<Vec<ExtrusionLine>>,
    infill_contour: Vec<ExPolygon>,
    ts: &TopSurfaceInputs<'_>,
    config: &ArachneRecordConfig,
    spacings: &ArachneSpacings,
    last: &[ExPolygon],
    wall_0_inset: i64,
    layer_height: i64,
    params: ArachneWallParams,
    scale: CoordinateScale,
    inner_loop_number: i32,
) -> Result<GeneratedSurfaceWalls, SliceError> {
    let mut bounds = BoundingBox::from_expolygons(&infill_contour);
    if let Some(bounds) = bounds.as_mut() {
        bounds.offset(
            scale
                .checked_scale(1e-4)
                .ok_or_else(|| invalid("Arachne epsilon exceeds the coordinate range"))?,
        );
    }
    let upper_clipped = match bounds {
        Some(bounds) => {
            if ts.interface_shells
                && let Some(surfaces) = ts.upper_same_region
            {
                clip_clipper_expolygons_with_subject_bbox(
                    &surfaces
                        .iter()
                        .map(|surface| surface.as_parts().1.clone())
                        .collect::<Vec<_>>(),
                    bounds,
                )
            } else {
                clip_clipper_expolygons_with_subject_bbox(ts.upper_slices, bounds)
            }
        }
        None => Vec::new(),
    };
    let mut top_expolygons =
        difference_ex_polygons(&infill_contour, &upper_clipped).map_err(geometry_error)?;
    if top_expolygons.is_empty() {
        // No top surface on this island (`:2242-2247`): regenerate as if
        // the feature were disabled — `inner_loop_number + 2` walls.
        let regenerated = wall_toolpaths::generate(
            &to_polygons(last),
            wall_toolpaths::RawWallToolPathConfig {
                outer_spacing: spacings.ext_perimeter_spacing,
                inner_spacing: spacings.perimeter_spacing,
                inset_count: usize::try_from(inner_loop_number + 2)
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
        let infill =
            union_ex(&regenerated.inner_contour, FillRule::NonZero).map_err(geometry_error)?;
        return Ok(GeneratedSurfaceWalls {
            toolpaths: regenerated.toolpaths,
            infill_contour: infill,
        });
    }
    // Remove bridges over lower slices (`:2196-2203`).
    if let (Some(lower), Some(bounds)) = (ts.lower_slices, bounds.as_ref()) {
        let bridge_offset = spacings.ext_perimeter_spacing.max(ts.perimeter_width) as f32;
        let lower_clipped = clip_clipper_expolygons_with_subject_bbox(lower, *bounds);
        let bridges = offset_expolygons(
            &difference_ex_polygons(&top_expolygons, &lower_clipped).map_err(geometry_error)?,
            bridge_offset,
            JoinType::Miter,
            MITER_LIMIT,
        )
        .map_err(geometry_error)?;
        top_expolygons = difference_ex(&top_expolygons, &bridges).map_err(geometry_error)?;
    }
    // Filter thin areas and expand to hide the wall line (`:2205-2214`).
    let scaled_1e5 = scale.checked_scale(0.00001).unwrap_or(0) as f32;
    let min_width_scaled = scale
        .checked_scale(ts.min_width_top_surface_mm)
        .ok_or_else(|| invalid("min_width_top_surface exceeds the coordinate range"))?
        as f32;
    let top_surface_min_width = f32::max(
        spacings.ext_perimeter_spacing as f32 / 4.0 + scaled_1e5,
        min_width_scaled / 4.0,
    );
    top_expolygons = offset2_ex(
        &top_expolygons,
        -top_surface_min_width,
        top_surface_min_width + (ts.perimeter_width * 85 / 100) as f32,
        JoinType::Miter,
        MITER_LIMIT,
    )
    .map_err(geometry_error)?;
    let not_top = difference_ex(&infill_contour, &top_expolygons).map_err(geometry_error)?;
    top_expolygons = intersection_ex(&top_expolygons, &infill_contour).map_err(geometry_error)?;
    // Inner walls on the not-top areas (`:2221-2223`).
    let not_top_polygons = to_polygons(
        &offset_expolygons(&not_top, wall_0_inset as f32, JoinType::Miter, MITER_LIMIT)
            .map_err(geometry_error)?,
    );
    let mut inner_generated = wall_toolpaths::generate(
        &not_top_polygons,
        wall_toolpaths::RawWallToolPathConfig {
            outer_spacing: spacings.perimeter_spacing,
            inner_spacing: spacings.perimeter_spacing,
            inset_count: usize::try_from(inner_loop_number + 1)
                .map_err(|_| invalid("wall_loops underflow"))?,
            outer_wall_inset: 0,
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
    let mut toolpaths = toolpaths;
    if !toolpaths.is_empty() {
        for lines in &mut inner_generated.toolpaths {
            for line in lines {
                line.inset_index += 1;
            }
        }
    }
    toolpaths.extend(inner_generated.toolpaths);
    let final_contour = union_expolygons(
        &top_expolygons,
        &union_ex(&inner_generated.inner_contour, FillRule::NonZero).map_err(geometry_error)?,
    )
    .map_err(geometry_error)?;
    Ok(GeneratedSurfaceWalls {
        toolpaths,
        infill_contour: final_contour,
    })
}

/// `to_polygons` (`ExPolygons.hpp`): contours and holes as one polygon set.
fn to_polygons(expolygons: &[ExPolygon]) -> Vec<crate::geometry::Polygon> {
    let mut polygons = Vec::new();
    for expolygon in expolygons {
        polygons.push(expolygon.contour().clone());
        polygons.extend(expolygon.holes().iter().cloned());
    }
    polygons
}

fn geometry_error(error: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput(format!("Arachne top-surface geometry failed: {error:?}"))
}

fn invalid(message: &str) -> SliceError {
    SliceError::InvalidInput(message.to_owned())
}
