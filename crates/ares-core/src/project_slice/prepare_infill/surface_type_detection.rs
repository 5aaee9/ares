mod cracks;
mod extra_bridge;
mod geometry;
mod preflight;
mod stage;
#[cfg(test)]
mod tests;
pub(in crate::project_slice) mod types;

#[cfg(test)]
pub(in crate::project_slice) use geometry::GeometryStep;
pub(in crate::project_slice) use types::{
    PreparedPostSurfaceTypeDetection, PreparedSurfaceTypeObject,
};

use crate::{
    SliceError,
    project_slice::{incomplete_sink, perimeters::layer_region::PreparedPostLayerRegionPerimeters},
};

#[cfg(test)]
thread_local! {
    static INVOCATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(in crate::project_slice) fn prepare(
    prepared: PreparedPostLayerRegionPerimeters,
) -> Result<PreparedPostSurfaceTypeDetection, SliceError> {
    #[cfg(test)]
    INVOCATIONS.with(|count| count.set(count.get() + 1));

    if let Err(error) = preflight::validate(&prepared.predecessor.as_classic().resolved.objects) {
        dispose_predecessor(prepared);
        return Err(error);
    }
    let staged = match stage::project(&prepared) {
        Ok(staged) => staged,
        Err(error) => {
            dispose_predecessor(prepared);
            return Err(error);
        }
    };
    let PreparedPostLayerRegionPerimeters {
        predecessor,
        objects,
    } = prepared;
    assert_eq!(objects.len(), staged.len());
    let objects = objects
        .into_iter()
        .zip(staged)
        .map(|(object, staged)| {
            assert_eq!(object.records.len(), staged.records.len());
            let records = object
                .records
                .into_iter()
                .zip(staged.records)
                .map(|(source, staged)| match (source, staged) {
                    (Some(source), Some(staged)) => Some(types::materialize_record(source, staged)),
                    (None, None) => None,
                    _ => unreachable!("staged O17 slots remain aligned with O16"),
                })
                .collect();
            PreparedSurfaceTypeObject { records }
        })
        .collect();
    Ok(PreparedPostSurfaceTypeDetection {
        predecessor: predecessor.into_classic(),
        objects,
    })
}

fn dispose_predecessor(prepared: PreparedPostLayerRegionPerimeters) {
    let PreparedPostLayerRegionPerimeters {
        predecessor,
        objects,
    } = prepared;
    for object in objects {
        incomplete_sink::consume_layer_region_perimeter_object(object);
    }
    incomplete_sink::consume_boxed_post_classic_traversal(predecessor.into_classic());
}

#[cfg(test)]
pub(in crate::project_slice) fn invocations() -> usize {
    INVOCATIONS.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::project_slice) fn reset_invocations() {
    INVOCATIONS.with(|count| count.set(0));
}

#[cfg(test)]
pub(in crate::project_slice) fn reset_geometry_hooks() {
    tests::reset_geometry_hooks();
}

#[cfg(test)]
pub(in crate::project_slice) fn fail_geometry_at(step: GeometryStep) {
    tests::fail_at(step);
}

#[cfg(test)]
pub(in crate::project_slice) fn geometry_events() -> Vec<GeometryStep> {
    tests::geometry_events()
}

#[cfg(test)]
pub(in crate::project_slice) fn stage_for_test(
    prepared: &PreparedPostLayerRegionPerimeters,
) -> Result<(), SliceError> {
    stage::project(prepared).map(drop)
}
