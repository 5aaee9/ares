use super::*;

#[test]
fn default_matches_upstream_path_vertex() {
    let vertex = PathVertex::default();
    assert_eq!(vertex.position, [f32::MAX, f32::MAX, f32::MAX]);
    assert_eq!(vertex.role, GCodeExtrusionRole::None);
    assert_eq!(vertex.move_type, MoveType::Noop);
    assert_eq!(vertex.times, [0.0, 0.0]);
    assert_eq!(vertex, PathVertex::DUMMY_PATH_VERTEX);
}

#[test]
fn helper_methods_match_move_and_role_semantics() {
    let mut vertex = PathVertex {
        move_type: MoveType::Extrude,
        ..PathVertex::default()
    };
    assert!(vertex.is_extrusion());
    assert!(!vertex.is_option());
    assert!(!vertex.is_custom_gcode());

    vertex.role = GCodeExtrusionRole::Custom;
    assert!(vertex.is_custom_gcode());

    vertex.move_type = MoveType::Travel;
    assert!(vertex.is_travel());
    assert!(!vertex.is_custom_gcode());

    vertex.move_type = MoveType::Wipe;
    assert!(vertex.is_wipe());

    vertex.move_type = MoveType::PausePrint;
    assert!(vertex.is_option());
}

#[test]
fn volumetric_rates_use_feedrates_and_mm3_per_mm() {
    let vertex = PathVertex {
        feedrate: 10.0,
        actual_feedrate: 8.0,
        mm3_per_mm: 0.25,
        ..PathVertex::default()
    };
    assert_eq!(vertex.volumetric_rate(), 2.5);
    assert_eq!(vertex.actual_volumetric_rate(), 2.0);
}
