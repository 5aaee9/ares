use crate::{
    geometry::Polygon,
    project_slice::perimeters::classic::entity_collections::ExtrusionEntity,
    project_slice::prepare_infill::{
        vertical_shell_projection::types::VerticalShellProjectionObject,
        vertical_shell_regularization::types::VerticalShellRegularizationObject,
        vertical_shell_trimming::{PreparedPostVerticalShellTrim, types::VerticalShellTrimObject},
        vertical_shells::types::VerticalShellCacheObject,
    },
};

pub(in crate::project_slice::tests::prepare_infill) fn cache_snapshot(
    objects: &[VerticalShellCacheObject],
) -> Vec<usize> {
    let mut snapshot = vec![objects.as_ptr() as usize, objects.len()];
    for object in objects {
        snapshot.extend([object.records.as_ptr() as usize, object.records.len()]);
        for cache in object.records.iter().flatten() {
            paths_snapshot(&mut snapshot, &cache.top_surfaces);
            paths_snapshot(&mut snapshot, &cache.bottom_surfaces);
            paths_snapshot(&mut snapshot, &cache.holes);
        }
    }
    snapshot
}

pub(in crate::project_slice::tests::prepare_infill) fn projection_snapshot(
    objects: &[VerticalShellProjectionObject],
) -> Vec<usize> {
    let mut snapshot = vec![objects.as_ptr() as usize, objects.len()];
    for object in objects {
        snapshot.extend([object.records.as_ptr() as usize, object.records.len()]);
        for projection in object.records.iter().flatten() {
            paths_snapshot(&mut snapshot, &projection.shell);
            paths_snapshot(&mut snapshot, &projection.holes);
        }
    }
    snapshot
}

pub(in crate::project_slice::tests::prepare_infill) fn trim_snapshot(
    objects: &[VerticalShellTrimObject],
) -> Vec<usize> {
    let mut snapshot = vec![objects.as_ptr() as usize, objects.len()];
    for object in objects {
        snapshot.extend([object.records.as_ptr() as usize, object.records.len()]);
        for trim in object.records.iter().flatten() {
            paths_snapshot(&mut snapshot, &trim.shell);
        }
    }
    snapshot
}

fn paths_snapshot(snapshot: &mut Vec<usize>, paths: &[Polygon]) {
    snapshot.extend([paths.as_ptr() as usize, paths.len()]);
    for path in paths {
        snapshot.extend([path.points().as_ptr() as usize, path.points().len()]);
    }
}

pub(in crate::project_slice::tests::prepare_infill) fn all_predecessor_points(
    input: &PreparedPostVerticalShellTrim,
) -> Vec<usize> {
    let mut points =
        super::super::super::vertical_shell_projection::predecessor_geometry_point_buffers(
            &input.predecessor,
        );
    for record in input
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
    {
        points.extend(
            record
                .perimeters
                .iter()
                .flat_map(|collection| &collection.entities)
                .flat_map(|entity| match entity {
                    ExtrusionEntity::Loop(ordered) => &ordered.extrusion_loop.paths,
                    ExtrusionEntity::MultiPath(multi_path) => &multi_path.paths,
                })
                .map(|path| path.polyline.points.as_ptr() as usize),
        );
        for surface in record.slices.iter().chain(&record.fill_surfaces) {
            expolygon_points(&mut points, surface.as_parts().1);
        }
        for expolygon in record
            .fill_expolygons
            .iter()
            .chain(&record.fill_no_overlap_expolygons)
        {
            expolygon_points(&mut points, expolygon);
        }
    }
    points.extend(cache_points(&input.caches));
    points.extend(projection_points(&input.projections));
    points.extend(trim_points(&input.trims));
    points
}

fn expolygon_points(points: &mut Vec<usize>, expolygon: &crate::geometry::ExPolygon) {
    points.push(expolygon.contour().points().as_ptr() as usize);
    points.extend(
        expolygon
            .holes()
            .iter()
            .map(|hole| hole.points().as_ptr() as usize),
    );
}

fn cache_points(objects: &[VerticalShellCacheObject]) -> impl Iterator<Item = usize> + '_ {
    objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|cache| {
            cache
                .top_surfaces
                .iter()
                .chain(&cache.bottom_surfaces)
                .chain(&cache.holes)
        })
        .map(|path| path.points().as_ptr() as usize)
}

fn projection_points(
    objects: &[VerticalShellProjectionObject],
) -> impl Iterator<Item = usize> + '_ {
    objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|projection| projection.shell.iter().chain(&projection.holes))
        .map(|path| path.points().as_ptr() as usize)
}

fn trim_points(objects: &[VerticalShellTrimObject]) -> impl Iterator<Item = usize> + '_ {
    objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|trim| &trim.shell)
        .map(|path| path.points().as_ptr() as usize)
}

pub(in crate::project_slice::tests::prepare_infill) fn regularization_allocations(
    objects: &[VerticalShellRegularizationObject],
) -> Vec<usize> {
    let mut allocations = vec![objects.as_ptr() as usize];
    for object in objects {
        allocations.push(object.records.as_ptr() as usize);
        for regularization in object.records.iter().flatten() {
            allocations.extend(
                (!regularization.regularized_shell.is_empty())
                    .then_some(regularization.regularized_shell.as_ptr() as usize),
            );
            for expolygon in &regularization.regularized_shell {
                allocations.push(expolygon.contour().points().as_ptr() as usize);
                allocations.extend(
                    (!expolygon.holes().is_empty()).then_some(expolygon.holes().as_ptr() as usize),
                );
                allocations.extend(
                    expolygon
                        .holes()
                        .iter()
                        .map(|hole| hole.points().as_ptr() as usize),
                );
            }
        }
    }
    allocations
}

pub(in crate::project_slice::tests::prepare_infill) fn regularization_point_buffers(
    objects: &[VerticalShellRegularizationObject],
) -> Vec<usize> {
    objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|regularization| &regularization.regularized_shell)
        .flat_map(|expolygon| std::iter::once(expolygon.contour()).chain(expolygon.holes().iter()))
        .map(|path| path.points().as_ptr() as usize)
        .collect()
}
