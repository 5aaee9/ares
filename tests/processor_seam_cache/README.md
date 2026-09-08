# Source-owned seam/cache output regressions

Owner: OrcaSlicer 2.4.2 `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`GCode/GCodeProcessor.cpp::process_G1`, `TimeMachine::calculate_time`, and
`GCodeProcessor.hpp::SeamsDetector`. See the seam-cache ARD and plan.

All `.orca.gcode` files are complete fresh actual-AppImage output, not Ares or
source-built traces. AppImage binary SHA256:
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.
Wrapper used: `/home/indexyz/ares/scripts/orca-parity.sh`, SHA256
`01764d2db02c6febcb8f16992bbe34482bfb180ba9205956c5700fc588186932`.
Original project bytes are unchanged copies of the preserved logical-cursor
`mutations/{anchor,two-walls,wipe-off,solid-rectilinear}/project.3mf`.
`SHA256SUMS` records every input/reference identity. Actual output includes
complete effective config: Marlin2, 1100 print/retract/travel acceleration,
0.01 junction deviation, zero filament-load delay. Other selected options and
all generated commands remain in the artifacts, not normalized away.

The fixture-local `.gitattributes` preserves verbatim G-code data, including
checkout newline bytes: `-text` disables Git newline conversion. The local
whitespace setting permits captured trailing whitespace and blank EOF lines;
never trim or normalize these bytes. Ordinary text diffs remain visible, and
source/docs whitespace checks outside `*.gcode` in this directory are unchanged.
This data policy also covers the exact `classification-injected.gcode` bytes.

`classification.3mf` derives from the same anchor by appending exactly
`classification-injected.gcode` to `machine_start_gcode` in
`Metadata/project_settings.config` (all other archive entry contents and config
values unchanged). Its actual-AppImage output covers closed external paths
terminated by Wipe, Travel, or inward motion before M204; open external paths;
closed inner-wall paths; overhang continuation; and termination by a different
extrusion role. It includes F-only G1 and intervening M204. No fake reference or
instrumented timing values are used. E-only Wipe, arc/scarf/spiral seam behavior
is deferred, not claimed by this fixture.

The existing private processor `process` byte seam is tested by
`processor/seam_tests.rs`. Replay strips generated M73 updates, restores the
first progress and time placeholders, and leaves all movement/config bytes
intact. It asserts the complete returned byte vector equals the actual output,
without even generator normalization (the input already has the same line).
Optional **test-only** `ARES_SEAM_TEST_ARTIFACTS` retains full input/expected/
actual bytes and never changes the assertions. No production environment hook
or new public API is introduced.

## Repeat tests and external integration

```sh
CARGO_TARGET_DIR=/absolute/new-own-target \
ARES_SEAM_TEST_ARTIFACTS=/absolute/new-evidence/replay \
  cargo nextest run -p ares-core -E 'test(processor::)'
```

Public CLI on published source has unrelated missing logical-cursor commands.
The rectilinear output therefore does not directly expose this cache correction;
its complete raw residual must not be called a pass. The authorized integration
uses **external source copies**, never candidate routing files:

```sh
python3 tests/processor_seam_cache/reproduce.py \
  --source "$PWD" --candidate "$(git rev-parse HEAD)" \
  --partial /home/indexyz/.local/state/ares-parity/2026-09-07-coordinator/wave3-handoffs/logical-cursor-current.patch \
  --wrapper /home/indexyz/ares/scripts/orca-parity.sh \
  --appdir /tmp/squashfs-root \
  --lib-cache /home/indexyz/.local/state/ares-parity/logical-cursor-e098f64/orca-libs.cache \
  --evidence /absolute/new-external-evidence
```

The script requires the exact published base
`057f131b9ebb50538f865ffd770a89a1e79248dd` and partial patch SHA256
`408c9123b51b4d4c752bb443a68a52bdc55f6fa71b40571f59496c77261aace4`.
It builds four explicitly archived source identities in separate owned targets,
records binary/archive hashes and commands/exits, captures fresh actual output,
retains full raw diffs, and compares every byte except one independently
validated generator identity/timestamp line. Anchor/two-wall integration must
pass before and after; wipe-off/rectilinear must fail before and pass after.
Published CLI differences remain separately recorded, not accepted or hidden.

Initial evidence lives under
`/home/indexyz/.local/state/ares-parity/2026-09-07-coordinator/wave6-seam-writer/`:
`baseline-comparisons.json`, `changed-comparisons.json`,
`integration-comparisons.json`, `replay-before/`, `replay-after/`, full diffs,
logs and exits. The wipe-off source-built combo from wave5 qualified only for
wipe-off. Rectilinear combo was disqualified; none of its times are used here.

This is five processor cases/four integration projects, not all profiles,
options, full producer parity, or browser/Windows/macOS runtime certification.
