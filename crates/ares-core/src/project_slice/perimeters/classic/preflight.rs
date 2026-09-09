use crate::{ObjectOptions, ProcessCounterboreHoleBridging, RegionOptions, SliceError};

use super::super::types::{PerimeterInputRecord, PostPerimeterInputPrintObject};

#[derive(Clone, Copy)]
pub(super) struct ValidatedClassicConfig {
    pub(super) wall_loops: i32,
    pub(super) precise_outer_wall: bool,
    pub(super) detect_overhang_wall: bool,
    pub(super) only_one_wall_top: bool,
    pub(super) gap_infill_speed: f64,
    pub(super) surface_simplify_resolution: f64,
    pub(super) support_nozzle_diameter: f64,
}

pub(super) struct ValidatedClassicObject {
    pub(super) records: Vec<Option<ValidatedClassicConfig>>,
}

pub(super) struct ClassicValidationContext<'a> {
    pub(super) resolved_objects:
        &'a [crate::project::effective_config::types::ResolvedProjectObject],
    pub(super) enable_arc_fitting: bool,
    pub(super) resolution: f64,
    pub(super) nozzle_diameters: &'a crate::OrcaFloats,
    pub(super) scale: crate::geometry::CoordinateScale,
}

struct RecordValidationContext<'a> {
    object_options: &'a ObjectOptions,
    region: &'a RegionOptions,
    enable_arc_fitting: bool,
    scaled_resolution: f64,
    nozzle_diameters: &'a crate::OrcaFloats,
    scale: crate::geometry::CoordinateScale,
}

pub(super) fn validate_project(
    objects: &[PostPerimeterInputPrintObject],
    context: ClassicValidationContext<'_>,
) -> Result<Vec<ValidatedClassicObject>, SliceError> {
    if !context.resolution.is_finite() {
        return Err(invalid("invalid Orca option resolution"));
    }
    let effective_resolution = if context.resolution > 1e-4 {
        context.resolution
    } else {
        1e-4
    };
    let scaled_resolution = effective_resolution / context.scale.factor();

    objects
        .iter()
        .map(|object| {
            let (source_object_index, _) = object.identity();
            let resolved = context
                .resolved_objects
                .iter()
                .find(|resolved| resolved.source_object_index == source_object_index)
                .expect("Task 22N object must retain its resolved source object");
            let records = object
                .as_parts()
                .1
                .iter()
                .map(|record| {
                    record
                        .as_ref()
                        .map(|record| {
                            validate_record(
                                object,
                                record,
                                RecordValidationContext {
                                    object_options: &resolved.object,
                                    region: object.region_options(record),
                                    enable_arc_fitting: context.enable_arc_fitting,
                                    scaled_resolution,
                                    nozzle_diameters: context.nozzle_diameters,
                                    scale: context.scale,
                                },
                            )
                        })
                        .transpose()
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ValidatedClassicObject { records })
        })
        .collect()
}

fn validate_record(
    object: &PostPerimeterInputPrintObject,
    record: &PerimeterInputRecord,
    context: RecordValidationContext<'_>,
) -> Result<ValidatedClassicConfig, SliceError> {
    let RecordValidationContext {
        object_options,
        region,
        enable_arc_fitting,
        scaled_resolution,
        nozzle_diameters,
        scale,
    } = context;
    if region.detect_thin_wall.0
        && !thin_wall_is_provably_inactive(object, record, region, nozzle_diameters, scale)
    {
        return Err(unsupported("detect_thin_wall"));
    }
    if region.overhang_reverse.0 && has_layer_overhang(object, record) {
        return Err(unsupported("overhang_reverse"));
    }
    if region.extra_perimeters_on_overhangs.0
        && object.lower_slices(record).is_some()
        && region.detect_overhang_wall.0
        && region.wall_loops.0 > 0
        && i32::try_from(record.layer_id).is_ok_and(|layer| layer > object_options.raft_layers.0)
        && has_layer_overhang(object, record)
    {
        return Err(unsupported("extra_perimeters_on_overhangs"));
    }
    if region.counterbore_hole_bridging != ProcessCounterboreHoleBridging::None
        && object
            .current_surfaces(record)
            .iter()
            .any(|surface| !surface.as_parts().1.holes().is_empty())
    {
        return Err(unsupported("counterbore_hole_bridging"));
    }

    let nozzle_index = region
        .outer_wall_filament_id
        .0
        .checked_sub(1)
        .and_then(|index| usize::try_from(index).ok())
        .filter(|index| *index < nozzle_diameters.0.len())
        .unwrap_or(0);
    let support_nozzle_diameter = nozzle_diameters
        .0
        .get(nozzle_index)
        .map(|value| value.0)
        .ok_or_else(|| invalid("invalid Orca option nozzle_diameter"))?;

    let fuzzy_skin = crate::perimeters::FuzzySkinConfig::from_region(region);
    let simplify_resolution =
        if enable_arc_fitting && !fuzzy_skin.should_fuzzify(record.layer_id, 0, true) {
            0.2 * scaled_resolution
        } else {
            scaled_resolution
        };
    let first_object_layer =
        i32::try_from(record.layer_id).ok() == Some(object_options.raft_layers.0);
    let mut wall_loops = region.wall_loops.0;
    if region.alternate_extra_wall.0
        && record.layer_id % 2 == 1
        && region.sparse_infill_density.0 > 0.0
    {
        wall_loops += 1;
    }
    if region.only_one_wall_first_layer.0 && first_object_layer {
        wall_loops = wall_loops.min(1);
    }
    Ok(ValidatedClassicConfig {
        wall_loops,
        precise_outer_wall: region.precise_outer_wall.0
            && region.wall_sequence == crate::ProcessWallSequence::InnerOuter,
        detect_overhang_wall: region.detect_overhang_wall.0,
        only_one_wall_top: region.only_one_wall_top.0,
        gap_infill_speed: region.gap_infill_speed.0,
        surface_simplify_resolution: simplify_resolution,
        support_nozzle_diameter,
    })
}

fn thin_wall_is_provably_inactive(
    object: &PostPerimeterInputPrintObject,
    record: &PerimeterInputRecord,
    region: &RegionOptions,
    nozzle_diameters: &crate::OrcaFloats,
    scale: crate::geometry::CoordinateScale,
) -> bool {
    let maximum_nozzle = nozzle_diameters
        .0
        .iter()
        .map(|diameter| diameter.0)
        .fold(0.0, f64::max);
    let minimum_span = 2.0 * maximum_nozzle * f64::from(region.wall_loops.0.max(1) + 1);
    object.current_surfaces(record).iter().all(|surface| {
        let expolygon = surface.as_parts().1;
        if !expolygon.holes().is_empty() || expolygon.contour().points().len() != 4 {
            return false;
        }
        let points = expolygon.contour().points();
        let axis_aligned = points
            .iter()
            .zip(points.iter().cycle().skip(1))
            .all(|(first, second)| first.x() == second.x() || first.y() == second.y());
        if !axis_aligned {
            return false;
        }
        let (min_x, max_x, min_y, max_y) = points.iter().fold(
            (i64::MAX, i64::MIN, i64::MAX, i64::MIN),
            |(min_x, max_x, min_y, max_y), point| {
                (
                    min_x.min(point.x()),
                    max_x.max(point.x()),
                    min_y.min(point.y()),
                    max_y.max(point.y()),
                )
            },
        );
        scale
            .unscale(max_x - min_x)
            .min(scale.unscale(max_y - min_y))
            >= minimum_span
    })
}

fn has_layer_overhang(
    object: &PostPerimeterInputPrintObject,
    record: &PerimeterInputRecord,
) -> bool {
    let Some(lower) = object.lower_slices(record) else {
        return false;
    };
    let current = object
        .current_surfaces(record)
        .iter()
        .map(|surface| surface.as_parts().1)
        .collect::<Vec<_>>();
    current.len() != lower.len()
        || current
            .iter()
            .zip(lower)
            .any(|(current, lower)| *current != lower)
}

fn invalid(message: &str) -> SliceError {
    SliceError::InvalidInput(message.to_owned())
}

fn unsupported(key: &str) -> SliceError {
    SliceError::UnsupportedProjectFeature(key.to_owned())
}
