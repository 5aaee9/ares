use crate::project_slice::{
    perimeters,
    perimeters::classic::entity_collections::ExtrusionEntity,
    prepare_infill::{fill_surfaces, surface_type_detection},
    region_slices::RegionSurface,
    tests::support::KsrArchive,
};

#[test]
fn task22o18_moves_all_o17_allocations_and_retags_only_in_place() {
    let detected = surface_type_detection::prepare(
        perimeters::prepare_post_layer_region_perimeters(&KsrArchive::new().bytes()).unwrap(),
    )
    .unwrap();
    let predecessor = std::ptr::from_ref(detected.predecessor.as_ref());
    let before = allocation_snapshot(&detected.objects);
    let output = fill_surfaces::prepare(detected);
    assert_eq!(std::ptr::from_ref(output.predecessor.as_ref()), predecessor);
    assert_eq!(allocation_snapshot(&output.objects), before);
}

pub(in crate::project_slice::tests::prepare_infill) fn allocation_snapshot(
    objects: &[surface_type_detection::PreparedSurfaceTypeObject],
) -> Vec<usize> {
    let mut output = vec![objects.as_ptr() as usize, objects.len()];
    for object in objects {
        output.extend([object.records.as_ptr() as usize, object.records.len()]);
        for record in &object.records {
            output.push(usize::from(record.is_some()));
            if let Some(record) = record {
                record_allocations(&mut output, record);
            }
        }
    }
    output
}

fn record_allocations(
    output: &mut Vec<usize>,
    record: &surface_type_detection::types::PreparedSurfaceTypeRecord,
) {
    output.extend([record.perimeters.as_ptr() as usize, record.perimeters.len()]);
    for collection in &record.perimeters {
        output.extend([
            collection.entities.as_ptr() as usize,
            collection.entities.len(),
        ]);
        for entity in &collection.entities {
            let paths = match entity {
                ExtrusionEntity::Loop(ordered) => &ordered.extrusion_loop.paths,
                ExtrusionEntity::MultiPath(_) => panic!("classic collections keep loop entities"),
            };
            output.extend([paths.as_ptr() as usize, paths.len()]);
            for path in paths {
                path_allocations(output, path);
            }
        }
    }
    output.extend([record.thin_fills.as_ptr() as usize, record.thin_fills.len()]);
    for entity in &record.thin_fills {
        thin_allocations(output, entity);
    }
    surface_allocations(output, &record.slices);
    surface_allocations(output, &record.fill_surfaces);
    expolygon_allocations(output, &record.fill_expolygons);
    expolygon_allocations(output, &record.fill_no_overlap_expolygons);
}

fn thin_allocations(
    output: &mut Vec<usize>,
    entity: &perimeters::classic::gap_extrusion::GapFillEntity,
) {
    match entity {
        perimeters::classic::gap_extrusion::GapFillEntity::Path(path) => {
            path_allocations(output, path);
        }
        perimeters::classic::gap_extrusion::GapFillEntity::Loop(paths) => {
            output.extend([paths.as_ptr() as usize, paths.len()]);
            for path in paths {
                path_allocations(output, path);
            }
        }
    }
}

fn path_allocations(
    output: &mut Vec<usize>,
    path: &perimeters::classic::materialize::ExtrusionPath,
) {
    output.extend([
        path.polyline.points.as_ptr() as usize,
        path.polyline.points.len(),
    ]);
}

fn surface_allocations(output: &mut Vec<usize>, surfaces: &[RegionSurface]) {
    output.extend([surfaces.as_ptr() as usize, surfaces.len()]);
    for surface in surfaces {
        expolygon_geometry(output, surface.as_parts().1);
    }
}

fn expolygon_allocations(output: &mut Vec<usize>, expolygons: &[crate::geometry::ExPolygon]) {
    output.extend([expolygons.as_ptr() as usize, expolygons.len()]);
    for expolygon in expolygons {
        expolygon_geometry(output, expolygon);
    }
}

fn expolygon_geometry(output: &mut Vec<usize>, expolygon: &crate::geometry::ExPolygon) {
    output.extend([
        expolygon.contour().points().as_ptr() as usize,
        expolygon.contour().points().len(),
        expolygon.holes().as_ptr() as usize,
        expolygon.holes().len(),
    ]);
    for hole in expolygon.holes() {
        output.extend([hole.points().as_ptr() as usize, hole.points().len()]);
    }
}
