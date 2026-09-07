use super::*;

fn vertex(
    layer_id: u32,
    z: f32,
    move_type: MoveType,
    role: GCodeExtrusionRole,
    times: [f32; 2],
) -> PathVertex {
    PathVertex {
        layer_id,
        position: [0.0, 0.0, z],
        move_type,
        role,
        times,
        ..PathVertex::default()
    }
}

#[test]
fn update_accumulates_layer_times_z_and_view_range_data() {
    let mut layers = Layers::default();
    layers.update(
        &vertex(
            0,
            0.2,
            MoveType::Extrude,
            GCodeExtrusionRole::Perimeter,
            [1.0, 2.0],
        ),
        7,
    );
    layers.update(
        &vertex(
            0,
            0.2,
            MoveType::Travel,
            GCodeExtrusionRole::None,
            [3.0, 4.0],
        ),
        8,
    );
    layers.update(
        &vertex(
            1,
            0.4,
            MoveType::Extrude,
            GCodeExtrusionRole::InternalInfill,
            [5.0, 6.0],
        ),
        9,
    );

    assert_eq!(layers.count(), 2);
    assert_eq!(layers.get_zs(), vec![0.2, 0.4]);
    assert_eq!(layers.get_times(TimeMode::Normal), vec![4.0, 5.0]);
    assert_eq!(layers.get_times(TimeMode::Stealth), vec![6.0, 6.0]);
    assert_eq!(layers.get_layer_time(TimeMode::Normal, 0), 4.0);
    assert_eq!(layers.get_layer_z(1), 0.4);
}

#[test]
fn update_detects_pause_and_custom_gcode_colorprint_options() {
    let mut layers = Layers::default();
    layers.update(
        &vertex(
            0,
            0.0,
            MoveType::PausePrint,
            GCodeExtrusionRole::None,
            [0.0, 0.0],
        ),
        0,
    );
    layers.update(
        &vertex(
            1,
            1.0,
            MoveType::CustomGCode,
            GCodeExtrusionRole::None,
            [0.0, 0.0],
        ),
        1,
    );

    assert!(layers.layer_contains_colorprint_options(0));
    assert!(layers.layer_contains_colorprint_options(1));
    assert!(!layers.layer_contains_colorprint_options(2));
}

#[test]
fn custom_extrusion_does_not_capture_layer_z() {
    let mut layers = Layers::default();
    layers.update(
        &vertex(
            0,
            2.0,
            MoveType::Extrude,
            GCodeExtrusionRole::Custom,
            [0.0, 0.0],
        ),
        0,
    );
    assert_eq!(layers.get_layer_z(0), 0.0);
}

#[test]
fn layer_lookup_returns_first_layer_with_z_at_or_above_query() {
    let mut layers = Layers::default();
    layers.update(
        &vertex(
            0,
            0.2,
            MoveType::Extrude,
            GCodeExtrusionRole::Perimeter,
            [0.0, 0.0],
        ),
        0,
    );
    layers.update(
        &vertex(
            1,
            0.4,
            MoveType::Extrude,
            GCodeExtrusionRole::Perimeter,
            [0.0, 0.0],
        ),
        1,
    );

    assert_eq!(layers.get_layer_id_at(0.1), 2);
    assert_eq!(layers.get_layer_id_at(0.2), 2);
    assert_eq!(layers.get_layer_id_at(0.3), 2);
    assert_eq!(layers.get_layer_id_at(0.4), 2);
    assert_eq!(layers.get_layer_id_at(0.5), 0);
}

#[test]
fn reset_clears_layers_and_view_range() {
    let mut layers = Layers::default();
    layers.update(
        &vertex(
            0,
            0.2,
            MoveType::Extrude,
            GCodeExtrusionRole::Perimeter,
            [1.0, 1.0],
        ),
        3,
    );
    layers.set_view_range(1, 3);
    layers.reset();

    assert!(layers.empty());
    assert_eq!(layers.get_view_range(), [0, 0]);
}
