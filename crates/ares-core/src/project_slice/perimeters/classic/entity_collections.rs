// Reachable loop-only `traverse_loops` seam from OrcaSlicer v2.4.2
// `PerimeterGenerator.cpp:230-280` and caller `1443-1450`.

mod orientation;
#[cfg(test)]
mod tests;
mod traverse;
mod types;

pub(in crate::project_slice) use orientation::orient_loop;
pub(in crate::project_slice) use types::{
    ExtrusionEntity, ExtrusionEntityCollection, PreparedEntityCollectionObject,
    PreparedEntityCollectionRecord, PreparedEntityCollectionSurface,
    PreparedPostClassicEntityCollections,
};
// The multi-path vocabulary is test-constructed until the arachne
// materialization seam builds real entities.
#[cfg_attr(not(test), allow(unused_imports))]
pub(in crate::project_slice) use types::{ExtrusionMultiPath, OrderedExtrusionLoop};

use super::{
    chained_loops::{
        PreparedChainedLoopObject, PreparedChainedLoopRecord, PreparedChainedLoopSurface,
        PreparedPostClassicChainedLoops,
    },
    traversal::{ClassicTraversalRecord, PostClassicTraversalPrintObject},
};
pub(in crate::project_slice) fn finish(
    prepared: PreparedPostClassicChainedLoops,
) -> PreparedPostClassicEntityCollections {
    let PreparedPostClassicChainedLoops {
        predecessor,
        objects,
    } = prepared;
    assert_eq!(objects.len(), predecessor.objects.len());
    let objects = objects
        .into_iter()
        .enumerate()
        .map(|(index, loops)| transform_object(loops, &predecessor.objects[index]))
        .collect();
    PreparedPostClassicEntityCollections {
        predecessor,
        objects,
    }
}

fn transform_object(
    loops: PreparedChainedLoopObject,
    traversal: &PostClassicTraversalPrintObject,
) -> PreparedEntityCollectionObject {
    assert_eq!(loops.records.len(), traversal.records.len());
    let records = loops
        .records
        .into_iter()
        .enumerate()
        .map(|(index, loops)| match (loops, &traversal.records[index]) {
            (None, None) => None,
            (Some(loops), Some(seeds)) => Some(transform_record(
                loops,
                seeds,
                traversal.wall_direction(index),
            )),
            _ => panic!("O5/O8 optional record alignment is invariant"),
        })
        .collect();
    PreparedEntityCollectionObject { records }
}

fn transform_record(
    loops: PreparedChainedLoopRecord,
    seeds: &ClassicTraversalRecord,
    wall_direction: crate::ProcessWallDirection,
) -> PreparedEntityCollectionRecord {
    assert_eq!(loops.surfaces.len(), seeds.surfaces.len());
    let surfaces = loops
        .surfaces
        .into_iter()
        .enumerate()
        .map(|(index, loops)| transform_surface(loops, &seeds.surfaces[index], wall_direction))
        .collect();
    PreparedEntityCollectionRecord { surfaces }
}

fn transform_surface(
    loops: PreparedChainedLoopSurface,
    seeds: &super::traversal::PreparedTraversalSurface,
    wall_direction: crate::ProcessWallDirection,
) -> PreparedEntityCollectionSurface {
    assert_eq!(loops.source_index, seeds.source_index);
    PreparedEntityCollectionSurface {
        source_index: loops.source_index,
        collection: traverse::traverse_loops(loops.roots, &seeds.roots, wall_direction),
    }
}
