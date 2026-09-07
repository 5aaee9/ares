// Ports rendering-neutral data from AGPL-licensed OrcaSlicer `src/libvgcode/include/Types.hpp` and `src/Types.cpp`.

pub type Vec3 = [f32; 3];
pub type Mat4x4 = [f32; 16];
pub type Color = [u8; 3];
pub type Palette = Vec<Color>;
pub type AABox = [Vec3; 2];
pub type Interval = [usize; 2];

pub const DEFAULT_TRAVELS_RADIUS_MM: f32 = 0.1;
pub const MIN_TRAVELS_RADIUS_MM: f32 = 0.05;
pub const MAX_TRAVELS_RADIUS_MM: f32 = 1.0;
pub const DEFAULT_WIPES_RADIUS_MM: f32 = 0.1;
pub const MIN_WIPES_RADIUS_MM: f32 = 0.05;
pub const MAX_WIPES_RADIUS_MM: f32 = 1.0;
pub const DUMMY_COLOR: Color = [64, 64, 64];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ViewType {
    Summary,
    FeatureType,
    ColorPrint,
    Speed,
    ActualSpeed,
    Height,
    Width,
    VolumetricFlowRate,
    ActualVolumetricFlowRate,
    LayerTimeLinear,
    LayerTimeLogarithmic,
    FanSpeed,
    Temperature,
    PressureAdvance,
    Acceleration,
    Jerk,
    Tool,
}

impl ViewType {
    pub const COUNT: usize = 17;

    pub const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MoveType {
    Noop,
    Retract,
    Unretract,
    Seam,
    ToolChange,
    ColorChange,
    PausePrint,
    CustomGCode,
    Travel,
    Wipe,
    Extrude,
}

impl MoveType {
    pub const COUNT: usize = 11;

    pub const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum GCodeExtrusionRole {
    None,
    Perimeter,
    ExternalPerimeter,
    OverhangPerimeter,
    InternalInfill,
    SolidInfill,
    TopSolidInfill,
    Ironing,
    BridgeInfill,
    GapFill,
    Skirt,
    SupportMaterial,
    SupportMaterialInterface,
    WipeTower,
    Custom,
    BottomSurface,
    InternalBridgeInfill,
    Brim,
    SupportTransition,
    Mixed,
}

impl GCodeExtrusionRole {
    pub const COUNT: usize = 20;

    pub const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum OptionType {
    Travels,
    Wipes,
    Retractions,
    Unretractions,
    Seams,
    ToolChanges,
    ColorChanges,
    PausePrints,
    CustomGCodes,
}

impl OptionType {
    pub const COUNT: usize = 9;

    pub const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TimeMode {
    Normal,
    Stealth,
}

impl TimeMode {
    pub const COUNT: usize = 2;

    pub const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ColorRangeType {
    Linear,
    Logarithmic,
}

impl ColorRangeType {
    pub const COUNT: usize = 2;

    pub const fn index(self) -> usize {
        self as usize
    }
}

pub const fn move_type_to_option(move_type: MoveType) -> Option<OptionType> {
    match move_type {
        MoveType::Travel => Some(OptionType::Travels),
        MoveType::Wipe => Some(OptionType::Wipes),
        MoveType::Retract => Some(OptionType::Retractions),
        MoveType::Unretract => Some(OptionType::Unretractions),
        MoveType::Seam => Some(OptionType::Seams),
        MoveType::ToolChange => Some(OptionType::ToolChanges),
        MoveType::ColorChange => Some(OptionType::ColorChanges),
        MoveType::PausePrint => Some(OptionType::PausePrints),
        MoveType::CustomGCode => Some(OptionType::CustomGCodes),
        MoveType::Noop | MoveType::Extrude => None,
    }
}

pub fn lerp_color(c1: Color, c2: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    [
        lerp_channel(c1[0], c2[0], t),
        lerp_channel(c1[1], c2[1], t),
        lerp_channel(c1[2], c2[2], t),
    ]
}

fn lerp_channel(c1: u8, c2: u8, t: f32) -> u8 {
    ((1.0 - t) * f32::from(c1) + t * f32::from(c2)) as u8
}

#[cfg(test)]
mod tests;
