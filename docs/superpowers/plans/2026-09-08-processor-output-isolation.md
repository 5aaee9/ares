# Processor isolation execution plan

Boundary: [processor output isolation ARD](../../architecture/processor-output-isolation.md).
This is a `libslic3r::GCodeProcessor` rewrite-boundary correction, not a new Ares
pipeline or full-goal acceptance. Included/deferred behavior is fixed by the ARD.

1. Verify worktree/ref, source owner and Coordinator contamination artifacts.
   Preserve verified baseline binary identity separately from the local build.
2. Before production changes, add dedicated CLI tests and the byte-identical
   anchor 3MF fixture. Run Nextest to reproduce external sentinel substitution
   through a child process; preserve the failure and actual output artifacts.
3. Delete processor-local ambient input and dump hooks plus logging-only work.
   Move existing estimator, arc accounting, delay scheduling and motion-limit
   responsibilities into ordinary modules, changing no timing/ordering formulas.
   Every changed Rust file must be under 400 physical lines.
4. Re-run focused CLI tests and processor/core Nextest. Build a local release
   CLI in its own target directory. Preserve complete baseline/candidate outputs
   with unset, readable sentinel and unreadable-path child environments. Require
   byte equality except for one validated generator timestamp line.
5. Run real Orca 2.4.2 through the approved wrapper on the anchor and preserve
   raw reference/candidate residual diffs and SHA256 identities. No parity claim
   follows from the isolation fix; no comparator edit is permitted.
6. Run format, changed-crate Clippy, WASM core/adapter checks, diff/LOC/scope checks
   and staged-index inspection. Record command exits/full log paths and residual
   baseline failures. No unrelated broad sweep or source-token pin tests.
7. Only with successful isolation regressions, commit code/tests/docs together
   using Conventional Commit and create local `coord/parity-processor-w2`
   without force, push or merge. Coordinator owns independent six-axis review,
   integration and publication.

## Execution record

- Initial worktree: `/home/indexyz/ares.pi-subagents-processor-isolation-fix-d25f0bf-ed4d-s0-t0`.
- Initial ref: `pi-subagents/processor-isolation-fix-d25f0bf-ed4d-s0-t0`.
- Initial HEAD: `570ff3ad544e682212be7c547c3ae9789f309560`; clean index/worktree.
- Plan and included/deferred boundary recorded before tests or production edits.
- Evidence root: `/home/indexyz/.local/state/ares-parity/2026-09-08-processor-w2`.
  All native and WASM Cargo commands used its own `target` directory, with
  `--locked --offline`; toolchain was rustc/cargo 1.96.0, not CI's pinned 1.91.

### Red, correction, green

The new CLI Nextest target failed all four tests on unchanged production code:
readable sentinel replaced the entire output (CLI exit 0), missing and existing
directory paths caused CLI exit 101, and the dump gate created a file.
`cli-red.log` records Nextest exit 100. An earlier module-path compile error
(exit 101) was corrected before this behavioral red and retained separately.
`red-artifacts.json` records both verified baseline and own pre-fix debug binary
runs, commands and full hashes; both produced the identical 58-byte sentinel.

Removed exactly five environment gates, the logging-only block-line-id clone,
and file dump work. The remaining implementation was mechanically split into
`estimate`, `arc_accounting`, `schedule`, and `motion/limits`. Helper visibility
is restricted to parent modules; no API or arithmetic was added. Existing
processor tests changed imports only. An initial stale `delays` import compile
error was fixed and retained in the evidence logs. `check-extraction.log`
compares all 20 retained function bodies to the base after deleting the
specified hooks/clone, ignoring only whitespace and declaration visibility.
This is local refactor evidence, not an upstream source-token assertion test.

### Validation results

Set `E=/home/indexyz/.local/state/ares-parity/2026-09-08-processor-w2` and
`CARGO_TARGET_DIR=$E/target` for the Cargo commands below.

| Command | Exit | Evidence |
| --- | --- | --- |
| `cargo nextest run --locked --offline -p ares-cli --test processor_output_isolation` | 0 | `cli-green.log`: 4/4 passed |
| `cargo nextest run --locked --offline -p ares-core -E 'test(project_slice::gcode_emit::processor)'` | 0 | `processor-nextest.log`: 28 passed, 6764 excluded/skipped |
| `cargo nextest run --locked --offline -p ares-core` | 0 | `core-nextest.log`: 6791 passed, 1 skipped |
| `cargo build --locked --offline -p ares-cli --release` | 0 | `release-build.log` |
| `cargo clippy --locked --offline -p ares-core -p ares-cli --all-targets` | 0 | `clippy.log`; existing warnings remain, including moved estimator nesting |
| `cargo check --locked --offline -p ares-core -p ares-wasm --target wasm32-unknown-unknown` | 0 | `wasm-check.log` |
| `cargo fmt --all -- --check` | 0 | `fmt-check.log` |
| `git diff --check` | 0 | `diff-check.log` |
| `node "$E/candidate-artifacts.mjs"` | 0 | `candidate-artifacts.json`: complete output comparisons |

The release candidate SHA256 is
`71772eeee4e6629b988bbccd944d74788e0dde1d518e91c7ffe8fc5f6a2f3b88`.
Unset, readable sentinel, missing path and directory path all produce the same
normalized complete 192697-byte raw output; the verified baseline and own
pre-fix debug output also match after the single generator-line normalization.
The shared normalized SHA256 is
`cbafdd577f2e3a9077badbe6d6726951de6b2957440b982210ccba3a524b021f`.
Raw baseline-to-candidate diff has only the timestamp change; normalized diff
is empty (exit 0).

Real Orca was rerun with `ORCA_APPDIR=/tmp/squashfs-root` using
`/home/indexyz/ares/scripts/orca-parity.sh --slice 0 --outputdir "$E/orca"`
on the Coordinator anchor project. Exit 0; `orca-command.txt` contains the
complete command, `orca.stdout`/`orca.stderr` preserve process output.
The actual AppImage SHA256 is
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.
The fresh reference SHA256 is
`58aa036a00e6fef91b7fd9cf211d23360c966a729519c557dcbc6765c3b41a63`.
`orca-candidate.raw.diff` remains 1104 lines (exit 1); timestamp-normalized diff
remains 1097 lines (exit 1). Ordered travel and other residuals are not removed
or weakened. No reference was substituted into a candidate run.

`scope-check.log` verifies only owned paths changed, all changed Rust files
are below 400 lines (maximum 380), no source-splitting includes, no production
processor OS hooks, separate test bodies, and empty staged index before commit.
Known independent KSR filament failure remains unmodified and was not rerun in
this bounded lane. Other core instrumentation and full printer/option/artifact
coverage remain deferred, as do Windows/macOS native and browser runtime runs.
Coordinator owns the required independent review and any integration/publication.
