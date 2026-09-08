use std::sync::atomic::Ordering;

use crate::{
    SliceError,
    project_slice::{
        perimeters,
        prepare_infill::surface_type_detection::{
            self, GeometryStep, fail_geometry_at, geometry_events, reset_geometry_hooks,
        },
    },
};

use super::super::super::support::KsrArchive;

#[test]
fn task22o17_geometry_failure_consumes_the_unmoved_predecessor_transactionally() {
    let prepared =
        perimeters::prepare_post_layer_region_perimeters(&KsrArchive::new().bytes()).unwrap();
    let (weak, dropped) = prepared.predecessor.as_classic().drop_probe_observer();
    reset_geometry_hooks();
    fail_geometry_at(GeometryStep::TopSafetyDifference);
    let error = match surface_type_detection::prepare(prepared) {
        Err(error) => error,
        Ok(_) => panic!("injected O17 geometry failure must not produce a successor"),
    };
    assert_eq!(
        error,
        SliceError::InvalidInput(
            "surface-type detection geometry is outside the supported Clipper range".to_owned()
        )
    );
    assert!(geometry_events().contains(&GeometryStep::TopSafetyDifference));
    assert!(weak.upgrade().is_none());
    assert!(dropped.load(Ordering::SeqCst));
    reset_geometry_hooks();
}
