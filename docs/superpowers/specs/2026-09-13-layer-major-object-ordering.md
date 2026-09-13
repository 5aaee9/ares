# Spec: layer-major object ordering port

## Observable contract

Multi-object plates print layer-major like upstream: every layer chunk
aggregates all objects whose layers share the print z, and the objects
within a chunk follow the chained instance order. The arachne
`default-prisms` full-output test no longer diverges on object order or
`; printing object ... id:` values (remaining divergences there are the
tracked M73/timing and arachne-geometry buckets).

## Source-owned boundaries

- `OrcaSlicer/src/libslic3r/GCode.cpp:1835-1870`
  `collect_layers_to_print(Print)` — merge every print object's layers
  into chunks by print z.
- `OrcaSlicer/src/libslic3r/ShortestPath.cpp:2015-2042`
  `chain_print_object_instances` — multi-fragment greedy TSP over the
  instance centers (`chain_segments_greedy_`, `ShortestPath.cpp:92-410`),
  including the `start_near` seed endpoint, chain-id union-find and the
  final walk.
- `GCode.cpp:5114-5131` — the per-layer call seeds the chain with the
  wipe-tower point (`Point wt_pos(x, y)` truncates the millimetre config
  values into scaled integers, so the seed sits microns from the origin)
  and REVERSES the resulting path.
- `GCode.cpp:4118-4167` `sort_print_object_instances` — the chunk's
  objects emit in that reversed chained order, each around its own
  `set_origin(unscale(instance.shift))`.
- `GCode.cpp:8074-8100, 2697` `set_object_info` — `PrintObject::m_id`
  (the `; printing object ... id:` value) is only assigned when the
  exclude-object labeling runs; it stays 0 otherwise.

## Rust destination boundary

`ares-core::project_slice::gcode_emit::layers` owns the port:

- `layers/object_order.rs` — the chained instance order (scaled integer
  distances, seed endpoint, union-find, heap of pending endpoints).
- `layers.rs::append` — merged layer-chunk loop (print-z groups, chained
  entry order, per-entry origin/labels/geometry, per-chunk boundary,
  skirt, brim, cooling and layer-end timelapse).
- `layers/boundary.rs` — the once-per-chunk `change_layer` block, now
  receiving the chunk's z/height from the merged schedule.
- `gcode_emit::footprint::object_center` still supplies the instance
  centers; `gcode_emit::object` gates the object-id assignment on the
  exclude-object labeling.

## Included

Merged layer chunks, chained per-chunk object order (seeded, reversed),
per-object origins inside the chunk, print preamble outside the cooling
rewrite window, layer-end timelapse per chunk, object-id gating, and the
property tests for the chain (collinearity, uniqueness, cluster
priority, prisms order).

## Deferred

- Differing per-object layer heights inside one chunk (upstream groups
  by print z with EPSILON; Ares groups by exact accumulated z — the
  common equal-height case is exact, mixed-height plates may chunk
  differently until `collect_layers_to_print`'s EPSILON pairing lands).
- The remaining arachne prisms divergences (M73 placement, arachne
  perimeter geometry) stay tracked as their own buckets.
