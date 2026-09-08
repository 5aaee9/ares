# Processor seam / time-cache association

## Source-owned bounded decision

Owner: OrcaSlicer 2.4.2 `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/GCode/GCodeProcessor.cpp::process_G1` (3819–4246) and
`TimeMachine::calculate_time` (480–574). A block records the next move-vertex
index before seam detection inserts a vertex. A closed external perimeter can
therefore associate the following moving block with a Seam instead of its
actual Travel/Wipe/Extrude vertex. Its time still counts; its normal-mode cache
entry does not. This is not a textual pre-WIPE rule.

Destination: `ares-core/src/project_slice/gcode_emit/processor/estimate.rs`
and a small source-owned seam-association child. Include reached planar G1
external-perimeter start/closure, overhang continuation, role changes, F-only
nonmovement, intervening nonmovement commands and first moving wipe/travel/
inward termination. Preserve existing motion arithmetic, counters, scheduler,
E-state and total-time accumulation. Replace the empirical adjacency exclusion;
no second heuristic or fallback. Tests use the existing processor byte-in/out
seam and public CLI project output, never private cache fields or source tokens.

Deferred: complete move-vertex/render data, arc seam discretization, scarf and
spiral-vase layer-based detector resets, E-only Wipe cache classification,
offsets/tool changes beyond these reached inputs, full timing parity, routing/fill/E-state/counter corrections,
all-printer/options coverage and Tier 1 runtime certification. Existing
estimator remains a temporary compatibility shell, not a new Ares pipeline.
No `libvgcode` or native runtime behavior is added. Existing E-only exclusion
is retained; only the reached moving seam association is changed.

## Evidence policy

Fresh actual AppImage output is authoritative. Only a separately validated
generator identity/timestamp line may be normalized. The wave5 wipe-off combo
instrument qualified only for wipe-off; rectilinear combo is disqualified and
must not supply timing values. Complete original/changed output and residual
diffs remain external. Logical-cursor partial, if needed, is applied only to an
identified external source copy and labelled integration evidence, never part
of the candidate. Red required regressions mean partial/no candidate.
