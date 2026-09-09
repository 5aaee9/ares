// Arachne integration seam 3 tests: the variable-width to constant-width
// sub-path segmentation port of `thick_polyline_to_multi_path`
// (OrcaSlicer `VariableWidth.cpp:5-102`) reached through
// `extrusion_paths_append` (`Arachne/utils/ExtrusionLine.cpp:298-302`).

use crate::{
    arachne::{ExtrusionJunction, ExtrusionLine},
    geometry::{CoordinateScale, Point},
    project_slice::perimeters::{
        arachne::variable_width::append_extrusion_paths,
        classic::materialize::{ExtrusionPath, ExtrusionRole},
        types::Flow,
    },
};

const SCALE: CoordinateScale = CoordinateScale::Normal;
// `scaled<float>(0.05)` split tolerance and `SCALED_EPSILON` merge tolerance.
const TOLERANCE: f64 = 0.05 / SCALE.factor();
const MERGE_TOLERANCE: f64 = 1e-4 / SCALE.factor();
const ROUNDED_RECTANGLE_FACTOR: f32 = (1.0 - 0.25 * std::f64::consts::PI) as f32;

fn flow() -> Flow {
    Flow {
        width: 0.4,
        height: 0.2,
        spacing: 0.35,
        nozzle_diameter: 0.4,
        bridge: false,
        mm3_per_mm: 0.08,
    }
}

fn line(junctions: &[(i64, i64, i64)]) -> ExtrusionLine {
    let mut line = ExtrusionLine::new(0, false);
    for &(x, y, width) in junctions {
        line.push(ExtrusionJunction::new(Point::new(x, y), width, 0));
    }
    line
}

fn convert(junctions: &[(i64, i64, i64)]) -> Vec<ExtrusionPath> {
    let mut paths = Vec::new();
    append_extrusion_paths(
        &mut paths,
        &line(junctions),
        ExtrusionRole::ExternalPerimeter,
        flow(),
        SCALE,
    )
    .expect("the segmentation port converts source-valid junction widths");
    paths
}

fn points(path: &ExtrusionPath) -> Vec<(i64, i64)> {
    path.polyline
        .points
        .iter()
        .map(|point| (point.x, point.y))
        .collect()
}

#[test]
fn uniform_width_junctions_merge_into_one_constant_width_path() {
    let paths = convert(&[
        (0, 0, 200_000),
        (1_000_000, 0, 200_000),
        (2_000_000, 0, 200_000),
    ]);
    assert_eq!(paths.len(), 1);
    assert_eq!(
        points(&paths[0]),
        vec![(0, 0), (1_000_000, 0), (2_000_000, 0)]
    );
    let expected_width = 200_000_f32 * SCALE.factor() as f32 + 0.2 * ROUNDED_RECTANGLE_FACTOR;
    assert_eq!(paths[0].width, expected_width);
    assert_eq!(paths[0].height, 0.2);
    // The sub-path flow is the source flow re-widthed by `Flow::with_width`
    // (`VariableWidth.cpp:69-75`).
    assert_eq!(
        paths[0].mm3_per_mm,
        flow().with_width(expected_width).unwrap().mm3_per_mm
    );
}

#[test]
fn width_ramp_splits_at_the_tolerance_into_chained_sub_paths() {
    let paths = convert(&[(0, 0, 200_000), (4_000_000, 0, 400_000)]);
    // 0.2mm to 0.4mm over 4mm splits into four 1mm sub-lines whose widths
    // step by the 0.05mm tolerance (`VariableWidth.cpp:27-56`).
    assert_eq!(paths.len(), 4);
    let expected_points = [
        vec![(0, 0), (1_000_000, 0)],
        vec![(1_000_000, 0), (2_000_000, 0)],
        vec![(2_000_000, 0), (3_000_000, 0)],
        vec![(3_000_000, 0), (4_000_000, 0)],
    ];
    for (path, expected) in paths.iter().zip(expected_points) {
        assert_eq!(points(path), expected);
    }
    // Each sub-path takes its flow width from the wider endpoint
    // (`fmax(line.a_width, line.b_width)`).
    let expected_widths: Vec<f32> = [250_000_f32, 300_000_f32, 350_000_f32, 400_000_f32]
        .into_iter()
        .map(|width| width * SCALE.factor() as f32 + 0.2 * ROUNDED_RECTANGLE_FACTOR)
        .collect();
    let widths: Vec<f32> = paths.iter().map(|path| path.width).collect();
    assert_eq!(widths, expected_widths);
}

#[test]
fn width_step_beyond_the_merge_tolerance_starts_a_new_path() {
    // Adjacent junction widths differ by less than the split tolerance, so no
    // subdivision happens, but the rebuilt flow widths differ by more than
    // `SCALED_EPSILON`, so the sub-paths split (`VariableWidth.cpp:80-97`).
    let paths = convert(&[
        (0, 0, 200_000),
        (1_000_000, 0, 200_000),
        (2_000_000, 0, 240_000),
    ]);
    assert_eq!(paths.len(), 2);
    assert_eq!(points(&paths[0]), vec![(0, 0), (1_000_000, 0)]);
    assert_eq!(points(&paths[1]), vec![(1_000_000, 0), (2_000_000, 0)]);
    let first_width = paths[0].width;
    let second_width = paths[1].width;
    assert!(
        f64::from((first_width - second_width).abs()) / SCALE.factor() > MERGE_TOLERANCE
            && f64::from((first_width - second_width).abs()) / SCALE.factor() <= TOLERANCE
    );
}

#[test]
fn tiny_line_splices_into_the_current_path_endpoint() {
    // The middle junction duplicates its predecessor, so the middle thick line
    // is shorter than `SCALED_EPSILON` and splices into the running path
    // (`VariableWidth.cpp:16-24`).
    let paths = convert(&[
        (0, 0, 200_000),
        (1_000_000, 0, 200_000),
        (1_000_000, 0, 200_000),
        (2_000_000, 0, 200_000),
    ]);
    assert_eq!(paths.len(), 1);
    assert_eq!(
        points(&paths[0]),
        vec![(0, 0), (1_000_000, 0), (2_000_000, 0)]
    );
}

#[test]
fn trailing_tiny_line_after_a_width_step_stays_spliced() {
    // The 0.2mm to 0.25mm width step finishes the first path; the trailing
    // duplicate junction is tiny and splices into the running second path
    // (`VariableWidth.cpp:16-24`).
    let paths = convert(&[
        (0, 0, 200_000),
        (1_000_000, 0, 200_000),
        (2_000_000, 0, 250_000),
        (2_000_000, 0, 250_000),
    ]);
    assert_eq!(paths.len(), 2);
    assert_eq!(points(&paths[0]), vec![(0, 0), (1_000_000, 0)]);
    assert_eq!(points(&paths[1]), vec![(1_000_000, 0), (2_000_000, 0)]);
    assert_eq!(paths[1].role, ExtrusionRole::ExternalPerimeter);
}
