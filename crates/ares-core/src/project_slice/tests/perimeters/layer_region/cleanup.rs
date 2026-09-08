use crate::project_slice::tests::deep_cleanup_support::{
    deepen_both_tree_families, run_on_constrained_stack,
};

use crate::{
    SliceError,
    project_slice::{
        consume_post_layer_region_perimeters, incomplete_sink,
        perimeters::{layer_region, prepare_post_classic_infill_boundary},
    },
};

use super::super::super::support::{ksr_project, metadata};

#[test]
fn task22o16_success_cleanup_with_both_deep_predecessors_fits_constrained_stack() {
    let mut source = prepare_post_classic_infill_boundary(ksr_project()).unwrap();
    deepen_both_tree_families(&mut source.predecessor);
    run_on_constrained_stack(move || {
        let output = layer_region::finish(source);
        for object in output.objects {
            incomplete_sink::consume_layer_region_perimeter_object(object);
        }
        incomplete_sink::consume_boxed_post_classic_traversal(output.predecessor.into_classic());
    });
}

#[test]
fn task22o16_incomplete_lifecycle_with_both_deep_predecessors_fits_constrained_stack() {
    let mut source = prepare_post_classic_infill_boundary(ksr_project()).unwrap();
    deepen_both_tree_families(&mut source.predecessor);
    run_on_constrained_stack(move || {
        let output = layer_region::finish(source);
        assert_eq!(
            consume_post_layer_region_perimeters(output, metadata()).unwrap_err(),
            SliceError::ProjectSlicingIncomplete
        );
    });
}
