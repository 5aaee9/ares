use super::*;

#[test]
fn input_data_default_is_empty_and_not_spiral_vase() {
    let data = GCodeInputData::default();
    assert!(!data.spiral_vase_mode);
    assert!(data.vertices.is_empty());
    assert!(data.tools_colors.is_empty());
    assert!(data.color_print_colors.is_empty());
}

#[test]
fn color_print_default_ids_and_times_are_zero() {
    let color_print = ColorPrint::default();
    assert_eq!(color_print.extruder_id, 0);
    assert_eq!(color_print.color_id, 0);
    assert_eq!(color_print.layer_id, 0);
    assert_eq!(color_print.times, [0.0, 0.0]);
}
