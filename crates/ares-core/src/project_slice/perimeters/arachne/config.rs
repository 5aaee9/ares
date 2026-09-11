// Arachne wall-path parameter preparation and the first-slice scope gate
// from OrcaSlicer v2.4.2 `Arachne/WallToolPaths.cpp:26-71::make_paths_params`
// (called by `PerimeterGenerator.cpp:2166`) and the per-record dispatch scope
// of `PerimeterGenerator::process_arachne`
// (`PerimeterGenerator.cpp:2093-2519`). Configurations outside the ported
// vertical slice stay typed-rejected; there is no classic fallback.

use crate::{ObjectOptions, OrcaFloats, RegionOptions, SliceError, geometry::CoordinateScale};

const EPSILON: f64 = 1e-4;

/// `Arachne::WallToolPathsParams` as produced by `make_paths_params`, in
/// scaled coordinates (`WallToolPaths.cpp:26-71`).
#[derive(Clone, Copy, Debug)]
pub(in crate::project_slice) struct ArachneWallParams {
    pub(in crate::project_slice) min_feature_size: i64,
    pub(in crate::project_slice) min_length_factor: f64,
    pub(in crate::project_slice) min_bead_width: i64,
    pub(in crate::project_slice) wall_transition_filter_deviation: i64,
    pub(in crate::project_slice) wall_transition_length: i64,
    pub(in crate::project_slice) wall_transition_angle: f64,
    pub(in crate::project_slice) wall_distribution_count: i32,
    pub(in crate::project_slice) wall_maximum_resolution: i64,
    pub(in crate::project_slice) wall_maximum_deviation: i64,
}

/// The per-record configuration `process_arachne` derives before generating
/// walls (`PerimeterGenerator.cpp:2107-2125,2113-2141,2273-2280`).
#[derive(Clone, Copy, Debug)]
pub(in crate::project_slice) struct ArachneRecordConfig {
    pub(in crate::project_slice) params: ArachneWallParams,
    pub(in crate::project_slice) surface_simplify_resolution: f64,
    pub(in crate::project_slice) is_bottom_layer: bool,
    pub(in crate::project_slice) is_topmost_layer: bool,
    pub(in crate::project_slice) only_one_wall_first_layer: bool,
    pub(in crate::project_slice) only_one_wall_top: bool,
    pub(in crate::project_slice) raft_layers: i32,
    pub(in crate::project_slice) precise_outer_wall: bool,
    pub(in crate::project_slice) outer_wall_first: bool,
}

pub(in crate::project_slice) fn validate_record(
    region: &RegionOptions,
    object: &ObjectOptions,
    layer_id: usize,
    upper_layer_index: Option<usize>,
    enable_arc_fitting: bool,
    resolution: f64,
    nozzle_diameters: &OrcaFloats,
    scale: CoordinateScale,
) -> Result<ArachneRecordConfig, SliceError> {
    // Scope gates for deferred upstream behavior: typed rejections, never a
    // classic fallback. Each names the upstream slice that lands it.
    if crate::perimeters::FuzzySkinConfig::from_region(region).should_fuzzify(layer_id, 0, true) {
        return Err(unsupported("fuzzy_skin"));
    }
    if region.overhang_reverse.0 {
        // `detect_steep_overhang` / `reorient_perimeters`
        // (`PerimeterGenerator.cpp:449-478,2472-2474`).
        return Err(unsupported("overhang_reverse"));
    }
    if region.only_one_wall_top.0 && upper_layer_index.is_some() {
        // Top-surface single-wall regeneration
        // (`PerimeterGenerator.cpp:2168-2224`).
        return Err(unsupported("only_one_wall_top"));
    }
    if region.alternate_extra_wall.0 && layer_id % 2 == 1 && region.sparse_infill_density.0 > 0.0 {
        // Alternating extra wall (`PerimeterGenerator.cpp:2116-2117`).
        return Err(unsupported("alternate_extra_wall"));
    }

    let nozzle_index = region
        .outer_wall_filament_id
        .0
        .checked_sub(1)
        .and_then(|index| usize::try_from(index).ok())
        .filter(|index| *index < nozzle_diameters.0.len())
        .unwrap_or(0);
    let _ = nozzle_index;
    // `m_scaled_resolution` floor of `1e-4` and the arc-fitting resolution
    // reduction (`PerimeterGenerator.cpp:2119-2122`).
    let effective_resolution = if resolution > EPSILON {
        resolution
    } else {
        EPSILON
    };
    let scaled_resolution = effective_resolution / scale.factor();
    let surface_simplify_resolution = if enable_arc_fitting {
        0.2 * scaled_resolution
    } else {
        scaled_resolution
    };
    Ok(ArachneRecordConfig {
        params: make_paths_params(layer_id, object, nozzle_diameters, scale)?,
        surface_simplify_resolution,
        is_bottom_layer: i32::try_from(layer_id).is_ok_and(|layer| layer == object.raft_layers.0),
        is_topmost_layer: upper_layer_index.is_none(),
        only_one_wall_first_layer: region.only_one_wall_first_layer.0,
        only_one_wall_top: region.only_one_wall_top.0,
        raft_layers: object.raft_layers.0,
        precise_outer_wall: region.precise_outer_wall.0
            && region.wall_sequence == crate::ProcessWallSequence::InnerOuter,
        outer_wall_first: is_outer_wall_first(region, layer_id),
    })
}

/// `is_outer_wall_first` (`PerimeterGenerator.cpp:2273-2280`): the layer-0
/// rule disables the inner-outer-inner sequence.
pub(in crate::project_slice) fn is_outer_wall_first(
    region: &RegionOptions,
    layer_id: usize,
) -> bool {
    if layer_id == 0 {
        region.wall_sequence == crate::ProcessWallSequence::OuterInner
    } else {
        matches!(
            region.wall_sequence,
            crate::ProcessWallSequence::OuterInner | crate::ProcessWallSequence::InnerOuterInner
        )
    }
}

/// `make_paths_params` (`WallToolPaths.cpp:26-71`).
fn make_paths_params(
    layer_id: usize,
    object: &ObjectOptions,
    nozzle_diameters: &OrcaFloats,
    scale: CoordinateScale,
) -> Result<ArachneWallParams, SliceError> {
    let min_nozzle_diameter = nozzle_diameters
        .0
        .iter()
        .map(|diameter| diameter.0)
        .fold(f64::INFINITY, f64::min);
    if !min_nozzle_diameter.is_finite() {
        return Err(invalid("invalid Orca option nozzle_diameter"));
    }
    let scaled_percent = |percent: f64| -> Result<i64, SliceError> {
        scale
            .checked_scale(percent * 0.01 * min_nozzle_diameter)
            .ok_or_else(|| invalid("arachne wall parameters exceed the coordinate range"))
    };
    let scaled_float = |value: f64, key: &str| -> Result<i64, SliceError> {
        scale
            .checked_scale(value)
            .ok_or_else(|| SliceError::InvalidInput(format!("invalid Orca option {key}")))
    };
    Ok(ArachneWallParams {
        min_feature_size: scaled_percent(object.min_feature_size.0)?,
        min_length_factor: object.min_length_factor.0,
        min_bead_width: if layer_id == 0 {
            scaled_percent(object.initial_layer_min_bead_width.0)?
        } else {
            scaled_percent(object.min_bead_width.0)?
        },
        wall_transition_filter_deviation: scaled_percent(
            object.wall_transition_filter_deviation.0,
        )?,
        wall_transition_length: scaled_percent(object.wall_transition_length.0)?,
        wall_transition_angle: object.wall_transition_angle.0,
        wall_distribution_count: object.wall_distribution_count.0,
        wall_maximum_resolution: scaled_float(
            object.wall_maximum_resolution.0,
            "wall_maximum_resolution",
        )?,
        wall_maximum_deviation: scaled_float(
            object.wall_maximum_deviation.0,
            "wall_maximum_deviation",
        )?,
    })
}

fn invalid(message: &str) -> SliceError {
    SliceError::InvalidInput(message.to_owned())
}

fn unsupported(key: &str) -> SliceError {
    SliceError::UnsupportedProjectFeature(key.to_owned())
}
