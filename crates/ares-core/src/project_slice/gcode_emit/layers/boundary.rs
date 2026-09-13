//! `GCode.cpp::change_layer` boundary emission: the layer-change labels,
//! retraction deferral and state advance that open every layer chunk.
#[cfg(test)]
mod tests;

use super::super::{
    GenerationMetadata, PreparedPostClassicTraversal, SliceError, footprint,
    format_processor_float, layer_gcode, machine, motion, spiral_vase, timelapse,
};

pub(super) struct Boundary<'a> {
    pub traversal: &'a PreparedPostClassicTraversal,
    pub layer_change_template: &'a layer_gcode::LayerChangeTemplate,
    pub metadata: GenerationMetadata,
    pub first_layer_bounds: Option<footprint::FirstLayerBounds>,
}

/// Advances the emission state into the new layer. `change_layer`
/// increments `m_layer_index` BEFORE its change-layer retract
/// (`GCode.cpp:5690-5696`), so the deferred hop's
/// `retract_lift_enforce` bottom/top gate must see the NEW layer
/// index — the index advance strictly precedes the deferral.
pub(in crate::project_slice::gcode_emit::layers) fn advance_for_layer(
    state: &mut motion::EmitState,
    precise_layer_z: f64,
    layer_z: f32,
    layer_index: usize,
    layer_retract_pending: bool,
) {
    state.layer_z = f64::from(layer_z);
    state.source_layer_z = precise_layer_z;
    state.layer_index = layer_index;
    if layer_retract_pending {
        motion::defer_layer_change_lift(state);
    }
}

pub(super) struct BoundaryAdvance<'a> {
    pub layer_z: f32,
    pub layer_height: f32,
    pub timelapse_context: timelapse::Context<'a>,
}

/// Emits the `change_layer` block for `layer_index` (`GCode.cpp:5685-5718`
/// and its callers): Z/HEIGHT headers, the before-layer-change template,
/// the deferred change-layer retract, and the per-state layer advance.
pub(super) fn append<'a>(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    spiral: &mut spiral_vase::SpiralVaseFilter,
    Boundary {
        traversal,
        layer_change_template,
        metadata,
        first_layer_bounds,
    }: Boundary<'a>,
    layer_index: usize,
    previous_layer_z: f32,
    layer_z: f32,
    layer_height: f32,
    second_layer_done: &mut bool,
    bed_cache: i32,
) -> Result<BoundaryAdvance<'a>, SliceError> {
    state.physical_fan_speed = state.part_fan_speed;
    let tags = state.tags;
    output.extend_from_slice(tags.layer_change().as_bytes());
    output.push(b'\n');
    // Upstream's writer z at the layer-change retract is still the
    // previous layer's z (`change_layer` does not move z); the
    // start-gcode z remains authoritative for the first layer.
    if layer_index > 0 {
        state.writer_z = Some(f64::from(previous_layer_z));
    }
    let header = format!(
        "{}\n{}\n",
        tags.z(&format_processor_float(f64::from(layer_z))),
        tags.height(&format_processor_float(f64::from(layer_height))),
    );
    output.extend_from_slice(header.as_bytes());
    let timelapse_context = timelapse::Context {
        traversal,
        layer: timelapse::TimelapseLayer {
            index: layer_index,
            z: f64::from(layer_z),
            max_z: f64::from(layer_z),
        },
        metadata,
        first_layer_bounds,
    };
    let timelapse_at_layer_change = !tags.is_bbl()
        && !traversal
            .resolved
            .views
            .runtime_gcode
            .time_lapse_gcode
            .0
            .is_empty();
    layer_gcode::append_before_layer_change_gcode(
        output,
        layer_gcode::LayerTemplateContext {
            traversal,
            layer_index,
            layer_z: f64::from(layer_z),
            totals: state.extrusion_totals(),
            context: layer_change_template,
        },
    )?;
    // Upstream's `change_layer` retract (`retract_when_changing_layer`)
    // also DEFERS the hop (`maybe_zlift`, `GCodeWriter.cpp:626-648`):
    // `m_to_lift` survives into the new layer's first travel, which
    // raises to layer+hop and descends at the target. Capture the
    // flag BEFORE the wipe flush consumes it; defer only here (the
    // c7ea935f lesson: no already-retracted mid-print deferrals).
    let layer_retract_pending = state.pending_layer_retract
        && state.options.z_hop > 0.0
        && state.options.retraction_length > 0.0
        && !state.lifted
        && state.pending_lift.is_none();
    motion::flush_pending_retract_wipe(output, state);
    // Pending object-end labels flush after the layer-change
    // retract/wipe, before the layer-change gcode
    // (`GCode.cpp:5699` `change_layer`).
    motion::append_exclude_end(output, state);
    // The layer-start retraction mirrors the `change_layer`/BBS
    // layer-start retract, which is gated on
    // `retract_when_changing_layer` (`GCode.cpp:5206`, `GCode.cpp:5693`);
    // the compatible-flavor `change_layer` variant additionally
    // requires `will_move_z` — the nozzle must actually change Z
    // from wherever the start g-code left it (`GCode.cpp:5693`).
    // BBL layer starts retract whenever the flag is set.
    let will_move_z = state
        .writer_z
        .is_none_or(|writer_z| (f64::from(layer_z) - writer_z).abs() > 1.0e-4);
    if layer_index == 0
        && state.options.retract_when_changing_layer
        && (state.tags.is_bbl() || will_move_z)
    {
        motion::retract_before_layer(output, state);
    }
    if timelapse_at_layer_change {
        timelapse::append_and_track(output, state, timelapse_context)?;
    }
    spiral.append_layer_z(output, layer_index, f64::from(layer_z));
    layer_gcode::append_layer_change(
        output,
        layer_gcode::LayerTemplateContext {
            traversal,
            layer_index,
            layer_z: f64::from(layer_z),
            totals: state.extrusion_totals(),
            context: layer_change_template,
        },
    )?;
    // A deferred previous-layer retraction lifts above the new layer's
    // print Z (`GCodeWriter::travel_to_z` during layer transition).
    // The lift gate evaluates at the writer's z before the layer move
    // (`GCode.cpp:5690` change_layer retract precedes travel_to_z;
    // `GCodeWriter.cpp:633-639` gates on `m_pos.z()`).
    let previous_state_layer_z = state.layer_z;
    advance_for_layer(
        state,
        f64::from(layer_z),
        layer_z,
        layer_index,
        layer_retract_pending,
    );
    motion::flush_pending_retract_lift(output, state, previous_state_layer_z);
    motion::begin_layer(
        output,
        state,
        layer_index,
        f64::from(layer_z),
        f64::from(layer_height),
    );
    // Second-layer transition: bed temperature for the remaining
    // layers (`GCode.cpp:4777-4830`), once per slice.
    if layer_index == 1 && !*second_layer_done {
        *second_layer_done = true;
        machine::append_second_layer_transition(output, traversal, bed_cache);
    }
    // Orca `GCode.cpp:5205-5210`: core-xy BBL printers emit the eager
    // change-layer lift (the second change-layer retract is a no-op
    // for E once retracted) and the timelapse gcode inside the new
    // layer's window, after the layer-change acceleration.
    let bbl_layer_start = state.tags.is_bbl() && !state.traditional_timelapse;
    if bbl_layer_start && state.options.retract_when_changing_layer {
        motion::flush_pending_retract_eager(output, state);
    }
    if bbl_layer_start {
        timelapse::append_and_track(output, state, timelapse_context)?;
    }
    Ok(BoundaryAdvance {
        layer_z,
        layer_height,
        timelapse_context,
    })
}
