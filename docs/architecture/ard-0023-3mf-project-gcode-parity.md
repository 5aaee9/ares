# ARD-0023: 3MF project-to-G-code parity boundary

## Status

Accepted

## Context

Ares must slice `tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf` into the
reference `ksr_fdmtest_v4.gcode`. Except for the generator timestamp line, the
result must be byte-for-byte identical. The fixture is a Bambu/Orca project
archive containing 6,109 vertices, 12,234 triangles, one printable instance,
and 653 project settings. Its reference output contains 460 layers and 269,330
lines.

The existing Ares 3MF branch only recognizes the ZIP signature and constructs
an empty model. The existing STL-oriented slicing pipeline was introduced as an
early compatibility scaffold. It approximates layer planning, polygon work,
perimeters, infill, ordering, and G-code generation and therefore cannot be
extended with fixture-specific adjustments to satisfy exact Orca parity.

Before the Option pinning cleanup, the repository contained thousands of
`PrintConfig` source-line modules and tests. They recorded individual
OrcaSlicer source lines, neighboring boundaries, or deferred behavior without
implementing runtime behavior, increasing maintenance and build cost without
providing slicing parity.

## Upstream baseline

All behavior for this parity target is derived from OrcaSlicer `v2.4.2`, commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Later upstream `main` behavior is
not silently substituted.

The owning upstream boundaries include:

- `src/libslic3r/Format/bbs_3mf.hpp::load_bbs_3mf` and
  `src/libslic3r/Format/bbs_3mf.cpp::_BBS_3MF_Importer` for archive, model,
  configuration, component, instance, and transform loading.
- `src/libslic3r/Config.*`, `PrintConfig.*`, and `PrintApply.cpp` for typed
  option deserialization, legacy handling, extruder/filament sizing, and FDM
  normalization.
- `src/libslic3r/Model.*`, `TriangleMesh.*`, `TriangleMeshSlicer.*`,
  `PrintObjectSlice.cpp`, `Layer.*`, and `Surface.*` for model and layer
  geometry.
- `src/libslic3r/PerimeterGenerator.*`, `Fill/*`, `ClipperUtils.*`,
  `PrintObject.cpp`, and `Print.cpp` for classic walls, surfaces, infill, brim,
  path ordering, and the print state machine.
- `src/libslic3r/GCode.*`, `GCodeWriter.*`, `PlaceholderParser.*`, and
  `GCode/GCodeProcessor.*` for command emission, custom G-code, configuration
  blocks, statistics, and time estimation.

## Decision

### Project input

`ares-core` will gain an in-memory Bambu/Orca 3MF project loader. It will parse
the ZIP package, XML models, nested components, build items, object/volume
metadata, plate metadata, and embedded project configuration without direct
filesystem access. The loader will preserve Orca's transform composition and
coordinate conversion semantics.

The archive is an untrusted public input boundary. Entry counts, individual and
total expanded sizes, compression ratios, XML depth/attributes/text, and JSON
document sizes are bounded before allocation. OPC relationship targets are
normalized in package space; traversal, ambiguous encodings, duplicate
normalized entries, encrypted entries, DTDs, and external/general entity
declarations are rejected.

For 3MF projects, embedded configuration is authoritative. External JSON may
not override project options. The existing explicit `SliceOptions` API remains
available for STL callers, while project slicing uses a separate entry point so
an ignored or ambiguous override cannot occur.

### Configuration

Configuration will be represented by concrete serde structs grouped by the
owning Orca config class, with concrete field types and enums for every known
option. Known structures and options may not use `serde_json::Value`, an erased
value map, generic `ConfigValue`, or runtime type inspection. Tests may use
dynamic values. A production field whose upstream schema is genuinely open may
use a dynamic payload only after a source-cited review records the containing
struct and field in an explicit allowlist; such a payload may not select option
types or slicing behavior. Unknown project option keys are rejected until a
reviewed option slice adds a typed field or approved open-field exception. The
port will implement Orca's deserialization, legacy normalization, active
extruder/filament selection, vector resizing, and FDM normalization before
runtime behavior or config-header export consumes a typed field.

Options are implemented one behavior at a time. Every behavioral port requires
a focused test tied to its owning Orca source path. Merely copying a serialized
string into an untyped container does not count as implementing an option.

### Geometry and G-code

The exact fixture path will replace, rather than grow, the early Ares
approximation. Rust module organization may differ from C++, but coordinate
rounding, polygon semantics, path ordering, extrusion calculation, formatting,
custom G-code expansion, and post-processing must preserve observable Orca
behavior.

The current `pipeline`, floating-point `segments`/`contours`, simplified
perimeter/infill generators, and current G-code formatter are temporary
compatibility shells. Each is removed or narrowed when its upstream-owned
replacement becomes active. No second fallback path remains after replacement.

### Golden comparison

The reference G-code is test data only. Production code must not read it,
identify the fixture filename, branch on model hashes, or invoke/link an Orca
binary.

The golden comparator permits exactly one difference:

- reference: `; generated by OrcaSlicer 2.4.2 on <timestamp>`
- Ares: `; generated by Ares 2.4.2 on <timestamp>`

Each side must independently match its required line shape. The comparator then
normalizes that complete line and compares every remaining byte. Model/total
time estimates, filament statistics, config serialization, whitespace, command
ordering, and final newlines are not normalized.

Amendment (2026-09-12, live-oracle printer sweep): some vendor presets embed
an additional branded generator line in `machine_start_gcode` (e.g. the
Creality K2 family's `; generated by Creality_Print V7.0.1.4212 on <date> at
<time>`, which exists to suppress a printer-side warning). For these lines
only, the comparator additionally normalizes the trailing local date/time,
preserving the brand and version bytes exactly; any brand/version, spacing or
structural difference still fails. This is the same producer-local-time
identity allowance as the canonical line, not a general header tolerance.

### Source pinning cleanup

Tests whose only subject is an Orca source line, raw token text, neighboring
milestone boundary, or a declaration that explicitly implements no runtime
behavior will be removed. Private production modules that exist only to satisfy
those tests will be removed with them. Runtime option definitions, behavioral
tests, concise source citations, and architecture decisions remain.

Pinning cleanup is based on reachability and test intent, not filename alone.
It must not remove code used by project loading, configuration normalization,
slicing, G-code generation, or behavioral tests.

The active roadmap makes this parity program the next development chain. The
superseded `PrintConfig` one-source-line modules, tests, and milestone documents
are removed together; behavior/architecture documents and the M852 crate
partition checkpoint remain. Staged `PrintApply` cleanup is tracked separately.

### Portability and ownership

The four-crate boundary from ARD-0022 remains unchanged:

- `ares-core` owns the in-memory project, configuration, slicing, and G-code
  implementation.
- `ares-cli` owns filesystem and current-time acquisition for native use.
- `ares-wasm` exposes the same byte-oriented project API to browsers.
- `ares-vgcode` remains rendering-neutral and is not required by this target.

Dependencies used for archive, XML, or geometry work must support Windows,
macOS, Linux, and `wasm32-unknown-unknown` without native filesystem or C/C++
runtime requirements.

The two parity fixtures are committed directly to normal Git history (not LFS)
so fresh clones and CI receive identical bytes. Their SHA-256 identities are
part of the reviewed spec.

## Consequences

- The parity target is a sequence of independently tested upstream rewrite
  slices, but it is not complete until the full normalized golden comparison
  passes.
- Approximate existing tests may be replaced when they assert behavior that
  conflicts with Orca v2.4.2; unrelated public behavior is not refactored.
- Source citations move from one-module-per-source-line pinning to focused
  specs, plans, behavioral tests, and concise runtime metadata where useful.
- Exact time estimation and post-processing are part of the required output,
  not optional metadata work.

## Rejected alternatives

- Patch the current scaffold until this fixture passes | Its geometry and
  ordering abstractions do not preserve Orca semantics and would encourage
  fixture-specific branches.
- Read or copy the reference G-code at runtime | This is hardcoding and cannot
  generalize to future projects/options.
- Invoke or link OrcaSlicer | This is a legacy fallback, violates the Rust/WASM
  boundary, and does not implement Ares behavior.
- Accept semantic rather than byte parity | The explicit test contract requires
  complete output equality except for the generator timestamp line.
