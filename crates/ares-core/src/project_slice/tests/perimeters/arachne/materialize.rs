// Arachne materialization transaction tests (OrcaSlicer
// `PerimeterGenerator.cpp:2093-2519`): the Snapmaker A250 actual-default
// fixture from the wave-9 evidence produces real arachne perimeters — no
// classic fallback, no fabricated entities.

use crate::project_slice::perimeters::{
    classic::entity_collections::ExtrusionEntity, prepare_post_classic_prelude,
    prepare_post_layer_region_perimeters, types::PerimeterDispatch,
};

const FIXTURE: &[u8] =
    include_bytes!("../../../../../../../tests/parity/arachne/default-prisms.3mf");

#[test]
fn task22w17_fixture_materializes_real_arachne_perimeters() {
    let output = prepare_post_layer_region_perimeters(FIXTURE).unwrap();

    let mut walls = 0_usize;
    let mut loops = 0_usize;
    let mut multipaths = 0_usize;
    let mut variable_widths = 0_usize;
    let mut fill_regions = 0_usize;
    let count_loop =
        |paths: &[crate::project_slice::perimeters::classic::materialize::ExtrusionPath],
         variable_widths: &mut usize,
         walls: &mut usize| {
            for path in paths {
                *walls += 1;
                // Arachne beading rebuilds the flow width per constant-width
                // sub-path (`VariableWidth.cpp:5-102`); a fixed-flow classic
                // wall would reuse the flow width exactly.
                if path.width != 0.42 && path.width != 0.45 {
                    *variable_widths += 1;
                }
            }
        };
    for object in &output.objects {
        let entities = object.records.iter().flatten().flat_map(|record| {
            fill_regions += record.fill_surfaces.len();
            record.perimeters.iter().flat_map(|c| &c.entities)
        });
        for entity in entities {
            let paths = match entity {
                ExtrusionEntity::Loop(loop_) => {
                    loops += 1;
                    &loop_.extrusion_loop.paths
                }
                ExtrusionEntity::MultiPath(multi_path) => {
                    multipaths += 1;
                    &multi_path.paths
                }
            };
            count_loop(paths, &mut variable_widths, &mut walls);
        }
    }
    assert!(loops > 0, "the fixture must produce arachne wall loops");
    assert!(
        variable_widths > 0,
        "arachne wall widths must vary across sub-paths"
    );
    assert!(fill_regions > 0, "the islands keep infill boundaries");
    assert!(walls > 100);
}

#[test]
fn task22w18_no_classic_fallback_for_arachne_records() {
    // The classic chain must generate no classic surfaces for arachne
    // records: every perimeter entity comes from `process_arachne`.
    let prelude = prepare_post_classic_prelude(FIXTURE).unwrap();
    for object in &prelude.objects {
        let dispatches = object
            .object
            .as_parts()
            .1
            .iter()
            .flatten()
            .map(|record| record.dispatch)
            .collect::<Vec<_>>();
        assert!(
            dispatches
                .iter()
                .all(|dispatch| *dispatch == PerimeterDispatch::Arachne),
            "the fixture dispatches arachne on every populated layer"
        );
        for record in object.records.iter().flatten() {
            assert!(
                record.surfaces.is_empty(),
                "classic surface generation stays off for arachne records"
            );
        }
    }
}
