# Closest-line extension — bounded source rewrite

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

## Wave11 continuation: partial, no commit or candidate

On published37159663 the exact26-file wave5 patch was restored, then the three
wave10 boundary corrections reproduced. The unchanged CLI tests first produced
4 pass/2 fail; all seven freshly executed actual-AppImage references confirmed
six complete-green contexts and one-wall443/436. Wipe-off and solid-rectilinear
are now already full-green on this newer base, not gains attributed to routing.

The authorized displacement-first cast in both `path_shape` offsets is retained.
The attempted final `Vec<Point>::dedup` matched integer equality only but was
not safe for the current port: it removed70 legitimate emitted points in each
of the six other contexts and reduced one-wall to414/436. It was fully reverted
following Coordinator direction. The final cast-only artifacts again match the
six references completely, and one-wall remains443/436 with its original full
motion/M73/time failures. No assertion, fixture, simplification or shortest-path
arithmetic was changed; no guessed selective suppression was substituted.

Precise call-graph audit disproves a special append condition for one-wall:
all active routes call `to_polyline`→`Polyline::append(Point)`. Qualified upstream
Z1.6 FINAL has one4714998 corner despite two post-simplification twins; Rust's
final raw-route API test has two. The next source qualification boundary is the
upstream/Rust neighbor-selection and resulting integer points before this
common conversion, not a formatted-coordinate or writer-emission heuristic.
See `2026-09-08-route-boundary-parity.md` for the exact source discrepancy,
state transitions, first differences and validation receipts. Evidence:
`/home/indexyz/ares.pi-subagents-route-completion-8bfde2d-8d9b-s0-t0/target/route-completion-evidence/`.
Coordinator approved validation-only partial preservation. Full printer/option/
Tier1 runtime parity and independent six-axis acceptance remain incomplete.

## Plan recorded before route edits

Restore the exact 18-file logical-cursor partial patch on published 057f131b;
its original base was 79b70da2. Preserve that staged history, not a claim that
the earlier partial was accepted. Evidence root:
`/home/indexyz/.local/state/ares-parity/2026-09-07-coordinator/wave5-route-writer/`.

Owner: OrcaSlicer 2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`libslic3r/GCode/AvoidCrossingPerimeters.cpp::extend_for_closest_lines`,
particularly `get_closer` and its replacement versus append/prepend branches
(lines 219–294). Rust boundary: `gcode_emit/motion/path/avoid_crossing/router`
and cohesive private child modules. The current router remains a temporary
compatibility shell around this upstream owner, not an independent pipeline.

Included: compare the current intersection's float squared distance with float
radius squared, require a strictly closer same-contour candidate for replacement,
and otherwise retain the crossing while inserting the selected endpoint hit.
Remove shipping router debug probes and split its 686 physical lines below400.
Deferred and unchanged: logical cursor, shared endpoint-contour selection,
closest-line collection, simplification, offsets/Clipper, shortest paths,
processor/time estimation, option expectations and shared strict comparator.
No guessed rounding fix, generic duplicate filter, runtime hooks or fallback.

1. Restore unchanged anchor XY/full-artifact/two-wall/sparse-run regressions.
   Generate a fresh actual-AppImage one-wall reference with provenance; add
   separate public CLI ordered436-XY and complete-artifact tests. Demonstrate red
   before route changes using this tree's own exclusive-target build.
2. Apply only the source-owned extension correction and cohesive split. Require
   all436 one-wall XY commands and complete artifact to match the real reference
   with only ARD-0023 generator identity/timestamp normalization.
3. Replay seven legal contexts with complete raw diffs, effective settings,
   hashes and actual exits. Compare against restored partial and preserved base;
   illegal sparse monotonic values are not coverage. Wipe-off/rectilinear M73
   residuals remain explicit failures; ask Coordinator for bounded disposition
   if they alone remain unchanged, rather than automatically commit or waive.
4. Run CLI, affected/full core Nextest, fmt/clippy, WASM compile, diff/LOC and
   repeated-output checks. Preserve partial patch if red; only authorized bounded
   success permits code/tests/docs commit on coord/parity-route-w5. Independent
   review and publication belong to Coordinator; full producer parity remains red.

## Executed scope and disposition

Actual initial/final HEAD: `057f131b9ebb50538f865ffd770a89a1e79248dd`.
Managed branch remains `pi-subagents/route-extension-fix-9296c5b-a3b2-s0-t0`.
Restored patch SHA256:
`7d7c64e677a2fb35c80156466f5909fbda417cf1c6f5703cc44eddeed3bc7833`.
The earlier partial's logical-cursor/lifecycle fixes and four original CLI
regressions remain unchanged. No candidate branch or commit was created.

The minimal extension repair uses f32 coordinate-difference squared norms and
f32 radius squared, preserving strict closer-than comparison. The insertion
branch now retains existing crossings; nearby eligible crossings alone are
replaced. `router/closest_lines.rs` owns that operation; `router/path_shape.rs`
contains unchanged moved shortest-direction, offset and simplification bodies.
No shipping router probes remain and no new hooks were introduced. All touched
Rust is below400 physical lines (maximum398).

Both new one-wall CLI tests failed before production edits (411/436) and remain
red afterward (443/436). The Z0.62 ordered travel defect is repaired, not the
complete artifact. Remaining one-wall output has seven extra XY commands,
six changed corner commands, M73 differences, and estimated total13m38s versus
Orca13m37s. First remaining XY failure: Z1.6 outer-wall approach, ramp Z2,
extra `G1 X164.715 Y179.715`. No qualified exact request/candidate/intersection
capture exists for that remaining route; it is a read-only follow-up target.
No generic deduplication or timing correction was attempted.

Coordinator explicitly acknowledged validation-only completion and partial/no
candidate after this result. This is not the bounded-candidate/M73-only case:
one-wall motion itself is still red. Keep both failing tests and all reference
bytes intact in the preserved patch. Independent review remains required.

## Seven legal contexts, fresh actual-AppImage replay

All oracle and Ares invocations exit0. Restored outputs match the preserved
partial for all seven contexts; corrected repeated outputs are stable under
only the allowed generator normalization. Effective settings for all six
exercised keys equal the request on every output. Every full raw diff is retained
(including generator-only differences, raw diff exit1); normalized aggregate
full-artifact validation exits1, not success.

| Context | Corrected/reference XY | Complete artifact | Changed by extension repair |
|---|---:|---|---|
| Anchor |770/770|pass except generator|no|
| Two walls |623/623|pass except generator|no|
| Translation +13,-17mm |770/770|pass except generator|no|
| Legal solid/top/bottom monotonicline |1251/1251|pass except generator|no|
| Wipe off |699/699|FAIL M73 P61 R5 placement|no|
| One wall |443/436|FAIL motion, M73, total time|yes|
| Legal solid/top/bottom rectilinear |770/770|FAIL M73 P61 R5 placement|no|

Wipe-off places M73 before rather than after `G1 X161.752 Y170.871 Z7.04`.
Solid-rectilinear places it before the final wipe extrusion rather than after
`G1 Z7.04 F7200`. They are unchanged from the restored partial, not waived.
Full diffs against the earlier published-base executable also remain available.
That preserved executable is the independently built79b70da2 binary from wave3
(SHA256 f245856a6fd2fc318f9facc1a1642eb4648a49b610257ceca3d1b898521ef2ac),
not a fresh057f131b build.057f131b's intervening changes are test-body moves and
strict test comparison; current-tree restored/red and corrected executables
were freshly built in this wave's exclusive target. Baseline counts, outputs
and differences are diagnosis only, never substitute reference or coverage.

## Commands and evidence receipts

D is the evidence root above; native commands use `CARGO_TARGET_DIR=$D/target`,
WASM uses the separate `$D/target-wasm`. Each numbered log has an actual `.exit`.

| Command | Exit | Result/log |
|---|---:|---|
| `cargo nextest run -p ares-cli --test avoid_crossing_travel --test logical_cursor_parity --test closest_line_extension` restored |100|01-cli-red:4 pass/2 fail|
| Same after extension repair |100|03-cli-green:4 pass/2 fail (filename is intent, not success)|
| `python3 $D/replay.py` first attempt |1|04-replay:missing historical anchor baseline output; no coverage|
| Same corrected external evidence script |1|06-replay:all7 produced;3 complete-artifact failures|
| `cargo nextest run -p ares-core -E 'test(avoid_crossing) \| test(gcode_emit::motion)'` |0|05-core-focused:59 pass|
| `cargo nextest run -p ares-core` |0|07-core-full:6791 pass/1 skipped|
| `cargo check -p ares-wasm --target wasm32-unknown-unknown` |0|08-wasm:compile only|
| `cargo clippy -p ares-core -p ares-cli --all-targets` |0|09-clippy:existing warnings|
| `cargo fmt --all -- --check` |0|10-fmt-check|
| `git diff --check` |0|11-diff-check|
| `python3 $D/classify.py` |0|12-classify:M73 residual isolation only, not parity|
| `python3 $D/source_checks.py` |0|13-source-checks:LOC/probe/unchanged-test/move audit and full binary patch|
| `git apply --check --reverse $D/route-extension-full.patch` |0|14-patch-check|

Python executable:
`/nix/store/3n4qphl9s728sz8frmpqqrv9b1m87g68-python3-3.14.7/bin/python3`.
AppImage identity is recorded in `oracle-identities.sha256`; one-wall fixture
identity and initial fresh-reference command are in its README. Replay command
arguments/exits are in `commands.json`, per-case inputs/effective configs/full
outputs/raw diffs in `mutations/`, and all hashes in `identities.sha256`.
`changed-files.txt`, `diff-summary.txt` and `route-extension-full.patch` retain
the complete code/tests/docs/data partial. No files staged. No upstream edits,
shared targets, pushes, merges or publication. No browser runtime, Windows/macOS
runtime, full supported-printer/value/range sweep or all-producer parity claim.
