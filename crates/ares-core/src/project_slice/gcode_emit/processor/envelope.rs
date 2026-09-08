//! Normal-mode MachineEnvelopeConfig bridge (`GCodeProcessor.cpp:2087–2122`).
use super::ProcessorLimits;
use crate::{GCodeFlavor, MachineEnvelopeOptions, OrcaFloats};

impl ProcessorLimits {
    pub(in crate::project_slice::gcode_emit) fn from_config(
        machine: &MachineEnvelopeOptions,
        gcode_flavor: GCodeFlavor,
        bbl_printer: bool,
    ) -> Self {
        let defaults;
        let machine = if matches!(
            gcode_flavor,
            GCodeFlavor::MarlinLegacy
                | GCodeFlavor::MarlinFirmware
                | GCodeFlavor::Klipper
                | GCodeFlavor::RepRapFirmware
        ) {
            machine
        } else {
            // TimeProcessor::reset keeps MachineEnvelopeConfig's schema defaults
            // when apply_config does not copy the effective printer envelope.
            defaults = MachineEnvelopeOptions::default();
            &defaults
        };
        Self {
            print_acceleration: normal(&machine.machine_max_acceleration_extruding),
            retract_acceleration: normal(&machine.machine_max_acceleration_retracting),
            travel_acceleration: if gcode_flavor.supports_separate_travel_acceleration() {
                normal(&machine.machine_max_acceleration_travel)
            } else {
                // Legacy Marlin/Klipper have no separate travel cap, even though
                // apply_config copies the extruding array into the travel array.
                0.0
            },
            gcode_flavor,
            bbl_printer,
            junction_deviation: normal(&machine.machine_max_junction_deviation),
            max_feedrate: [
                &machine.machine_max_speed_x,
                &machine.machine_max_speed_y,
                &machine.machine_max_speed_z,
                &machine.machine_max_speed_e,
            ]
            .map(normal),
            max_acceleration: [
                &machine.machine_max_acceleration_x,
                &machine.machine_max_acceleration_y,
                &machine.machine_max_acceleration_z,
                &machine.machine_max_acceleration_e,
            ]
            .map(normal),
            jerk: [
                &machine.machine_max_jerk_x,
                &machine.machine_max_jerk_y,
                &machine.machine_max_jerk_z,
                &machine.machine_max_jerk_e,
            ]
            .map(normal),
        }
    }
}

fn normal(values: &OrcaFloats) -> f64 {
    // get_option_value returns float, indexing by time mode, not extruder.
    f64::from(values.0.first().map_or(0.0, |value| value.0 as f32))
}

#[cfg(test)]
mod tests;
