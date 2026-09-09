// Variable-width to constant-width sub-path segmentation from OrcaSlicer
// v2.4.2 `VariableWidth.cpp:5-102::thick_polyline_to_multi_path`, reached by
// arachne through `extrusion_paths_append(ExtrusionLine)`
// (`Arachne/utils/ExtrusionLine.cpp:298-302`): junction widths clip into
// constant-width `ExtrusionPath`s whose flow width is rebuilt per sub-path.

use crate::{
    SliceError,
    arachne::ExtrusionLine,
    geometry::{CoordinateScale, Line, Point, ThickLine, ThickPolyline},
    project_slice::perimeters::{
        classic::materialize::{ExtrusionPath, ExtrusionRole, Point3, Polyline3},
        types::Flow,
    },
};

const SPLIT_TOLERANCE_MM: f64 = 0.05;
#[cfg_attr(not(test), allow(dead_code))]
const MERGE_TOLERANCE_MM: f64 = 1e-4;

/// `extrusion_paths_append(dst, extrusion, role, flow)`
/// (`Arachne/utils/ExtrusionLine.cpp:298-302`): append the constant-width
/// sub-paths of one variable-width arachne wall line.
#[cfg_attr(not(test), allow(dead_code))]
pub(in crate::project_slice) fn append_extrusion_paths(
    destination: &mut Vec<ExtrusionPath>,
    extrusion: &ExtrusionLine,
    role: ExtrusionRole,
    flow: Flow,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    destination.extend(thick_polyline_to_multi_path(
        &extrusion.to_thick_polyline(),
        role,
        flow,
        scale,
    )?);
    Ok(())
}

/// `thick_polyline_to_multi_path` returns one multi-path; callers flatten its
/// sub-paths, so the sub-path vector is the slice this seam owns.
fn thick_polyline_to_multi_path(
    thick_polyline: &ThickPolyline,
    role: ExtrusionRole,
    flow: Flow,
    scale: CoordinateScale,
) -> Result<Vec<ExtrusionPath>, SliceError> {
    let tolerance = SPLIT_TOLERANCE_MM / scale.factor();
    let merge_tolerance = MERGE_TOLERANCE_MM / scale.factor();
    let mut lines = thick_polyline.thicklines();
    let mut paths: Vec<ExtrusionPath> = Vec::new();
    let mut path: Option<ExtrusionPath> = None;

    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        let line_len = Line::new(line.a, line.b).length();
        if line_len < merge_tolerance {
            // The line is so tiny that we don't care about its width when we
            // connect it to another line (`VariableWidth.cpp:16-24`).
            connect_tiny_line(&mut path, &mut paths, &mut lines, index, line);
            index += 1;
            continue;
        }

        let thickness_delta = (line.a_width - line.b_width).abs();
        if thickness_delta > tolerance {
            split_thick_line(&mut lines, index, line, line_len, tolerance);
            continue;
        }

        let width = line.a_width.max(line.b_width) as f32;
        let new_flow = if role == ExtrusionRole::OverhangPerimeter && flow.bridge {
            flow
        } else {
            flow.with_width(
                width * scale.factor() as f32
                    + flow.height * (1.0 - 0.25 * std::f64::consts::PI) as f32,
            )?
        };
        match path.as_mut() {
            None => {
                path = Some(ExtrusionPath {
                    polyline: Polyline3 {
                        points: vec![point3(line.a), point3(line.b)],
                        fitting: Vec::new(),
                    },
                    role,
                    can_reverse: true,
                    mm3_per_mm: new_flow.mm3_per_mm,
                    width: new_flow.width,
                    height: new_flow.height,
                });
            }
            Some(current) => {
                let width_delta =
                    f64::from((current.width - new_flow.width).abs()) / scale.factor();
                if width_delta <= merge_tolerance {
                    // Width difference within the accepted tolerance.
                    current.polyline.points.push(point3(line.b));
                } else {
                    // Initialize a new path from this line.
                    paths.push(path.take().expect("the current path exists"));
                    continue;
                }
            }
        }
        index += 1;
    }

    if let Some(path) = path.filter(|path| path.polyline.points.len() >= 2) {
        paths.push(path);
    }
    Ok(paths)
}

/// Tiny-line connection (`VariableWidth.cpp:16-24`): splice the line into the
/// current path, the next line, or the last finished path.
fn connect_tiny_line(
    path: &mut Option<ExtrusionPath>,
    paths: &mut [ExtrusionPath],
    lines: &mut [ThickLine],
    index: usize,
    line: ThickLine,
) {
    if let Some(current) = path.as_mut() {
        *current
            .polyline
            .points
            .last_mut()
            .expect("a started path has an endpoint") = point3(line.b);
    } else if index + 1 < lines.len() {
        lines[index + 1].a = line.a;
    } else if let Some(last) = paths.last_mut() {
        *last
            .polyline
            .points
            .last_mut()
            .expect("a finished path has an endpoint") = point3(line.b);
    }
}

/// Width-ramp subdivision (`VariableWidth.cpp:27-56`): replace one thick line
/// with constant-width sub-lines spaced by the split tolerance.
fn split_thick_line(
    lines: &mut Vec<ThickLine>,
    index: usize,
    line: ThickLine,
    line_len: f64,
    tolerance: f64,
) {
    let segments = ((line.a_width - line.b_width).abs() / tolerance).ceil() as usize;
    let segment_length = line_len / segments as f64;
    let dx = (line.b.x() - line.a.x()) as f64;
    let dy = (line.b.y() - line.a.y()) as f64;
    let norm = (dx * dx + dy * dy).sqrt();
    let direction = (dx / norm, dy / norm);
    let mut points = Vec::with_capacity(segments + 1);
    let mut widths = Vec::with_capacity(segments * 2);
    points.push(line.a);
    widths.push(line.a_width);
    for segment in 1..segments {
        let distance = segment as f64 * segment_length;
        points.push(Point::new(
            (line.a.x() as f64 + direction.0 * distance) as i64,
            (line.a.y() as f64 + direction.1 * distance) as i64,
        ));
        let width = line.a_width + distance * (line.b_width - line.a_width) / line_len;
        widths.extend([width, width]);
    }
    points.push(line.b);
    widths.push(line.b_width);
    let replacements = (0..segments).map(|segment| {
        ThickLine::with_widths(
            points[segment],
            points[segment + 1],
            widths[2 * segment],
            widths[2 * segment + 1],
        )
    });
    lines.splice(index..=index, replacements);
}

const fn point3(point: Point) -> Point3 {
    Point3 {
        x: point.x(),
        y: point.y(),
        z: 0,
    }
}
