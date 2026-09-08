# Route boundary arithmetic (bounded wave10)

## Wave16 execution plan and authorized fan continuation

Base3c5cd20c plus the exact30-file wave14 patch (SHA256
8627519453f3720668131bf41e9158ee6f7ea4933a1f8d748ee2db0a40878654).
Fresh actual-AppImage references reproduced six complete-green contexts and
one-wall438/436 RED before edits. A captured empty-top-subtraction regression
first failed `None` versus `Some([])`. Pinned `AvoidCrossingPerimeters.cpp`
1099–1134,1259–1264,1285–1288 owns successful empty-boundary direct travel.
Rust builder outcomes distinguish unavailable/empty/ready; only successful
empty geometry returns zero interiors. Existing rectangle dispatch for other
outcomes and errors, cursor scaling, shortest direction and emission stay intact.

The route fix reaches436/436 exact ordered one-wall XY and removes all M73/time
differences, but fresh full artifacts still differ by one missing `M106 S255`
at line4950, layer65 outer-wall start following layer64 Internal Bridge.
Coordinator explicitly authorized a bounded fan-emission continuation, not an
unrequested expansion. Owner: pinned `GCode.cpp:6889–6911` role markers and
`GCode/CoolingBuffer.cpp:780,851–869,984–1006` deferred fan requests. END forces
emission even when the adjacent equal-speed overhang START is suppressed.
Rust destination: `gcode_emit/motion/fan.rs` carries the END emission request
through `gcode_emit/cooling.rs` until the actual layer baseline is known.
A separate source-named test must fail before the force-condition correction.
No timing, speed selection, fan mover, fixture or comparator changes are allowed.
Broader cooling architecture and other existing scaffold behavior are deferred.

Require all seven complete fresh artifacts and stable repeats after only the
validated generator line, effective exports, focused/core-full Nextest,
fmt/clippy/WASM compile, diff/LOC and empty index. Evidence belongs to this
worktree's `target/route-empty-evidence/`, with finite900s command receipts.
One code/tests/docs commit and absent local `coord/parity-route-w16` candidate
are allowed only on bounded success. Independent six-axis review, publication
and main merge belong to Coordinator. Full1001-printer/legal-domain/range and
Tier1 runtime acceptance remain open; this is a libslic3r rewrite slice with
no libvgcode change, not an independently designed pipeline.

## Wave16 outcome: bounded seven-artifact candidate

Both required corrections have source-named RED→GREEN tests. The empty-boundary
case first returned `None` instead of `Some([])`; after the fix its ordered
one-wall route is436/436. Six other complete outputs stayed unchanged; one-wall's
only residual was a missing `M106 S255` at line4950. The separately authorized
role-END force correction repairs that residual. No fixture, comparator, timing,
M73, fan speed, cursor-scaling or rectangle-shell implementation was changed.
Coordinator additionally approved deletion of the fan owner's two pre-existing
`ARES_DUMP_FAN` filesystem/environment diagnostic blocks. The final seven complete
outputs remain identical to pre-cleanup output except permitted generator lines.

Evidence D is `/home/indexyz/ares.pi-subagents-route-empty-boundary-fix-e418317-9ad0-s0-t0/target/route-empty-evidence/`.
All seven actual-AppImage runs exit0; the executable hash is
64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60.
Every final/repeat Ares execution exits0. All seven full outputs and ordered
XY sequences match, repeats are stable, and exercised settings match complete
effective exports. Counts: anchor770, two-walls623, translated770,
solid-monotonicline1251, wipe-off699, solid-rectilinear770, one-wall436.
Only the independently calendar/shape-validated generator line is normalized.

Receipts: `logs/03-baseline` exits1 (expected6/7 complete-green,438/436);
`04-tdd-red` exits100 (one required route failure); `08-green` exits1
(route-only436/436, missing fan); `11-fan-red` exits100; `13-fan-green`
exits0 (7 tests); `16-cli-focused` exits0 (6 tests); `17-core-full` exits0
(6828 passed/3 skipped,249.067s). The initial focused rebuild `06-core-focused`
exits101 because one test helper still called `unwrap` on the new build enum;
that missed test-only migration was repaired, and `10-core-focused` exits0
(71 tests, including Z1.6). `18-clippy`, `19-wasm`, `20-fmt-check` exit0.
`23-final-replay` proves7/7 full-green after diagnostic cleanup. Final exact-source
receipts24–30 all exit0: CLI6/6; core6828 passed/3 skipped (244.204s); clippy
(existing warnings only); WASM compile; fmt; source diff check; empty index.
The source audit has37 files, max398 physical Rust lines, no include macros,
no touched-core FS/env probes, unchanged CLI/fixture bytes, and an absent
candidate before creation. No skipped tests count as coverage.

Receipt31's audit exits1 on an overly strict input-archive identity assumption:
the inherited replay recipe reserializes only the anchor JSON settings (equal
parsed values); all other ZIP members are exact. One/two-wall input bytes are
exact. All three fresh reference outputs match preserved fixtures under the
same generator-only rule. Receipt32 validates this explicit provenance and
all seven full outputs/effective settings/cleanup neutrality, exits0. This input
provenance check does not normalize or tolerate any G-code difference.

`route-only/` preserves the honest intermediate fan residual and entire outputs;
`mutations/CASE/` preserves final/raw/repeated outputs, full diffs, requests,
effective exports and exits. `checkpoint.tar` preserves the exact restored
30-file source state. The final complete patch, changed-files/LOC/source audit
and hashes retain all inherited work plus this bounded correction. A local
`coord/parity-route-w16` candidate requires independent six-axis review; no push,
merge or publication is performed by this writer. Exhaustive1001-printer/domain/
range parity and Tier1 runtime acceptance remain open.

## Historical disposition: wave14 shortest-direction partial, no candidate

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

## Wave11 approved continuation plan

Restore the exact26-file patch on37159663, then reproduce the three wave10
boundary corrections and their unchanged captured-route tests. Fresh actual
AppImage references for all seven legal contexts must first reproduce six
complete-green artifacts and one-wall RED443/436. The two remaining authorized
production changes are confined to `router/path_shape.rs` and `router.rs`:
`AvoidCrossingPerimeters.cpp:335–346` casts the normal displacement before
integer addition; `to_polyline` (348–355) uses `Polyline.hpp:59–65::append(Point)`
to suppress exactly equal consecutive integer points. Do not deduplicate
formatted coordinates, change simplification, direction selection or any other
rounding. The anchor's distinct integer twins must still emit70 repeated
formatted corner pairs. Existing captured raw-route tests and strict public CLI
regressions are the red/green gates; no expectation or reference bytes change.

Require one-wall436/436 and complete generator-only equality for all seven
contexts, effective exports, full diffs and stable repeats, then full core
Nextest, focused CLI, fmt/clippy/WASM compile, diff/LOC and clean index.
Evidence is owned under `target/route-completion-evidence/` in the wave11 tree.
Only bounded success permits one code/tests/docs Conventional Commit and local
candidate `coord/parity-route-w11`; independent review and publication remain
Coordinator-only. The shortest-direction arithmetic discrepancy and all other
deferred behavior below remain unchanged, not implicitly accepted.

## Source-owned plan, recorded before production changes

Base: `37159663447569aaff248fab5c4a23b487bdfe77`. First restore the exact
26-file wave5 route partial (SHA256
`0ff4fcdad4a822a295ac77e800ac6f1744ad8ba0ca044d0b9c77f7d9202759ec`).
The unchanged public CLI one-wall ordered-XY and complete-artifact tests must
fail before implementation; anchor and two-wall tests must remain green.

Owner: readonly Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`libslic3r/Flow.hpp::scaled_spacing`,
`GCode/AvoidCrossingPerimeters.cpp::get_perimeter_spacing` (492–508),
`get_boundary(const Layer&, float)` (1099–1134), and
`avoid_perimeters_inner` (565–595). The task's 1604–1640 citation refers to
another overload in this readonly source; the actual travel overload calls
`union_ex(inner_offset(layer.lslices, 1.5 * perimeter_spacing))` at1105.
The qualified exact-recipe trace independently records spacing189955,
boundary4715068, retry284932.5, extension379910 and Backward direction.

Included Rust destination: `gcode_emit/motion/path/avoid_crossing` boundary
construction and router radius input. Scale the existing single-region flow
spacing once, retain the scaled value with the built boundary, and derive
boundary/top insets and distinct retry/extension radii from it. Use ordinary
union rather than the safety-offset union. Add separate behavior test modules
for the captured boundary and raw route; keep existing CLI assertions and
reference bytes unchanged. No shipping probes, deduplication, rounding,
simplification, comparator or tolerance changes.

Deferred: multi-region average/default-nozzle sourcing, external/support routing,
existing rectangle compatibility scaffold, independent processor/M73 defects,
exhaustive1001-printer/legal-value/range coverage and Tier1 runtime acceptance.
Existing Ares routing remains a temporary shell around the cited libslic3r
concepts, not an independently designed pipeline. No libvgcode behavior changes.

Validation: establish CLI RED, then check exact boundary/raw route and unchanged
CLI output; run all seven legal mutation contexts with full diffs if one-wall
is green (also retain the previously green translated/monotonicline contexts).
Run focused/full core Nextest, fmt, clippy and WASM compile using private targets
and finite900s commands. Preserve a complete partial patch and report, without
commit/candidate, if a required artifact stays red. Commit code/tests/docs
with a Conventional Commit and create local `coord/parity-route-w10` without
force only on verified success; publication and independent reviews belong
to the Coordinator.

## Outcome: partial preservation, no commit/candidate

Coordinator approved validation-only completion after the corrected CLI retained
443/436 ordered one-wall XY. No further fix or scope widening is included.

`Boundary` now stores the once-scaled f32 spacing; builder offset and top inset
use that scaled value, `inner_offset` uses ordinary union, and router computes
retry and extension radii independently. Separate `tests/boundary_parity.rs`
asserts the captured contour and two complete interior routes. The boundary
changed from±4715077 RED to±4715068 GREEN. Both route assertions stay red,
preserving the qualified values rather than changing expectations.

The temporary stderr-only instrument was separately approved, built and removed.
All production source files match pre-instrument snapshots byte-for-byte.
Instrumented/corrected CLI one-wall artifacts are each169991 bytes and equal
after only validated generator normalization. At actual Z1.6:

- request start(-3559422,4728540), end(4890001,4890001);
- spacing189955, boundary±4715068, retry284932.5, extension379910;
- retry start(-3559522,4715068), end(4715168,4715068);
- extensions: segment0 (-3559522,4715068), distance8274590, dnr=true;
  segment3 (4715068,4715068), distance37720544, dnr=true;
- Backward chosen; the boundary/intersection values agree with qualified upstream.

Therefore the corrections reach this code path, but do not repair one-wall
formatted output. The independent residual source owners are:

1. `AvoidCrossingPerimeters.cpp:335–346` casts normal displacement before
   integer addition; Rust `path_shape::{vertex_offset,middle_point_offset}`
   casts the sum4714997.2893218817, giving4714997 instead of4714998.
2. `to_polyline` (348–355) calls `Polyline.hpp:59–65::append(Point)`, removing
   only exact consecutive integer duplicates. Both simplifiers retain the
   false/true dnr twins; Rust directly collects them instead of using this
   upstream conversion behavior. No deduplication change was made.
3. `get_shortest_direction` (389–423) totals remain different: upstream
   forward21171364/backward-2311090, Rust11741228/-9430136 at this request.
   Both choose Backward here. No shortest-direction changes were made.

### Fresh actual-AppImage replay

All7 actual oracle invocations and21 current-tree Ares invocations exit0;
all six exercised keys match requested effective-export settings on both sides.
All corrected/repeated outputs are stable, and all corrected outputs equal
current-base restored outputs after generator-only normalization.

| Context | Ordered XY corrected/reference | Complete artifact |
|---|---:|---|
| Anchor |770/770|GREEN|
| Two walls |623/623|GREEN|
| Translation +13,-17mm |770/770|GREEN|
| Legal solid/top/bottom monotonicline |1251/1251|GREEN|
| Wipe off |699/699|GREEN|
| One wall |443/436|RED motion/M73/time|
| Legal solid/top/bottom rectilinear |770/770|GREEN|

Wipe-off and solid-rectilinear were already complete-green before these edits
on published37159663 plus the restored route partial. Their newer-base M73
corrections are not credited to this lane. Historical wave5 complete artifacts
therefore differ for those cases and one-wall; all historical comparisons are
diagnostic, never reference or coverage. Fresh AppImage outputs match all three
preserved reference fixtures except their validated generator timestamps.

### Evidence location and receipts

Owned worktree:
`/home/indexyz/ares.pi-subagents-route-boundary-fix-4fe56e9-51ca-s0-t0`.
Evidence subdirectory: `.route-boundary-evidence/`; commands have numbered
`.log` and actual `.exit` receipts. `mutations/CASE/` contains full project,
requested config, mutation description, actual-Orca/restored/corrected/repeat/
historical outputs, effective exports, full raw and XY diffs, logs and results.
`commands.json` records exact replay argv/exits; `identities.sha256` hashes all
replay data and executables. `route-boundary-full.patch`, `patch.sha256`,
`changed-files.txt`, `source-identities.sha256`, and `diff-summary.txt` preserve
the complete28-file partial. No staged files; HEAD remains37159663.

Initial CLI red and corrected/final CLI:4 pass/2 fail, exit100. Initial captured
core tests:0 pass/2 fail, exit100. Boundary green afterward; raw routes remain
red. Instrument build/run, source-restoration audit, validated-neutrality check,
fmt, clippy and final WASM compile exit0. Clippy retains existing warnings.
The first full-core run stopped early on the deliberate red tests; a subsequent
`--no-fail-fast` run is the full-suite receipt, not the early run:
6797 passed,2 captured-route failures,1 skipped, exit100 (351.425s).
Final focused run:60 passed,2 captured-route failures, exit100.
The final rebuilt production CLI is byte-identical to the corrected binary
used for all seven replays (SHA256
`5f815a4eebd9ed3af3a7a4e3239b63d00ed8cbbb7d236f913ff080a2239248be`).

Two diagnostic command problems are retained honestly: plain `python3` was not
on PATH (exit127; rerun via the installed absolute Python path); the replay
script's historical-equality assumption failed on the newer base (exit1 after
all7 contexts completed). An optional second trace lookup also failed after
the Z1.6 capture/neutrality check succeeded (exit1). Independent complete
validation in23-validation-summary exits0 and proves6/7 green, stable outputs,
reference identity and the requested Z1.6 values; it does not waive one-wall.

No1001-printer/default/legal-value/range completion, browserWASM/Windows/macOS/
Linux runtime matrix or independent six-axis acceptance is claimed. WASM is
compile-only. Rejected, skipped and unexecuted cases supply no coverage.

## Wave11 result: cast-only partial; suppression experiment reverted

Worktree `/home/indexyz/ares.pi-subagents-route-completion-8bfde2d-8d9b-s0-t0`,
HEAD37159663447569aaff248fab5c4a23b487bdfe77 throughout. Evidence D is its
`target/route-completion-evidence/`. Exact26-file patch applied first, then the
wave10 three production corrections and unchanged separate boundary/raw-route
tests copied from the preserved28-file tree. Fresh current-source CLI RED:
4 pass/2 fail. Full actual-AppImage seven-case replay established the required
six full-green contexts and unchanged one-wall443/436 before further edits.
Captured boundary tests:1 pass/2 raw-route failures, with4714997 twins.

Both authorized edits were then tried: displacement-first integer addition in
both `path_shape` offset functions, and integer-only `Vec<Point>::dedup` after
router simplification/collection. No formatted-coordinate filtering was used.
Unexpectedly the latter removed70 legitimate emitted points from each other
context, and29 from one-wall. All six public CLI tests failed. After escalation,
Coordinator ordered the dedup fully reverted while retaining the cast-order
repair. Final source audit proves `router.rs` is byte-identical to wave10;
only `router/path_shape.rs` differs among Rust files beyond wave10. No further
production changes were made. All inherited tests and fixture bytes are intact.

### Fresh references and every state transition

Actual AppImage SHA25664515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60
and readonly upstream8500fcdccaa10b5099ac20d252af3a7c560046f1 independently verified.
All7 oracle and35 replay Ares invocations exit0, using owned inputs, cache/HOME/XDG and
native targets. Each reference was generated afresh in this wave. Requested
six exercised keys match complete effective exports on both sides. Final
cast-only repeats are stable; the three committed reference fixtures match the
fresh oracle artifacts after only validated generator timestamp normalization.
The `green` filenames denote the failed intended-green experiment, not success.

| Context | Boundary baseline | Cast+dedup experiment | Final cast-only / Orca | Final complete artifact |
|---|---:|---:|---:|---|
| Anchor |770|700|770/770|GREEN, generator only|
| Two walls |623|553|623/623|GREEN, generator only|
| Translation +13,-17mm |770|700|770/770|GREEN, generator only|
| Solid/top/bottom monotonicline |1251|1181|1251/1251|GREEN, generator only|
| Wipe off |699|629|699/699|GREEN, generator only|
| Solid/top/bottom rectilinear |770|700|770/770|GREEN, generator only|
| One wall |443|414|443/436|RED motion/M73/time|

All final complete artifacts equal the initial boundary-state artifacts after
only generator normalization. Anchor retains140 exact formatted corner commands
(70 duplicate pairs). One-wall still has seven extra XY and six changed corner
commands; estimated total13m38s versus13m37s. First full normalized difference
is line485: expected `M73 P11 R12`, actual `G1 X164.715 Y179.715 Z.88`.
First ordered-XY difference is command82 at the Z1.6 approach/rampZ2: expected
`G1 X164.89 Y179.89`, actual extra `G1 X164.715 Y179.715`.
Every raw state-transition diff, reference diff, normalized diagnostic diff,
complete G-code, project, request, effective export, invocation log/exit and
result JSON remains in `D/mutations/CASE/`. Raw diffs exit1, including
complete-green cases whose only difference is the generator line.

### Corrected suppression interpretation and exact next source boundary

The proposed separate one-wall suppression site does **not** exist in the
active source. Both internal/external `travel_to` branches call
`avoid_perimeters` (691–700), which collects `avoid_perimeters_inner`'s result
and calls `to_polyline` (348–355) → `Polyline.hpp:59–65::append(Point)` for every
point. Diagnostic conversions664/676 are discarded; vector append686 preserves
travel records. The alternative1536 overload is inside `#if 0` beginning1340.
No source permits a selective dnr/extension condition. The initial claim that
this conversion alone could fix one-wall without harming the current Rust
anchor was disproved by complete-output tests, not waived.

The required next-wave question was: does the qualified upstream **FINAL** route
(not merely post-simplification) contain one or two4714998 corners? The existing
qualified exact-recipe trace (SHA256564affb9237ee57db3744dd2e08523f2bb57b66121391d047d259d519382fd5b)
answers it: line523 SIMPLIFY_POST contains two; line524 FINAL contains one:
`(-3559422,4728540),(-3559522,4714968),(4714998,4714998),(4890001,4890001)`.
The FINAL hook is immediately before `travel_to` returns. Thus this captured
merge precedes G-code writer emission, not `GCodeWriter::travel_to_xy` filtering.
The later diagnostic final-trace.log additionally records TO_POLYLINE_OUT with
one at651 and FINAL at652. Rust's final uninstrumented raw-route API regression
returns interiors `(-3559522,4714968),(4714998,4714998),(4714998,4714998)`.
That is current test output, not a newly instrumented actual-CLI integer trace.

Readonly audit found another concrete port difference needing qualification:
`find_first_different_vertex<false>` (299–315) initializes `line_idx` to
`(point_idx+1)%size` even for backward search; Rust `previous_different` starts
index-1. Upstream vertex normal passes prior/next indices, while Rust passes
middle and relies on helper scanning. Qualified one-wall Z0.62 FINAL152 has
**distinct** corners4714998 and4715068, both formatting identically; this
illustrates why route point identity must be established before conversion.
Next qualification must compare anchor and one-wall left/middle/right inputs,
normal displacement and final integer points through this shared conversion.
Do not invent a selective append condition. No helper, simplification,
shortest-direction or other rounding change is included. Coordinator approved
validation-only partial preservation, no commit/ref/candidate.

### Final validation and preserved patch

Each command uses `D/run.sh` with finite900s timeout and a real `.exit` receipt.
Native target `D/target`, clippy `D/target-clippy`, WASM `D/target-wasm` are owned.

| Receipt / command | Exit | Result |
|---|---:|---|
|01 CLI Nextest three integration targets |100|4 pass/2 fail before edits|
|02 fresh actual-AppImage references |0|7 executed|
|03 captured raw core RED |100|1 pass/2 fail|
|04 boundary baseline full replay |1|six green; one-wall443/436|
|05 `cargo fmt --all` |0|formatted|
|06 cast+dedup CLI |100|0 pass/6 fail|
|07 initial WASM compile |0|cast+dedup state, compile only|
|08 cast+dedup full replay |1|all7 red; repeated outputs stable|
|09 final cast-only CLI |100|4 pass/2 fail|
|10 final cast-only replay |1|six green; one-wall443/436|
|11 `cargo nextest run -p ares-core --no-fail-fast` |100|6798 pass/1 captured twin failure/1 skipped;246.247s test runtime|
|12 `cargo clippy -p ares-core -p ares-cli --all-targets` |0|existing warnings|
|13 final `cargo check -p ares-wasm --target wasm32-unknown-unknown` |0|compile only, existing warnings|
|14 `cargo fmt --all -- --check` |0|clean|
|15 `git diff --check` |0|clean|
|16 `git diff --cached --exit-code` |0|no staged files|
|17 final focused boundary/raw-route tests |100|2 pass/1 twin failure; captured retry now passes|
|18 final artifact/identity/repeat audit |0|six green/one red proven; no parity waiver|
|19 initial full patch checkpoint |0|28 files, reverse-apply0, max398 Rust LOC|

`D/route-completion-partial.patch`, `patch.sha256`, `changed-files.txt`,
`diff-summary.txt`, source/output identities and receipts preserve the28-file
partial. No include macros, shipping instrumentation, shared targets, upstream
edits, source-built reference, new fallback, commit, push, merge or publication.
The full1001-printer/default/Boolean/Enum/range producer goal remains incomplete;
WASM compilation is not browser runtime or Windows/macOS/Linux matrix coverage.
Skipped and unexecuted cases do not count. Independent six-axis review remains
required; the cast-only partial is not a bounded-success candidate.

## Wave14 shortest-direction execution plan (before production edits)

Restore checkpoint1177266b on37159663; independently reproduce six full-green
contexts and one-wall438/436 RED with fresh actual-AppImage references.
Owner: pinned8500fcdc `AvoidCrossingPerimeters.cpp:390–420::get_shortest_direction`
→ existing `router/path_shape.rs::shortest_direction_is_forward`. The wave13
qualified segment1→0 trace proves two mistranslated endpoint operands. Add
separate captured source-named tests and establish RED before rewriting the
four endpoint-distance subtractions in source order with float distances/norms
and strict `<`. Retain FFD/append, cursor conversion, timing and M73 unchanged.
The one-unit logical re-scaling drift is recorded, not fixed or accepted here.
Existing routing remains a temporary shell for libslic3r, not a new pipeline;
libvgcode does not own route selection.

Require one-wall436/436 and seven complete fresh artifacts equal except the
validated generator line, effective exports and stable repeats. Recheck Z1.6,
focused CLI, full core Nextest, fmt/clippy, WASM compile, diff/LOC and clean index.
Only verified bounded success allows a single code/tests/docs Conventional
Commit and absent local `coord/parity-route-w14` ref. Otherwise preserve the
complete partial and exact first divergence, asking before widening. Exhaustive
1001-printer/legal-domain/range and Tier1 runtime parity remain open; independent
six-axis review and publication belong to Coordinator. Evidence and finite900s
receipts are under this worktree's `target/route-shortest-evidence/`.
