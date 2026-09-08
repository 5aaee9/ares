use super::{layers, perimeters};
use perimeters::classic::entity_collections::ExtrusionEntity;

#[cfg(test)]
mod infill_boundary;
mod layer_region;
mod project_context;
pub(super) mod surface_type_detection;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(super) use infill_boundary::consume_infill_boundary_object;
pub(super) use layer_region::consume_layer_region_perimeter_object;

#[inline(never)]
pub(super) fn consume_boxed_post_classic_traversal(
    prepared: Box<perimeters::classic::PreparedPostClassicTraversal>,
) {
    let perimeters::classic::PreparedPostClassicTraversal {
        project,
        resolved,
        config_block,
        scale,
        objects,
        #[cfg(test)]
        drop_probe,
    } = *prepared;
    project_context::observe(&project, &resolved);
    for object in objects {
        consume_traversal_object(object);
    }
    let _ = (project, resolved, config_block, scale);
    #[cfg(test)]
    let _ = drop_probe;
}

#[inline(never)]
pub(super) fn consume_perimeter_append_object(
    object: perimeters::classic::perimeter_append::PreparedPerimeterAppendObject,
) {
    for record in object.records.into_iter().flatten() {
        for surface in record.surfaces {
            let _ = surface.source_index;
            consume_inactive_post_collection(surface.inactive);
            consume_appended_collections(surface.appended.collections);
        }
    }
}

#[inline(never)]
pub(super) fn consume_medial_gap_object(
    object: perimeters::classic::medial_gap::PreparedMedialGapObject,
) {
    for record in object.records.into_iter().flatten() {
        for surface in record.surfaces {
            consume_medial_gap_surface(surface);
        }
    }
}

#[inline(never)]
pub(super) fn consume_gap_extrusion_object(
    object: perimeters::classic::gap_extrusion::PreparedGapExtrusionObject,
) {
    for record in object.records.into_iter().flatten() {
        for surface in record.surfaces {
            consume_gap_extrusion_surface(surface);
        }
    }
}

fn consume_gap_extrusion_surface(
    surface: perimeters::classic::gap_extrusion::PreparedGapExtrusionSurface,
) {
    let _ = surface.source_index;
    consume_inactive_post_collection(surface.inactive);
    consume_appended_collections(surface.appended.collections);
    if let Some(medial) = surface.medial {
        consume_medial_domain(medial);
    }
    for entity in surface.gap_fill.entities {
        consume_gap_entity(entity);
    }
    for expolygon in surface.remaining {
        consume_expolygon(expolygon);
    }
}

fn consume_medial_domain(medial: perimeters::classic::medial_gap::MedialGapDomain) {
    for polyline in medial.polylines {
        let _ = (polyline.points, polyline.width, polyline.endpoints);
    }
    consume_pre_medial_gap_domain(medial.predecessor);
}

fn consume_gap_entity(entity: perimeters::classic::gap_extrusion::GapFillEntity) {
    match entity {
        perimeters::classic::gap_extrusion::GapFillEntity::Path(path) => consume_gap_path(path),
        perimeters::classic::gap_extrusion::GapFillEntity::Loop(paths) => {
            for path in paths {
                consume_gap_path(path);
            }
        }
    }
}

fn consume_expolygon(expolygon: crate::geometry::ExPolygon) {
    let (contour, holes) = expolygon.into_parts();
    let _ = contour.into_points();
    for hole in holes {
        let _ = hole.into_points();
    }
}

fn consume_gap_path(path: perimeters::classic::materialize::ExtrusionPath) {
    let _ = (
        path.polyline.points,
        path.role,
        path.mm3_per_mm,
        path.width,
        path.height,
    );
}

fn consume_medial_gap_surface(surface: perimeters::classic::medial_gap::PreparedMedialGapSurface) {
    let _ = surface.source_index;
    consume_inactive_post_collection(surface.inactive);
    consume_appended_collections(surface.appended.collections);
    if let Some(medial) = surface.medial {
        for polyline in medial.polylines {
            let _ = (polyline.points, polyline.width, polyline.endpoints);
        }
        consume_pre_medial_gap_domain(medial.predecessor);
    }
}

#[inline(never)]
pub(super) fn consume_gap_domain_object(
    object: perimeters::classic::gap_domain::PreparedGapDomainObject,
) {
    for record in object.records.into_iter().flatten() {
        for surface in record.surfaces {
            let _ = surface.source_index;
            consume_inactive_post_collection(surface.inactive);
            consume_appended_collections(surface.appended.collections);
            if let Some(pre_medial) = surface.pre_medial {
                consume_pre_medial_gap_domain(pre_medial);
            }
        }
    }
}

fn consume_pre_medial_gap_domain(pre_medial: perimeters::classic::gap_domain::PreMedialGapDomain) {
    let _ = (pre_medial.min, pre_medial.max);
    for expolygon in pre_medial.expolygons {
        let (contour, holes) = expolygon.into_parts();
        let _ = contour.into_points();
        for hole in holes {
            let _ = hole.into_points();
        }
    }
}

fn consume_inactive_post_collection(
    inactive: perimeters::classic::perimeter_append::InactivePostCollectionBranches,
) {
    let perimeters::classic::perimeter_append::InactiveOverhangReorientation::Disabled {
        overhang_reverse_internal_only,
    } = inactive.overhang_reorientation;
    let _ = overhang_reverse_internal_only;
    let perimeters::classic::perimeter_append::InactiveWallReordering::InnerOuter { outer_brim } =
        inactive.wall_reordering;
    match outer_brim {
        perimeters::classic::perimeter_append::InactiveOuterBrimReordering::LaterLayer {
            layer_id,
            brim_type,
            brim_width,
        } => {
            let _ = (layer_id, brim_type, brim_width);
        }
        perimeters::classic::perimeter_append::InactiveOuterBrimReordering::DifferentBrimType {
            brim_type,
            brim_width,
        } => {
            let _ = (brim_type, brim_width);
        }
        perimeters::classic::perimeter_append::InactiveOuterBrimReordering::WidthNotPositive {
            brim_width,
        } => {
            let _ = brim_width;
        }
    }
}

fn consume_appended_collections(
    collections: Vec<perimeters::classic::entity_collections::ExtrusionEntityCollection>,
) {
    for collection in collections {
        for entity in collection.entities {
            consume_entity(entity);
        }
    }
}

fn consume_entity(entity: ExtrusionEntity) {
    let paths = match entity {
        ExtrusionEntity::Loop(ordered) => {
            let _ = (ordered.inset_idx, ordered.extrusion_loop.role);
            ordered.extrusion_loop.paths
        }
        ExtrusionEntity::MultiPath(multi_path) => multi_path.paths,
    };
    for path in paths {
        let _ = path.polyline.fitting;
        consume_gap_path(path);
    }
}

#[inline(never)]
#[cfg(test)]
pub(super) fn consume_raw_path_object(
    object: perimeters::classic::materialize::PreparedRawPathObject,
) {
    for record in object.records.into_iter().flatten() {
        for surface in record.surfaces {
            let _ = surface.source_index;
            perimeters::classic::materialize::tree::consume_nodes(surface.roots);
        }
    }
}

#[inline(never)]
pub(super) fn consume_traversal_object(
    object: perimeters::classic::PostClassicTraversalPrintObject,
) {
    for (record_index, record) in object.records.iter().enumerate() {
        let Some(record) = record else { continue };
        for surface in &record.surfaces {
            let mut pending = surface.roots.iter().collect::<Vec<_>>();
            while let Some(seed) = pending.pop() {
                let _ = object.lower_series(record_index, seed.route).len();
                pending.extend(&seed.children);
            }
        }
    }
    let (hierarchy_object, records) = object.into_parts();
    for record in records.into_iter().flatten() {
        let _ = (
            record.layer_height,
            record.overhang_flow,
            record.branch,
            record.overhang_reverse,
        );
        for surface in record.surfaces {
            let _ = surface.source_index;
            consume_seeds(surface.roots);
        }
    }
    consume_hierarchy_object(hierarchy_object);
}

fn consume_seeds(mut pending: Vec<perimeters::classic::traversal::TraversalSeed>) {
    while let Some(mut seed) = pending.pop() {
        let _ = (
            seed.polygon,
            seed.depth,
            seed.is_contour,
            seed.is_smaller_width_perimeter,
            seed.extrusion_role,
            seed.loop_role,
            seed.route,
            seed.width,
            seed.mm3_per_mm,
        );
        pending.append(&mut seed.children);
    }
}

fn consume_hierarchy_object(object: perimeters::classic::PostClassicHierarchyPrintObject) {
    let (onion_object, records) = object.into_parts();
    for surface in records
        .into_iter()
        .flatten()
        .flat_map(|record| record.surfaces)
    {
        let _ = surface.source_index;
        consume_loops(surface.roots);
        for loops in surface.remaining_contours {
            consume_loops(loops);
        }
        for loops in surface.remaining_holes {
            consume_loops(loops);
        }
    }
    consume_onion_object(onion_object);
}

fn consume_loops(loops: Vec<perimeters::classic::hierarchy::PerimeterGeneratorLoop>) {
    let mut pending = loops;
    while let Some(loop_) = pending.pop() {
        let _ = (
            loop_.polygon,
            loop_.is_contour,
            loop_.is_smaller_width_perimeter,
            loop_.depth,
        );
        pending.extend(loop_.children);
    }
}

fn consume_onion_object(object: perimeters::classic::PostClassicOnionPrintObject) {
    let (top_split_object, onion_records) = object.into_parts();
    for surface in onion_records
        .into_iter()
        .flatten()
        .flat_map(|record| record.surfaces)
    {
        let _ = (
            surface.source_index,
            surface.initial_loop_number,
            surface.effective_loop_number,
            surface.last,
            surface.gaps,
        );
        for shell in surface.shells {
            let _ = (shell.depth, shell.normal, shell.smaller_width);
        }
    }
    let (prelude_object, top_split_records) = top_split_object.into_parts();
    for record in top_split_records.into_iter().flatten() {
        for surface in record.surfaces {
            let _ = (
                surface.source_index,
                surface.initial_loop_number,
                surface.effective_loop_number,
                surface.normal_first_offset,
                surface.smaller_first_offset,
                surface.remaining,
                surface.top_fills,
                surface.fill_clip,
                surface.outcome,
                surface.upper_source,
            );
        }
    }
    let (perimeter_input_object, classic_records) = prelude_object.into_parts();
    for record in classic_records.into_iter().flatten() {
        let _ = (
            record.perimeter_width,
            record.perimeter_spacing,
            record.external_width,
            record.external_spacing,
            record.external_to_internal_spacing,
            record.solid_infill_spacing,
            record.minimum_spacing,
            record.external_minimum_spacing,
            record.smaller_external_minimum_spacing,
            record.has_gap_fill,
            record.smaller_external_flow,
            record.lower_slices_polygons,
            record.lower_polygons_series,
            record.external_lower_polygons_series,
            record.smaller_external_lower_polygons_series,
            record.surface_simplify_resolution,
        );
        for surface in record.surfaces {
            let _ = (
                surface.source_index,
                surface.kind,
                surface.thickness,
                surface.thickness_layers,
                surface.bridge_angle,
                surface.extra_perimeters,
                surface.loop_number,
                surface.polygons,
            );
        }
    }
    let (compensation_object, perimeter_inputs) = perimeter_input_object.into_parts();
    let _ = perimeter_inputs;
    let (region_object, lslices) = compensation_object.into_parts();
    let (plan, volume_slices, regions) = region_object.into_parts();
    let _ = (volume_slices, regions, lslices);
    let layers::PlannedPrintObject {
        source_object_index,
        transform_index,
        layers,
    } = plan;
    let _ = (source_object_index, transform_index);
    for layers::PlannedLayer {
        id,
        height,
        print_z,
        slice_z,
    } in layers
    {
        let _ = (id, height, print_z, slice_z);
    }
}
