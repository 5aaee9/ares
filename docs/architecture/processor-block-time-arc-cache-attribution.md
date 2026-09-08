# Processor block-time trapezoid and arc cache attribution

## Source-owned bounded decision

Owner: OrcaSlicer 2.4.2 `8500fcdccaa10b5099ac20d252af3a7c560046f1` (read-only),
`src/libslic3r/GCode/GCodeProcessor.cpp`:

- `calculate_trapezoid` (255-274) with `estimated_acceleration_distance`
  (127-129), `intersection_distance` (131-132), `speed_from_distance`
  (134-138), `acceleration_time_from_distance` (140-143): every phase time
  derives from an accel/decel DISTANCE; a distance whose velocity difference
  points the wrong way clamps to zero (`std::max(0.0f, ...)`), so the phase
  contributes zero time, never a negative one.
- `TimeBlock::time()` / `Trapezoid` (`GCodeProcessor.hpp:436-476`):
  `acceleration_time(entry, accel) + cruise_time() +
  deceleration_time(distance, accel)` with the zero-`cruise_feedrate` guard.
- Arc `g1_times_cache` attribution: each discretized internal `process_G1`
  consumes its own `m_g1_line_id` (`GCodeProcessor.cpp:3868`) carrying
  `remaining_internal_g1_lines = segments - i` (4761-4763, 4813-4815); the
  cache entry push (574) skips non-Extrude/Travel/Wipe blocks (485-487); the
  export loop calls `process_line_move(g1_lines_counter +
  internal_g1_lines_counter)` for `G2`/`G3` lines (1466) so the arc line's
  own marker reads the second-to-last internal segment's entry and the
  trailing segment lands on the next motion line's lookup.

Destination: `ares-core/src/project_slice/gcode_emit/processor/motion/planner.rs::
block_time` (+ `speed_from_distance`/`acceleration_time_from_distance`) and
`processor/estimate.rs` arc `block_line_ids` per-segment assignment and
`G2`/`G3` lookup id. The previous velocity-difference shortcut
`(cruise-entry)/accel + cruise + (cruise-exit)/accel` produced negative
accel/decel terms whenever `exit > cruise` (or `entry > cruise`) — BIQU B1
END-code `G1 E-2 Z0.2 F2400` block (cruise 6, entry 0.3, exit 10, accel 100,
distance 0.2) lost exactly 0.04 s. The previous shared arc id made the arc
line's lookup read no arc segment at all, deferring the traversal time past
the next motion line (FLSun S1 start-gcode purge arc).

Included: distance-based trapezoid times for all blocks (including the
clamped-intersection short-block branch); per-internal-segment g1 line ids
for arcs; arc line lookup id `counter + internal`. Excluded (unchanged):
footer sub-second `%fs` formatting (`time.rs::duration`, wave19 group G-B),
marker-stream ordering/seam placement (wave19 group G-C remainder), G4 `M400`
combined spelling, stealth mode. Existing estimator remains a temporary
compatibility shell around the cited upstream owner; no new Ares pipeline
behavior.

## Evidence

Red-then-green tests in `processor/trapezoid_tests.rs` (wrong-direction
decel/accel contribute zero time, BIQU block state) and
`processor/arc_cache_tests.rs` (arc traversal time attributed to the arc
line's own marker). Three precision pins in `processor/tests.rs` shifted by
~3.2e-6 s because upstream computes phase times through `sqrt` round-trips,
not the algebraic shortcut; `processor/zero_acceleration_tests.rs` helical
bytes now include the `M73` marker at the arc line (segment 15 of 16), which
is what upstream's lookup emits.

Replay evidence with the generator/timestamp-only normalization against
immutable wave18 actual-AppImage artifacts:

- The five wave18-reverify PASS printers stay byte-identical: Afinia H+1(HS)
  (33 arcs) and BLOCKS Pro S100 (12 arcs) included — the per-segment lookup
  reproduces Orca's emission decisions exactly where the totals already
  matched.
- FLSun S1 `case-WXegdS`: first difference moved from line 34 (M73 value
  after the start-gcode purge arc, `P6 R1` vs `P17 R0`) to line 414; only 12
  diff lines remain (marker-stream ordering class).
- BIQU B1 `case-bfdChK`: the line-224 M73 placement first difference is gone;
  the only remaining diff is the known G-B footer `%fs` line.
- KSR `project_matches_orca_242_semantically`: unchanged +3 s residual at
  byte 96 (separate root cause, wave20 reverify lane).
