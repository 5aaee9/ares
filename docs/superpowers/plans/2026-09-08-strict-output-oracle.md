# Plan: strict supplied-output oracle

## Approved source boundary

ARD-0023 remains accepted and unchanged. Validate Orca 2.4.2, upstream
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, at the external emitted stream:
`libslic3r/GCode.cpp::GCode::do_export` (2030; generator at 2576,
statistics/config at 3535–3555, object identities at 5349–5351/5472–5475),
`GCodeWriter.cpp` (temperature 117–192, acceleration 216, travel 601/685/812,
extrusion 933/958/977, retraction 1004–1056, fan 1095/1133), and
`GCode/GCodeProcessor.cpp::run_post_process` (981; time rewriting 1113–1117,
M73 mask 1768). These are source citations, not source-token acceptance tests.

Rust destination is exclusively the existing `ares-cli` test harness:
`ksr_fdmtest_v4/golden.rs`, its separate tests, `compare_actual`, and
`orca_parity` artifact/replay gates. No runtime, adapter, geometry, config,
libvgcode or upstream edits. Existing semantic parsers remain explicitly
**diagnostic-only** with their behavioral tests retained, not a fallback.
This validates upstream-owned rewrite output, not a new Ares pipeline.

## Contract and sequence

1. Change the classic partial-success regression to require strict rejection of
   the **unchanged independently generated pair**, saved complete streams,
   SHA-256/length/error evidence and unknown legacy producer provenance. Run
   current source before gate changes to prove the false PASS.
2. Compose existing `normalize_one_generator_line` and `first_difference` into
   one `compare_ordered_bytes_generator_only` function. Independently shaped
   Orca/Ares generator lines are the **only** normalization. Object identity,
   travel, time, M73, config, statistics, ordering, whitespace and EOF stay strict.
   Wire artifacts after saving both streams, and native/browser `compare_actual`.
   Delete only the now-orphaned object-ID collapse helper and its dedicated test.
3. Add bounded sensitivity contracts in a separate module: 13 families with
   change/deletion/reordering, eight generator-contract templates (invalid shapes
   tested independently on both sides), and six whitespace/EOF boundaries.
   Real classic records are used where present; arc/multiobject snippets are
   explicitly synthetic comparator-only inputs. Assert anchors, ordering and
   actual mutation. Equivalent-body positives are comparator contracts, **not**
   Ares E2E passes or invented Orca replay references.
4. Re-run current-source focused/regression tests, exact original/classic replay
   subprocesses in unique external roots, native KSR before/after, browser wiring
   and feasible runtime, broader CLI, fmt, Clippy, diff/LOC and build identities.
   Save exact exits/logs/hashes. A native/browser output mismatch remains a real
   failure even if stricter comparison exposes an earlier byte.
5. Commit docs/code/tests together only after oracle/replay regressions pass;
   local `coord/parity-strict-oracle-w4` for independent review. Coordinator alone
   integrates/publishes; no push/merge or reference reset.

## Evidence semantics and excluded coverage

`ordered_bytes_generator_only` describes strict equality of **supplied paired
streams**. Saved hashes attest bytes, not independent producer identity.
Reference provenance remains unknown for legacy replay. `not_checked` retains
all-printer/default/domain inventory, requested-versus-effective config coverage,
all plates/artifact inventory and reference producer identity. Comparing retained
config bytes is not checking that every legal option was exercised effectively.

The full 1,001 actual-default printer goal, every legal Boolean/Enum and range
min/max/seeded interior, inventory/provenance repairs, all-plate/artifact capture,
source-owned slicing repairs and Tier-1 execution are not closed by this slice.
The original Ender-3 rejection and classic divergence must remain errors. No
extra nondeterminism normalization, expected G-code edit, ARD amendment or
producer-pass claim is authorized. Historical partial verdicts are not rewritten
as strict successes. See the convergence spec/plan for prior wave evidence.

## Execution record

Evidence root:
`/home/indexyz/.local/state/ares-parity/2026-09-08-strict-oracle-w4` (`E`).
All builds use external `$E/target`, temp files `$E/tmp`, and the explicit
compile-time claim `79b70da235cef5446ae633bada7c6404b901db8b+strict-oracle-wip`.
Executable hashes, not that dirty-build revision claim, identify tested binaries.

- Current-source replay red: `cargo nextest run -p ares-cli --test orca_parity
  -E 'test(replay_tests::)' --no-fail-fast` → **100**, 10 passed/1 failed.
  `red.log` shows unchanged classic falsely exiting 0 under the old gate;
  `red-current-source-tests` and `red.sha256` preserve that binary.
- Focused strict oracle/replay: same command with filter
  `test(replay_tests::) or test(ordered_bytes_tests::)` → **0**, 15 passed;
  `green.log`. Final focused run adds `--test ksr_fdmtest_v4` with the same
  filter: **0**, 19 passed (`focused-final.log`), exercising shared contracts in
  both binaries. No Ares slicing success is implied by these harness regressions.
  Red replay binary SHA-256:
  `732cbd9d3fb33b6546b572f4ae9f92a65463c2bf202bea09a54c1df5ab3e3c48`;
  final replay binary:
  `24b61a8fad48e821ac7b3b8265ba3c17770c087c893f42fe4beb29f1d9ddd440`.
- `replay.py before|after <saved-current-source-binary>` executes each exact
  `--exact replay::orca_parity_replay_sweep --nocapture` subprocess. Script exits
  **0** both times; `exact-before.json`/`exact-after.json` record exact command,
  cwd, env, binary hashes, exits, summaries and manifests. Original exits
  **101 → 101**, `ARES_ERROR` (`wall_generator`); classic **0 → 101**, now
  `DIVERGENT` at normalized byte 863, line 32, M73 P4 R8 versus M73 P2 R8.
  Python hashlib independently verifies every saved manifest file length/hash.
  Full streams/error/diffs remain under `exact-replay/{before,after}/`.
- Classic reference SHA-256 remains
  `3ed255be8081c66c1a512a838040752cdb076d3aeae9be9a632d0f2e43fabb97`;
  Ares remains `1ff0f80f3ae17bed696397886f9939ba50e32c8e4139b3eb02c15bc21bd0cbe3`.
  The changed verdict exposes existing bytes; it does not change generated output.
- Native KSR before: `cargo nextest run -p ares-cli --test ksr_fdmtest_v4
  -E 'test(=project_matches_orca_242_semantically)' --no-fail-fast` → **100**,
  filament 11335.74 versus 11335.55 mm, `ksr-before.log`. After moving test bodies
  into `tests.rs`, filter `test(project_matches_orca_242_semantically)` → **100**,
  now first raw timing difference at byte 96, line 3: 1h43m49s/1h48m58s versus
  1h43m52s/1h49m1s, `ksr-after.log`. This is an existing raw mismatch newly
  exposed, not a runtime regression or parity fix. The legacy selector suffixes
  remain for existing callers, but both call the same strict `compare_actual`;
  `ARES_BROWSER_GCODE` no longer selects any tolerant acceptance mode.
- Browser tooling inspected: `project-slice.spec.mjs` invokes Nextest by the
  browser selector suffix with `ARES_BROWSER_GCODE`; `project-slice-page.mjs`
  calls `sliceProject`. No browser caller or adapter source was changed.
  `cargo build -p ares-wasm --target wasm32-unknown-unknown --release
  --target-dir "$E/wasm-target"` and `wasm-bindgen ... --target web --out-dir
  "$E/target/wasm-browser"` both exit **0**. Owned browser output links expose
  these external generated files at the existing server's `target/wasm-browser`
  route. `npm --prefix crates/ares-wasm/tests/browser ci` uses `$E/npm-cache`.
- `npm --prefix crates/ares-wasm/tests/browser test` initially exits **1** before
  execution because Chromium cannot load `libglib-2.0.so.0` (`browser.log`).
  After supplying existing read-only Nix library directories via the saved
  `browser-env.sh`, it executes real Chromium 143.0.7499.4: **1** overall,
  export-hook check passed, project test failed at the strict gate (nested
  Nextest **100**), same timing byte 96 (`browser-final.log`). This verifies
  browser wiring/runtime, not browser parity. No browser binary/cache patch.
- `node "$E/capture-browser.mjs"` → **0** verifies expected rejection, saving
  complete real browser output, project, legacy reference, subprocess logs and
  SHA-256/length manifest under `browser-capture/`. Its actual comparator command
  is `cargo nextest run -p ares-cli browser_output_matches_orca_242_semantically
  --run-ignored all`, exit **100**. Browser Ares SHA-256:
  `c88f3c475e303287a004466876c6f08e8126c836927ed149324d9eb975db91a8`.
  Native CLI output is also saved under `native-capture/` (slice exit **0** is
  output production, not a comparison pass). `captured-identities.txt` binds
  native bytes/executable and current-source raw/generated WASM artifacts.
- Broader CLI command: `cargo nextest run -p ares-cli -E
  'not binary(orca_parity) or (binary(orca_parity) and not test(smoke::) and
  not test(option_coverage::orca_parity_option_coverage))' --no-fail-fast` →
  **100**, 128 passed/1 KSR failed/27 excluded (`cli-nextest-final.log`).
  Initial run had 14 additional missing-upstream-path failures, preserved in
  `cli-nextest.log`; owned ignored links to read-only upstream `src`/`resources`
  fixed test setup, not source behavior. Offline returns/domain scaffold tests
  are not fresh printer/domain coverage. No full 1,001 sweep was run.
- `cargo clippy -p ares-cli --all-targets` → **0** (`clippy.log`); existing core,
  smoke and runner warnings remain, none in changed files. `cargo fmt --all --
  --check` → **0** (`fmt.log`). Rust test bodies moved to separate `tests.rs`;
  all touched Rust files remain below 400 physical lines (maximum 307).
  `git diff --check` → **0** (`diff-check.log`). `identities.txt` records compiler,
  Cargo/Nextest/Node/wasm-bindgen versions, binary hashes and upstream/AppImage
  identities. Semantic behavioral
  tests are retained unchanged; only the dedicated ID-collapse test was deleted.

Linux native and Linux-hosted browser execution are evidenced, with real output
mismatches retained. Windows/macOS execution remains unverified. No fresh Orca
capture was performed in this slice: committed independent captures were replayed
unchanged, and the supplied AppImage SHA-256/upstream revision were rechecked.
Full-goal completion and independent six-axis acceptance remain Coordinator work.
