# Arachne traverse_extrusions conversion — integration seam 3

## Contract and status

Owning upstream: OrcaSlicer 2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`
(`src/libslic3r/PerimeterGenerator.cpp`, `src/libslic3r/VariableWidth.cpp`,
`src/libslic3r/Arachne/utils/ExtrusionLine.{hpp,cpp}`). This slice implements
deferred item 1 of the seam 2 plan
(`2026-09-09-arachne-entity-vocabulary.md`): the variable-width entity
conversion — `traverse_extrusions` (`PerimeterGenerator.cpp:370-574`, called
at `:2471` from `process_arachne`) turning ordered Arachne `ExtrusionLine`
walls into the seam 2 typed `ExtrusionEntity` vocabulary. Arachne wall
generation is NOT enabled here and still rejects with the typed
`SliceError::UnsupportedProjectFeature("wall_generator")`
(`project_slice/perimeters/preflight.rs`, proven unchanged by
`tests/perimeters/layer_region/predecessor.rs`).

## Upstream boundary

- `PerimeterGeneratorArachneExtrusion` (`PerimeterGenerator.cpp:363-367`)
  pairs one `Arachne::ExtrusionLine*` with the `is_contour` flag from the
  candidate traversal (`:2363`).
- The ordinary branch (`:520-525`) converts each line through
  `extrusion_paths_append(paths, *extrusion, role, flow)`
  (`Arachne/utils/ExtrusionLine.cpp:298-302`):
  `ExtrusionLine::to_thick_polyline` (`ExtrusionLine.hpp:205-224`, already
  ported on the Rust `ExtrusionLine`) then
  `thick_polyline_to_multi_path` (`VariableWidth.cpp:5-102`) with the
  `scaled<float>(0.05)` split tolerance and `SCALED_EPSILON` merge tolerance.
  Junction widths clip into constant-width sub-paths: width-ramp subdivision
  (`:27-56`), tiny-line splicing (`:16-24`), per-sub-path flow re-widthing
  `flow.with_width(unscale<float>(fmax(a_width, b_width)) + height *
  (1 - 0.25 π))` (`:69-75`) and merge-tolerance splitting (`:80-97`).
- Closed lines append `ExtrusionLoop(std::move(paths), is_contour ?
  elrDefault : elrHole)` oriented by
  `(wall_direction == CounterClockwise) == (is_contour || pg_extrusions.size() == 2)`
  (`:529-545`), keep the `ExtrusionEntity::inset_idx` base default -1, and
  the Orca thin-wall-hole entity order reverses for
  `!is_contour && pg_extrusions.size() == 2 && wall_sequence != OuterInner`
  (`:546-548`).
- Open lines append `ExtrusionMultiPath` entities that chain sub-paths
  end-to-start and split into a new entity on a discontinuity
  (`:546-566`).
- `steep_overhang_contour`/`steep_overhang_hole` are outputs the caller feeds
  to `reorient_perimeters` (`:2472`); the ordinary branch sets both when
  `overhang_reverse && layer_id % 2 == 1 && layer_id > raft_layers`
  (`:375-377`, `:520-524`).
- Deferred seams that fail closed here with typed
  `UnsupportedProjectFeature`: the Z-interpolating overhang clipping branch
  (`:391-519` — `clip_extrusion` ClipperLib_Z width interpolation, steep
  overhang detection, start-point rechaining; key `detect_overhang_wall`)
  and the junction fuzzifier `apply_fuzzy_skin` (`:387`,
  `FuzzySkin.cpp:685`; key `fuzzy_skin`). No classic fallback exists on
  either branch.

## Rust destination boundary

- `project_slice/perimeters/arachne/` (new module, declared in
  `perimeters.rs`): `traverse.rs` owns `traverse_extrusions`,
  `PerimeterGeneratorArachneExtrusion`, `TraverseExtrusionsContext` (the
  `PerimeterGenerator` fields the function reads) and
  `TraverseExtrusionsOutcome` (collection + steep flags);
  `variable_width.rs` owns the `extrusion_paths_append` /
  `thick_polyline_to_multi_path` port.
- Reuses the seam 2 vocabulary in place: `ExtrusionEntity::{Loop, MultiPath}`,
  `ExtrusionEntityCollection`, `OrderedExtrusionLoop`, `ExtrusionMultiPath`,
  `ExtrusionLoopRole`, `ExtrusionPath`/`Polyline3` and the classic
  `orient_loop` orientation helper (`entity_collections/orientation.rs`,
  visibility widened to `pub(in crate::project_slice)`; classic behavior
  unchanged).
- The classic gap-fill conversion (`classic/gap_extrusion/variable_width.rs`,
  the BBS `thick_polyline_to_extrusion_paths_2` port) is untouched: upstream
  keeps `VariableWidth.cpp:5-102` and `:105+` as separate algorithms with
  different grouping semantics.
- `crate::arachne::ExtrusionLine` is re-exported unconditionally (it already
  flowed through `wall_toolpaths::GeneratedWallToolPaths`).
- Tests: `project_slice/tests/perimeters/arachne/{traverse,variable_width}.rs`
  — loop roles/orientation/inset defaults, thin-wall-hole reversal and its
  OuterInner exception, open multi-path chaining, uniform/ramp/step/tiny-line
  width semantics with the source tolerance constants, per-inset flow
  selection, steep-flag layer parity, empty-line skipping, and both fail-closed
  branches.

## Included and deferred

Included: the conversion layer, vocabulary reuse, fail-closed deferred
branches, seam tests, plan doc. Classic algorithms, ordering and output bytes
are untouched (Loop-only classic path proven byte-identical below).

Deferred (updated from the seam 2 plan):

1. Z-interpolating overhang clipping + start-point rechaining
   (`PerimeterGenerator.cpp:391-519`) — needs the ClipperLib_Z width
   interpolation consumer over `geometry/clipper`'s `execute_z_paths`.
2. Junction fuzzifier (`FuzzySkin.cpp:685`).
3. Arachne config/surface/materialize transaction
   (`PerimeterGenerator.cpp:2093-2519`, `Arachne/WallToolPaths.cpp`) that
   dispatches into this conversion, replacing every deferred-seam marker
   reached by real multi-paths.
4. Constrained candidate traversal (`PerimeterGenerator.cpp:2269-2379`) that
   produces the ordered `PerimeterGeneratorArachneExtrusion` input.
5. `reorient_perimeters` (`PerimeterGenerator.cpp:2472-2475`) consuming the
   steep flags.
6. Strict public ordered-output comparison and four-platform execution.

## Validation

- `cargo nextest run -p ares-core`: 6861 run, 6861 passed, 3 skipped
  (inherited ignores; 16 new seam tests included).
- `cargo nextest run -p ares-cli --test logical_cursor_parity --test
  closest_line_extension --test avoid_crossing_travel`: 6/6 pass; anchor,
  two-wall and one-wall complete G-code artifacts stay byte-identical to the
  stored Orca references.
- `cargo fmt --all -- --check`: clean. `cargo clippy -p ares-core
  --all-targets`: warning set identical to the base commit `b05a97b8`.
  `cargo check -p ares-wasm --target wasm32-unknown-unknown` and
  `cargo check -p ares-cli --all-targets`: clean.
- Pre-existing, unchanged from the base commit: the full `ares-cli` suite
  reports 138 passed / 41 failed / 2 skipped both with and without this
  change (environment-dependent orca_parity preset and ksr cases); this seam's
  CLI suites pass.

A bounded green conversion slice does not complete the all-printer
domain/full-output parity goal.
