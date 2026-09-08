use crate::{GenerationMetadata, SliceError};

use super::{incomplete_sink, perimeters, prepare_infill};

#[inline(never)]
pub(super) fn consume_post_horizontal_shell_propagation(
    prepared: prepare_infill::horizontal_shell_propagation::PreparedPostHorizontalShellPropagation,
    metadata: GenerationMetadata,
) -> Result<Vec<u8>, SliceError> {
    prepare_infill::horizontal_shell_propagation::dispose(prepared);
    let _ = metadata;
    Err(SliceError::ProjectSlicingIncomplete)
}

#[inline(never)]
pub(super) fn consume_post_horizontal_shell_promotion(
    prepared: prepare_infill::horizontal_shell_promotion::PreparedPostHorizontalShellPromotion,
    metadata: GenerationMetadata,
) -> Result<Vec<u8>, SliceError> {
    prepare_infill::horizontal_shell_promotion::dispose(prepared);
    let _ = metadata;
    Err(SliceError::ProjectSlicingIncomplete)
}

#[inline(never)]
pub(super) fn consume_post_vertical_shell_assignment(
    prepared: prepare_infill::vertical_shell_assignment::PreparedPostVerticalShellAssignment,
    metadata: GenerationMetadata,
) -> Result<Vec<u8>, SliceError> {
    prepare_infill::vertical_shell_assignment::dispose(prepared);
    let _ = metadata;
    Err(SliceError::ProjectSlicingIncomplete)
}

#[inline(never)]
pub(super) fn consume_post_vertical_shell_filtering(
    prepared: prepare_infill::vertical_shell_filtering::PreparedPostVerticalShellFiltering,
    metadata: GenerationMetadata,
) -> Result<Vec<u8>, SliceError> {
    prepare_infill::vertical_shell_filtering::dispose(prepared);
    let _ = metadata;
    Err(SliceError::ProjectSlicingIncomplete)
}

#[inline(never)]
pub(super) fn consume_post_vertical_shell_regularization(
    prepared: prepare_infill::vertical_shell_regularization::PreparedPostVerticalShellRegularization,
    metadata: GenerationMetadata,
) -> Result<Vec<u8>, SliceError> {
    prepare_infill::vertical_shell_regularization::dispose(prepared);
    let _ = metadata;
    Err(SliceError::ProjectSlicingIncomplete)
}

#[inline(never)]
pub(super) fn consume_post_vertical_shell_trim(
    prepared: prepare_infill::vertical_shell_trimming::PreparedPostVerticalShellTrim,
    metadata: GenerationMetadata,
) -> Result<Vec<u8>, SliceError> {
    prepare_infill::vertical_shell_trimming::dispose(prepared);
    let _ = metadata;
    Err(SliceError::ProjectSlicingIncomplete)
}

#[inline(never)]
pub(super) fn consume_post_vertical_shell_projection(
    prepared: prepare_infill::vertical_shell_projection::PreparedPostVerticalShellProjection,
    metadata: GenerationMetadata,
) -> Result<Vec<u8>, SliceError> {
    prepare_infill::vertical_shell_projection::dispose(prepared);
    let _ = metadata;
    Err(SliceError::ProjectSlicingIncomplete)
}

#[inline(never)]
pub(super) fn consume_post_vertical_shell_cache(
    prepared: prepare_infill::vertical_shells::PreparedPostVerticalShellCache,
    metadata: GenerationMetadata,
) -> Result<Vec<u8>, SliceError> {
    prepare_infill::vertical_shells::dispose(prepared);
    let _ = metadata;
    Err(SliceError::ProjectSlicingIncomplete)
}

#[inline(never)]
pub(super) fn consume_post_layer_region_perimeters(
    prepared: perimeters::layer_region::PreparedPostLayerRegionPerimeters,
    metadata: GenerationMetadata,
) -> Result<Vec<u8>, SliceError> {
    let perimeters::layer_region::PreparedPostLayerRegionPerimeters {
        predecessor,
        objects: layer_region_objects,
    } = prepared;
    for object in layer_region_objects {
        incomplete_sink::consume_layer_region_perimeter_object(object);
    }
    consume_post_classic_traversal_context(predecessor.into_classic(), metadata)
}

#[inline(never)]
fn consume_post_classic_traversal_context(
    predecessor: Box<perimeters::classic::PreparedPostClassicTraversal>,
    metadata: GenerationMetadata,
) -> Result<Vec<u8>, SliceError> {
    incomplete_sink::consume_boxed_post_classic_traversal(predecessor);
    let _ = metadata;
    Err(SliceError::ProjectSlicingIncomplete)
}
