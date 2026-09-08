use crate::project_slice::{
    perimeters::classic::{
        entity_collections::ExtrusionEntity, traversal::PreparedPostClassicTraversal,
    },
    prepare_infill::{
        surface_type_detection::PreparedSurfaceTypeObject,
        vertical_shell_projection::types::VerticalShellProjectionObject,
        vertical_shells::types::VerticalShellCacheObject,
    },
    tests::prepare_infill::vertical_shell_projection::{
        predecessor_geometry_point_buffers as classic_geometry_point_buffers,
        predecessor_snapshot as classic_predecessor_snapshot,
    },
};

pub(super) fn predecessor_snapshot(predecessor: &PreparedPostClassicTraversal) -> Vec<usize> {
    classic_predecessor_snapshot(predecessor)
}

pub(super) fn projection_snapshot(objects: &[VerticalShellProjectionObject]) -> Vec<usize> {
    let mut snapshot = vec![objects.as_ptr() as usize, objects.len()];
    for object in objects {
        snapshot.extend([object.records.as_ptr() as usize, object.records.len()]);
        for record in &object.records {
            snapshot.push(usize::from(record.is_some()));
            if let Some(projection) = record {
                snapshot_paths(&mut snapshot, &projection.shell);
                snapshot_paths(&mut snapshot, &projection.holes);
            }
        }
    }
    snapshot
}

pub(super) fn cache_snapshot(objects: &[VerticalShellCacheObject]) -> Vec<usize> {
    let mut snapshot = vec![objects.as_ptr() as usize, objects.len()];
    for object in objects {
        snapshot.extend([object.records.as_ptr() as usize, object.records.len()]);
        for record in &object.records {
            snapshot.push(usize::from(record.is_some()));
            if let Some(cache) = record {
                snapshot_paths(&mut snapshot, &cache.top_surfaces);
                snapshot_paths(&mut snapshot, &cache.bottom_surfaces);
                snapshot_paths(&mut snapshot, &cache.holes);
            }
        }
    }
    snapshot
}

fn snapshot_paths(snapshot: &mut Vec<usize>, paths: &[crate::geometry::Polygon]) {
    snapshot.extend([paths.as_ptr() as usize, paths.len()]);
    for path in paths {
        snapshot.extend([path.points().as_ptr() as usize, path.points().len()]);
    }
}

pub(super) fn predecessor_geometry_point_buffers(
    predecessor: &PreparedPostClassicTraversal,
    objects: &[PreparedSurfaceTypeObject],
    caches: &[VerticalShellCacheObject],
    projections: &[VerticalShellProjectionObject],
) -> Vec<usize> {
    let mut points = classic_geometry_point_buffers(predecessor);
    surface_type_point_buffers(&mut points, objects);
    cache_point_buffers(&mut points, caches);
    projection_point_buffers(&mut points, projections);
    points
}

fn surface_type_point_buffers(points: &mut Vec<usize>, objects: &[PreparedSurfaceTypeObject]) {
    for record in objects
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
        for fill in &record.thin_fills {
            match fill {
                crate::project_slice::perimeters::classic::gap_extrusion::GapFillEntity::Path(
                    path,
                ) => points.push(path.polyline.points.as_ptr() as usize),
                crate::project_slice::perimeters::classic::gap_extrusion::GapFillEntity::Loop(
                    paths,
                ) => points.extend(
                    paths
                        .iter()
                        .map(|path| path.polyline.points.as_ptr() as usize),
                ),
            }
        }
        for surface in record.slices.iter().chain(&record.fill_surfaces) {
            expolygon_point_buffers(points, surface.as_parts().1);
        }
        for expolygon in record
            .fill_expolygons
            .iter()
            .chain(&record.fill_no_overlap_expolygons)
        {
            expolygon_point_buffers(points, expolygon);
        }
    }
}

fn expolygon_point_buffers(points: &mut Vec<usize>, expolygon: &crate::geometry::ExPolygon) {
    points.push(expolygon.contour().points().as_ptr() as usize);
    points.extend(
        expolygon
            .holes()
            .iter()
            .map(|hole| hole.points().as_ptr() as usize),
    );
}

fn cache_point_buffers(points: &mut Vec<usize>, objects: &[VerticalShellCacheObject]) {
    points.extend(
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
            .map(|path| path.points().as_ptr() as usize),
    );
}

fn projection_point_buffers(points: &mut Vec<usize>, objects: &[VerticalShellProjectionObject]) {
    points.extend(
        objects
            .iter()
            .flat_map(|object| object.records.iter().flatten())
            .flat_map(|projection| projection.shell.iter().chain(&projection.holes))
            .map(|path| path.points().as_ptr() as usize),
    );
}
