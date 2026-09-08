# Bounded FloatOrPercent harness correction

## Source-owned included/deferred plan (before implementation)

Base 057f131b9ebb50538f865ffd770a89a1e79248dd; read-only upstream
8500fcdccaa10b5099ac20d252af3a7c560046f1. Test-only destination:
`ares-cli/tests/orca_parity/option_coverage{.rs,/}` and `runner{.rs,/}`.
No producer, core, libvgcode, strict comparator or fixture changes.
ARD-0023 generator-only ordered-byte equality remains mandatory.

1. Reproduce current emitted plans omitting literal widths and current aggregation
   accepting partially rejected requirements. Keep Boolean/Enum behavioral tests.
2. For `line_width` and `bridge_line_width` only, emit separate mm/percent plans:
   raw schema bounds, contextual bounds, GUI missing-percent threshold, zero
   sentinel, dependencies, deterministic decimal interiors, and separately
   classified negative/context probes. Do not execute extreme schema maxima.
   Definitions: PrintConfig.cpp:1339–1352,2410–2420. `max_literal=10` is GUI
   metadata (Config.hpp::ConfigOptionDef, GUI/Field.cpp), NOT a legality cap.
3. Explicit single-nozzle 0.4/layer 0.2 context only. Print.cpp::validate
   (1543–1556,1663–1689) admits zero or positive widths above layer height,
   through 5*nozzle for line width; bridge through nozzle, with positive lower
   boundary zero only when BOTH thick flags are true. PrintConfig.cpp:10487–10545
   supplies bridge/nozzle and raw numeric validation. Config.cpp:321–336 uses
   approximate endpoint acceptance (default precision 4), rejects negatives
   when min=0, and permits NaN only for nullable options. Do not claim exact
   real-number raw boundaries or probe the 1e-4 tolerance edge here.
   Flow.cpp::extrusion_width/new_from_config_width distinguishes line literal
   zero auto from zero percent; bridge zero resolves solid-infill dependency.
   Validation candidates are NOT successful Flow/slicing coverage.
4. Replace blanket build-error rejection with typed stage evidence, concurrent
   pipe drainage and terminate/reap timeout handling. Unknown/process/I/O failures
   fail closed. Recognize only source-attributable normal validation exits
   (OrcaSlicer.cpp CLI validation and Utils.hpp -18/-51); preserve full commands,
   status, stdout/stderr. Count comparison only after strict comparator execution;
   reject baseline-only, rejected and unexecuted requirements as domain completion.
5. Verify typed magnitude AND percent tag from exported
   Metadata/project_settings.config (bbs_3mf.cpp::_add_project_config_file_to_archive,
   ConfigOptionFloatOrPercent); process ownership from Preset.cpp::s_Preset_print_options.
   At most two fresh actual-AppImage cases: line_width=0.6 and 150%, explicitly
   exported nozzle/layer/dependencies. Save presets, input, commands and hashes.
   Keep actual strict producer failure RED; application success is separate.
6. Focused Nextest red/green, fmt, clippy, strict-gate regressions, physical LOC
   below 400 per changed Rust file, separate tests. External evidence root:
   `/home/indexyz/.local/state/ares-parity/2026-09-07-coordinator/wave6-domains-writer/`.
   Local candidate only if required bounded regressions pass, otherwise preserve
   binary-capable patch and report partial. No publication.

Deferred: remaining options and unbounded/mixed-nozzle generation; every printer's
actual defaults; object/region overrides and bridge activation; full tolerance-edge
semantics; zero-percent runtime feasibility; producer/full ordered artifact parity;
Tier1 browser WASM/Windows/macOS execution. Existing smoke normalization is a
compatibility harness, never actual-default proof. No full sweep in this vertical.

## Execution and evidence

Let `E` be the external evidence root above. Compiler: rustc 1.96.0
(`ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96`, x86_64-unknown-linux-gnu).
All Cargo commands use
`CARGO_TARGET_DIR=$E/target`; no shared target. Builds declare base revision
plus `domains-red` or `domains-wip`; these dirty-build claims are not commit
identities. Executable SHA-256 and source overlays are retained externally.
The owned ignored `OrcaSlicer` symlink exposes the supplied read-only checkout;
no upstream, fixtures, core, producer or comparator files changed.

### Red/green harness behavior

- Before implementation, `cargo nextest run -p ares-cli --test orca_parity
  -E 'test(option_coverage::)' --no-fail-fast` exited **100**: 6 passed,
  3 failed (`red-nextest.log`). Current-source generation emitted only
  `0%`, `10%`, percentage interior; partial rejection incorrectly passed;
  unexecuted requirements reported REJECTED. Saved `red-tests` SHA-256:
  `e5f9ffcb34c764db950e38ad2d06b419019cfa620fc55b0da7e4bf32f57ea919`.
- The focused domain/runner/replay command (`focused-final.log`) exits **0**,
  32 passed, including two gated OFFLINE returns (not case execution).
  Broader offline parity command (`offline-parity-tests.log`) exits **0**,
  83 passed / 26 excluded. Boolean/Enum behavioral tests remain unchanged;
  only the partial-rejection expectation changes to INCOMPLETE.
- `verify-exact.py` exits **0**. The same three original behavioral selectors
  exit **101 → 0**; actual owner-reference replay remains **101 → 101**,
  zero strict comparisons (`ARES_ERROR`); classic remains **101 → 101**,
  one strict comparison per replay, `DIVERGENT`. Full streams, raw diff,
  manifests and exact commands/exits are in `exact-results.json` and
  `strict-{owner,classic}-{before,after}`. Manifest lengths/SHA-256 were checked.
- `emitted-plans.log` retains both complete emitted plans. Line width:
  `0`, `2`, `1.134222`, `0%`, `500%`, `366.7208%`; bridge:
  `0`, `0.4`, `0.289068`, `0%`, `100%`, `60.22655%`.
  These are finite validation candidates only. Raw maxima 1000/100 remain
  visible by unit. Invalid/context probes are labelled nonexecuted and add
  zero parity coverage. Both thick flags are required to change bridge's
  positive lower boundary; single-flag cases and zero tags are tested.
- Spawn/wait/timeout/I/O/missing artifact/signal/generic nonzero versus
  attributable -18/-51 are exercised using bounded captured values, not
  hostile processes. Full binary output/status persistence is tested.
  Pipe drainage and normal completion execute on the actual owner pair;
  timeout termination/reaping is implemented, not a live timeout experiment.
  Generic -51 "Too small line width" is not attributable: CLI omits opt_key.
- Initial intermediate builds failed twice (**101**) for a missing test import
  and a `json!` expression syntax error. Both logs remain (`focused-first.log`,
  `focused-pre-owner.log`); neither is counted as behavioral red evidence.

### Exactly two fresh actual-Orca owner cases

`bash $E/run-owner-pair.sh` exits **0**, application test passed. Its test
executable (`owner-tests`) SHA-256 is
`e6cc62770f24e8cbb977800ee84c5fd6493d414ed7bacabe5facf2e3f8e2e97d`;
`owner-source-files.tar`, `owner-tracked.patch`, and `owner-identities.sha256`
retain the exact harness source overlay/build. AppImage executable SHA-256:
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.
Wrapper SHA-256:
`01764d2db02c6febcb8f16992bbe34482bfb180ba9205956c5700fc588186932`.
No source-built Orca trace or fake reference was used.

Four Orca stages (two exports, two slices) exit **0**. Flattened original and
written presets, model, exported archives, all command/stdout/stderr/status
records, full G-code and application JSON remain under `actual-owner-pair`.
The explicit process dependency bundle is layer_height=0.2,
thick_bridges=0, thick_internal_bridges=0; machine nozzle remains [0.4].
There are no smoke/default normalization calls or producer-enabling overrides
in this pair. The existing runner compatibility-list insertion is retained.
Exported `internal_solid_infill_line_width=0` is separately recorded, not changed.

| requested | exported typed value/tag | project SHA-256 | reference SHA-256 |
|---|---|---|---|
| 0.6 | 0.6 / false | c06ce260d7aeee1be07ea6984600f683bf9fe7a1c4e300030fc8ba7a20c63228 | 9e588277e9ca3cda9798ae184e437ce5f181eb423559af502f52210ac9acd1d7 |
| 150% | 150 / true | feac67abbeea1d2cb9a69f39cf6276f8c8ae1ea9d6bbb8edc4448751a6ae4936 | 89675af70484adc72509d40850dad8fe290f271416729de85f26048810bb0f43 |

Both relevant reference G-code config blocks agree with the requested tag and
magnitude and retain the dependency values. This proves project-level process
application, not object/region activation or actual-default profile coverage.
Both Ares attempts fail **ARES_ERROR: unsupported project feature:
wall_generator**; no Ares stream exists, strict-compared=0, domain passes=0.
No classic substitution is made to hide that producer error.

### Other gates and remaining failures

- Generator-only strict sensitivity contracts: **0**, 8 passed across both
  integration binaries (`strict-gate-tests.log`). No comparator changed.
- Native KSR strict gate: **100**, existing first mismatch byte 96, line 3,
  time 1h43m49s/1h48m58s versus 1h43m52s/1h49m1s (`native-ksr.log`).
  Fresh CLI capture exits **0** as output production only; complete bytes and
  raw failing diff (diff exit **1**) are in `native-capture`.
- The retained genuine wave4 browser bytes are copied with their original
  manifest into `retained-browser`. Current native strict gate on those bytes
  exits **100**, same timing mismatch (`retained-browser-gate.log`). This is
  a retained-output gate regression, NOT a fresh browser/WASM execution.
- `cargo fmt --all -- --check`: **0**. `cargo clippy -p ares-cli --all-targets`:
  **0** (`clippy-verified.log`). Existing core warnings and unchanged
  runner compatibility-list/smoke warnings remain; new nesting warnings were
  fixed, with earlier logs retained. All 12 changed Rust files are below
  400 physical lines (maximum 303); tests are in separate modules.

Only the bounded harness correction is a local review candidate. No legal
width domain was fully executed. Zero-percent Flow feasibility, extreme/raw
probes, mixed nozzle intersections, every printer/default/other option,
all ordered artifacts, producer repairs and fresh Tier1 execution remain open.
Legacy non-width generation stays unverified and cannot report complete PASS
without effective-application proof. Independent six-axis review and any
publication belong to the Coordinator; no global parity conclusion follows.

## Independent-review follow-up plan (before changes)

Resume clean `coord/parity-domains-w6` at d528739a0b61e5569de3722c090c0b18d65edc7d.
The three review findings are accepted; no domain arithmetic, producer or
comparator scope is reopened. Evidence goes under the existing isolated root's
`review-fixes/`; its existing owned Cargo target may be reused.

1. Red caller subprocess tests: an existing configured executable plus missing,
   relative, repository-local or unwritable artifact destination must fail
   initialization with its cause, never return a successful offline skip.
   Change the runner to `Result<Option<_>, _>` and propagate errors through all
   smoke/option callers. `None` remains only genuinely unconfigured operation;
   explicitly requested sweeps require a configured/available runner.
2. Red bounded launch test: a relative executable valid in the caller directory
   must still launch after the command cwd changes. Resolve the executable at
   the caller boundary and record its absolute path in command evidence.
3. Red lifecycle test: a short-lived helper descendant inherits output handles
   past direct-child termination. The command deadline must not wait for EOF.
   Replace pipes/readers with file-backed capture and immutable, length-bounded
   snapshots. Bound direct-child polling and shutdown/reaping; preserve primary
   Timeout/Wait through later capture/persistence errors. Retain live capture
   files but mark timeout snapshots incomplete and live files non-authoritative.
   The Coordinator explicitly approved this portable solution: process-tree
   termination is deferred, no native dependency is required, and no claim is
   made that every descendant stopped. No reader threads are abandoned.
4. Re-run focused runner/option/replay/ordered-byte regressions, fmt/clippy,
   exact before/after bounded helper cases and one fresh actual owner export
   pair (execution logic changed). Preserve all reference and producer failures.
   Commit a normal fast-forward follow-up only after these regressions pass;
   retain the old candidate and evidence, no amend/rebase/force/publication.

Ownership remains test-only CLI orchestration around the cited libslic3r
configuration/export boundary and OrcaSlicer.cpp CLI. Tier1 native std process
and regular-file APIs are used; browser core remains untouched. Process-tree
termination, full domain/default coverage, producer parity and fresh cross-OS
execution remain deferred. Finite command deadlines assume responsive host
filesystem/process APIs; no hard realtime guarantee for stalled kernel I/O.

### Review-fix execution record

`R=$E/review-fixes`. The original candidate, evidence and producer failures are
retained unchanged. Rust/Cargo identities and isolated target remain as above.

- Before fixes, focused Nextest for the new initialization/path/lifecycle tests
  exits **100**, four failures (`R/red.log`). The caller regression observes
  20 false successful smoke skips and four option failures missing the useful
  root cause. Relative launch reports Spawn/ENOENT. A 500-ms timeout returns
  after **3.03 seconds**, waiting for descendant-held pipe EOF. A separate
  capture-persistence failure changes Timeout to Io. Saved red executable:
  `99f869233f39cd305871392a9a4b769022aba20a0af7303e46892529a36609ca`;
  `red-source-overlay.tar` and `red-tracked.patch` preserve its source, including
  the timeout-parameter test seam (no lifecycle repair before the red run).
- After fixes, the original four regressions pass. Focused runner/option/replay/
  ordered-byte Nextest across both integration binaries exits **0**, 47 passed
  (`R/focused.log`). Helper/offline entrypoints are not Orca/domain coverage.
  `cargo clippy -p ares-cli --all-targets` exits **0**; only pre-existing core,
  smoke nesting and unchanged compatibility-list warnings remain.
- `R/verify-exact.py` exits **0**; `exact-results.json` retains all commands,
  environment changes, binary hashes and statuses. Four original regressions
  change **101 → 0**; explicit printer sweep initialization with each bad root
  changes **0 → 101**, before any inventory/case execution. Initialization is
  also exercised through selected printer/process, cluster, dump and option
  callers, not just option coverage.
- The safe descendant helper stays alive for three seconds; the repaired
  command call returns in **519.6 ms**, preserving Timeout, captured binary
  prefixes, `kill=Ok`, and a reaped direct child. The test then waits for the
  bounded descendant's completion marker. Retained artifacts under `R/tmp/
  lifecycle-*` include immutable snapshots, full command/status/lifecycle JSON,
  and continuing live files. Late descendant output changes the live file but
  not either snapshot. Status marks the snapshot incomplete and live files
  non-authoritative. Primary Timeout/Wait is preserved through later capture
  errors; non-lifecycle capture failures fail closed as Io.
- Shutdown polls for at most a two-second grace after kill. If a child cannot
  be observed reaped, its PID and unreaped state remain recorded and the stage
  fails; it is never input rejection or PASS. There is no process-tree kill
  guarantee and no abandoned reader thread. Snapshot reads are bounded by the
  observed file length, not EOF or subsequent descendant output.
- One fresh actual owner pair ran because execution logic changed:
  `bash $R/run-owner-pair.sh` exits **0**; four Orca stages exit **0**, all direct
  children observed reaped. `R/actual-owner-pair` retains original/written
  presets, model, commands, live captures, immutable stdout/stderr snapshots,
  statuses/hashes, archives, full reference G-code and application/producer
  manifests. Executable SHA-256 for this run:
  `b1f9979f247a804f3756b061bb72bacd19c737914ca40afeb9b71480b816e834`;
  the supplied AppImage and wrapper hashes remain unchanged. No source-built
  reference or extra physical-equivalence substitution is used.
- Both requests still export exactly `0.6`/percent=false and `150%`/percent=true,
  with nozzle[0.4], layer0.2, thick flags0/0 and solid-infill width0. New project
  hashes are `47672ffe3e0c321e161c5471984eb84c448c78a61a38df3e2b81328f0aaa233b`
  and `47a01f896c1b3c27ae2c2730ca8517d8665e609143231013f01d96943192c113`;
  reference hashes are `b60fe73925d078274b0759a3102c593617a6bc46235d6e16c76ca9c040cb9398`
  and `f666551149f6ae7459f175296b8a0983ec5cd2ecb6f88ca8e12ee100826c2ffd`.
  Both Ares attempts remain **ARES_ERROR wall_generator**, zero strict
  comparisons and zero completed width domains.
- Exact owner and retained-classic replays stay **101 → 101**. Classic still
  diverges at byte863/line32 (M73); complete paired bytes, errors and raw diffs
  remain under `R/strict-*`. The prior native/browser KSR timing failures and
  their complete artifacts remain at the original evidence root; no KSR or
  fresh cross-platform success is inferred from this follow-up.

Only initialization propagation, executable path resolution and bounded command
capture/shutdown are included in this follow-up. No domain arithmetic, producer,
strict comparator, fixture or upstream changes are made. Fresh Windows/macOS/
browser execution, descendant process-tree termination, all-printer/default/
legal-option coverage and complete ordered-output parity remain open. This is
a local fast-forward review candidate, not Coordinator publication or global
acceptance.
