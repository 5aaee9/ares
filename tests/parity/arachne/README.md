# First Arachne perimeter slice — executed red, NOT parity coverage

OrcaSlicer 2.4.2 upstream `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Runtime `/tmp/squashfs-root/bin/orca-slicer`, SHA256
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.

Selected actual defaults, without any option overrides:
- Machine: **Snapmaker A250 (0.4 nozzle)**.
- Its named process: **0.16 Optimal @Snapmaker (0.4 nozzle)**.
- Its inherited default filament: **Snapmaker PLA**.
- Source Arachne setter: `resources/profiles/Snapmaker/process/fdm_process_common.json:32`.

`prepare.mjs` recursively flattens the exact named inheritance chains and writes
three watertight, 2mm-high STL objects: a 20x16mm wide prism, a narrow-neck prism
(two 8x10mm lobes joined by a 4x0.6mm neck), and a 20x20mm prism with a 6x6mm
through-hole. Interior faces are omitted. All three objects (136 triangles)
were arranged and exported together by the actual AppImage, then sliced from
that exported 3MF. No config ZIP editing, classic override, fabricated reference,
or output normalization was applied to the stored files.

`effective.json` is the exact `Metadata/project_settings.config` archive entry.
It proves `wall_generator=arachne`, `wall_sequence=inner wall/outer wall`,
`precise_outer_wall=1`, `wall_loops=3`, `layer_height=0.16`, both one-wall
options false, `fuzzy_skin=none`, and `post_process=[]`. Machine, process and
filament identities above also occur in that effective export.
`capture.json` records argv, exits and SHA256 identities; `SHA256SUMS` checks
the input presets/meshes, effective config and output bytes.

The initial Prusa MK3S actual-default attempt exported but the AppImage refused
to slice its `post_process:[""]` (CLI -19, process exit237). It was NOT counted
as coverage or silently edited. Supervisor approved switching to this Snapmaker
selection. Its empty default post-process string exports naturally as `[]`.

## Regression

`cargo nextest run -p ares-core --test arachne_default_prisms`

The public test requests complete ordered G-code equality. On published source
`37159663447569aaff248fab5c4a23b487bdfe77` it executed red:
`ARES_ERROR: UnsupportedProjectFeature("wall_generator")` (nextest exit100).
It deliberately remains red in this partial patch, without `ignore` or an
expected-error assertion. The separate strict comparator independently validates
one generator identity/calendar timestamp per stream and compares every other
byte. Its mutation test rejects changed travel, time, config and statistics.
This partial work provides zero supported-printer or option-domain coverage.

Only the reusable `WallToolPaths::getRegionOrder` component was implemented;
public dispatch and classic algorithms were not changed. See the source-owned
plan for remaining materialization and integration seams. Native Linux tests and
WASM compilation are not four-platform execution proof.
