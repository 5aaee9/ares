use crate::project_slice::{
    fill_entities::{self, FillExtrusionEntity},
    perimeters::classic::{entity_collections::ExtrusionEntity, gap_extrusion::GapFillEntity},
    tests::prepare_infill::group_fills::focused::fixture::graph,
};

fn entity_sub_paths(
    entity: &ExtrusionEntity,
) -> &[crate::project_slice::perimeters::classic::materialize::ExtrusionPath] {
    match entity {
        ExtrusionEntity::Loop(ordered) => &ordered.extrusion_loop.paths,
        ExtrusionEntity::MultiPath(multi_path) => &multi_path.paths,
    }
}

#[test]
fn task22o91_stage_materializes_all_objects_and_layers_in_order() {
    let graph = graph();
    fill_entities::reset_hooks();

    let prepared = fill_entities::prepare(graph).unwrap();

    assert_eq!(fill_entities::invocations(), 1);
    assert_eq!(fill_entities::disposals(), 0);
    assert_eq!(prepared.objects.len(), 1);
    assert!(!prepared.objects[0].is_empty());
    assert!(prepared.objects[0].iter().all(|layer| {
        layer
            .collections
            .iter()
            .all(|collection| collection.entities.iter().all(valid_fill_entity))
    }));
    let thin_inventory = prepared.objects[0]
        .iter()
        .flat_map(|layer| &layer.thin_fills)
        .fold((0_usize, 0_usize, 0_usize), |mut counts, entity| {
            counts.0 += 1;
            match entity {
                GapFillEntity::Path(path) => {
                    counts.1 += 1;
                    counts.2 += path.polyline.points.len();
                }
                GapFillEntity::Loop(paths) => {
                    counts.1 += paths.len();
                    counts.2 += paths
                        .iter()
                        .map(|path| path.polyline.points.len())
                        .sum::<usize>();
                }
            }
            counts
        });
    assert!(thin_inventory.0 > 0);
    assert_eq!(thin_inventory.0, thin_inventory.1);
    assert!(thin_inventory.2 >= thin_inventory.1 * 2);
    let perimeter_inventory = prepared.objects[0]
        .iter()
        .flat_map(|layer| &layer.perimeters)
        .fold(
            (0_usize, 0_usize, 0_usize, 0_usize),
            |mut counts, collection| {
                counts.0 += 1;
                counts.1 += collection.entities.len();
                counts.2 += collection
                    .entities
                    .iter()
                    .map(|entity| entity_sub_paths(entity).len())
                    .sum::<usize>();
                counts.3 += collection
                    .entities
                    .iter()
                    .flat_map(entity_sub_paths)
                    .map(|path| path.polyline.points.len())
                    .sum::<usize>();
                counts
            },
        );
    assert!(
        [
            perimeter_inventory.0,
            perimeter_inventory.1,
            perimeter_inventory.2,
            perimeter_inventory.3,
        ]
        .into_iter()
        .all(|count| count > 0)
    );
    assert!(perimeter_inventory.2 >= perimeter_inventory.1);
    assert!(perimeter_inventory.3 >= perimeter_inventory.2 * 2);
    let source = &prepared.predecessor.predecessor.predecessor.predecessor;
    assert!(
        source
            .objects
            .iter()
            .all(|object| object.records.iter().all(|record| {
                record.as_ref().is_none_or(|record| {
                    record.perimeters.is_empty() && record.thin_fills.is_empty()
                })
            }))
    );
    fill_entities::dispose(prepared);
    assert_eq!(fill_entities::disposals(), 1);
}

fn valid_fill_entity(entity: &FillExtrusionEntity) -> bool {
    match entity {
        FillExtrusionEntity::Path(path) => path.polyline.is_valid(),
        FillExtrusionEntity::VariableWidth(GapFillEntity::Path(path)) => {
            path.polyline.points.len() >= 2
        }
        FillExtrusionEntity::VariableWidth(GapFillEntity::Loop(paths)) => {
            !paths.is_empty() && paths.iter().all(|path| path.polyline.points.len() >= 2)
        }
    }
}

#[test]
fn task22o91_stage_is_repeatable_for_independent_graphs() {
    let first = fill_entities::prepare(graph()).unwrap();
    let second = fill_entities::prepare(graph()).unwrap();

    assert_eq!(first.objects, second.objects);

    fill_entities::dispose(first);
    fill_entities::dispose(second);
}
