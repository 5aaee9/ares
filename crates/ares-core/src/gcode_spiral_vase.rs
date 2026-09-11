mod project;

pub(crate) use project::{
    ProjectSpiralVaseConfig, ProjectSpiralVaseLayer, ProjectSpiralVaseRunState,
};

use crate::{LayerExtrusionMoves, Point2, SliceError, SliceOptions, ToolpathMoveKind};

#[derive(Clone, Copy, Debug, PartialEq)]
struct SpiralVaseConfig {
    starting_flow_ratio: f64,
    finishing_flow_ratio: f64,
    transition_layer_index: Option<usize>,
    smooth_xy: bool,
    max_xy_smoothing: f64,
}

impl SpiralVaseConfig {
    fn from_options(options: &SliceOptions) -> Result<Self, SliceError> {
        let starting_flow_ratio = options.range_f64("spiral_starting_flow_ratio", 0.0, 0.0, 1.0)?;
        let finishing_flow_ratio =
            options.range_f64("spiral_finishing_flow_ratio", 0.0, 0.0, 1.0)?;
        let spiral_mode = options.bool_option("spiral_mode", false)?;
        let smooth_xy = spiral_mode && options.bool_option("spiral_mode_smooth", false)?;
        let transition_layer_index = if spiral_mode && options.use_relative_e_distances()? {
            Some(options.shell_layer_options()?.bottom_shell_layers())
        } else {
            None
        };
        let nozzle_diameter = options.nozzle_diameters()?[0];
        let max_xy_smoothing = parse_max_xy_smoothing(options, nozzle_diameter)?;
        Ok(Self {
            starting_flow_ratio,
            finishing_flow_ratio,
            transition_layer_index,
            smooth_xy,
            max_xy_smoothing,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpiralVaseRunState {
    config: SpiralVaseConfig,
    previous_layer_points: Vec<Point2>,
}

impl SpiralVaseRunState {
    pub(crate) fn from_options(options: &SliceOptions) -> Result<Self, SliceError> {
        Ok(Self {
            config: SpiralVaseConfig::from_options(options)?,
            previous_layer_points: Vec::new(),
        })
    }

    pub(crate) fn layer_state(
        &mut self,
        layer_index: usize,
        final_layer: bool,
        layer: &LayerExtrusionMoves,
    ) -> SpiralVaseLayerState {
        let transition_out = crate::gcode_spiral_vase_transition::SpiralVaseTransitionOutState::new(
            self.config.finishing_flow_ratio,
            layer,
            self.config.transition_layer_index.is_some() && final_layer,
        );
        let previous_layer_points = std::mem::take(&mut self.previous_layer_points);
        let transition_in = self.config.transition_layer_index == Some(layer_index);
        let smooth_xy = self.config.smooth_xy
            && self.config.max_xy_smoothing > 0.0
            && previous_layer_points.len() > 1;
        let total_xy = if transition_in || smooth_xy {
            total_print_xy(layer)
        } else {
            0.0
        };
        SpiralVaseLayerState {
            starting_flow_ratio: self.config.starting_flow_ratio,
            total_xy,
            printed_xy: 0.0,
            previous_point: None,
            transition_in: transition_in && total_xy > f64::EPSILON,
            transition_out,
            smooth_xy,
            max_xy_smoothing: self.config.max_xy_smoothing,
            previous_layer_points,
            current_layer_points: Vec::new(),
            last_emitted_point: None,
        }
    }

    pub(crate) fn finish_layer(
        &mut self,
        mut layer_state: SpiralVaseLayerState,
        writer: &mut crate::gcode_writer::GCodeWriter,
        gcode_comments: bool,
    ) -> String {
        self.previous_layer_points = layer_state.take_current_layer_points();
        layer_state.transition_out.finish(writer, gcode_comments)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpiralVaseLayerState {
    starting_flow_ratio: f64,
    total_xy: f64,
    printed_xy: f64,
    previous_point: Option<Point2>,
    transition_in: bool,
    transition_out: crate::gcode_spiral_vase_transition::SpiralVaseTransitionOutState,
    smooth_xy: bool,
    max_xy_smoothing: f64,
    previous_layer_points: Vec<Point2>,
    current_layer_points: Vec<Point2>,
    last_emitted_point: Option<Point2>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SpiralVaseMoveCommand {
    pub(crate) kind: ToolpathMoveKind,
    pub(crate) point: Point2,
    pub(crate) current_e: f64,
    pub(crate) target_e: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SpiralVaseAdjustedMove {
    pub(crate) point: Point2,
    pub(crate) e_position: f64,
    pub(crate) e_offset_delta: f64,
}

impl SpiralVaseLayerState {
    pub(crate) fn observe_transition_out(
        &mut self,
        command: crate::gcode_spiral_vase_transition::TransitionOutMoveCommand,
    ) {
        self.transition_out.observe_move(command);
    }

    pub(crate) fn adjusted_move(
        &mut self,
        command: SpiralVaseMoveCommand,
    ) -> SpiralVaseAdjustedMove {
        let previous = self.previous_point;
        self.previous_point = Some(command.point);
        if command.kind != ToolpathMoveKind::Print {
            self.last_emitted_point = Some(command.point);
            return SpiralVaseAdjustedMove {
                point: command.point,
                e_position: command.target_e,
                e_offset_delta: 0.0,
            };
        }
        self.current_layer_points.push(command.point);
        let mut e_position = command.target_e;
        let Some(previous) = previous else {
            let point = self.smoothed_point(command.point, 0.0);
            self.last_emitted_point = Some(point);
            return SpiralVaseAdjustedMove {
                point,
                e_position,
                e_offset_delta: 0.0,
            };
        };
        let xy_distance = distance(previous, command.point);
        if self.transition_in || self.smooth_xy {
            self.printed_xy += xy_distance;
        }
        if self.transition_in && xy_distance > f64::EPSILON {
            let progress = (self.printed_xy / self.total_xy).clamp(0.0, 1.0);
            let factor = self.starting_flow_ratio + progress * (1.0 - self.starting_flow_ratio);
            e_position = command.current_e + (command.target_e - command.current_e) * factor;
        }
        let progress = if self.total_xy > f64::EPSILON {
            (self.printed_xy / self.total_xy).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let point = self.smoothed_point(command.point, progress);
        let emitted_previous = self.last_emitted_point.unwrap_or(previous);
        self.last_emitted_point = Some(point);
        if xy_distance > f64::EPSILON {
            let smoothed_distance = distance(emitted_previous, point);
            e_position = command.current_e
                + (e_position - command.current_e) * smoothed_distance / xy_distance;
        }
        SpiralVaseAdjustedMove {
            point,
            e_position,
            e_offset_delta: e_position - command.target_e,
        }
    }

    fn take_current_layer_points(&mut self) -> Vec<Point2> {
        std::mem::take(&mut self.current_layer_points)
    }

    fn smoothed_point(&self, point: Point2, progress: f64) -> Point2 {
        if !self.smooth_xy {
            return point;
        }
        // Upstream `SpiralVaseHelpers` runs entirely in float: distance,
        // projection, and the `(1-factor)` interpolation all round at
        // f32 before the X/Y words are formatted.
        let p = [point.x() as f32, point.y() as f32];
        let previous = self
            .previous_layer_points
            .iter()
            .map(|point| [point.x() as f32, point.y() as f32])
            .collect::<Vec<_>>();
        let Some(previous) = nearest_point_on_polyline(&previous, p) else {
            return point;
        };
        if distance_f32(previous, p) < self.max_xy_smoothing as f32 {
            let factor = progress as f32;
            Point2::new(
                f64::from(previous[0] * (1.0 - factor) + p[0] * factor),
                f64::from(previous[1] * (1.0 - factor) + p[1] * factor),
            )
        } else {
            point
        }
    }
}

fn total_print_xy(layer: &LayerExtrusionMoves) -> f64 {
    let mut previous = None;
    let mut total = 0.0;
    for move_ in layer.moves() {
        let point = move_.point();
        if move_.kind() == ToolpathMoveKind::Print
            && let Some(start) = previous
        {
            total += distance(start, point);
        }
        previous = Some(point);
    }
    total
}

fn distance(start: Point2, end: Point2) -> f64 {
    ((end.x() - start.x()).powi(2) + (end.y() - start.y()).powi(2)).sqrt()
}

fn distance_f32(start: [f32; 2], end: [f32; 2]) -> f32 {
    ((end[0] - start[0]).powi(2) + (end[1] - start[1]).powi(2)).sqrt()
}

fn nearest_point_on_polyline(points: &[[f32; 2]], point: [f32; 2]) -> Option<[f32; 2]> {
    points
        .windows(2)
        .map(|segment| project_point_to_segment(segment[0], segment[1], point))
        .min_by(|left, right| distance_f32(*left, point).total_cmp(&distance_f32(*right, point)))
}

fn project_point_to_segment(start: [f32; 2], end: [f32; 2], point: [f32; 2]) -> [f32; 2] {
    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let length_squared = dx * dx + dy * dy;
    if length_squared <= f32::EPSILON {
        return start;
    }
    let t = (((point[0] - start[0]) * dx + (point[1] - start[1]) * dy) / length_squared)
        .clamp(0.0, 1.0);
    [start[0] + dx * t, start[1] + dy * t]
}

fn parse_max_xy_smoothing(options: &SliceOptions, nozzle_diameter: f64) -> Result<f64, SliceError> {
    let value = options.values().get("spiral_mode_max_xy_smoothing");
    let value = match value {
        Some(value) => crate::options::parsing::parse_non_negative_numeric_or_percent_over_base(
            "spiral_mode_max_xy_smoothing",
            value,
            nozzle_diameter,
        )?,
        None => 2.0 * nozzle_diameter,
    };
    if value <= 1000.0 {
        Ok(value)
    } else {
        Err(SliceError::InvalidInput(
            "spiral_mode_max_xy_smoothing is out of range".to_owned(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ExtrusionMove, PrintPathRole};

    #[test]
    fn spiral_starting_flow_ratio_uses_print_xy_only_but_observes_travel_start() {
        let mut state = SpiralVaseLayerState {
            starting_flow_ratio: 0.0,
            total_xy: 10.0,
            printed_xy: 0.0,
            previous_point: None,
            transition_in: true,
            transition_out:
                crate::gcode_spiral_vase_transition::SpiralVaseTransitionOutState::disabled(),
            smooth_xy: false,
            max_xy_smoothing: 0.0,
            previous_layer_points: Vec::new(),
            current_layer_points: Vec::new(),
            last_emitted_point: None,
        };

        assert_eq!(
            state.adjusted_move(command(ToolpathMoveKind::Travel, 100.0, 0.0, 0.0, 0.0)),
            adjusted(100.0, 0.0, 0.0, 0.0)
        );
        assert_eq!(
            state.adjusted_move(command(ToolpathMoveKind::Print, 103.0, 4.0, 0.0, 1.0)),
            adjusted(103.0, 4.0, 0.5, -0.5)
        );
        assert_eq!(
            state.adjusted_move(command(ToolpathMoveKind::Print, 106.0, 8.0, 0.5, 1.5)),
            adjusted(106.0, 8.0, 1.5, 0.0)
        );
    }

    #[test]
    fn xy_smoothing_uses_previous_original_layer_points_and_scales_e() {
        let mut run = SpiralVaseRunState {
            config: SpiralVaseConfig {
                starting_flow_ratio: 0.0,
                finishing_flow_ratio: 0.0,
                transition_layer_index: None,
                smooth_xy: true,
                max_xy_smoothing: 10.0,
            },
            previous_layer_points: Vec::new(),
        };
        let first_layer = layer_moves(0.0);
        let mut first = run.layer_state(0, false, &first_layer);
        first.adjusted_move(command(ToolpathMoveKind::Travel, 0.0, 0.0, 0.0, 0.0));
        first.adjusted_move(command(ToolpathMoveKind::Print, 10.0, 0.0, 0.0, 1.0));
        first.adjusted_move(command(ToolpathMoveKind::Print, 20.0, 0.0, 1.0, 2.0));
        run.previous_layer_points = first.take_current_layer_points();

        let second_layer = layer_moves(4.0);
        let mut second = run.layer_state(1, false, &second_layer);
        second.adjusted_move(command(ToolpathMoveKind::Travel, 0.0, 4.0, 0.0, 0.0));
        let first_smoothed =
            second.adjusted_move(command(ToolpathMoveKind::Print, 10.0, 4.0, 0.0, 1.0));
        let second_smoothed =
            second.adjusted_move(command(ToolpathMoveKind::Print, 20.0, 4.0, 1.0, 2.0));

        assert_eq!(first_smoothed.point, Point2::new(10.0, 2.0));
        assert!((first_smoothed.e_position - 1.0198039027185568).abs() < 1e-12);
        assert_eq!(second_smoothed.point, Point2::new(20.0, 4.0));
        assert!((second_smoothed.e_position - 2.019803902718557).abs() < 1e-12);
        let original_points = vec![Point2::new(10.0, 4.0), Point2::new(20.0, 4.0)];
        assert_eq!(second.take_current_layer_points(), original_points);
    }

    fn command(
        kind: ToolpathMoveKind,
        x: f64,
        y: f64,
        current_e: f64,
        target_e: f64,
    ) -> SpiralVaseMoveCommand {
        SpiralVaseMoveCommand {
            kind,
            point: Point2::new(x, y),
            current_e,
            target_e,
        }
    }

    fn layer_moves(y: f64) -> LayerExtrusionMoves {
        LayerExtrusionMoves::new(
            0,
            0.2,
            vec![
                ExtrusionMove::new(
                    ToolpathMoveKind::Travel,
                    PrintPathRole::ExternalPerimeter,
                    Point2::new(0.0, y),
                    None,
                ),
                ExtrusionMove::new(
                    ToolpathMoveKind::Print,
                    PrintPathRole::ExternalPerimeter,
                    Point2::new(10.0, y),
                    Some(1.0),
                ),
                ExtrusionMove::new(
                    ToolpathMoveKind::Print,
                    PrintPathRole::ExternalPerimeter,
                    Point2::new(20.0, y),
                    Some(2.0),
                ),
            ],
            2.0,
        )
    }

    fn adjusted(x: f64, y: f64, e_position: f64, e_offset_delta: f64) -> SpiralVaseAdjustedMove {
        SpiralVaseAdjustedMove {
            point: Point2::new(x, y),
            e_position,
            e_offset_delta,
        }
    }
}
