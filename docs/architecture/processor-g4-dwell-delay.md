# Processor G4 dwell delay

## Source-owned bounded decision

Owner: OrcaSlicer 2.4.2 `8500fcdccaa10b5099ac20d252af3a7c560046f1` (read-only),
`src/libslic3r/GCode/GCodeProcessor.cpp::process_G4` (4848-4856):
`process_G4` reads `S` seconds or `P` milliseconds
(`value_s += value_p * 0.001`) and passes the sum to
`simulate_st_synchronize(value_s)` (6011-6013), whose default
`EMoveType::Noop` target attributes the dwell to the first block processed by
the next `calculate_time` pass. The C++ `has_value('S') || has_value('P')`
short-circuits, so a parseable `S` suppresses the `P` lookup entirely.
Command dispatch (1784) is keyed on the exact `G4` command word.

Destination: `ares-core/src/project_slice/gcode_emit/processor/delays.rs::
command_delay`. The `G4` branch returns `S` seconds when `S` parses, else
`P * 0.001`; `estimate.rs::synchronizes_planner` already flushed the planner
for `G4` lines carrying `S`/`P`, and `schedule.rs` already attributes pending
`DelayTarget::Any` seconds to the first block of the next emitted batch, so
the dwell reaches the machine total, the `g1_times_cache` progress values and
`prepare`/first-layer accumulation without further changes.

Included: `G4` dwell seconds in normal-mode total time, M73 progress values
and prepare/first-layer time; `S`-precedence over `P`; milliseconds
conversion. Excluded (unchanged): `M400` combined `S + P * 0.001` spelling,
planner arithmetic, M73 placement/counter semantics, footer sub-second
formatting, stealth mode. Existing estimator remains a temporary
compatibility shell around the cited upstream owner; no new Ares pipeline
behavior.

## Evidence

Red-then-green unit tests in `processor/tests.rs` cover `G4 S10` (+10 s
total), `G4 P500` (+0.5 s), `G4 S10 P500` (`S` precedence), and the M73/footer
shift through `process`. Replay evidence with the generator/timestamp-only
normalization against immutable actual-AppImage artifacts:

- Artillery `case-TQeCzN` (3x `G4 P500`): complete output now byte-identical
  (wave8 strict envelope replay passes; previously DIVERGENT with the
  16m10s-vs-16m12s footer residual).
- CR-10 Max `case-qAOxED` and `case-VOCOfp` (`G4 S10`): footers byte-equal
  at 8m 59s (previously 8m 49s, exactly -10 s); remaining first difference
  moved from line 32 (M73 value) to line 1663 (M73 placement, the separate
  marker-stream cache-attribution defect).
- `case-yiVK0r` (`G4 S10`): footer byte-equal at 8m 55s; remaining first
  difference is M73 placement at line 121.
- The five wave18-reverify PASS printers and the Afinia envelope replay stay
  byte-identical.

Remaining after this slice: M73 marker placement/count attribution
(`g1_times_cache` id capture), sub-second first-layer footer formatting,
Ender-3 V4 ~1 s block-time drift, and the non-timing divergences stay open.
