# Processor isolation input

`project.3mf` is a byte-identical copy of the Coordinator's small parity anchor:
`2026-09-07-coordinator/anchor-baseline/project.3mf`.
SHA256: `200980b8217b4a5f0dfb8022e0a837c662c15fcaaaeca7d7a545eb7be250a63d`.
This fixture exercises the real public 3MF CLI and processor boundary; it does
not claim all-printer/default-option or full Orca output parity. No reference
G-code is loaded by the implementation or substituted into sliced output.

The tests set variables only on child `std::process::Command` instances.
Complete output equality permits only the single validated generator timestamp
line to differ; ordered travel, extrusion, progress, configuration, statistics,
whitespace and trailing bytes all participate in comparison.
