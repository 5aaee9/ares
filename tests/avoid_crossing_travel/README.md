# Avoid-crossing travel fixture

## Current bounded status (wave16)

All six CLI regressions now pass unchanged, including all436 one-wall ordered
XY commands and the complete one-wall artifact. Seven fresh actual-AppImage
contexts match complete outputs after only validated generator normalization;
repeats are stable. Empty-boundary direct travel and the separately authorized
role-END fan-emission correction close the historical failures below. Fixtures
and reference bytes are unchanged. See
`docs/superpowers/plans/2026-09-08-route-boundary-parity.md` for source ownership,
full artifacts/effective exports and validation receipts. This bounded candidate
still requires independent review and does not complete the1001-printer/legal-
domain/range or Tier1 runtime goal. The following capture history is retained
for provenance, not as the current failure status.

`anchor.3mf` is the supplied cube10 printer-option anchor project, unchanged.
`anchor.orca.gcode` is complete output from independently running OrcaSlicer
2.4.2's AppImage executable, SHA256
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.

Generation (exit 0):

```sh
ORCA_APPDIR=/tmp/squashfs-root /home/indexyz/ares/scripts/orca-parity.sh \
  --slice 0 \
  --outputdir /home/indexyz/.local/state/ares-parity/travel-fix-322a3890/orca-anchor \
  "$PWD/tests/avoid_crossing_travel/anchor.3mf"
```

The external CLI regression compares **every ordered `G1 X...` command without
an E field**, without removing repeated rounded coordinates. It does not certify
other G-code commands, timing, M73, configuration, statistics, or whole-output
parity. Full reference bytes are retained for independent review.

Current-source TDD baseline: 700 XY commands; reference: 770. The recovered
lifecycle/filter component restores 70 commands but remains red at six sparse
travel destinations. Separating the generator cursor from writer-only inward
motion fixes all six. `logical_cursor_parity` additionally checks complete
layers for the three reversed runs and complete anchor bytes (only the
ARD-0023 generator line is normalized).

`two-walls.3mf` changes only `wall_loops` to 2 in the anchor configuration;
`two-walls.orca.gcode` is fresh complete AppImage output (same executable above),
produced on 2026-09-08 at 07:07:46. Its complete-artifact regression first failed
on an extra corner command when routing started at the writer's inward point.
Routing from the logical cursor fixes it without suppressing duplicate commands.

This is still a partial worktree, not an accepted candidate: the required
one-wall mutation retains travel defects and wipe-off retains an M73 placement
residual. Complete mutation inputs/outputs/effective settings and raw differences
are preserved in `/home/indexyz/.local/state/ares-parity/logical-cursor-e098f64/`.
See the scoped logical-cursor plan for all validation results and residuals.

Use `cargo nextest run -p ares-cli --test avoid_crossing_travel`.
`ARES_TRAVEL_BIN` optionally selects an independently built baseline binary for
red verification; otherwise the current worktree's Cargo-built CLI is used.

## One-wall extension regression (wave5, still red)

`one-wall.3mf` changes the anchor's `wall_loops` to1; input SHA256:
`ae9cde3053ba1a22be87a7cc4677b71c088b24f2741d9311a9723a58f922140b`.
`one-wall.orca.gcode` is verbatim fresh output from the same actual AppImage
above, generated2026-09-08 at08:43:12, SHA256:
`783be63a6380a30c2ca5f329d63e9a2e6f552a3133912aa221ca7353113ebb00`.
Command (exit0), with D equal to the wave5 evidence root below:

```sh
ORCA_LIB_CACHE="$D/orca-libs.cache" ORCA_APPDIR=/tmp/squashfs-root \
  /home/indexyz/ares/scripts/orca-parity.sh --slice 0 \
  --outputdir "$D/mutations/one-wall/orca" \
  "$PWD/tests/avoid_crossing_travel/one-wall.3mf"
```

`closest_line_extension` checks all436 ordered XY commands and independently
the complete artifact, normalizing only the validated generator line. Both
fail before the extension fix (411 XY) and afterward (443 XY); no reference
bytes or assertions were changed to accept those failures. The original anchor
and two-wall tests stay unchanged and pass. The fixture-local G-code attributes
preserve verbatim whitespace and newlines; they do not relax source checks.

D=`/home/indexyz/.local/state/ares-parity/2026-09-07-coordinator/wave5-route-writer/`.
A second fresh reference capture in the seven-case replay differs only by its
generator timestamp; full raw outputs/diffs and provenance are external.
This data is a failing regression in a preserved partial patch, not an accepted
candidate or a claim of full option/printer/Tier1 parity.

## Wave14 partial preservation

Fixture bytes and existing CLI assertions are unchanged. The FFD+append
checkpoint reproduces438/436; the shortest-direction endpoint-operand correction
repairs the six wrong corners but retains438/436 due to the final Z10.4 detour.
The two required one-wall tests still fail. All six other fresh actual-AppImage
contexts are complete-green; this is not accepted route parity or a candidate.
See the latest disposition in `docs/superpowers/plans/2026-09-08-route-boundary-parity.md`.
