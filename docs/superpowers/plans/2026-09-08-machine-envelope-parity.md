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
