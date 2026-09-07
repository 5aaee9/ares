use super::*;

#[test]
fn range_orders_reversed_bounds_and_resets() {
    let mut range = Range::new(8, 3);
    assert_eq!(range.get(), [3, 8]);
    range.set_min(10);
    assert_eq!(range.get(), [8, 10]);
    range.reset();
    assert_eq!(range.get(), [0, 0]);
}

#[test]
fn range_clamps_another_range_inside_itself() {
    let range = Range::new(10, 20);
    let mut other = Range::new(0, 30);
    range.clamp(&mut other);
    assert_eq!(other.get(), [10, 20]);
}

#[test]
fn view_range_cascades_clamps() {
    let mut view = ViewRange::default();
    view.set_full(10, 20);
    view.set_enabled(12, 18);
    view.set_visible(0, 30);
    assert_eq!(view.get_visible(), [12, 18]);
    view.set_full(14, 16);
    assert_eq!(view.get_enabled(), [14, 16]);
    assert_eq!(view.get_visible(), [14, 16]);
    view.reset();
    assert_eq!(view.get_full(), [0, 0]);
    assert_eq!(view.get_enabled(), [0, 0]);
    assert_eq!(view.get_visible(), [0, 0]);
}
