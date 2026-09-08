# Arachne extrusion entity vocabulary — integration seam 2

## Contract and status

Owning upstream: OrcaSlicer 2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`
(`src/libslic3r/ExtrusionEntity.hpp`, `PerimeterGenerator.cpp`). This slice
implements deferred item 1 of the seam 1 plan
(`2026-09-08-arachne-predecessor-envelope.md`): the typed extrusion entity
vocabulary so arachne's open variable-width multi-path entities can coexist
with classic ordered loops inside `ExtrusionEntityCollection`. Arachne wall
generation is NOT enabled here and still rejects with the typed
`SliceError::UnsupportedProjectFeature("wall_generator")`
(`project_slice/perimeters/preflight.rs`, proven unchanged by
`tests/perimeters/layer_region/predecessor.rs`).

## Upstream boundary

- `ExtrusionEntity.hpp` models `ExtrusionEntityCollection::entities` as
  `ExtrusionEntitiesPtr`, a heterogeneous vector of `ExtrusionEntity`
  subclasses; classic and arachne generators append into the same collection.
- `ExtrusionEntity` carries `int inset_idx = -1` ("Used for inner/outer/inner
  mode - classic perimeter generator"); classic loops set it from loop depth
  (`PerimeterGenerator.cpp:270` `eloop->inset_idx = loop.depth`).
- `ExtrusionMultiPath` (`ExtrusionEntity.hpp:377`) is a "Single continuous
  extrusion path, possibly with varying extrusion thickness, extrusion height
  or bridging / non bridging": `ExtrusionPaths paths`, `is_loop() == false`,
  `role()` from the first sub-path, `first_point()`/`last_point()` from the
  first/last sub-path endpoints. Variable width lives per sub-path
  (`ExtrusionPath::width`, `mm3_per_mm`, `height`).
- `traverse_extrusions` (`PerimeterGenerator.cpp:370-574`) appends
  `ExtrusionLoop` for closed arachne wall lines and builds `ExtrusionMultiPath`
  entities for open lines whose clipped constant-width sub-paths chain
  end-to-end, splitting into a new entity on a discontinuity
  (`PerimeterGenerator.cpp:553-566`).
- Downstream upstream owners that already handle both kinds uniformly:
  `LayerRegion::simplify_multi_path` (`LayerRegion.cpp:1089-1125`) simplifies
  each sub-path exactly like a loop sub-path;
  `SeamPlacer.cpp:405-438 extract_perimeter_polygons` scans sub-paths for the
  external role only for loops while multi-paths contribute through their
  entity role, and `ExtrusionEntity::collect_points` appends every multi-path
  sub-path point; `ExtrusionEntity::first_point` is the first sub-path first
  point for both kinds.
- `GCode::extrude_multi_path` (`GCode.cpp:6038-6080`, adaptive-PA averaging
  and wipe accumulation) is NOT ported here; perimeter emission stays loop
  only until the arachne materialization seam lands.

## Rust destination boundary

- `project_slice/perimeters/classic/entity_collections/types.rs`: new
  `ExtrusionEntity { Loop(OrderedExtrusionLoop), MultiPath(ExtrusionMultiPath) }`
  enum and `ExtrusionMultiPath { paths: Vec<ExtrusionPath> }`.
  `ExtrusionEntityCollection::entities` becomes `Vec<ExtrusionEntity>`.
  `ExtrusionEntity::inset_idx()` mirrors the upstream base default (-1 for
  multi-paths, loop depth for loops) and `first_point3()` mirrors
  `ExtrusionEntity::first_point`.
- `entity_collections/traverse.rs` wraps classic output in
  `ExtrusionEntity::Loop`; classic ordering, orientation and inset indices are
  untouched.
- Consumers with upstream-defined uniform behavior handle both kinds:
  `perimeter_append.rs::reorder_walls` through `inset_idx()`
  (`PerimeterGenerator.cpp:1484-1536`), `path_simplification.rs` sub-path
  simplification (`LayerRegion.cpp:1089-1125`), `seam_candidates.rs`
  extraction (`SeamPlacer.cpp:405-438`), `extrusion_islands.rs` first-point
  island assignment, `seam_placement/alignment/preparation.rs` external-width
  scan and per-entity candidate association over sub-paths.
- Consumers whose upstream behavior requires unported machinery keep classic
  behavior with an explicit `unreachable!` deferred-seam marker on the
  multi-path arm: G-code perimeter emission
  (`gcode_emit/motion/perimeter.rs`, `GCode.cpp:6038`), aligned-seam
  `place_loop` (`seam_placement.rs`, `SeamPlacer.cpp:1500` takes loops) and
  staggered inner-wall splitting (`seam_placement/runtime.rs`). No multi-path
  can reach them today because arachne dispatch is typed-rejected before
  materialization; the markers fail loudly instead of silently reusing classic
  loop behavior when the materialization seam (seam 3) lands.
- `motion.rs` crossed the 400-LOC limit with this change, so perimeter
  emission moved to `gcode_emit/motion/perimeter.rs` (346 + 69 lines).
- Seam tests: `project_slice/tests/perimeters/classic/entity_collections/
  multi_path.rs` — coexistence and source order, variable-width sub-path flow
  retention, inset default semantics, first-point semantics, and the
  `SeamPlacer.cpp` multi-path extraction arm.

## Included and deferred

Included: entity vocabulary + collection re-typing, uniform-behavior consumer
ports, deferred-seam markers, seam tests, plan doc. Classic algorithms,
ordering and output bytes are untouched (Loop-only classic path proven
byte-identical below).

Deferred (updated from the seam 1 plan):

1. Variable-width conversion proper: clipping arachne `ExtrusionLine`
   junction widths into constant-width sub-paths and chaining them
   (`PerimeterGenerator.cpp:397-523`) — the vocabulary now exists to receive
   it.
2. Arachne config/surface/materialize transaction
   (`PerimeterGenerator.cpp:2093-2519`, `Arachne/WallToolPaths.cpp`) to
   `project_slice/perimeters/arachne/`; the arachne envelope variant and the
   dispatch that selects it land here, replacing every deferred-seam marker
   reached by real multi-paths.
3. Constrained candidate traversal (`PerimeterGenerator.cpp:2269-2379`) plus
   the existing `getRegionOrder` component.
4. Envelope propagation into later prepare_infill predecessors, only as each
   stage gains arachne behavior.
5. Strict public ordered-output comparison and four-platform execution.

Known pre-existing debt observed, not changed: `seam_placement.rs` already
exceeded 400 LOC before this seam (532 at the base commit); splitting it stays
with its owning slice.

## Validation

- `cargo nextest run -p ares-core`: 6839 run, 6839 passed, 3 skipped
  (inherited ignores; 5 new seam tests included).
- `cargo nextest run -p ares-cli --test logical_cursor_parity --test
  closest_line_extension`: 5/5 pass; anchor, two-wall and one-wall complete
  G-code artifacts stay byte-identical to the stored Orca references.
- `cargo fmt --all -- --check`: clean. `cargo clippy -p ares-core
  --all-targets`: warning set identical to the base commit (only pre-existing
  line-shifted warnings). `cargo check -p ares-wasm --target
  wasm32-unknown-unknown` and `cargo check -p ares-cli --all-targets`: clean.

A bounded green structural slice does not complete the all-printer
domain/full-output parity goal.
