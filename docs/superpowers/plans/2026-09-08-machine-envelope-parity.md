# Configured machine envelope (bounded rewrite)

Owner: OrcaSlicer 2.4.2 `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`libslic3r/GCode/GCodeProcessor.cpp::apply_config(PrintConfig)` 2087–2122,
`get_axis_max_feedrate/acceleration/jerk` 5742–5822 and
`TimeProcessor::reset`; `GCodeWriter::supports_separate_travel_acceleration`.
Destination: core `project_slice/gcode_emit.rs` effective-config seam,
`processor::ProcessorLimits`, `estimate` and `motion/limits` initialization.
No libvgcode behavior belongs in this bridge.

Included: normal-mode configured XYZ/E feedrate, acceleration and classic jerk;
MarlinFirmware-only junction deviation; supported-flavor config gate and
upstream defaults for other flavors; print/retract/travel initialization.
The emission flag does not gate estimator configuration. Existing processor is
only a temporary compatibility shell for the cited upstream estimator.

Deferred: stealth estimates, minimum feedrate application, other planner
arithmetic, geometry, progress/cache/seam rules, SET_VELOCITY_LIMIT VELOCITY
parsing (explicitly supervisor-deferred), all-printer/option parity and Tier1
runtime certification. No comparator or expected-byte changes.

Plan before production edits:
1. Add separate processor replay tests reading complete immutable wave8 Afinia
   and Artillery references and their embedded effective config. Capture RED at
   ordered output, with an existing actual-Orca positive replay regression.
2. Mechanically extract the oversized emission loop to a cohesive child module;
   verify existing green output before/after. Supervisor approved this necessary
   <400-line split, with no emission semantics changes.
3. Port the typed machine envelope bridge and focused flavor/override tests.
   Re-run complete replay and report exact residual first differences, not PASS
   for partially improved fixtures.
4. Run nextest, fmt, clippy and available Tier1 checks in an owned target with
   finite 900s command bounds. Keep unchanged CLI anchors; commit only verified
   bounded correction. Full user goal remains incomplete.

## Implemented boundary and evidence

`ProcessorLimits::from_config` transports normal-mode XYZ/E arrays through the
existing effective printer configuration seam. It copies configured limits only
for MarlinLegacy, MarlinFirmware, Klipper and RepRapFirmware; other flavors keep
`TimeProcessor::reset`'s `MachineEnvelopeConfig` schema defaults (`PrintConfig.cpp`
4497–4567, 4600–4637; `PrintConfig.hpp` default-constructor cache). Normal values
are converted through upstream's float return type. Classic jerk comes from
`machine_max_jerk_*`, not process `default_jerk`; only MarlinFirmware activates
JD. Klipper/legacy retain zero separate travel cap and initial travel acceleration
1250, even if the configured travel array differs. M201/M203/M205 and all other
existing command overrides, scheduling and emission remain unchanged.

The necessary move-only extraction puts layer-chunk emission in `layers.rs`
(378 lines), leaving the driver below 400. Anchor/one-wall complete CLI outputs
were identical before/after extraction and after envelope initialization under
the existing validated generator-only comparator. Both CLI cases were already
reference-red (normalized first differing byte 6804/10926); neither is claimed
as reference parity. Existing processor anchor replay is complete byte-green
before and after. Existing seam fixtures were not modified; their touched test
module now reads fixture bytes at test runtime instead of include macros.

Evidence root (owned worktree): `evidence/`. Commands used owned
`CARGO_TARGET_DIR=$PWD/target-envelope` and finite 900s bounds.

- RED `cargo nextest run -p ares-core -E 'test(envelope_replay_tests)'
  --run-ignored all`: exit 100, anchor passed; Afinia byte2787/line118 and
  Artillery byte483/line18 failed. Full input/actual/expected bytes in `red/`.
- Final explicit processor run with `--run-ignored all --no-fail-fast`: exit100,
  43/44 pass. Afinia complete 107618-byte replay passes; Artillery remains RED
  byte2665/line149 (expected `M73 P2 R15`, actual next G1). Its initial R16 and
  first 148 lines now match, but total16m10s still differs from actual16m12s.
  Complete Artillery output/diff is retained, not treated as passing coverage.
- Ordinary processor run: exit0, 42/42 pass. Eight focused envelope tests cover
  first retract/Z, Artillery Z-limited displacement, command overrides, flavor
  gates, zero acceleration defaults, normal-mode float conversion, emission
  flag independence and JD gate. External replay diagnostics are ignored by
  default because they require immutable wave8 artifacts; both were explicitly
  executed, including the failing Artillery test. They never silently skip.
- `cargo nextest run --workspace --no-fail-fast`: exit100, 6954/6995 passed;
  40 failures require the absent worktree-local upstream/profile/reference
  environment. KSR parity also remains red. A separate published-HEAD baseline
  run reproduces its identical byte96/line3 failure and6321946-byte length.
  Two final additional envelope tests passed separately. No skipped/unexecuted
  test, semantic-tolerance test or environment failure counts as parity evidence.
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets`: exit0.
  Existing warnings remain, including mechanically moved layer-loop warnings.
- `cargo check`: browser WASM core/adapter/view data and Windows GNU/macOS
  workspace checks exit0. Linux builds/tests execute natively. Cross-checks do
  not certify native Windows/macOS runtime behavior.

Fresh actual executable SHA256
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`, via the
unchanged wrapper with `ORCA_APPDIR=/tmp/squashfs-root`, sliced both exact wave8
inputs (`--slice 0 --outputdir ... input.3mf`), exit0 each. Full outputs and
`result.json` are in `fresh-orca/`; both match archived actual references with
only validated generator-line normalization. Afinia full CLI output now matches
fresh actual Orca. Artillery full CLI output remains line149-red, with later
motion differences deferred. No reference bytes or comparator were changed.

Browser WASM release build + wasm-bindgen exit0. Test-only Playwright script
`evidence/browser-envelope.mjs` runs both complete 3MFs through `sliceProject` in
Chromium143.0.7499.4, exit0 after supplying existing native libraries (two earlier
launch attempts failed before execution). Both browser outputs equal native
outputs generator-only; Afinia also equals fresh actual Orca. Complete bytes,
logs and strict residual diffs remain in `browser-output/`. This demonstrates
these two browser executions, not Tier1 or all-option certification.

Reproduce external replays with
`ARES_ENVELOPE_FIXTURES=/home/indexyz/.local/state/ares-parity/2026-09-07-coordinator/wave8-smoke-sample/corrected/artifacts`
and `ARES_ENVELOPE_OUTPUT=$PWD/evidence/replay`, using the nextest command above.
All artifacts are diagnostic/test-only; production has no FS/env hook. These
classic-normalized smoke inputs are not all1001 actual-default printer coverage.
Independent six-axis review/publication remains Coordinator-owned.

## P1 correction plan: zero and sub-unit axis acceleration

Source: `GCodeProcessor.cpp:4066–4073` clamps each moving axis even when
its configured acceleration is zero and stores block acceleration without a
unit floor; `PrintConfig.cpp:4529–4548` declares axis acceleration coFloats with
minimum zero. `get_option_value` returns zero for empty arrays. Destination:
`processor/motion.rs` linear and segmented block construction only; libvgcode
has no ownership in this correction.

Before production changes:
1. Add separate behavioral tests through `ProcessorLimits::from_config` for
   zero/sub-unit Z limits on linear and helical arc moves, plus empty Z arrays.
   Execute them RED against the envelope candidate at `82694b4c`.
2. Remove the positive-only axis clamp gate and block acceleration unit floor
   in both constructors. Do not change centripetal acceleration, scheduling,
   cache, emission, seams, command parsing, comparator or reference bytes.
3. Re-run focused/envelope/core nextest, explicit complete envelope replays,
   fmt/clippy and browser WASM compile checks in an owned target (900s bounds).
   Afinia must stay byte-exact; retain and compare full Artillery residual bytes.
   Record any regression rather than widening scope; only commit verified work.

Supervisor-approved test setup correction after the first focused run:
13 existing synthetic timing tests relied on zero axis defaults meaning
unlimited. Give only those setups explicit nonbinding positive axis limits,
without changing their assertions or fixture bytes. Move the new tests into a
separate motion child module to derive exact arc expectations from source
segment deltas (division versus separately rounded direction multiplication),
not tolerance or magic ULP values. Production defaults and planner stay intact.

### Correction validation

Evidence: owned `evidence/accel-zero/`, target `target/accel-zero`; every Cargo
validation command bounded by `timeout 900`. New separate tests cover zero and
0.5 Z acceleration in linear blocks and helical arc segments, and empty Z arrays
in both constructors. Existing assertions, comparator and reference bytes are
unchanged. Touched Rust files are 374, 370 and 78 physical lines.

- TDD `cargo nextest run -p ares-core -E 'test(axis_acceleration_tests)'
  --no-fail-fast`: exit100, all five RED before production changes (zero yielded
  1250; sub-unit yielded 1). First post-fix focused run exposed 13 synthetic
  setup regressions and the exact arc expectation rounding issue, retained in
  `focused.log`; the supervisor approved the test-only corrections above.
- Baseline core before correction: exit0, 6805/6805. Baseline production rerun
  with corrected synthetic setups: exit0, 6805/6805. Final `cargo nextest run
  -p ares-core --no-fail-fast`: exit0, 6810/6810, three ignored tests not credited.
- Final `cargo nextest run -p ares-core -E 'test(gcode_emit::processor)'
  --no-fail-fast`: exit0, 47/47, including eight envelope unit tests and all five
  new axis tests.
- Explicit replay command from above with `--no-fail-fast`: baseline and final
  exit100, Afinia and anchor PASS, Artillery unchanged RED at byte2665/line149.
  Full input/actual/expected files are retained in `baseline-replay/` and
  `final-replay/`. `cmp` of baseline/final complete actual bytes exits0 for all
  three, not merely their first differences. Afinia actual/reference `cmp`
  exits0; both SHA256 are
  `6cb5cc59480e0bafbdda8d862477f49b31232f1c139599e5b63da0c9a56ee6e7`.
  Artillery actual SHA256 before/after is
  `54de7c4cc5847ea4381776a52c8e03c972057fe8a3415ac2d543a8267dc75c18`.
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets`, and
  `cargo check --target wasm32-unknown-unknown -p ares-core -p ares-wasm
  -p ares-vgcode`: exit0 each. Existing clippy warnings remain; no browser or
  Windows/macOS runtime execution or fresh AppImage slicing in this correction.

Search found no production caller relying on default zero axes as unlimited:
`ProcessorLimits::default` is test-only; `MotionState::with_limits` overwrites
all axis defaults from config. Existing downstream planner zero-acceleration
arithmetic can produce NaN (observed with the old synthetic zero setups);
changing that arithmetic is explicitly outside this block-construction fix.
Zero-axis full timing parity is therefore not claimed. Artillery and the full
all1001/options/artifacts/Tier1 goal remain incomplete; no failed, ignored or
unexecuted case is credited as parity. Independent review/publication remains
Coordinator-owned.
