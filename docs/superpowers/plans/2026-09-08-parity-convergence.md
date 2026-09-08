# Plan: parity convergence — fail-closed replay wave

Historical wave record: the [strict oracle follow-up](2026-09-08-strict-output-oracle.md)
replaces the partial acceptance gate below with generator-only ordered bytes.
Classic's historical partial PASS is now a strict rejection, not a repaired
slicing case; missing inventory/effective-config/plate/provenance coverage remains.
The [bounded width-domain follow-up](2026-09-08-float-percent-domain-parity.md)
separates units/schema/context and stage failures for two keys, with two real
process-owner export proofs. Both fresh producer attempts remain ARES_ERROR;
no width domain, actual-default matrix or Tier1 parity completion is claimed.

## Approved seam and source boundary

Test project bytes through `ares_core::slice_project` against independently
executed OrcaSlicer output. This is a validation slice of
`libslic3r/Format/bbs_3mf.cpp::_BBS_3MF_Importer`,
`Print.cpp::Print::export_gcode`, `GCode.cpp::GCode::do_export`,
`GCodeWriter.*`, and `GCode/GCodeProcessor.cpp::{process_file,run_post_process}`.
Upstream read-only reference: 8500fcdccaa10b5099ac20d252af3a7c560046f1.
Rust owner: `crates/ares-cli/tests/orca_parity{.rs,/}`; no core edits.
The semantic comparator is a temporary diagnostic scaffold, not the final
whole-output acceptance oracle. No fallback or source-token tests are added.

## This wave

1. Capture a fresh reference with the independent Orca AppImage on the tracked
   Ender-3 project. Preserve its bytes and provenance as test data.
2. Before implementation, run current-source subprocess regressions at the
   replay entrypoint: empty inventory, missing reference, unwritable artifacts,
   actual divergent reference, and paired-artifact success requirements.
3. Make explicit replay fail closed; require a caller-selected external
   `ARES_PARITY_ARTIFACT_ROOT`. Save input and complete outputs before comparison,
   with SHA-256 manifests, executable identity, honest reference provenance,
   and diagnostics for missing data or slicing errors. Remove the fixed
   `/tmp/kobra2` dump from the touched comparison path. Keep the old entrypoint's
   cached-fixture replay purpose, with no compatibility fallback.
4. Re-run exact original cases and regressions with current-source builds.
   Run focused Nextest, broader CLI tests where feasible, rustfmt, Clippy,
   diff/LOC checks. Record commands, exits, artifact paths and build identities.
5. Commit tested code, fixtures and these docs together; create local review
   branch `coord/parity-harness-w1`. Independent review is required before use.

## Deferred convergence (not acceptance from this wave)

- All 1,001 supported printer presets with their actual default process and
  filament profiles; substitutions and smoke normalization cannot certify them.
- `PrintConfig.*` / `Preset.cpp` legal Boolean/Enum domains, separate literal
  and percent min/max/seeded interiors, and requested-versus-effective exported
  config checks. The 302 baseline-only generated cases are a known gap, not
  legal-domain coverage. Rejection, invalid setup and missing evidence never PASS.
- Complete G-code including XY travel/order, timing, M73, config, statistics,
  and all generated plates/artifacts; no comparator loosening or tolerances.
- Fail-open live runner/vendor/option enumeration and cache identity remain
  separate bounded validation slices; replaying legacy bytes cannot establish
  their missing producer provenance.
- Source-owned runtime fixes and removal of remaining temporary debug hooks
  require isolated approved lanes, not edits in this harness wave.

## Review/fix/reverify protocol

Use medium read-only scouts for independent diagnosis and high isolated writers
for one approved source-owned correction each. The coordinator, not workers,
assigns scope and publication. After convergence, independently review six axes:
source fidelity/no pinning; legal-domain/default-profile coverage; full output
and comparator sensitivity; reproducible artifacts/build identity; correctness
and regression tests; architecture/portability/maintainability. Every finding
routes to a bounded writer, then original-case and changed-case revalidation,
then independent re-review. Repeat until all six axes and full user coverage
are green. This wave does not start or complete that final acceptance loop.

## Execution record

Evidence root `E=/home/indexyz/.local/state/ares-parity/2026-09-08-harness-w1`.
Managed worktree: `/home/indexyz/ares.pi-subagents-harness-fix-d869825-34be-s0-t0`,
base `431fb36141d9d6e2b60cc7149d35891978aa0fbb`.
All harness builds used this worktree's `target`, not a shared baseline target.
`rustc 1.96.0 (ac68faa20 2026-05-25)`, test profile optimized + debuginfo.
Compile-time revision claim for dirty validation builds:
`ARES_PARITY_BUILD_REVISION=431fb36141d9d6e2b60cc7149d35891978aa0fbb+harness-wip`.
This is explicitly a claim; executable SHA-256 is independently measured.

### Red then green, external seam

- Fresh independent Orca original and classic-reference commands both exited 0;
  full command/input/output identities are in `tests/parity/replay/README.md`.
- The original tracked Ender-3 project is rejected by Ares (`wall_generator`).
  It remains an explicit error regression. A separately labelled classic fixture
  changes only that config value and reaches the partial-semantic success path.
- Before implementation, `cargo nextest run -p ares-cli --test orca_parity
  -E 'test(replay_tests::)' --no-fail-fast` exited **100**, five failures:
  empty inventory, missing reference, unwritable artifact destination, divergent
  real-reference mutation, and missing paired artifacts. Full log:
  `$E/red-classic-nextest.log`. Current-source red executable SHA-256:
  `c4d612b4e092ad2d09c3c356bb47b52f5d69d42ee429c3ddd7135e404a2aa7ee`,
  preserved at `$E/red-current-source-tests` (not the scout's prebuilt binary).
  An initial test-authoring compile error (`LowerHex` on sha2's digest array,
  exit 101, `$E/red-nextest.log`) was fixed before behavioral red evidence.
- Final same focused command: **0**, 11/11 harness regressions pass,
  `$E/focused-final.log`. Final precommit executable SHA-256:
  `a0968911f906b205f8d91fd41afd37133db8f5cc606ee9621df16079bb86c562`.
- Exact before/after subprocess matrix command:
  `/nix/store/gxzhl7aaiid7zp3y47jqqiq7zg5mqpwp-python3-3.14.6/bin/python3
  "$E/verify_replay.py"`, exit **0**. It invokes each test binary with
  `--exact replay::orca_parity_replay_sweep --nocapture`, configuring separate
  `ARES_PARITY_REPLAY` and `ARES_PARITY_ARTIFACT_ROOT` for every scenario.
  `$E/exact-replay-final-results.json` records every exact command, environment,
  cwd, binary hash, log path and exit. The six bad cases (empty, missing ref,
  unwritable artifacts, unwritable report, original rejection, divergence)
  change from exit **0** to **101**; classic remains exit **0**. Saved files'
  hashes and byte lengths were independently checked by Python hashlib.
  Full artifacts and logs: `$E/exact-replay-final/{before,after}/<case>/`.
- Classic's raw whole-output diff remains **6,099 lines** at
  `$E/exact-replay-final/after/classic/full-output.diff`. This is not parity;
  M73 and motion-order differences remain. A passing harness test that asserts
  a failed replay is not a repaired slicing case.

### Broader validation and residual failure provenance

- `cargo nextest run -p ares-cli -E 'not binary(orca_parity) or
  (binary(orca_parity) and not test(smoke::) and not
  test(option_coverage::orca_parity_option_coverage))' --no-fail-fast`:
  **100**, 117 passed / 1 failed / 27 excluded, `$E/cli-nextest.log`.
  Live sweeps/dump paths were not invoked; they still need their own corrections.
  Offline entrypoint returns in this run are not executed parity comparisons;
  existing domain/source-shape tests are not legal-domain acceptance evidence.
- Failure: `ksr_fdmtest_v4::project_matches_orca_242_semantically`, expected
  filament 1 length **11335.74 mm**, actual **11335.55 mm**.
  Exact single-test command on both base and changed source:
  `cargo nextest run -p ares-cli --test ksr_fdmtest_v4
  -E 'test(=project_matches_orca_242_semantically)' --no-fail-fast`.
  Both exit **100** with the identical mismatch. Base was exported with
  `git archive 431fb36141d9d6e2b60cc7149d35891978aa0fbb` into
  `$E/base-source`; separate `CARGO_TARGET_DIR=$E/base-target`. Same compiler,
  profile and revision-claim environment; `ARES_BROWSER_GCODE` unset, comparator
  `semantic::compare` on both. Logs `$E/ksr-clean-base.log` and
  `$E/ksr-changed.log`; hashes `$E/ksr-build-identities.txt`. All 4,733 archived
  regular source files verified unchanged after execution (exit 0,
  `$E/base-snapshot-verification.log`). No core/comparator fix or exclusion was
  used to turn the broader result green.
- `cargo nextest run -p ares-core --no-fail-fast`: **0**, 6,791 passed / 1
  skipped, `$E/core-nextest-final.log`. Initial attempt reached the tool's
  300-second timeout and is not a completed result (`$E/core-nextest.log`);
  the rerun completed in 331 seconds.
- `cargo clippy -p ares-cli --all-targets`: **0**,
  `$E/clippy-verified.log`; existing core, smoke and runner warnings remain,
  no warning from changed files. Strict all-features `-D warnings` was not run
  or claimed green.
- `cargo fmt --all -- --check`: **0**, `$E/fmt-final.log`.
  `git diff --cached --check` and changed-Rust physical LOC checks: **0**.
  Initial staged whitespace check exited **2** on exact upstream fixture
  trailing spaces and EOF blanks only. Coordinator approved fixture-local
  `.gitattributes` (`*.gcode -text whitespace=-blank-at-eol,-blank-at-eof`),
  preserving full bytes and visible text diffs without changing Rust/docs checks.
  Staged blob SHA-256 identities are verified against the capture hashes.
  All four
  changed Rust files are below 400 lines; regression tests are a separate file.

Only the fail-closed replay/artifact correction is ready for independent review.
The 1,001/default/domain/full-output goal, 302 baseline-only cases, whole-output
comparator gaps, live fail-open harness paths and runtime debug hooks remain open.
