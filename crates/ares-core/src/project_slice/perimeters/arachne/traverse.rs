// `traverse_extrusions` port from OrcaSlicer v2.4.2
// `PerimeterGenerator.cpp:370-574` (called from `process_arachne` at
// `PerimeterGenerator.cpp:2471`): converts ordered arachne wall lines into
// the typed `ExtrusionEntityCollection` vocabulary — closed lines become
// `ExtrusionLoop` entities (`:529-545`) and open lines become
// `ExtrusionMultiPath` entities that split on discontinuities (`:546-566`).
// The overhang-clipping branch (`:391-519`, `clip_extrusion` Z interpolation,
// steep-overhang detection and start-point rechaining) and the junction
// fuzzifier (`Feature/FuzzySkin/FuzzySkin.cpp:685`) are later seams: they fail
// closed with a typed `UnsupportedProjectFeature` instead of falling back.

use crate::{
    ProcessWallDirection, ProcessWallSequence, SliceError,
    arachne::ExtrusionLine,
    geometry::{
        BoundingBox, CoordinateScale, Polygon, clip_clipper_polygons_with_subject_bbox,
        clip_open_path_widths,
    },
    perimeters::FuzzySkinConfig,
    project_slice::perimeters::{
        classic::{
            chained_loops::{ExtrusionLoop, ExtrusionLoopRole},
            entity_collections::{
                ExtrusionEntity, ExtrusionEntityCollection, ExtrusionMultiPath,
                OrderedExtrusionLoop, orient_loop,
            },
            materialize::{ExtrusionPath, ExtrusionRole},
            shortest_path::chain_and_reorder_extrusion_paths,
        },
        types::Flow,
    },
};

use super::variable_width;

/// `PerimeterGeneratorArachneExtrusion` (`PerimeterGenerator.cpp:363-367`):
/// one ordered wall line plus the contour flag computed by the candidate
/// traversal (`is_contour()` at `:2363`).
#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct PerimeterGeneratorArachneExtrusion {
    pub(in crate::project_slice) extrusion: ExtrusionLine,
    pub(in crate::project_slice) is_contour: bool,
}

/// The `PerimeterGenerator` fields `traverse_extrusions` reads
/// (`PerimeterGenerator.cpp:370-574`): `lower_slices_polygons` is the
/// nozzle-half-grown lower slice set prepared by `process_arachne`
/// (`PerimeterGenerator.cpp:2107-2111`).
#[derive(Clone, Copy)]
pub(in crate::project_slice) struct TraverseExtrusionsContext<'a> {
    pub(in crate::project_slice) layer_id: usize,
    pub(in crate::project_slice) raft_layers: i32,
    pub(in crate::project_slice) detect_overhang_wall: bool,
    pub(in crate::project_slice) overhang_reverse: bool,
    pub(in crate::project_slice) wall_direction: ProcessWallDirection,
    pub(in crate::project_slice) wall_sequence: ProcessWallSequence,
    pub(in crate::project_slice) fuzzy_skin: FuzzySkinConfig,
    pub(in crate::project_slice) perimeter_flow: Flow,
    pub(in crate::project_slice) ext_perimeter_flow: Flow,
    pub(in crate::project_slice) overhang_flow: Flow,
    pub(in crate::project_slice) lower_slices_polygons: &'a [Polygon],
    pub(in crate::project_slice) scale: CoordinateScale,
}

/// `traverse_extrusions` returns the collection plus the steep-overhang flags
/// its caller feeds to `reorient_perimeters` (`PerimeterGenerator.cpp:2472`).
#[derive(Debug)]
pub(in crate::project_slice) struct TraverseExtrusionsOutcome {
    pub(in crate::project_slice) collection: ExtrusionEntityCollection,
    // Consumed by `reorient_perimeters` (`PerimeterGenerator.cpp:2472-
    // 2474`), a deferred seam behind the `overhang_reverse` scope gate.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::project_slice) steep_overhang_contour: bool,
    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::project_slice) steep_overhang_hole: bool,
}

pub(in crate::project_slice) fn traverse_extrusions(
    pg_extrusions: Vec<PerimeterGeneratorArachneExtrusion>,
    context: &TraverseExtrusionsContext,
) -> Result<TraverseExtrusionsOutcome, SliceError> {
    // Detect steep overhangs. Only calculate overhang degree on even (from
    // GUI POV) layers (`PerimeterGenerator.cpp:375-377`).
    let overhangs_reverse = context.overhang_reverse && context.layer_id % 2 == 1;
    let above_raft = i32::try_from(context.layer_id).is_ok_and(|layer| layer > context.raft_layers);
    let thin_wall_context = pg_extrusions.len() == 2;

    let mut collection = ExtrusionEntityCollection::default();
    let mut steep_overhang_contour = false;
    let mut steep_overhang_hole = false;

    for pg_extrusion in pg_extrusions {
        let extrusion = pg_extrusion.extrusion;
        if extrusion.junctions.is_empty() {
            continue;
        }

        let is_external = extrusion.inset_index == 0;
        let role = if is_external {
            ExtrusionRole::ExternalPerimeter
        } else {
            ExtrusionRole::Perimeter
        };

        let is_contour = !extrusion.is_closed || pg_extrusion.is_contour;
        // `apply_fuzzy_skin(extrusion, perimeter_generator, is_contour)`
        // (`PerimeterGenerator.cpp:387`, `FuzzySkin.cpp:685`) — the junction
        // fuzzifier is a later seam; an active skin fails closed.
        if context
            .fuzzy_skin
            .should_fuzzify(context.layer_id, extrusion.inset_index, is_contour)
        {
            return Err(unsupported("fuzzy_skin"));
        }

        let mut paths: Vec<ExtrusionPath> = Vec::new();
        if context.detect_overhang_wall && above_raft {
            append_overhang_clipped_paths(&mut paths, &extrusion, role, context)?;
        } else {
            if overhangs_reverse && above_raft {
                // Always reverse if detect overhang wall is not enabled
                // (`PerimeterGenerator.cpp:520-524`).
                steep_overhang_contour = true;
                steep_overhang_hole = true;
            }
            variable_width::append_extrusion_paths(
                &mut paths,
                &extrusion,
                role,
                if is_external {
                    context.ext_perimeter_flow
                } else {
                    context.perimeter_flow
                },
                context.scale,
            )?;
        }

        // Append paths to collection (`PerimeterGenerator.cpp:528-566`).
        if paths.is_empty() {
            continue;
        }
        if extrusion.is_closed {
            let mut extrusion_loop = ExtrusionLoop {
                paths,
                role: if pg_extrusion.is_contour {
                    ExtrusionLoopRole::Default
                } else {
                    ExtrusionLoopRole::Hole
                },
            };
            // ExtrusionLoops occasionally have significantly disconnected
            // paths, triggering these asserts upstream too
            // (`PerimeterGenerator.cpp:537-543`).
            debug_assert!(
                extrusion_loop
                    .paths
                    .iter()
                    .skip(1)
                    .all(|path| path.polyline.points.len() >= 2)
            );
            debug_assert!(
                extrusion_loop
                    .paths
                    .windows(2)
                    .all(|pair| pair[0].polyline.points.last() == pair[1].polyline.points.first())
            );
            debug_assert_eq!(
                extrusion_loop
                    .paths
                    .first()
                    .and_then(|path| path.polyline.points.first()),
                extrusion_loop
                    .paths
                    .last()
                    .and_then(|path| path.polyline.points.last())
            );
            orient_loop(
                &mut extrusion_loop,
                context.wall_direction,
                pg_extrusion.is_contour,
                thin_wall_context,
            );
            collection
                .entities
                .push(ExtrusionEntity::Loop(OrderedExtrusionLoop {
                    extrusion_loop,
                    // Arachne loops keep the `ExtrusionEntity::inset_idx` base
                    // default (`ExtrusionEntity.hpp`); classic sets loop depth.
                    inset_idx: -1,
                }));
            // Orca: Reverse the order of paths for thin wall holes. We define
            // thin wall hole as a hole with only one perimeter
            // (`PerimeterGenerator.cpp:546-548`).
            let thin_wall_hole = !pg_extrusion.is_contour && thin_wall_context;
            if thin_wall_hole && context.wall_sequence != ProcessWallSequence::OuterInner {
                collection.entities.reverse();
            }
        } else {
            debug_assert!(
                paths
                    .iter()
                    .skip(1)
                    .all(|path| path.polyline.points.len() >= 2)
            );
            debug_assert!(
                paths
                    .windows(2)
                    .all(|pair| pair[0].polyline.points.last() == pair[1].polyline.points.first())
            );
            append_open_multi_path(&mut collection, paths);
        }
    }

    Ok(TraverseExtrusionsOutcome {
        collection,
        steep_overhang_contour,
        steep_overhang_hole,
    })
}

/// Open wall lines append `ExtrusionMultiPath` entities, splitting into a new
/// entity when consecutive sub-paths do not chain end-to-start
/// (`PerimeterGenerator.cpp:546-566`).
fn append_open_multi_path(collection: &mut ExtrusionEntityCollection, paths: Vec<ExtrusionPath>) {
    let mut paths = paths.into_iter();
    let mut current = vec![paths.next().expect("an open wall line has a path")];
    for path in paths {
        let chains = current
            .last()
            .expect("a multi-path keeps its last sub-path")
            .polyline
            .points
            .last()
            == path.polyline.points.first();
        if !chains {
            collection
                .entities
                .push(ExtrusionEntity::MultiPath(ExtrusionMultiPath {
                    paths: std::mem::take(&mut current),
                }));
        }
        current.push(path);
    }
    collection
        .entities
        .push(ExtrusionEntity::MultiPath(ExtrusionMultiPath {
            paths: current,
        }));
}

fn unsupported(key: &str) -> SliceError {
    SliceError::UnsupportedProjectFeature(key.to_owned())
}

const SCALED_EPSILON_MM: f64 = 1e-4;

/// The overhang detection branch of `traverse_extrusions`
/// (`PerimeterGenerator.cpp:389-524`): intersect the variable-width wall
/// polyline with the nozzle-half-grown lower slices, append the outside
/// remainder as overhang perimeter, then re-chain from a start point. The
/// upstream `clip_extrusion` (`:311-357`) interpolates the wall width at
/// true subject/clip crossings; this engine marks those crossings with
/// sentinel widths, so any non-positive output width fails closed instead
/// of emitting an interpolated fallback.
fn append_overhang_clipped_paths(
    paths: &mut Vec<ExtrusionPath>,
    extrusion: &ExtrusionLine,
    role: ExtrusionRole,
    context: &TraverseExtrusionsContext<'_>,
) -> Result<(), SliceError> {
    let subject = extrusion
        .junctions
        .iter()
        .map(|junction| (junction.point.x(), junction.point.y(), junction.width))
        .collect::<Vec<_>>();
    let mut bounds = BoundingBox::from_polygon(&extrusion.to_polygon())
        .ok_or_else(|| unsupported("detect_overhang_wall"))?;
    bounds.offset(
        context
            .scale
            .checked_scale(SCALED_EPSILON_MM)
            .ok_or_else(|| unsupported("detect_overhang_wall"))?,
    );
    let lower = clip_clipper_polygons_with_subject_bbox(context.lower_slices_polygons, bounds);
    let clip = |operation| {
        clip_open_path_widths(&subject, &lower, operation).map_err(|_| {
            SliceError::InvalidInput(
                "Arachne overhang clipping is outside the supported Clipper range".to_owned(),
            )
        })
    };
    let is_external = extrusion.inset_index == 0;
    let inside = clip(crate::geometry::ClipOperation::Intersection)?;
    let overhang = clip(crate::geometry::ClipOperation::Difference)?;
    if inside
        .iter()
        .chain(&overhang)
        .any(|path| path.iter().any(|&(_, _, width)| width <= 0))
    {
        return Err(unsupported("detect_overhang_wall"));
    }
    variable_width::append_clipped_paths(
        paths,
        &inside,
        role,
        if is_external {
            context.ext_perimeter_flow
        } else {
            context.perimeter_flow
        },
        context.scale,
    )?;
    variable_width::append_clipped_paths(
        paths,
        &overhang,
        ExtrusionRole::OverhangPerimeter,
        context.overhang_flow,
        context.scale,
    )?;
    if !paths.is_empty() {
        // Reapply the nearest point search for a starting point
        // (`PerimeterGenerator.cpp:437-462,499-500`).
        let start = if extrusion.is_closed {
            paths[0].polyline.points[0]
        } else {
            select_open_start(paths)
        };
        chain_and_reorder_extrusion_paths(paths, [start.x, start.y]);
    }
    Ok(())
}

/// Start-point preference for open walls (`PerimeterGenerator.cpp:441-463`):
/// the only end occurring once, preferring a non-overhang end. Upstream scans
/// an unordered map; first-seen order keeps the selection deterministic.
fn select_open_start(
    paths: &[ExtrusionPath],
) -> crate::project_slice::perimeters::classic::materialize::Point3 {
    let start = paths[0].polyline.points[0];
    let mut order = Vec::new();
    let mut occurrence = std::collections::HashMap::new();
    for path in paths {
        let first = path.polyline.points[0];
        let last = *path.polyline.points.last().expect("a path has an endpoint");
        let overhang = path.role == ExtrusionRole::OverhangPerimeter;
        for point in [first, last] {
            let entry = occurrence.entry((point.x, point.y)).or_insert_with(|| {
                order.push((point.x, point.y));
                (0_usize, false)
            });
            entry.0 += 1;
            entry.1 |= overhang;
        }
    }
    let mut fallback = None;
    for key in order {
        let Some(&(occurrences, overhang)) = occurrence.get(&key) else {
            continue;
        };
        if occurrences != 1 {
            continue;
        }
        let point = paths
            .iter()
            .flat_map(|path| path.polyline.points.iter())
            .find(|point| (point.x, point.y) == key)
            .expect("the occurrence key comes from a path point");
        if !overhang {
            return *point;
        }
        fallback = fallback.or(Some(*point));
    }
    fallback.unwrap_or(start)
}
