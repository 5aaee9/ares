# Avoid-crossing travel waypoint correction (scoped work plan)

## Latest disposition: wave17 independent-review P1 corrections

Base `c90d3598ba2b023d9fb3cde8be0b83cfe2954da7`; scope is only the two
independent route-review P1s. Upstream remains read-only Orca2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

- `Polyline.hpp:59–65::append(Point)` can return a singleton for equal integer
  endpoints. Rust `motion/path/avoid_crossing.rs::route` now saturates the
  interior count at zero, preserving append suppression and caller-owned
  endpoints. The square `(4.89,4.89)` same-point route outside the safe zone
  and the `start_travel.rs` equal-XY slope-Z caller both first panicked on
  subtraction overflow; both now pass, including exact emitted XYZ bytes.
- `GCode/CoolingBuffer.cpp:780,851–869` suppresses inactive conditional STARTs
  regardless of physical fan speed, while END still forces emission. Rust
  `gcode_emit/cooling.rs::resolve_role_fans` uses that emission condition alone;
  physical speed differing from baseline no longer activates an inactive START.
  A complete two-marker sequence with physical50/baseline100 first emitted two
  `M106 S255` commands; it now emits one. The motion marker test also explicitly
  sets internal-bridge speed50 and checks its preceding fixed-speed marker.

No fan speed selection, timing, M73, cursor arithmetic, router append behavior,
fixture, comparator, or rectangle-shell change. These are corrections to the
cited libslic3r rewrite boundaries, not new Ares pipeline behavior; libvgcode is
unchanged. Existing unavailable-geometry rectangle scaffolding is not expanded
or accepted as parity. External/support/multi-region routing and logical
re-scaling drift remain deferred.

Evidence D (owned, with full artifacts, raw/normalized/ordered-XY diffs, effective
exports, input/output identities, commands and real exits):
`/home/indexyz/ares.pi-subagents-route-review-fixes-0ebec5a-143e-s0-t0/target/route-review-evidence/`.
`D/replay.py` reuses the exact seven wave16 inputs and adds `internal-fan-50`,
changing only one-wall's `internal_bridge_fan_speed` to `["50"]`. All eight
references are fresh actual-AppImage executions, not copied/fake references.
Actual executable SHA256:
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.
Each reference and both candidate runs exit0. Complete artifacts and repeats
are byte-identical after only independently shape/calendar-validated generator
identity/timestamp normalization. Exact ordered XY counts: anchor770,
two-walls623, translated770, solid-monotonicline1251, wipe-off699,
solid-rectilinear770, one-wall436, internal-fan-50 436. Effective exports confirm
explicit internal fan50 in both producers. No tolerance or fallback reference.

Validation receipts in `D/logs/` (each command has a finite900s timeout):
`01-tdd-red` exit100,5 passed/3 expected failures; `02-tdd-green` exit0,24 passed;
`03-core-full` exit0,6831 passed/3 skipped,246.979s; `05-wasm` exit0,compile-only;
`06-fresh-references` exit0,8 actual Orca executions; `07-cli-focused` exit0,6/6;
`08-clippy` exit0,existing warnings; `09-full-replay` exit0,8/8 complete-green
and stable repeats; `10-fmt` and `11-fmt-check` exit0. Receipt04 failed exit127
because `python3` was absent from PATH; receipt06 reran with the explicit Nix
Python executable. `12-audit`, `13-diff-check` and `14-empty-index` exit0:
original seven input archives are exact; the new archive differs only in the
explicit fan setting; touched Rust modules are at most263 physical lines.
No failed/skipped/unexecuted case counts as coverage.

This bounded eight-artifact pass does **not** complete the full1001-printer,
default/legal-Boolean/Enum/range or Tier1 runtime goal. BrowserWASM is compile-
only; Windows/macOS runtime validation remains open. Independent review and
publication/main merge remain Coordinator-owned.

## Historical disposition: wave16 bounded seven-artifact parity

On published3c5cd20c plus the exact30-file shortest-direction partial, fresh
actual-AppImage output reproduced six complete-green contexts and one-wall
438/436 RED. Wave15's qualified trace established an empty internal boundary
after top-surface subtraction on the last layer's outer-wall approach (Z10.4
ramp, printed Z10.0/layer70). `AvoidCrossingPerimeters.cpp:1259–1264,1285–1288`
skips routing and returns `{start,end}` there. Rust now distinguishes successful
empty construction from unavailable geometry and returns zero interior points;
other failures still reach the unchanged temporary rectangle shell. The direct
route regression failed before the correction; unavailable-boundary and captured
Z1.6/twin/shortest-direction behavior remain covered and unchanged.

That correction achieved exact436/436 ordered XY and removed all M73/time
differences, leaving only a missing `M106 S255` at reference line4950. Coordinator
then authorized a small fan-owner correction. At layer65, Internal Bridge END
follows an equal-speed Outer wall overhang START. `CoolingBuffer.cpp:780,851–869,
984–1006` suppresses the START request but unconditionally emits on END. Rust
now preserves that END force through deferred conditional fan resolution without
changing speed selection. A separate source-named regression first failed.
Coordinator also authorized removing the touched cooling owner's two pre-existing
filesystem/environment diagnostic blocks; fresh full output remains byte-neutral.

All seven contexts now match complete fresh actual-Orca artifacts after only
independently validated generator identity/timestamp normalization. One-wall436,
anchor770, two-walls623, translated770, solid-monotonicline1251, wipe-off699 and
solid-rectilinear770 ordered XY commands match exactly, including repeated rounded
coordinates. Full outputs, repeats, raw/normalized/XY diffs, effective exports,
identities and actual command exits are in the wave16 worktree's
`target/route-empty-evidence/`. Fixture/reference/comparator bytes are unchanged.
Focused CLI6/6 and full core6828 passed/3 skipped; fmt/clippy and WASM compile
pass. The build-result enum intentionally stays inline to avoid an extra allocation.

This is a bounded libslic3r rewrite candidate, not completion of the user's
1001-printer/default/legal-Boolean/Enum/range matrix or Tier1 runtime goal.
BrowserWASM is compile-only; Windows/macOS runtime validation is not claimed.
Known logical re-scaling drift, external/support/multi-region routing and existing
rectangle compatibility scaffolding remain deferred. No libvgcode behavior changes.
Skipped/rejected/unexecuted cases count for no coverage. Independent six-axis
review and publication/main-merge approval remain Coordinator-only. Detailed plan
and receipts: `2026-09-08-route-boundary-parity.md`. Older dispositions below are
historical, not current failures.

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

## Historical disposition: wave11 cast-only partial, no candidate

The exact26-file route partial was restored on37159663, followed by the three
wave10 boundary fixes. Fresh actual-AppImage output reproduced six full-green
contexts and one-wall RED443/436 before the two authorized edits. Casting the
normal displacement before integer addition matches
`AvoidCrossingPerimeters.cpp:335–346` and is retained. Adding integer-equal
consecutive suppression at the final router collection was tested and **reverted**:
it reduced anchor770→700 and one-wall443→414, making all seven artifacts red.
No formatted-coordinate filter was used. The final cast-only state restores the
six green contexts and anchor's140 identical formatted corner commands, but
one-wall remains443/436. All final artifacts are unchanged from the restored
boundary state after only validated generator normalization.

The initial assumption that suppression could be applied here without affecting
anchor was disproved. There is no separate upstream one-wall append site:
active `travel_to` calls `avoid_perimeters` (691–700), then `to_polyline`
(348–355) and `Polyline.hpp:59–65::append(Point)` for every point. The alternative
overload is under `#if 0`. A selective dnr/extension condition would be invented.
The qualified Z1.6 trace has two4714998 corners after simplification but **one**
in `travel_to`'s FINAL result, so this particular discrepancy precedes writer
emission. The cast-only Rust raw-route test still returns two4714998 corners.
Readonly follow-up also found different neighbor selection in
`find_first_different_vertex<false>` (299–315): upstream initializes its scan
at index+1 even for backward search; Rust starts index-1. No neighbor-selection,
simplification, shortest-direction or other rounding change is included.

Coordinator approved validation-only preservation, not a commit/ref. Detailed
state-transition artifacts, source findings and receipts are in the wave11
section of `2026-09-08-route-boundary-parity.md` and the owned evidence directory
`/home/indexyz/ares.pi-subagents-route-completion-8bfde2d-8d9b-s0-t0/target/route-completion-evidence/`.
The full1001-printer/legal-value/range/Tier1 goal and independent review remain
open; bounded six-green output is not completion. Historical outcomes follow.

This wave is not full-output parity acceptance. The external seam is committed
project bytes → CLI G-code, checked against independently executed OrcaSlicer
2.4.2 AppImage output (upstream 8500fcdccaa10b5099ac20d252af3a7c560046f1).

Owner: `libslic3r/GCode/AvoidCrossingPerimeters.cpp`,
`AvoidCrossingPerimeters::init_layer`, `travel_to`, `init_boundary`, and
`MinDistanceVisitor`. Rust destination:
`ares-core::project_slice::gcode_emit::motion::path::{avoid_crossing,start_travel}`.

1. Preserve the anchor project and independently generate its reference output.
   Add a CLI regression comparing ordered XY travel commands (including repeated
   rounded coordinates), and prove red using the verified baseline executable.
2. Test the scout's ranked hypotheses individually: safe-zone-before-internal-
   boundary initialization with actual travel endpoint bounds; closest-segment
   deduplication; scaled spacing and ordinary union arithmetic. No hypothesis is
   established by old instrumented traces. Retain only verified corrections.
3. If raw waypoints recover, audit downstream near-point filtering against
   `GCode.cpp::travel_to`'s emission from travel point index 1. Remove shipping
   probes in touched modules and split cohesive modules below 400 physical lines.
4. Re-run anchor and representative option variants with fresh Orca outputs,
   focused nextest, fmt, clippy, diff and LOC checks. Record full raw residual
   differences separately; do not loosen the existing comparator.

Included: internal boundary lifecycle, endpoint bounds and the minimal routing /
waypoint emission correction established by output tests. Deferred: external
multi-object routing, support boundary expansion, broad EdgeGrid changes, timing,
M73/config/statistics parity and exhaustive preset/option coverage. Existing Rust
routing is a temporary compatibility shell for the cited upstream implementation,
not an independent pipeline. No rectangle fallback or hardcoded duplicate may be
introduced as a correction; unrelated scaffolding is not expanded.

The scout's earlier claims about numerical root cause and instrumented reference
builds are unverified. Acceptance requires fresh executable output evidence.

## Partial outcome (2026-09-08)

Fresh CLI red baseline: 700 ordered XY travel commands versus Orca770. Lazy
initialization using actual endpoint bounds restores a fifth raw waypoint on
all70 affected routes. Removing the two downstream near-point filters preserves
it in output: candidate770, with140 exact `G1 X164.715 Y179.715` commands versus
baseline70 and reference140. Diagnostic insertion audit establishes zero other
XY changes from baseline; it is not used to relax the committed comparator.

The unchanged exact all-XY regression remains red at six existing destination
commands. Three subsequent sparse-infill XY sequences (20 positions each) are
exact reversals, verified from full output bytes. Source owner for follow-up is
`GCode::extrude_infill` / `ExtrusionEntityCollection::chained_path_from`, not the
owned detour router. Coordinator explicitly stopped this lane at that boundary:
no commit or candidate branch until the combined required regression passes.
Closest-segment dedup, direction-formula and f32-distance experiments showed no
output benefit and were reverted; offset/union arithmetic was not changed.

Evidence directory:
`/home/indexyz/.local/state/ares-parity/travel-fix-322a3890/`.
`ordered-differences.log` records all six destination differences and the
insertion/reversal assertions. `candidate-vs-gt.diff` retains the full raw red
diff: final candidate and reference each7966 lines, with94 aligned differing
lines (not merely travel). Core nextest6791 passed/1 skipped; focused core2
passed; exact CLI nextest failed (exit100); fmt and changed-crate all-target
clippy exited0 with existing warnings. Broader preset/option replay and additional
reference variants are not run in this stopped partial lane. Core full-suite
validation preceded the final source-comment-only clarification; final CLI and
clippy were re-run afterward. Existing rectangle fallback and probes in the
untouched router remain explicit delivery risks, not accepted parity behavior.

## Logical-cursor continuation (2026-09-08, partial; no candidate)

The preserved route component was replayed separately against published base
`79b70da235cef5446ae633bada7c6404b901db8b`: current-source baseline red700/770,
route-only red770/770 at the same six destinations. `GCode::_extrude` sets
`m_last_pos` to the clipped path endpoint (GCode.cpp:7259), while
`extrude_loop`'s inward command updates only `GCodeWriter` (6032). Rust now
preserves its existing scaled generator cursor and uses it for chaining;
physical writer XY and the inward output remain intact. All770 ordered XY
commands and the complete anchor artifact now agree, except the allowed
ARD-0023 generator identity/timestamp line.

The first required two-wall mutation exposed an extra corner command introduced
by the recovered route component. `GCode::travel_to` builds its initial polyline
from `last_pos()` (7356–7362), and `AvoidCrossingPerimeters::travel_to` likewise
uses `gcodegen.last_pos()` (1239). Rust route planning still used writer XY.
At Z9.16, logical start (4890001,4860001) produces one (4715006,4715006)
interior corner; writer start (4851363,4879648) produces it twice. Using the
logical route start fixes the complete two-wall artifact; no duplicate-filter,
clipping epsilon, closest-line or shortest-path algorithm change was made.
This source-owned route-start correction is separate from the recovered route
lifecycle component and the infill-chaining cursor correction.

Required one-wall output remains red:411 versus436 ordered XY commands;
42 layers fail, all already failing on the exact published-base build, but some
already-failing routes change, so this is NOT evidence of zero new differences.
First missing waypoint at Z0.62 has the identical pre/post-simplification path
in baseline/route-only/final: [(3900880,-4151695),(4715006,4715006),
(4890001,4890001)], n=1. AppImage emits two corner commands. No inward move
is applicable with one wall. This missing vertex precedes the downstream
filters; exact upstream closest-line/intersection evidence is needed before a
router-algorithm correction. A one-unit logical-coordinate → millimetres →
scaled roundtrip also remains visible, not proven causal and not changed here.

Fresh anchor, two-wall, translated (+13,-17mm) and legal solid-monotonicline
cases match complete artifacts. Wipe-off and solid-rectilinear match all ordered
XY commands but each retains M73 placement differences. Illegal sparse monotonic
and monotonicline values are rejected by both executables and do not count as
coverage. Hole geometry was not available in this bounded fixture inventory.
No exhaustive printer/option/Tier1 runtime acceptance is claimed.

Full evidence: `/home/indexyz/.local/state/ares-parity/logical-cursor-e098f64/`;
source-state audit and commands: `docs/superpowers/plans/2026-09-08-logical-cursor-parity.md`.
All shipping probes in touched modules are removed. Untouched router/core probes
and existing rectangle scaffolding remain explicit risks. No candidate commit
is allowed with the unresolved required mutation failures.

## Closest-line continuation (2026-09-08, partial; no candidate)

Wave5 restored the exact18-file logical-cursor partial on published057f131b
(the partial's original base was79b70da2). The unchanged four CLI regressions
pass. A fresh actual-AppImage one-wall reference and two separate public CLI
regressions establish411/436 ordered XY and complete-artifact red before edits.

Owner `AvoidCrossingPerimeters.cpp::extend_for_closest_lines/get_closer`
(219–294) distinguishes replacement from endpoint insertion. Replacement now
requires the CURRENT crossing's float squared distance within radius squared
and a strictly closer same-contour candidate. Otherwise Rust retains the
crossing and prepends/appends the selected endpoint hit. No rounding,
simplification, shortest-path, logical-cursor or processor change is included.
Router shipping probes are removed; cohesive closest-lines and path-shape
children leave router275, closest-lines184 and path-shape176 physical lines.
Moved path geometry/simplification bodies are unchanged.

This correction is still insufficient: one-wall becomes443/436 ordered XY,
with seven extra commands and six changed corner commands. Z0.62 ordered XY
now agrees, but even its full layer retains M73 differences. The first remaining
XY failure is the Z1.6 approach (ramp Z2), an extra `G1 X164.715 Y179.715`.
Exact corrected request/candidate/intersection capture for that route is absent;
it is the next read-only diagnosis target, not grounds for guessed deduplication.
No new shipping instrumentation was added. The previous qualified trace proves
only the earlier Z0.62 endpoint-extension operation, not all remaining routes.

All seven fresh legal replays and all Ares invocations exit0; strict combined
artifact validation exits1. Anchor770, two-wall623, translated770 and legal
solid-monotonicline1251 match complete artifacts. Wipe-off699 and
solid-rectilinear770 retain their distinct M73 placement failures. Both are
byte-identical to the restored partial except the permitted generator line;
this route edit changes output only for one-wall among these seven cases.
Restored outputs match the preserved partial in all seven cases; repeat outputs
are stable. Full diffs against actual Orca, restored partial and preserved
published-base executable, effective settings and hashes are retained. Illegal
sparse monotonic values are not counted or rerun as coverage.

Coordinator explicitly directed validation-only completion and a partial patch:
no commit or candidate ref. No required assertion was weakened. Full-core
Nextest6791 pass/1 skipped; focused59 pass; CLI4 pass/2 fail; fmt/clippy/WASM
compile checks exit0 (existing warnings). This is not browser/native-platform
runtime acceptance, exhaustive printer/option coverage or producer parity.
Evidence and detailed receipts: `2026-09-08-closest-line-extension.md` and
`/home/indexyz/.local/state/ares-parity/2026-09-07-coordinator/wave5-route-writer/`.

## Scaled-boundary continuation (wave10, partial; no candidate)

On published37159663 plus the exact26-file wave5 partial, the unchanged CLI
seam remains red443/436 before and after the three captured corrections.
`Flow::scaled_spacing` truncates spacing before the float conversion in
`get_perimeter_spacing`; Rust now retains scaled spacing189955 with the
boundary and derives its inset284932.5, retry284932.5 and extension379910
without unscaling/rescaling. `get_boundary(layer,float)` at1099–1134 uses
plain `union_ex`, not the safety-offset union. The boundary unit now matches
all four±4715068 corners exactly (red±4715077 before correction).

A Coordinator-approved, temporary stderr-only instrument of the corrected CLI
proves those values reach the actual Z1.6 request. The complete instrumented
artifact equals the corrected production artifact after independently validated
generator-line normalization. Sources were restored byte-for-byte afterward;
no shipping probe, filesystem/environment hook or diagnostic branch remains.
The three boundary defects are repaired but were not sufficient to change
formatted one-wall output. All seven corrected artifacts equal the restored
artifacts under the same strict generator-only rule.

The captured Z1.6 intersections match upstream, and both select Backward.
The remaining raw corner is4714997 in Rust versus4714998 upstream:
`get_polygon_vertex_offset/get_middle_point_offset` (upstream335–346)
cast the displacement before adding the integer point; Rust
`router/path_shape.rs::{vertex_offset,middle_point_offset}` casts the sum.
This is a diagnosed next seam, not an authorized rounding correction here.

Both implementations preserve two identical corners with do-not-remove
false/true through `simplify_travel`. The qualified upstream FINAL contains
only one because `to_polyline` calls `Polyline::append(const Point&)`
(`Polyline.hpp:59–65`), which rejects an exactly equal integer predecessor.
Rust `router::avoid_perimeters` directly collects both. This is not evidence
for formatted-coordinate deduplication or simplification changes: neither was
made. Shortest-direction totals also differ (Rust forward11741228,
backward-9430136; upstream21171364,-2311090), despite the same direction;
that independent arithmetic remains deferred.

Fresh actual-AppImage replay now has six complete-artifact passes: anchor770,
two-walls623, translated770, solid-monotonicline1251, wipe-off699 and
solid-rectilinear770. The latter two were already green in the restored partial
on this newer published base; this lane must not claim credit for those M73
fixes. Only one-wall remains red443/436, with motion/M73/time differences.
Three new separate boundary/route behavior tests preserve the qualified exact
expectations; the boundary is green and both raw-route assertions remain red.
Coordinator directed partial preservation without commit/candidate. Full
printer/option/range coverage, Tier1 runtimes and independent acceptance remain
open. Plan and validation receipts: `2026-09-08-route-boundary-parity.md`.
