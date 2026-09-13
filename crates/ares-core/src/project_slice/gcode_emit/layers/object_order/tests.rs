use super::chain_instance_order;

/// Upstream default seed (`GCode.cpp:5115`): the wipe-tower config values
/// truncated into scaled integers — microns from the origin.
const SEED: (i64, i64) = (15, 220);

#[test]
fn single_and_empty_inputs_pass_through() {
    assert!(chain_instance_order(&[], SEED).is_empty());
    assert_eq!(chain_instance_order(&[(1.0, 1.0)], SEED), vec![0]);
}

#[test]
fn collinear_instances_chain_monotonically() {
    // 0 -- 1 -- 2: the multi-fragment greedy links the two shortest edges;
    // the seed (near the origin) starts the path at instance 0 and the
    // caller-facing chain is reversed, ending there.
    let order = chain_instance_order(&[(0.0, 0.0), (10.0, 0.0), (20.0, 0.0)], SEED);
    assert_eq!(order.len(), 3);
    let joined = order
        .iter()
        .map(|&index| index.to_string())
        .collect::<String>();
    assert!(
        joined == "012" || joined == "210",
        "collinear chain must be a monotone path, got {joined}"
    );
}

#[test]
fn every_instance_appears_exactly_once() {
    let centers: Vec<(f64, f64)> = (0..8)
        .map(|index| {
            let angle = index as f64 * 0.7;
            (angle.cos() * (index as f64 + 1.0), angle.sin() * 7.0)
        })
        .collect();
    let mut order = chain_instance_order(&centers, SEED);
    assert_eq!(order.len(), centers.len());
    order.sort_unstable();
    assert_eq!(order, (0..centers.len()).collect::<Vec<_>>());
}

#[test]
fn grid_clusters_link_before_spanning() {
    // Two tight pairs far apart: the greedy must connect inside the pairs
    // before the long spanning edge regardless of instance numbering.
    let order = chain_instance_order(
        &[
            (0.0, 0.0),   // 0
            (100.0, 0.0), // 1
            (0.5, 0.0),   // 2
            (100.5, 0.0), // 3
        ],
        SEED,
    );
    assert_eq!(order.len(), 4);
    let adjacent: Vec<(usize, usize)> = order.windows(2).map(|pair| (pair[0], pair[1])).collect();
    let pair_of = |index: usize| if index == 0 || index == 2 { 0 } else { 1 };
    assert!(
        adjacent
            .iter()
            .filter(|(a, b)| pair_of(*a) == pair_of(*b))
            .count()
            >= 1,
        "at least one tight pair must be linked adjacently: {adjacent:?}"
    );
}

#[test]
fn seed_near_origin_orders_prisms_like_upstream() {
    // The arachne default-prisms plate: instance centers (mm) for
    // [hole, wide, neck] in print-object order. Upstream seeds at the
    // truncated wipe-tower point, chains, reverses — producing
    // wide, hole, neck (`chain_print_object_instances` +
    // `std::reverse`, `GCode.cpp:5120-5125`).
    let mut order = chain_instance_order(
        &[
            (124.000001, 130.999995), // hole
            (104.0, 131.000005),      // wide
            (114.438309, 113.999995), // neck
        ],
        SEED,
    );
    order.reverse();
    assert_eq!(order, vec![1, 0, 2]);
}
