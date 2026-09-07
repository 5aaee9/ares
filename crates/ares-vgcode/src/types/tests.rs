use super::*;

#[test]
fn enum_discriminants_and_counts_match_upstream_order() {
    assert_eq!(ViewType::Summary as u8, 0);
    assert_eq!(ViewType::Tool as u8, 16);
    assert_eq!(ViewType::COUNT, 17);
    assert_eq!(MoveType::Noop as u8, 0);
    assert_eq!(MoveType::Extrude as u8, 10);
    assert_eq!(MoveType::COUNT, 11);
    assert_eq!(GCodeExtrusionRole::None as u8, 0);
    assert_eq!(GCodeExtrusionRole::Mixed as u8, 19);
    assert_eq!(GCodeExtrusionRole::COUNT, 20);
    assert_eq!(OptionType::Travels as u8, 0);
    assert_eq!(OptionType::CustomGCodes as u8, 8);
    assert_eq!(OptionType::COUNT, 9);
    assert_eq!(TimeMode::Normal as u8, 0);
    assert_eq!(TimeMode::Stealth as u8, 1);
    assert_eq!(TimeMode::COUNT, 2);
    assert_eq!(ColorRangeType::Linear as u8, 0);
    assert_eq!(ColorRangeType::Logarithmic as u8, 1);
    assert_eq!(ColorRangeType::COUNT, 2);
    assert_eq!(TimeMode::Stealth.index(), 1);
}

#[test]
fn move_types_map_to_matching_option_types() {
    assert_eq!(
        move_type_to_option(MoveType::Travel),
        Some(OptionType::Travels)
    );
    assert_eq!(move_type_to_option(MoveType::Wipe), Some(OptionType::Wipes));
    assert_eq!(
        move_type_to_option(MoveType::Retract),
        Some(OptionType::Retractions)
    );
    assert_eq!(
        move_type_to_option(MoveType::Unretract),
        Some(OptionType::Unretractions)
    );
    assert_eq!(move_type_to_option(MoveType::Seam), Some(OptionType::Seams));
    assert_eq!(
        move_type_to_option(MoveType::ToolChange),
        Some(OptionType::ToolChanges)
    );
    assert_eq!(
        move_type_to_option(MoveType::ColorChange),
        Some(OptionType::ColorChanges)
    );
    assert_eq!(
        move_type_to_option(MoveType::PausePrint),
        Some(OptionType::PausePrints)
    );
    assert_eq!(
        move_type_to_option(MoveType::CustomGCode),
        Some(OptionType::CustomGCodes)
    );
    assert_eq!(move_type_to_option(MoveType::Noop), None);
    assert_eq!(move_type_to_option(MoveType::Extrude), None);
}

#[test]
fn color_lerp_clamps_and_truncates_channels() {
    assert_eq!(
        lerp_color([0, 100, 200], [100, 200, 250], -0.5),
        [0, 100, 200]
    );
    assert_eq!(
        lerp_color([0, 100, 200], [100, 200, 250], 1.5),
        [100, 200, 250]
    );
    assert_eq!(lerp_color([0, 0, 0], [255, 127, 1], 0.5), [127, 63, 0]);
}
