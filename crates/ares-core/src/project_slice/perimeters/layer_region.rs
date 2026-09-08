// Layer-region perimeter outputs from OrcaSlicer v2.4.2 `LayerRegion.cpp:82-142`
// and the one-compatible-region branch in `Layer.cpp:185-226`.

#[cfg(test)]
mod tests;
mod types;

pub(in crate::project_slice) use types::{
    PostPerimeterPredecessor, PreparedLayerRegionPerimeterObject,
    PreparedLayerRegionPerimeterRecord, PreparedPostLayerRegionPerimeters,
};

use super::classic::{
    gap_extrusion::PreparedGapExtrusionSurface,
    infill_boundary::{
        PreparedInfillBoundaryObject, PreparedInfillBoundaryRecord,
        PreparedPostClassicInfillBoundary,
    },
};

#[cfg(test)]
thread_local! {
    static FINISH_INVOCATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(in crate::project_slice) fn finish(
    prepared: PreparedPostClassicInfillBoundary,
) -> PreparedPostLayerRegionPerimeters {
    #[cfg(test)]
    FINISH_INVOCATIONS.with(|count| count.set(count.get() + 1));

    validate_alignment(&prepared);
    let PreparedPostClassicInfillBoundary {
        predecessor,
        objects,
    } = prepared;
    let objects = objects.into_iter().map(materialize_object).collect();
    PreparedPostLayerRegionPerimeters {
        predecessor: PostPerimeterPredecessor::Classic(predecessor),
        objects,
    }
}

fn validate_alignment(prepared: &PreparedPostClassicInfillBoundary) {
    assert_eq!(prepared.objects.len(), prepared.predecessor.objects.len());
    for (source, traversal) in prepared.objects.iter().zip(&prepared.predecessor.objects) {
        let input_object = &traversal
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object;
        let identity = input_object.identity();
        assert_eq!(source.records.len(), input_object.records.len());
        for (source, input) in source.records.iter().zip(&input_object.records) {
            match (source, input) {
                (None, None) => {}
                (Some(_), Some(input)) => {
                    assert_eq!((input.source_object_index, input.transform_index), identity);
                    assert_eq!(input.compatible_region_ids, [input.region_id]);
                    let (post_region, _) = input_object.object.as_parts();
                    let (_, _, regions) = post_region.as_parts();
                    assert_eq!(
                        regions[input.current.region_index].as_parts().0,
                        input.region_id
                    );
                }
                _ => panic!("O16 predecessor record alignment is invariant"),
            }
        }
    }
}

pub(in crate::project_slice) fn materialize_object(
    source: PreparedInfillBoundaryObject,
) -> PreparedLayerRegionPerimeterObject {
    PreparedLayerRegionPerimeterObject {
        records: source
            .records
            .into_iter()
            .map(|record| record.map(materialize_record))
            .collect(),
    }
}

pub(super) fn materialize_record(
    source: PreparedInfillBoundaryRecord,
) -> PreparedLayerRegionPerimeterRecord {
    let PreparedInfillBoundaryRecord {
        surfaces,
        fill_surfaces,
        fill_no_overlap,
        overlap,
    } = source;
    let perimeter_count = surfaces
        .iter()
        .map(|surface| surface.appended.collections.len())
        .sum();
    let thin_fill_count = surfaces
        .iter()
        .map(|surface| surface.gap_fill.entities.len())
        .sum();
    let mut perimeters = Vec::with_capacity(perimeter_count);
    let mut thin_fills = Vec::with_capacity(thin_fill_count);
    for surface in surfaces {
        append_surface_outputs(surface, &mut perimeters, &mut thin_fills);
    }
    let fill_expolygons = fill_surfaces
        .iter()
        .map(|surface| surface.as_parts().1.clone())
        .collect();
    let _ = overlap;
    PreparedLayerRegionPerimeterRecord {
        perimeters,
        thin_fills,
        fill_surfaces,
        fill_expolygons,
        fill_no_overlap_expolygons: fill_no_overlap,
    }
}

fn append_surface_outputs(
    surface: PreparedGapExtrusionSurface,
    perimeters: &mut Vec<super::classic::entity_collections::ExtrusionEntityCollection>,
    thin_fills: &mut Vec<super::classic::gap_extrusion::GapFillEntity>,
) {
    let PreparedGapExtrusionSurface {
        appended,
        gap_fill,
        source_index: _,
        inactive: _,
        medial: _,
        remaining: _,
    } = surface;
    perimeters.extend(appended.collections);
    thin_fills.extend(gap_fill.entities);
}

#[cfg(test)]
pub(in crate::project_slice) fn finish_invocations() -> usize {
    FINISH_INVOCATIONS.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::project_slice) fn reset_finish_invocations() {
    FINISH_INVOCATIONS.with(|count| count.set(0));
}
