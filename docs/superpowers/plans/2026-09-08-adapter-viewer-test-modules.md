# Adapter/viewer test-module relocation

## Scope and source boundary

Layout-only refactor based on `79b70da235cef5446ae633bada7c6404b901db8b`.
Read AGENTS.md, Rust/TDD/diagnosing-bugs/conventional-commits skills, the
2026-09-07 structure audit, and ARD-0018/0020/0021/0023. The explicit separate
module requirement overrides the Rust skill's generic inline-test suggestion.
No functional red test is manufactured: establish existing discovery and passing
behavior first, relocate, then compare discovery, source bodies and results.

Upstream is read-only OrcaSlicer 2.4.2,
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Paths below are relative to its root.
All named production declarations and their existing citations remain unchanged.

| Owning upstream declarations | Existing Rust parent | Ordinary test destination |
| --- | --- | --- |
| `src/libvgcode/include/GCodeInputData.hpp::GCodeInputData`, `src/libvgcode/include/ColorPrint.hpp::ColorPrint` | `crates/ares-vgcode/src/input_data.rs` | `crates/ares-vgcode/src/input_data/tests.rs` |
| `src/libvgcode/src/Layers.hpp`, `src/libvgcode/src/Layers.cpp`: `Layers`, `Item`, update/lookup/reset | `crates/ares-vgcode/src/layers.rs` | `crates/ares-vgcode/src/layers/tests.rs` |
| `src/libvgcode/include/PathVertex.hpp`, `src/libvgcode/src/PathVertex.cpp`: `PathVertex`, defaults and helpers | `crates/ares-vgcode/src/path_vertex.rs` | `crates/ares-vgcode/src/path_vertex/tests.rs` |
| `src/libvgcode/src/Range.hpp`, `src/libvgcode/src/Range.cpp`, `src/libvgcode/src/ViewRange.hpp`, `src/libvgcode/src/ViewRange.cpp`: `Range`, `ViewRange` | `crates/ares-vgcode/src/range.rs` | `crates/ares-vgcode/src/range/tests.rs` |
| `src/libvgcode/include/Types.hpp`, `src/libvgcode/src/Types.cpp`: enum/data vocabulary, `move_type_to_option`, `lerp` | `crates/ares-vgcode/src/types.rs` | `crates/ares-vgcode/src/types/tests.rs` |
| Adapter around `ares-core`'s `libslic3r` rewrite: `src/libslic3r/Print.hpp`/`Print.cpp::Print`, `src/libslic3r/GCode.hpp`/`GCode.cpp::GCode`, `src/libslic3r/PrintConfig.hpp`/`PrintConfig.cpp` options; no upstream browser binding | `crates/ares-wasm/src/lib.rs` | `crates/ares-wasm/src/tests.rs` |

Included: move all six inline test-module bodies, preserving names, attributes,
assertions, imports and helper data; leave `#[cfg(test)] mod tests;` in each
parent. Only indentation/rustfmt layout may change. Existing viewer semantics
and native byte-adapter output/error assertions stay intact, including tests
whose names cite upstream ordering/defaults. No API, implementation, expectation,
feature, comparator, data fixture, or source-token-test deletion is authorized.
The viewer remains rendering-neutral. Browser byte bindings remain adapters to
`ares-core`, not an independent slicer pipeline. Existing STL slicing scaffolding
is retained unchanged as a temporary compatibility shell around the upstream
Print/config/GCode concepts; this relocation neither replaces nor endorses its
parity. No new pipeline, fallback, source-splitting include macro or OS hook.

Deferred: all additional upstream behavior, OpenGL/native viewer runtime, slicing
and G-code parity fixes, other core OS hooks, full ordered artifact comparisons,
all 1001 printer/default and legal option-domain coverage. The audit's remaining
35 inline test-body files (33 core source plus two core integration helpers) are
outside this writer's authority. Their audit paths, relative to `crates/ares-core/`,
are:

```text
src/arachne/wall_toolpaths/postprocess.rs
src/arachne/wall_toolpaths/stitch.rs
src/arachne/wall_toolpaths.rs
src/extrusion_entity.rs
src/fill/multiline/vline.rs
src/gap_fills/solid_surface.rs
src/gap_fills/wall.rs
src/gcode_adaptive_bed_mesh.rs
src/gcode_first_layer_print_placeholders.rs
src/gcode_line_numbers.rs
src/gcode_machine_limits.rs
src/gcode_object_labels.rs
src/gcode_role_fan.rs
src/gcode_spiral_vase.rs
src/gcode_thumbnails.rs
src/gcode_wipe_before_external_loop.rs
src/geometry/clipper/ordering/gcc.rs
src/model.rs
src/options/config_export/orca_block_keys.rs
src/print.rs
src/project/raw_settings.rs
src/project_slice/gcode_emit/footprint.rs
src/project_slice/gcode_emit/header.rs
src/project_slice/gcode_emit/motion/clip.rs
src/project_slice/gcode_emit/motion/format.rs
src/project_slice/gcode_emit/object.rs
src/project_slice/gcode_emit/value.rs
src/project_slice/path_simplification.rs
src/project_slice/seam_placement/alignment/context.rs
src/project_slice/seam_placement/sampling.rs
src/segments.rs
src/skirts/min_length.rs
src/surface.rs
tests/no_unapproved_dynamic_values/profile_shell/identity.rs
tests/no_unapproved_dynamic_values/profile_shell.rs
```

Aggregator-only modules elsewhere stay intact, including `ares-core/src/project.rs`.
No global test-layout or full parity completion is claimed.

## Execution and acceptance plan

1. Use a unique external target/artifact directory. Record cwd/ref/HEAD,
   toolchain/upstream identity and existing Nextest discovery for both crates;
   run all discovered tests before modifying Rust (expected 16 + 3, not assumed).
2. Relocate the six bodies to the exact destinations above. Compare production
   prefixes byte-for-byte, and test bodies against dedented, rustfmt-formatted
   originals, preserving literal helper bytes. Require all 12 Rust files <400
   physical lines and no inline test bodies in the six parents.
3. Run discovery and Nextest for both crates again; compare exact ordered names,
   counts and passing results. Run package fmt, workspace fmt check, package
   Clippy (including native tests), and both-crate WASM compile check. Keep full
   logs, real exits and test/build SHA-256 identities. Core dependency diagnostics
   are reported, not repaired in this scope.
4. Commit only the six parents, six children and this plan together after the
   bounded regressions pass. Create local candidate `coord/parity-testmods-w3`
   without force at the successful commit; no push/merge. Return full patch,
   clean status/no staged files and evidence for independent review.

Linux native tests and WASM compilation do not certify actual browser execution
or native Windows/macOS execution. No AppImage capture or full-output parity run
is needed to prove this layout-only relocation, and none is claimed.

## Validation record

Executed on Linux with rustc 1.96.0 (`ac68faa20`), Cargo 1.96.0 and Nextest
0.9.137. External evidence root: `/tmp/ares-adapter-testmods-6d93721f`;
all Cargo build commands set `CARGO_TARGET_DIR` to its `target` child. Full logs
and actual exit files are in its `logs` child. No shared target was used.

| Command | Exit | Evidence |
| --- | --- | --- |
| `cargo nextest list --locked --offline -p ares-vgcode -p ares-wasm --message-format json` (before and after) | 0 / 0 | `before-discovery.json`, `after-discovery.json` and corresponding stderr/exit files; raw JSON byte-identical, 16 + 3 tests |
| `cargo nextest run --locked --offline -p ares-vgcode -p ares-wasm` (before and after) | 0 / 0 | `before-tests.log`, `after-tests.log`: each 19 passed, zero skipped; each discovered test passed exactly once |
| `cargo fmt -p ares-vgcode -p ares-wasm` | 0 | `fmt.log` empty; extracted bodies remain exactly dedented originals |
| `cargo fmt --all --check` | 0 | `fmt-check.log` empty |
| `cargo clippy --locked --offline -p ares-vgcode -p ares-wasm --all-targets --all-features` | 0 | `clippy.log`: no adapter/viewer diagnostics; 108 warnings in unchanged core dependency |
| `cargo check --locked --offline -p ares-vgcode -p ares-wasm --target wasm32-unknown-unknown` | 0 | `wasm-check.log`: both crates check; nine existing core dependency warnings |
| `node /tmp/ares-adapter-testmods-6d93721f/verify-relocation.cjs` | 0 | `source-comparison.log`: six byte-identical production prefixes, six byte-identical dedented test bodies, exact 13-file scope |
| `node /tmp/ares-adapter-testmods-6d93721f/verify-discovery.cjs` | 0 | `discovery-comparison.log`: full raw JSON identical, same 19 individual passing results |

All twelve touched Rust files are below 400 physical lines: parents
33/125/96/131/186/57; children 19/156/47/35/73/32 in table order. Each parent has
only the ordinary test declaration, and no source-splitting macros were added.
No assertion, helper datum, test attribute or name changed; no test was added
or removed. `before-build-hashes.txt`, `after-build-hashes.txt` and
`wasm-check-build-hashes.txt` retain SHA-256 identities (the latter are compiler
metadata, not executable browser WASM captures).

Residuals: strict `-D warnings` and the pinned Rust 1.91 toolchain were not run;
core lint debt is not fixed here. Browser, Windows and macOS execution remain
unverified. This is a bounded layout candidate requiring independent review,
not certification of any full parity count or full ordered G-code artifacts.
