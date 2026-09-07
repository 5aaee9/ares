# Fail-closed replay fixtures (partial semantic evidence only)

Captured 2026-09-08 by an independent execution of the unmodified OrcaSlicer
2.4.2 AppImage. Source reference: OrcaSlicer
`8500fcdccaa10b5099ac20d252af3a7c560046f1`; no source-token acceptance test.

Executable: `/tmp/squashfs-root/bin/orca-slicer`, SHA-256
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.
Wrapper: `/home/indexyz/ares/scripts/orca-parity.sh` with
`ORCA_APPDIR=/tmp/squashfs-root`.

Inputs and full reference outputs:

| File | SHA-256 |
| --- | --- |
| `../orca_cli_ender3.3mf` (existing original) | `7fb06c23baed9c8e06757cc0f7128c827e0b232eaa2655af687e4708014037c5` |
| `ender3.orca.gcode` | `cfcab0b074ce4c92693196b2df492b65c19797d17f62cbd5bc340ee74462fd3f` |
| `ender3-classic.3mf` | `abd8882f86e789f727fe362a11e3b57eb52e846c712b7a0b102b0a4fe3d05bc3` |
| `ender3-classic.orca.gcode` | `3ed255be8081c66c1a512a838040752cdb076d3aeae9be9a632d0f2e43fabb97` |

The classic input is the original ZIP with only the effective
`Metadata/project_settings.config` value `wall_generator` changed from
`arachne` to `classic` (JSON whitespace reserialized; all other entry bytes
preserved). It is a separately labelled harness fixture, **not** a default
printer or all-domain acceptance case. The original is retained and Ares still
rejects it with `unsupported project feature: wall_generator`.

Actual commands (both exit 0), with `REPO` equal to the managed worktree
`/home/indexyz/ares.pi-subagents-harness-fix-d869825-34be-s0-t0` and `E` equal to
`/home/indexyz/.local/state/ares-parity/2026-09-08-harness-w1`:

```sh
ORCA_APPDIR=/tmp/squashfs-root /home/indexyz/ares/scripts/orca-parity.sh \
  --slice 0 --outputdir "$E/orca-ender3" "$REPO/tests/parity/orca_cli_ender3.3mf"
ORCA_APPDIR=/tmp/squashfs-root /home/indexyz/ares/scripts/orca-parity.sh \
  --slice 0 --outputdir "$E/orca-classic" "$REPO/tests/parity/replay/ender3-classic.3mf"
```

Full logs: `$E/orca-ender3.log`, `$E/orca-classic.log`; complete output folders
are the respective `--outputdir` paths. `plate_1.gcode` was copied verbatim to
this directory. Input and reference bytes above are Git data fixtures, not
production behavior or source splitting. The fixture-local `.gitattributes`
preserves exact G-code bytes across platform checkouts (`-text`) and exempts
only upstream trailing-space/EOF-blank data from whitespace checks. Normal text
diffs stay visible; Rust/docs/general whitespace checks are unchanged. Do not
trim or normalize these independently captured bytes.

The divergence regression changes one `M104 S` to `M104 S9` in a temporary
copy of the real classic reference. It must fail with DIVERGENT and save both
full streams. This is a falsifiable harness mutation, not a repaired Ares case.
The unmodified classic pair passes only `compare_ignoring_time`; its whole
output still differs substantially (initial capture: 6,099 unified-diff lines,
including M73 and motion ordering). No full-output parity is claimed.

The legacy replay layout contains no producer manifest. Its runtime manifest
therefore records reference producer provenance as unknown even for test copies
of these bytes; this README provides the independent capture provenance, not a
fabricated cache attestation. Runtime manifests bind saved bytes by SHA-256 and
identify the actual executing Ares test binary by SHA-256. Build revision is an
optional compile-time `ARES_PARITY_BUILD_REVISION` claim, explicitly labelled as
such; absent claims are unknown, not guessed from the runtime checkout.

To replay: create an external input directory containing `case.3mf` and
`case/plate_1.gcode`, then set `ARES_PARITY_REPLAY` to that directory and
`ARES_PARITY_ARTIFACT_ROOT` to a fresh absolute external directory. Run:

```sh
ARES_PARITY_BUILD_REVISION="$(git rev-parse HEAD)" \
  cargo nextest run -p ares-cli --test orca_parity \
  -E 'test(=replay::orca_parity_replay_sweep)'
```

Only use a clean revision claim for a clean build. A replay report is
`replay-summary.json`; per-case directories retain available `input.3mf`,
`orca.gcode`, `ares.gcode`, `error.txt` on failure, and `manifest.json`.
Files are never overwritten. Reusing a root with a summary fails instead of
replacing old evidence. An unset replay environment is an offline skip, not a
parity execution. Other live runner/default/domain/cache/multi-plate gaps remain
explicitly deferred in the convergence spec.
