# Logical G-code cursor parity — scoped implementation plan

## Latest disposition: wave14 shortest-direction partial, no candidate

The exact29-file FFD+append checkpoint1177266b was restored on37159663.
Fresh actual-AppImage replay first reproduced six complete-green artifacts and
one-wall438/436 RED. The source-owned `get_shortest_direction` rewrite now uses
stored distances as f32, float difference-vector norms, the four upstream
endpoint operands in order, and strict `<`. Captured tests first failed, then
passed; all six Z3.26–8.86 wrong corners are repaired. FFD/append, logical
re-scaling, timing, M73 and fixture/comparator bytes remain unchanged.

Required one-wall parity is still RED438/436: the first remaining XY difference
is command435 (1-based), final Z10.4 approach, expected direct
`G1 X164.89 Y179.89 Z10.4`, actual `G1 X155.285 Y171.735 Z10.4` followed by
`G1 X164.715 Y179.715` and `G1 X164.89 Y179.89`. Full output first differs at
line575 (`M73 P12 R11` expected, `M73 P12 R12` actual); later M73 placement and
missing `M106 S255` remain. No instrumentation was added: exact logical/physical
request, boundary and route decisions at Z10.4 need the next qualified trace,
not inference from formatted commands. The known one-unit cursor drift remains
separate and unmodified. Six other contexts remain completely green and all
seven corrected repeats are stable under generator-only normalization.

Coordinator approved validation-only partial preservation, no commit/ref.
Full core Nextest6806 passed/1 skipped; focused core69 passed including Z1.6
and three new direction tests; CLI4 passed/2 required failures. Fmt/clippy/WASM
compile exit0 (existing warnings). Evidence: this wave14 worktree's
`target/route-shortest-evidence/`, with full artifacts, effective exports,
raw/normalized diffs, command exits, hashes and complete partial patch. Full
1001-printer/legal-domain/range/Tier1 runtime parity and independent six-axis
review remain open. This bounded improvement is not completed route parity.

## Wave11 preservation receipt: partial, no candidate

Restored the exact26-file extension partial on37159663 and reproduced the three
wave10 boundary fixes before any further production change. Logical-cursor,
waypoint lifecycle, extension and all unchanged public CLI expectations remain
intact. Fresh actual-AppImage references prove all six non-one-wall contexts
complete-green on this base; one-wall starts and finishes443/436.

Only the source-owned displacement-first cast remains beyond wave10. Integer
consecutive deduplication at final route collection was tried and reverted after
it removed the anchor's70 required emitted duplicates (770→700); it made all
seven contexts red, including one-wall414/436. Final cast-only replay restores
all six full-green contexts and anchor140 formatted corner commands. Complete
final output equals the pre-edit boundary output in every context after only
validated generator-line normalization. No logical-cursor or writer behavior
was changed to mask the failed experiment.

The presumed selective upstream suppression site does not exist: the active
route call graph shares `to_polyline` and unconditional `Polyline::append(Point)`.
The qualified one-wall Z1.6 FINAL trace already has a single4714998 corner,
whereas the final Rust raw-route assertion shows two. Adjacent neighbor-selection
semantics need qualification before this common conversion can safely be ported;
no new conditions or geometry changes were authorized here. Coordinator approved
validation-only preservation with no commit/ref. Detailed source citations,
first differences, all state-transition artifacts and real-exit receipts are in
`2026-09-08-route-boundary-parity.md` and
`/home/indexyz/ares.pi-subagents-route-completion-8bfde2d-8d9b-s0-t0/target/route-completion-evidence/`.
This is not full1001-printer/legal-domain/Tier1 acceptance; review remains open.

## Boundary and contract

Pinned source: OrcaSlicer 2.4.2, `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
`libslic3r/GCode.cpp::_extrude` (7259) sets `m_last_pos` to the clipped
extrusion endpoint; `extrude_loop` (5993–6033) emits the inward move through
`GCodeWriter` without changing it; `extrude_infill` (6150–6168) chains using
that logical cursor. Rust destination is `gcode_emit/motion::{loop_paths,
local_cursor,state}`. Reuse existing scaled position only after auditing all
updates/consumers. Preserve physical writer XY and the emitted inward command.

The separate preserved correction ports `GCode/AvoidCrossingPerimeters.cpp`
`init_layer/travel_to/init_boundary` lifecycle and `GCode::travel_to` waypoint
emission. Apply its exact production patch only after reproducing the unchanged
public regression against this checkout's own build.

Included: generator-versus-writer position semantics, preserved route lifecycle
and waypoint retention, removal of shipping probes in touched files, cohesive
sub-400-line modules, CLI/project-bytes regression with complete real-AppImage
data. Existing emission organization remains a temporary compatibility shell
around these upstream concepts, not a new Ares pipeline.

Deferred: shortest-path/two-exchange algorithms, clipping/offset arithmetic,
processor/timing/feedrate defects, harness changes, broader printer/option parity.
No reversal heuristic, fallback, source-token assertions or expected-data weakening.

## TDD and validation

1. Copy only the unchanged public travel test and all its data from the saved
   patch; build current source in an exclusive target and establish 700 vs 770 red.
2. Apply the remaining minimal route correction separately; establish 770 with
   the six exact destination mismatches still red.
3. Audit scaled-state initialization/update consumers, correct the writer-only
   transition and chaining cursor, and retain the exact original XY assertion.
   Add a separate CLI test for complete ordered commands of all three affected
   sparse runs and observable inward movement.
4. Capture representative real-AppImage option mutations with full inputs,
   effective configuration, outputs, raw residual diffs and hashes. Unsupported
   cases remain explicit. Run focused CLI/core, full core nextest, fmt/clippy,
   wasm check and changed-Rust physical LOC/diff checks.
5. Commit code/tests/docs together and create `coord/parity-cursor-w3` only if
   the bounded correction and required regressions pass. Coordinator alone
   integrates/publishes; full active parity is not certified by this lane.

## Initial evidence

Worktree `/home/indexyz/ares.pi-subagents-logical-cursor-fix-e098f64-4254-s0-t0`,
branch `pi-subagents/logical-cursor-fix-e098f64-4254-s0-t0`, clean initial HEAD
`79b70da235cef5446ae633bada7c6404b901db8b`.
Read AGENTS.md, ARD-0023, O96 chaining ARD, Rust/TDD/diagnosis/commit skills,
complete sparse-destinations.json and both travel reports. Prior diagnosis
established strictly unequal endpoint distances; it did not build a corrected
executable. Runtime evidence for this run is isolated under
`/home/indexyz/.local/state/ares-parity/logical-cursor-e098f64/`.

## State audit and implemented components

- `EmitState` starts with `last_scaled_position=None`; gcode_emit.rs initializes
  the fixed model/extruder offset and takes initial writer XY from start G-code.
  Before the first generated path, `local_cursor` retains that existing behavior.
- `path::emit` clips scaled points before selecting `last_scaled`; constant,
  overhang-variable and scarf branches all save that endpoint after emission.
  This matches `GCode::_extrude`'s `set_last_pos(path.last_point())`.
- `start_travel` stores the target scaled point; `travel::retract_and_lift`
  stores each wipe endpoint, matching upstream Wipe::wipe (GCode.cpp:492).
  The fake wipe-before-external path stores the true perimeter start and then
  emits its pending return before extrusion; no new position state is added.
- Consumers: skirt seam targeting and start-travel identity already consume the
  scaled endpoint. Chaining/nearest seam now use that cursor rather than writer
  XY; inward movement updates only writer XY. `begin_layer` does not reset it.
- The route request now also starts at the generator cursor, as required by
  `GCode::travel_to` and `AvoidCrossingPerimeters::travel_to`. Physical writer XY
  remains separately available for emission. Existing physical-distance gates
  and floating roundtrip conversion are not globally rewritten in this lane.

Production components: exact preserved lazy boundary/endpoint-bound lifecycle
and retention of all interior waypoints; minimal inward/chaining cursor fix;
source-proved logical route-start correction after the two-wall red test.
Removed touched IORDER/PATH/boundary/start-travel shipping probes. No edits to
router.rs, shortest-path, two-exchange, clipping epsilon, processor or harness.
Removal of probes brings motion.rs below400 lines without another split.

## TDD and representative output results

All test builds use this worktree's exclusive external `target`. No inherited
executable was used for current-source red verification. Baseline and route-only
executables are separately retained.

| Case | Orca/Ares exits | Result after final route cursor fix |
|---|---|---|
| Unchanged anchor |0/0|770/770 exact XY; complete bytes match except generator|
| Two walls |0/0|623/623 exact XY; complete bytes match except generator|
| Translation +13,-17 mm |0/0|770/770 exact XY; complete bytes match except generator|
| Solid/top/bottom monotonicline |0/0|1251/1251 exact XY; complete bytes match except generator|
| Wipe off, no inward |0/0|699/699 exact XY; one M73 placement residual|
| One wall, no inward |0/0|411/436 XY;42 failing layers, full raw diff remains red|
| Solid/top/bottom rectilinear |0/0|770/770 exact XY; one M73 placement residual|
| Sparse monotonic / monotonicline |238/1 each|Both reject illegal values; not coverage|

Fresh AppImage command, for each case C under `$D/mutations`:
`ORCA_LIB_CACHE=$D/orca-libs.cache ORCA_APPDIR=/tmp/squashfs-root
/home/indexyz/ares/scripts/orca-parity.sh --slice 0 --outputdir $C/orca
$C/project.3mf`. Ares command: `$D/final-ares slice -o $C/final.gcode
$C/project.3mf`. Every invocation has its own stdout/stderr log and `.exit`.
Full G-code footer settings for both sides and requested configuration are
retained; effective values of all six exercised keys were asserted equal on
both sides to the requested values. No normalized-set acceptance was used.

## Validation receipt (external directory D above)

| Command | Exit | Evidence |
|---|---|---|
| `cargo nextest run -p ares-cli --test avoid_crossing_travel` before production |100|01-baseline-nextest.log,700/770|
| Same, recovered route only |100|02-route-nextest.log,770 and six destinations in02-six-destinations.log|
| Same, inward/chaining cursor |0|03-cursor-nextest.log|
| New complete-layer/full-artifact tests using route-only executable |100|04-sparse-red.log,both fail|
| Both CLI regression binaries |0|05-cli-green.log,3 pass|
| Two-wall full-artifact regression before logical route start |100|10-two-wall-red.log|
| Both CLI binaries after logical route start |0|13-route-cursor-green.log,4 pass|
| `cargo nextest run -p ares-core -E 'test(avoid_crossing) \| test(gcode_emit::motion)'` |0|19-final-core-focused.log,59 pass|
| `cargo nextest run -p ares-core` final source |0|20-final-core-full.log,6791 pass/1 skipped|
| `cargo fmt --all -- --check` |0|21-final-fmt.log|
| `cargo clippy -p ares-core -p ares-cli --all-targets` |0|17-clippy.log,existing warnings|
| `cargo check -p ares-wasm --target wasm32-unknown-unknown` exclusive target-wasm |0|16-wasm.log|
| Artifact/effective-config validation |1|22-artifact-validation.log; failures explicit above|

The initial external mutation generator failed (exit1) because Python zipfile
mutated reused ZipInfo offsets; corrected by copying each ZipInfo. Its incomplete
artifacts are preserved under `mutations-initial-failed`, not counted as cases.
Follow-up failed/missing input invocations are not successes. No hole fixture,
Windows/macOS runtime, browser runtime, all1001 profiles or full option sweep was
executed. Optional octagram replay was not pursued. No KSR timing claim.

## Disposition

Partial, no commit or candidate branch. Coordinator instructed withholding a
candidate while required mutations fail; one additional timeboxed first-divergence
diagnosis stopped at the identical one-wall raw path described in the ARD.
The correction demonstrably fixes the anchor and two-wall public regressions,
but does not certify the recovered route component across all required cases.
Complete binary-capable patch, source diff, hashes, status and unstaged receipt
are preserved externally for independent six-axis review and follow-up.

Final-source follow-up:23-final-cli.log exit0 (4 tests);24-final-clippy.log exit0
(existing warnings);25-final-wasm.log exit0. `final-source-ares` is the retained
final-source executable, with SHA256 in `final-source-ares.sha256`. Every legal
mutation was re-sliced with it (exit0 each), full raw differences retained as
`final-source-vs-orca.diff` (exit1 each).26-final-source-artifact-stability.log
exit0 verifies all seven outputs identical to the reported final artifacts
except the Ares timestamp. Diagnostics never substitute output.

## Wave5 continuation receipt (partial, no candidate)

The exact18-file partial above was restored on057f131b, preserving this history
and its original79b70da2 base. No logical-cursor or simplification changes were
made afterward. The separate closest-line source-owned correction is documented
in `2026-09-08-closest-line-extension.md`: current-intersection float radius test
and replacement versus endpoint insertion, with cohesive router split/probe
removal. Its new strict CLI tests remain red at443/436 one-wall XY; the original
four tests remain green. No one-wall complete-output success is claimed.

Fresh seven-case replay reproduces every preserved partial output before the
new route correction. After it, anchor/two-wall/translated/solid-monotonicline
full outputs remain exact; wipe-off and solid-rectilinear remain explicit,
unchanged M73 failures. One-wall changes but remains red. Every full raw diff,
effective config, executable/input/output hash and real exit is preserved in
`/home/indexyz/.local/state/ares-parity/2026-09-07-coordinator/wave5-route-writer/`.
Coordinator directed validation-only partial/no commit after this result.
This supersedes neither the original partial rejection nor ARD-0023 strictness.
