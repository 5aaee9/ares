# Arachne predecessor envelope — integration seam 1

## Contract and status

Owning upstream: OrcaSlicer 2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`
(`src/libslic3r/PerimeterGenerator.cpp`). This slice is the supervisor-approved
(wave9 handoff, seam 5) common post-perimeter predecessor boundary refactor on
top of `4255d6d5dda57e2c3fecfd6f73acd00c32419abe`. It is pure structural
preparation for the `PerimeterGenerator::process_arachne`
(`PerimeterGenerator.cpp:2093`) dispatch; arachne wall generation is NOT
enabled here and still rejects with the typed
`SliceError::UnsupportedProjectFeature("wall_generator")`.

## Upstream boundary

- Classic and arachne wall generation both end by appending perimeter entities
  and fill surfaces to the same `LayerRegion`-level state
  (`LayerRegion.cpp:82-142`): materialized perimeters, thin/gap fills, fill
  surfaces, fill expolygons and no-overlap regions are generator-agnostic.
- The prepared generator state that survives perimeter generation for
  downstream fill/seam/G-code stages is generator-specific: classic keeps the
  prepared classic traversal; `process_arachne` will keep arachne-generated
  state (`PerimeterGenerator.cpp:2097-2519`), which is not a classic hierarchy
  and must never be faked as one.

## Rust destination boundary

- `project_slice/perimeters/layer_region/types.rs` introduces
  `PostPerimeterPredecessor`, the minimal typed common envelope for classic and
  arachne post-perimeter results. `PreparedPostLayerRegionPerimeters` now owns
  the envelope instead of `Box<PreparedPostClassicTraversal>`. Only the
  `Classic(Box<PreparedPostClassicTraversal>)` variant exists today; an arachne
  variant joins when its materialization lands.
- Drills: `as_classic`, `as_classic_mut` (test-only) and `into_classic` are
  exhaustive single-arm matches. Adding the arachne variant later makes every
  drill a compile-time dispatch decision instead of a silent classic reuse.
- `prepare_infill/surface_type_detection.rs` (preflight, staging, forwarding,
  disposal) and `project_slice/test_consumers.rs` drill the envelope at this
  boundary; `PreparedPostSurfaceTypeDetection` and all later prepare_infill,
  seam-placement and G-code emission stages keep drilling the classic
  hierarchy unchanged.
- New seam tests live in
  `project_slice/tests/perimeters/layer_region/predecessor.rs`: envelope
  variant presence, drill identity (the boxed traversal allocation moves
  through `finish`/`prepare` without copying), and the unchanged typed arachne
  rejection at the boundary.

## Included and deferred

Included: envelope type + drills, boundary re-typing, drill updates, seam
tests, plan doc. Classic algorithms, ordering and output bytes are untouched.

Deferred (in dependency order, from the wave9 handoff):

1. Typed entity vocabulary and variable-width conversion
   (`PerimeterGenerator.cpp:370-574::traverse_extrusions` open/disconnected
   multi-path entities; classic `entity_collections/types.rs` stores loops
   only).
2. Supported-path clipping/rechain (`PerimeterGenerator.cpp:397-523`).
3. Arachne config/surface/materialize transaction
   (`PerimeterGenerator.cpp:2093-2519`, `Arachne/WallToolPaths.cpp`) to
   `project_slice/perimeters/arachne/`; the arachne envelope variant and the
   dispatch that selects it land here, mapping `process_arachne` orchestration
   without fabricating a classic hierarchy.
4. Constrained candidate traversal (`PerimeterGenerator.cpp:2269-2379`) plus
   the existing `getRegionOrder` component.
5. Envelope propagation into later prepare_infill predecessors, only as each
   stage gains arachne behavior.
6. Strict public ordered-output comparison and four-platform execution.

## Validation

- `cargo nextest run -p ares-core`: 6834 run, 6834 passed, 3 skipped
  (inherited ignores).
- Focused layer-region/surface-type suites and the three new seam tests pass.
- `cargo nextest run -p ares-cli --test logical_cursor_parity --test
  closest_line_extension`: 5/5 pass; anchor, two-wall and one-wall complete
  G-code artifacts stay byte-identical to the stored Orca references after
  validated generator-line normalization only.
- `cargo fmt --all -- --check`: clean. `cargo clippy -p ares-core
  --all-targets`: no warnings in changed files (existing workspace warnings
  unchanged). `cargo check -p ares-wasm --target wasm32-unknown-unknown`:
  clean.

A bounded green structural slice does not complete the all-printer
domain/full-output parity goal.
