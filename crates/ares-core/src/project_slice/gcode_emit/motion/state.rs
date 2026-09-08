use super::{MotionOptions, arc, set_accel_and_jerk, set_layer_acceleration_and_jerk};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::project_slice::gcode_emit) enum LiftMode {
    #[default]
    Normal,
    Slope,
    Spiral,
}

#[derive(Default)]
pub(in crate::project_slice::gcode_emit) struct EmitState {
    pub(in crate::project_slice::gcode_emit) x: f64,
    pub(in crate::project_slice::gcode_emit) y: f64,
    pub(in crate::project_slice::gcode_emit) e_position: f64,
    pub(in crate::project_slice::gcode_emit) offset: (f64, f64),
    pub(in crate::project_slice::gcode_emit) scale_factor: f64,
    pub(in crate::project_slice::gcode_emit) travel_feedrate: f64,
    pub(in crate::project_slice::gcode_emit) extrusion_feedrate: f64,
    pub(in crate::project_slice::gcode_emit) current_feedrate: f64,
    pub(in crate::project_slice::gcode_emit) options: MotionOptions,
    pub(in crate::project_slice::gcode_emit) small_area_flow:
        crate::extrusions::SmallAreaInfillFlowCompensation,
    pub(in crate::project_slice::gcode_emit) layer_index: usize,
    pub(in crate::project_slice::gcode_emit) spiral_vase: bool,
    pub(in crate::project_slice::gcode_emit) spiral_vase_layer: bool,
    pub(in crate::project_slice::gcode_emit) positioned: bool,
    /// The Z the machine start/filament g-code left the nozzle at; mirrors
    /// `GCodeWriter::m_pos(2)` for the `change_layer` `will_move_z` gate.
    pub(in crate::project_slice::gcode_emit) writer_z: Option<f64>,
    /// Generator `GCode::m_last_pos` in local scaled coordinates. Writer-only
    /// inward moves change x/y without changing this cursor; None precedes
    /// the first generated path (start G-code supplies the initial writer XY).
    pub(in crate::project_slice::gcode_emit) last_scaled_position: Option<(i64, i64)>,
    pub(in crate::project_slice::gcode_emit) last_feature: Option<&'static str>,
    pub(in crate::project_slice::gcode_emit) last_width: Option<f32>,
    pub(in crate::project_slice::gcode_emit) last_height: Option<f32>,
    pub(in crate::project_slice::gcode_emit) last_acceleration: Option<u32>,
    pub(in crate::project_slice::gcode_emit) last_jerk: Option<f64>,
    pub(in crate::project_slice::gcode_emit) layer_z: f64,
    pub(in crate::project_slice::gcode_emit) source_layer_z: f64,
    pub(in crate::project_slice::gcode_emit) scarf_z: Option<f64>,
    pub(in crate::project_slice::gcode_emit) retracted: bool,
    /// Orca `Extruder::used_filament()` = `m_absolute_E + m_retracted` —
    /// the cumulative forward extrusion through the writer's extrude()
    /// calls. Template output (start g-code) never touches it.
    pub(in crate::project_slice::gcode_emit) wipe_path: Vec<arc::Point>,
    pub(in crate::project_slice::gcode_emit) wipe_start: Option<arc::Point>,
    pub(in crate::project_slice::gcode_emit) lifted: bool,
    /// Upstream `m_lifted` as a DISTANCE (`GCodeWriter.cpp:770-830`):
    /// reduced by in-band z deltas, cleared on real z moves — the bool
    /// above cannot represent partial reduction.
    pub(in crate::project_slice::gcode_emit) lifted_amount: f64,
    pub(in crate::project_slice::gcode_emit) template_lifted: bool,
    pub(in crate::project_slice::gcode_emit) pending_lift: Option<LiftMode>,
    pub(in crate::project_slice::gcode_emit) part_fan_speed: u8,
    pub(in crate::project_slice::gcode_emit) physical_fan_speed: u8,
    pub(in crate::project_slice::gcode_emit) overhang_fan_active: bool,
    pub(in crate::project_slice::gcode_emit) overhang_fan_marker_layer: Option<usize>,
    pub(in crate::project_slice::gcode_emit) internal_bridge_fan_active: bool,
    pub(in crate::project_slice::gcode_emit) internal_bridge_fan_marker_layer: Option<usize>,
    pub(in crate::project_slice::gcode_emit) pending_object_start: Option<(u32, [u8; 12])>,
    pub(in crate::project_slice::gcode_emit) last_travel_acceleration: Option<u32>,
    pub(in crate::project_slice::gcode_emit) pending_exclude_start: Option<String>,
    pub(in crate::project_slice::gcode_emit) pending_exclude_end: Option<String>,
    pub(in crate::project_slice::gcode_emit) pending_object_stop_label: Option<u32>,
    pub(in crate::project_slice::gcode_emit) tags: super::super::tags::Tags,
    /// The one-shot routing disable armed after the first-layer skirt
    // (`disable_once`, `GCode.cpp:4448-4450`: a straight travel to the
    // first object point).
    pub(in crate::project_slice::gcode_emit) avoid_crossing_disabled_once: bool,
    pub(in crate::project_slice::gcode_emit) pending_layer_retract: bool,
    pub(in crate::project_slice::gcode_emit) layer_change_travel_pending: bool,
    pub(in crate::project_slice::gcode_emit) pending_wipe_before_external_target:
        Option<super::arc::Point>,
    pub(in crate::project_slice::gcode_emit) traditional_timelapse: bool,
    /// Orca `Extruder` state: `m_absolute_E` (signed cumulative E through
    /// the writer) and `m_retracted` (outstanding retraction amount).
    /// `used_filament()` returns `m_absolute_E` on share-extruder printers
    /// (`single_extruder_multi_material`), else `m_absolute_E +
    /// m_retracted`. Template output never touches these.
    pub(in crate::project_slice::gcode_emit) filament_used: f64,
    pub(in crate::project_slice::gcode_emit) retracted_amount: f64,
    /// Per-layer avoid-crossing routing boundary, built lazily on the first
    /// routed travel and dropped by `begin_layer`
    /// (`AvoidCrossingPerimeters::init_layer`).
    pub(in crate::project_slice::gcode_emit) avoid_boundary:
        Option<std::rc::Rc<super::path::Boundary>>,
}

#[derive(Clone, Copy)]
pub(in crate::project_slice::gcode_emit) struct LayerGeometry<'a> {
    /// Per-layer nearest-seam penalty data for the Nearest mode — the
    /// layer's candidates with emit-time gaussian distance selection
    /// (`SeamPlacer.cpp:1500-1560`).
    pub(in crate::project_slice::gcode_emit) nearest_seam_penalties:
        Option<&'a crate::project_slice::island_print_order::NearestSeamLayer>,
    /// `staggered_inner_seams` from the owning object (`SeamPlacer.cpp:1601`).
    pub(in crate::project_slice::gcode_emit) staggered_inner: bool,
    pub(in crate::project_slice::gcode_emit) internal_surfaces:
        &'a [crate::project_slice::region_slices::RegionSurface],
    pub(in crate::project_slice::gcode_emit) scale: crate::geometry::CoordinateScale,
    pub(in crate::project_slice::gcode_emit) previous_layer_boundary:
        Option<&'a crate::geometry::LineDistanceTree<'a>>,
    /// Current layer's merged slices plus the perimeter spacing and top
    /// surfaces — the avoid-crossing boundary inputs
    /// (`AvoidCrossingPerimeters.cpp:1099-1134`).
    pub(in crate::project_slice::gcode_emit) avoid_crossing: AvoidCrossingGeometry<'a>,
}

#[derive(Clone, Copy)]
pub(in crate::project_slice::gcode_emit) struct AvoidCrossingGeometry<'a> {
    pub(in crate::project_slice::gcode_emit) layer_slices: &'a [crate::geometry::ExPolygon],
    pub(in crate::project_slice::gcode_emit) perimeter_spacing: f32,
    pub(in crate::project_slice::gcode_emit) external_perimeter_width: f32,
    pub(in crate::project_slice::gcode_emit) top_surfaces: &'a [&'a crate::geometry::ExPolygon],
}

pub(in crate::project_slice::gcode_emit) fn begin_layer(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    layer_index: usize,
    layer_z: f64,
    layer_height: f64,
) {
    state.layer_index = layer_index;
    state.layer_change_travel_pending = true;
    state.avoid_boundary = None;
    state.last_height = Some(layer_height as f32);
    state.layer_z = layer_z;
    state.travel_feedrate = if layer_index == 0 {
        state.options.first_layer_travel_feedrate
    } else {
        state.options.travel_feedrate
    };
    let acceleration = match layer_index {
        0 => Some(state.options.initial_layer_acceleration),
        1 => Some(state.options.default_acceleration),
        _ => None,
    };
    if let Some(acceleration) = acceleration {
        let jerk = if state.options.default_jerk <= 0.0 {
            0.0
        } else if layer_index == 0 && state.options.initial_layer_jerk > 0.0 {
            state.options.initial_layer_jerk
        } else {
            state.options.default_jerk
        };
        set_layer_acceleration_and_jerk(output, state, acceleration, jerk);
    }
}

pub(in crate::project_slice::gcode_emit) fn queue_object_start(
    state: &mut EmitState,
    label_id: u32,
    encoded_labels: [u8; 12],
) {
    state.pending_object_start = Some((label_id, encoded_labels));
}

pub(in crate::project_slice::gcode_emit) fn queue_exclude_start(
    state: &mut EmitState,
    text: String,
) {
    state.pending_exclude_start = Some(text);
}

/// Mirrors `GCode.cpp:5478-5494`: an unflushed start label is dropped
/// (empty instance) and only then does the end label get armed.
pub(in crate::project_slice::gcode_emit) fn queue_exclude_end(state: &mut EmitState, text: String) {
    if state.pending_exclude_start.take().is_none() {
        state.pending_exclude_end = Some(text);
    }
}

impl EmitState {
    pub(in crate::project_slice::gcode_emit) fn extrusion_totals(
        &self,
    ) -> crate::project_slice::gcode_emit::layer_gcode::ExtrusionTotals {
        let used_filament = if self.options.single_extruder_multi_material {
            self.filament_used
        } else {
            self.filament_used + self.retracted_amount
        };
        crate::project_slice::gcode_emit::layer_gcode::ExtrusionTotals {
            diameter: 1.75,
            density: 1.24,
            used_filament,
        }
    }
}

pub(in crate::project_slice::gcode_emit) fn queue_object_stop_label(
    state: &mut EmitState,
    label_id: u32,
) {
    state.pending_object_stop_label = Some(label_id);
}

pub(in crate::project_slice::gcode_emit) fn append_exclude_end(
    output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    if let Some(label_id) = state.pending_object_stop_label.take() {
        output.extend_from_slice(
            format!("; stop printing object, unique label id: {label_id}\nM625\n").as_bytes(),
        );
    }
    if let Some(text) = state.pending_exclude_end.take() {
        output.extend_from_slice(text.as_bytes());
        if !state.options.use_relative_e_distances && text.lines().any(|line| line == "G92 E0") {
            state.e_position = 0.0;
        }
    }
}

pub(in crate::project_slice::gcode_emit) fn append_object_start(
    output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    // `add_object_change_labels` writes the end label before the start label
    // (`GCodeWriter.cpp:1197-1201`).
    append_exclude_end(output, state);
    if let Some((label_id, encoded_labels)) = state.pending_object_start.take() {
        output.extend_from_slice(
            format!("; start printing object, unique label id: {label_id}\nM624 ").as_bytes(),
        );
        output.extend_from_slice(&encoded_labels);
        output.push(b'\n');
    }
    if let Some(text) = state.pending_exclude_start.take() {
        if !state.options.use_relative_e_distances && state.e_position.abs() > f64::EPSILON {
            output.extend_from_slice(b"G92 E0\n");
            state.e_position = 0.0;
        }
        output.extend_from_slice(text.as_bytes());
    }
}

pub(in crate::project_slice::gcode_emit) fn begin_path_travel(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    destination_feature: &str,
    travel_distance: f64,
) {
    let acceleration = if state.options.default_acceleration == 0 {
        None
    } else if state.layer_index == 0 {
        (state.options.initial_layer_travel_acceleration > 0)
            .then_some(state.options.initial_layer_travel_acceleration)
    } else if travel_distance < state.options.retraction_minimum_travel {
        match destination_feature {
            "Overhang wall" => {
                (state.options.bridge_acceleration > 0).then_some(state.options.bridge_acceleration)
            }
            "Outer wall" => (state.options.outer_wall_acceleration > 0)
                .then_some(state.options.outer_wall_acceleration),
            _ => None,
        }
        .or((state.options.travel_acceleration > 0).then_some(state.options.travel_acceleration))
    } else {
        (state.options.travel_acceleration > 0).then_some(state.options.travel_acceleration)
    };
    let jerk = if state.options.default_jerk <= 0.0 {
        0.0
    } else if state.layer_index == 0 {
        state.options.travel_jerk
    } else if travel_distance < state.options.retraction_minimum_travel
        && matches!(destination_feature, "Outer wall" | "Overhang wall")
        && state.options.outer_wall_jerk > 0.0
    {
        state.options.outer_wall_jerk
    } else {
        state.options.travel_jerk
    };
    set_accel_and_jerk(output, state, acceleration.unwrap_or(0), jerk, true);
}
