# Spec: full-output Orca parity convergence

## Unchanged user goal

All 1,001 supported printer presets, with their real default process/filament
profiles, must produce the full OrcaSlicer artifacts through the public project
bytes/CLI seam. Every legal Boolean and Enum value and range min/max/seeded
interior must be applied to its upstream owner and verified in effective
exported configuration. Baseline-only plans, rejected cases, substitutions,
missing evidence and skipped offline tests do not satisfy coverage.
Full output includes deposited and travel/motion commands in order, timing,
M73, config, statistics and every generated plate/artifact. ARD-0023's allowed
generator identity/timestamp normalization is not permission to discard other
output. No source pinning, comparator weakening, new tolerance, or legacy
fallback is permitted.

## Source-owned boundaries

The accepted ARD-0023 and four-crate architecture remain authoritative.
`libslic3r/Format/bbs_3mf.cpp::_BBS_3MF_Importer` owns project interpretation;
`PrintConfig.*`, `Preset.cpp` and `PrintApply.cpp` own domains/configuration;
`Print::export_gcode`, `GCode::do_export`, `GCodeWriter` and
`GCodeProcessor::{process_file,run_post_process}` own emitted output.
Their Rust runtime destination remains `ares-core`, with no filesystem or
terminal behavior; `ares-cli` tests own external reference execution, replay and
artifact I/O. Viewer data stays rendering-neutral in `ares-vgcode`; browser
WASM calls the same core API. This wave validates these rewrites, not a new
Ares pipeline. Existing semantic comparison is only a temporary diagnostic
shell and cannot certify whole-output equality.

## Bounded wave: truthful replay and complete paired bytes

The existing `orca_parity_replay_sweep` continues to replay cached `.3mf` and
`<stem>/plate_1.gcode` inputs without invoking Orca. Explicit requests must fail
on empty/unreadable inventory, missing/unreadable bytes, slicing error,
divergence, or artifact/report write failure. With the replay environment unset
it may return an explicitly labelled offline skip, never an executed parity run.

Artifacts go only under caller-selected external `ARES_PARITY_ARTIFACT_ROOT`;
no tracked report or fixed temporary diagnostic paths. Each invocation/case
must preserve complete available project, reference and Ares output before
comparison, with SHA-256 content identities, case label, comparator mode,
executable identity/provenance metadata and useful errors when no Ares output
exists. Legacy reference producer provenance is unknown; hashes attest saved
bytes, not their origin. Supplied provenance must not be silently promoted to
verified provenance. Build revision claims must be distinguished from observed
checkout state; executable hashes identify the actual running build.

The current verdict must say **partial semantic evidence**, never full-output
parity. Successful harness regression tests may assert a real divergent replay
exits nonzero; that does not repair the underlying slicing difference.

## Current non-green facts and exclusions

Scout measurements at 431fb361: 1,001 printer labels, historical 702 PASS /
276 DIVERGENT / 23 ARES_ERROR; these are not fresh successful executions.
The comparator ignores XY travel, timing in this entrypoint, M73, config and
some statistics/order. Option generation contains 302 baseline-only cases;
legal-domain completeness and effective exports remain unverified. Live runner,
vendor enumeration, default-profile substitutions, cache provenance, plate
coverage and many runtime debug hooks remain outside this bounded correction.
No historical pass count or focused semantic success closes any of those gaps.

## Acceptance and review

Current-source red regressions must demonstrate the reachable false greens.
After correction, fail-closed cases must fail and a successful partial comparison
must retain verifiable paired bytes. Fresh independent Orca execution, command
exit codes, output paths, build identities, Nextest, fmt, Clippy and diff/LOC
checks accompany the commit. Docs land only with tested harness code.
Medium read-only scouts and high isolated writers operate under coordinator
scope authority. Final acceptance requires independent six-axis review of
source fidelity, domain/default coverage, complete-output oracle sensitivity,
artifact/build reproducibility, regression correctness, and architecture/
portability/maintainability. Findings require bounded fixes, original+changed
case re-verification and independent review again until all axes are green.
This wave is not overall completion or publication approval.
