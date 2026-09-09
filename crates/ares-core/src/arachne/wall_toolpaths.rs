mod outline;
mod postprocess;
pub(crate) mod region_order;
mod stitch;

use crate::geometry::{ClipperError, CoordinateScale, Polygon};

use super::{
    beading::factory::{BeadingStrategyFactoryConfig, make_strategy},
    extrusion_line::ExtrusionLine,
    trapezoidation::{SkeletalTrapezoidation, TrapezoidationConfig, TrapezoidationError},
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct RawWallToolPathConfig {
    pub(crate) outer_spacing: i64,
    pub(crate) inner_spacing: i64,
    pub(crate) inset_count: usize,
    pub(crate) outer_wall_inset: i64,
    pub(crate) layer_height: i64,
    pub(crate) min_bead_width: i64,
    pub(crate) min_feature_size: i64,
    pub(crate) transition_length: i64,
    pub(crate) transitioning_angle: f64,
    pub(crate) transition_filter_deviation: i64,
    pub(crate) wall_distribution_count: i32,
    pub(crate) min_length_factor: f64,
    pub(crate) wall_maximum_resolution: i64,
    pub(crate) wall_maximum_deviation: i64,
    pub(crate) is_top_or_bottom_layer: bool,
    pub(crate) coordinate_scale: CoordinateScale,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GeneratedWallToolPaths {
    pub(crate) toolpaths: Vec<Vec<ExtrusionLine>>,
    pub(crate) inner_contour: Vec<Polygon>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WallToolPathError {
    Geometry(ClipperError),
    Trapezoidation(TrapezoidationError),
}

impl From<ClipperError> for WallToolPathError {
    fn from(error: ClipperError) -> Self {
        Self::Geometry(error)
    }
}

impl From<TrapezoidationError> for WallToolPathError {
    fn from(error: TrapezoidationError) -> Self {
        Self::Trapezoidation(error)
    }
}

pub(crate) fn generate_raw(
    prepared_outline: &[Polygon],
    config: RawWallToolPathConfig,
) -> Result<Vec<Vec<ExtrusionLine>>, TrapezoidationError> {
    if config.inset_count == 0 {
        return Ok(Vec::new());
    }
    let rounded_rectangle_factor = 1.0 - 0.25 * std::f64::consts::PI;
    let scale = config.coordinate_scale.factor();
    let layer_height = (config.layer_height as f64 * scale) as f32;
    let extrusion_width = |spacing| {
        let spacing = spacing as f32 * scale as f32;
        (f64::from(spacing) + f64::from(layer_height) * rounded_rectangle_factor) as f32
    };
    let outer_width = f64::from(extrusion_width(config.outer_spacing));
    let inner_width = f64::from(extrusion_width(config.inner_spacing));
    let min_bead_width = config.min_bead_width as f64 * scale;
    let strategy = make_strategy(BeadingStrategyFactoryConfig {
        preferred_bead_width_outer: config.outer_spacing,
        preferred_bead_width_inner: config.inner_spacing,
        preferred_transition_length: config.transition_length,
        transitioning_angle: config.transitioning_angle as f32,
        print_thin_walls: true,
        min_bead_width: config.min_bead_width,
        min_feature_size: config.min_feature_size,
        wall_split_middle_threshold: (2.0 * min_bead_width / outer_width - 1.0).clamp(0.01, 0.99),
        wall_add_middle_threshold: (min_bead_width / inner_width).clamp(0.01, 0.99),
        max_bead_count: (config.inset_count * 2) as i64,
        outer_wall_offset: config.outer_wall_inset,
        inward_distributed_center_wall_count: config.wall_distribution_count,
        minimum_variable_line_ratio: 0.5,
        coordinate_scale: config.coordinate_scale,
    });
    let trapezoidation = SkeletalTrapezoidation::new(
        prepared_outline,
        strategy.as_ref(),
        TrapezoidationConfig {
            transitioning_angle: config.transitioning_angle,
            discretization_step_size: config.coordinate_scale.checked_scale(0.8).unwrap(),
            transition_filter_dist: config.coordinate_scale.checked_scale(100.0).unwrap(),
            allowed_filter_deviation: config.transition_filter_deviation,
            beading_propagation_transition_dist: config.transition_length,
            coordinate_scale: config.coordinate_scale,
        },
    )?;
    Ok(trapezoidation.generate_toolpaths(false))
}

pub(crate) fn generate(
    outline: &[Polygon],
    config: RawWallToolPathConfig,
) -> Result<GeneratedWallToolPaths, WallToolPathError> {
    let prepared_outline = outline::prepare(outline, config)?;
    let mut toolpaths = generate_raw(&prepared_outline, config)?;
    stitch::stitch_toolpaths(
        &mut toolpaths,
        config.inner_spacing - 1,
        config.coordinate_scale.checked_scale(0.01).unwrap(),
    );
    postprocess::remove_small_lines(
        &mut toolpaths,
        config.min_length_factor,
        config.is_top_or_bottom_layer,
    );
    let inner_contour = postprocess::separate_inner_contour(&mut toolpaths);
    postprocess::simplify_toolpaths(&mut toolpaths, config);
    Ok(GeneratedWallToolPaths {
        toolpaths,
        inner_contour,
    })
}

#[cfg(test)]
mod tests {
    use crate::geometry::{CoordinateScale, Point, Polygon};

    use super::{RawWallToolPathConfig, generate, generate_raw, outline};

    fn fixture() -> (Polygon, RawWallToolPathConfig) {
        let scale = CoordinateScale::Normal;
        let scaled = |value| scale.checked_scale(value).unwrap();
        (
            Polygon::new(vec![
                Point::new(0, 0),
                Point::new(scaled(10.0), 0),
                Point::new(scaled(10.0), scaled(10.0)),
                Point::new(0, scaled(10.0)),
            ]),
            RawWallToolPathConfig {
                outer_spacing: scaled(0.4),
                inner_spacing: scaled(0.4),
                inset_count: 26,
                outer_wall_inset: 0,
                layer_height: scaled(0.2),
                min_bead_width: scaled(0.34),
                min_feature_size: scaled(0.1),
                transition_length: scaled(0.4),
                transitioning_angle: f64::from((std::f32::consts::PI * 10.0_f32) / 180.0_f32),
                transition_filter_deviation: scaled(0.1),
                wall_distribution_count: 1,
                coordinate_scale: scale,
                min_length_factor: 0.5,
                wall_maximum_resolution: scaled(0.5),
                wall_maximum_deviation: scaled(0.025),
                is_top_or_bottom_layer: false,
            },
        )
    }

    #[test]
    fn task22o195_rectangle_generates_positive_width_raw_wall_toolpaths() {
        let (outline, config) = fixture();

        let toolpaths = generate_raw(&[outline], config).unwrap();

        assert!(toolpaths.iter().flatten().any(|line| {
            line.inset_index < config.inset_count
                && line.junctions.len() >= 2
                && line.junctions.iter().all(|junction| junction.width > 0)
        }));
    }

    #[test]
    fn task22o196_wall_toolpath_pipeline_stitches_closed_rectangle_lines() {
        let (outline, config) = fixture();
        let prepared = outline::prepare(std::slice::from_ref(&outline), config).unwrap();
        assert!(!prepared.is_empty());

        let generated = generate(&[outline], config).unwrap();

        assert!(
            generated
                .toolpaths
                .iter()
                .flatten()
                .any(|line| line.is_closed)
        );
        assert!(
            generated
                .toolpaths
                .iter()
                .flatten()
                .flat_map(|line| &line.junctions)
                .all(|junction| junction.width > 0)
        );
    }

    #[test]
    fn task22o203_prepares_fixture_domain_before_trapezoidation() {
        let (_, mut config) = fixture();
        config.outer_spacing = 377_079;
        config.inner_spacing = 377_079;
        config.inset_count = 29;
        let outline = Polygon::new(vec![
            Point::new(2_690_706, -20_263_054),
            Point::new(-1_602_493, -15_969_855),
            Point::new(-1_469_177, -15_836_537),
            Point::new(-862_793, -15_909_651),
            Point::new(-831_312, -15_909_411),
            Point::new(-1_519_521, -15_813_200),
            Point::new(-2_495_958, -15_601_912),
            Point::new(-3_040_178, -15_548_538),
            Point::new(-7_889_784, -15_548_538),
            Point::new(-5_249_484, -18_188_838),
            Point::new(-4_583_039, -17_856_194),
            Point::new(-3_765_477, -17_595_579),
            Point::new(-2_926_077, -17_465_343),
            Point::new(-2_073_923, -17_465_343),
            Point::new(-1_234_523, -17_595_579),
            Point::new(-416_961, -17_856_194),
            Point::new(347_353, -18_237_688),
            Point::new(1_041_851, -18_730_630),
            Point::new(1_480_065, -19_137_235),
            Point::new(1_657_323, -19_329_792),
            Point::new(2_027_204, -19_793_598),
            Point::new(2_170_378, -20_012_730),
            Point::new(2_499_256, -20_582_360),
        ]);

        let generated = generate(&[outline], config).unwrap();

        assert!(
            generated
                .toolpaths
                .iter()
                .flatten()
                .flat_map(|line| &line.junctions)
                .all(|junction| junction.width > 0)
        );
    }
}
