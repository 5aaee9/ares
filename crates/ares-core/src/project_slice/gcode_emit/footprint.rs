//! Per-instance object geometry for Klipper/Marlin object exclusion and
//! first-layer/timelapse bounds helpers.

use crate::geometry::{CoordinateScale, Point, douglas_peucker};
use crate::project_slice::gcode_emit::{
    brim::BrimPlan,
    skirt::{SkirtPlan, convex_hull},
};
use crate::project_slice::gcode_emit::{format_processor_float, tags::Tags};
use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;
use crate::{ProjectInstance, ProjectObject, ProjectVolumeType};

pub(super) type FirstLayerBounds = (f64, f64, f64, f64);

pub(super) fn first_layer_bounds(
    traversal: &PreparedPostClassicTraversal,
    skirt: Option<&SkirtPlan>,
    brim: Option<&BrimPlan>,
) -> Option<FirstLayerBounds> {
    let print = &traversal.resolved.views.full.process.print;
    let skirt_height = usize::try_from(print.skirt_height.0).unwrap_or_default();
    let infinite_skirt = print.draft_shield == crate::ProcessDraftShield::Enabled;
    let layer_limit = if infinite_skirt {
        usize::MAX
    } else {
        skirt_height
    };

    let mut bounds = None;
    for object in &traversal.objects {
        let compensation_object = &object
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object
            .object;
        let layers = compensation_object.as_parts().1;
        let selected_layers = if skirt.is_some() {
            layers.iter().take(layer_limit)
        } else {
            layers.iter().take(1)
        };
        for layer in selected_layers {
            for polygon in layer {
                include_polygon_bounds(polygon.contour(), traversal.scale, &mut bounds);
            }
        }
    }
    if let Some(skirt) = skirt {
        include_bounds(skirt.covered_bounds(traversal.scale), &mut bounds);
    }
    if let Some(brim) = brim {
        include_bounds(brim.covered_bounds(traversal.scale), &mut bounds);
    }

    let fallback_distance = if skirt.is_none() && skirt_height > 0 {
        print.skirt_distance.0
    } else {
        0.0
    };
    let (center_x, center_y) = model_center(traversal)?;
    bounds.map(|(min_x, min_y, max_x, max_y)| {
        (
            min_x - fallback_distance + center_x,
            min_y - fallback_distance + center_y,
            max_x - min_x + 2.0 * fallback_distance,
            max_y - min_y + 2.0 * fallback_distance,
        )
    })
}

fn include_bounds(
    additional: Option<(f64, f64, f64, f64)>,
    bounds: &mut Option<(f64, f64, f64, f64)>,
) {
    let Some((min_x, min_y, max_x, max_y)) = additional else {
        return;
    };
    *bounds = Some(match *bounds {
        Some((old_min_x, old_min_y, old_max_x, old_max_y)) => (
            old_min_x.min(min_x),
            old_min_y.min(min_y),
            old_max_x.max(max_x),
            old_max_y.max(max_y),
        ),
        None => (min_x, min_y, max_x, max_y),
    });
}

fn include_polygon_bounds(
    polygon: &crate::geometry::Polygon,
    scale: CoordinateScale,
    bounds: &mut Option<(f64, f64, f64, f64)>,
) {
    for point in polygon.points() {
        let x = scale.unscale(point.x());
        let y = scale.unscale(point.y());
        *bounds = Some(match *bounds {
            Some((min_x, min_y, max_x, max_y)) => {
                (min_x.min(x), min_y.min(y), max_x.max(x), max_y.max(y))
            }
            None => (x, y, x, y),
        });
    }
}

pub(super) fn model_bounds(
    traversal: &PreparedPostClassicTraversal,
) -> Option<(f64, f64, f64, f64)> {
    object_bounds(traversal.project.objects().first()?)
}

fn object_bounds(object: &ProjectObject) -> Option<(f64, f64, f64, f64)> {
    let instance_transform = object.instances().first()?.transform();
    let mut bounds = None::<(f64, f64, f64, f64)>;
    for volume in object
        .volumes()
        .iter()
        .filter(|volume| volume.volume_type() == ProjectVolumeType::ModelPart)
    {
        let transform = instance_transform.then(volume.transform());
        for &vertex in volume.mesh().vertices() {
            let point = transform.transform_point(vertex);
            bounds = Some(match bounds {
                Some((min_x, min_y, max_x, max_y)) => (
                    min_x.min(point.x),
                    min_y.min(point.y),
                    max_x.max(point.x),
                    max_y.max(point.y),
                ),
                None => (point.x, point.y, point.x, point.y),
            });
        }
    }
    bounds
}

/// World-space center of the `index`th project object: its build-item
/// transform places the sliced object on the plate (`bbs_3mf.cpp:3554-3560`
/// applies build-item transforms; `GCode.cpp:5380` `set_origin(unscale(
/// instance.shift))` re-origins coordinates per print object).
pub(super) fn object_center(
    traversal: &PreparedPostClassicTraversal,
    index: usize,
) -> Option<(f64, f64)> {
    let (min_x, min_y, max_x, max_y) = object_bounds(traversal.project.objects().get(index)?)?;
    Some(((min_x + max_x) * 0.5, (min_y + max_y) * 0.5))
}

pub(in crate::project_slice) fn model_center(
    traversal: &PreparedPostClassicTraversal,
) -> Option<(f64, f64)> {
    let (min_x, min_y, max_x, max_y) = model_bounds(traversal)?;
    Some(((min_x + max_x) * 0.5, (min_y + max_y) * 0.5))
}

pub(super) const EXCLUDE_FLAVORS: [crate::GCodeFlavor; 4] = [
    crate::GCodeFlavor::Klipper,
    crate::GCodeFlavor::MarlinLegacy,
    crate::GCodeFlavor::MarlinFirmware,
    crate::GCodeFlavor::RepRapFirmware,
];

/// One `EXCLUDE_OBJECT_DEFINE` / `M486 S..` record per print instance
/// (`GCode.cpp:8084-8108`).
pub(super) struct InstanceDefinition {
    pub(super) name: String,
    pub(super) unique_id: usize,
    /// `CENTER=.. POLYGON=..` body for Klipper; `None` degenerates to a skip.
    pub(super) klipper_body: Option<String>,
}

pub(super) fn definitions(traversal: &PreparedPostClassicTraversal) -> Vec<InstanceDefinition> {
    let scale = traversal.scale;
    let mut unique_id = 0;
    let mut output = Vec::new();
    for (object_index, object) in traversal.project.objects().iter().enumerate() {
        for (instance_index, instance) in object.instances().iter().enumerate() {
            output.push(InstanceDefinition {
                name: instance_name(object.name(), object_index as u32, instance_index as u32),
                unique_id,
                klipper_body: klipper_body(object, instance, scale),
            });
            unique_id += 1;
        }
    }
    output
}

impl InstanceDefinition {
    pub(super) fn append(&self, output: &mut Vec<u8>, klipper: bool, rrf: bool) {
        let text = if klipper {
            let Some(body) = &self.klipper_body else {
                return;
            };
            format!("EXCLUDE_OBJECT_DEFINE NAME={} {body}\n", self.name)
        } else if rrf {
            // RepRapFirmware takes the object name on the select line
            // (`GCode.cpp:8110-8113`).
            format!("M486 S{} A\"{}\"\nM486 S-1\n", self.unique_id, self.name)
        } else {
            format!("M486 S{}\nM486 A{}\nM486 S-1\n", self.unique_id, self.name)
        };
        output.extend_from_slice(text.as_bytes());
    }
}

/// Start/end label strings for the in-print object markers
/// (`GCode.cpp:5360-5372, 5478-5494`), or `None` when exclusion is disabled.
pub(super) fn in_print_labels(
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
) -> (Option<String>, Option<String>) {
    let settings = &traversal.resolved.views.full;
    if !EXCLUDE_FLAVORS.contains(&settings.printer.gcode.gcode_flavor)
        || !settings.process.print.exclude_object.0
        || Tags::of(traversal).is_bbl()
    {
        return (None, None);
    }
    let Some(object) = traversal.project.objects().get(object_index) else {
        return (None, None);
    };
    let Some(instance) = object.instances().first() else {
        return (None, None);
    };
    let preceding_instances = traversal.project.objects()[..object_index]
        .iter()
        .map(|object| object.instances().len())
        .sum::<usize>();
    let name = instance_name(object.name(), object_index as u32, instance.instance_id());
    let klipper = settings.printer.gcode.gcode_flavor == crate::GCodeFlavor::Klipper;
    let start = if klipper {
        format!("EXCLUDE_OBJECT_START NAME={name}\n")
    } else {
        format!("M486 S{preceding_instances}\n")
    };
    let mut end = if klipper {
        format!("EXCLUDE_OBJECT_END NAME={name}\n")
    } else {
        "M486 S-1\n".to_owned()
    };
    if !traversal
        .resolved
        .views
        .runtime_gcode
        .use_relative_e_distances
        .0
    {
        // `GCodeWriter::add_object_end_labels` resets E after the end marker
        // when extrusion is absolute (`GCodeWriter.cpp:1183-1192`).
        end.push_str("G92 E0\n");
    }
    (Some(start), Some(end))
}

fn klipper_body(
    object: &ProjectObject,
    instance: &ProjectInstance,
    scale: CoordinateScale,
) -> Option<String> {
    let mut min = [f64::INFINITY; 2];
    let mut max = [f64::NEG_INFINITY; 2];
    let mut points = Vec::new();
    for volume in object
        .volumes()
        .iter()
        .filter(|volume| volume.volume_type() == ProjectVolumeType::ModelPart)
    {
        let transform = instance.transform().then(volume.transform());
        for vertex in volume.mesh().vertices() {
            let transformed = transform.transform_point(*vertex);
            if !transformed.x.is_finite() || !transformed.y.is_finite() {
                return None;
            }
            min[0] = min[0].min(transformed.x);
            min[1] = min[1].min(transformed.y);
            max[0] = max[0].max(transformed.x);
            max[1] = max[1].max(transformed.y);
            let x = scale.checked_scale(transformed.x)?;
            let y = scale.checked_scale(transformed.y)?;
            points.push(Point::new(x, y));
        }
    }
    if points.is_empty() {
        return None;
    }
    let hull = douglas_peucker(&convex_hull(&points), 0.1);
    if hull.is_empty() {
        return None;
    }
    let unscaled = |point: &Point| [scale.unscale(point.x()), scale.unscale(point.y())];
    let coordinate = |point: &Point| {
        let [x, y] = unscaled(point);
        format!(
            "[{},{}]",
            format_processor_float(x),
            format_processor_float(y)
        )
    };
    let polygon = hull
        .iter()
        .chain(hull.first())
        .map(coordinate)
        .collect::<Vec<_>>()
        .join(",");
    Some(format!(
        "CENTER={},{} POLYGON=[{polygon}]",
        format_processor_float((min[0] + max[0]) * 0.5),
        format_processor_float((min[1] + max[1]) * 0.5),
    ))
}

/// `get_instance_name` (`GCode.cpp:4378-4386`): sanitize the object name,
/// compose `<name>_id_<object>_copy_<instance>`, then sanitize again.
fn instance_name(object_name: &str, object_id: u32, instance_id: u32) -> String {
    let composed = format!(
        "{}_id_{object_id}_copy_{instance_id}",
        sanitize_instance_name(object_name)
    );
    sanitize_instance_name(&composed)
}

/// `sanitize_instance_name` (`GCode.cpp:4364-4375`): collapse runs of
/// printable-special characters into `_`, then strip edge underscores.
fn sanitize_instance_name(name: &str) -> String {
    const SPECIAL: [char; 22] = [
        ' ', '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '=', '+', '[', ']', '{', '}', ';',
        ':', '"', ',', '\'',
    ];
    let mut output = String::with_capacity(name.len());
    let mut separator = false;
    for character in name.chars() {
        if SPECIAL.contains(&character) {
            separator = true;
        } else {
            if separator && !output.is_empty() {
                output.push('_');
            }
            separator = false;
            output.push(character);
        }
    }
    // Upstream erases at most one leading and one trailing underscore
    // (`GCode.cpp:4368-4375`).
    let stripped = output.strip_prefix('_').unwrap_or(&output);
    stripped.strip_suffix('_').unwrap_or(stripped).to_owned()
}

#[cfg(test)]
mod tests {
    use super::sanitize_instance_name;

    #[test]
    fn sanitize_matches_upstream_character_class() {
        assert_eq!(sanitize_instance_name("cube10.stl"), "cube10.stl");
        assert_eq!(sanitize_instance_name("a b!!c.stl"), "a_b_c.stl");
        assert_eq!(sanitize_instance_name("!!lead"), "lead");
        assert_eq!(sanitize_instance_name("trail!!"), "trail");
        assert_eq!(sanitize_instance_name("a:b,c\"d'e"), "a_b_c_d_e");
        assert_eq!(sanitize_instance_name("hyphen.dot/kept"), "hyphen.dot/kept");
        assert_eq!(sanitize_instance_name("__foo__"), "_foo_");
    }
}
