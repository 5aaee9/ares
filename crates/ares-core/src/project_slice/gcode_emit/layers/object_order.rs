//! `ShortestPath.cpp:2015-2042` `chain_print_object_instances`: order print
//! object instances with the multi-fragment greedy TSP over instance centers
//! (`chain_segments_greedy_`, `ShortestPath.cpp:92-410`). Each instance is a
//! degenerate segment whose two endpoints both sit at the instance center, so
//! reversal is always allowed and the produced order is the chained path.

#[derive(Clone, Copy)]
struct EndPoint {
    edge_out: Option<usize>,
    distance_out: f64,
    chain_id: usize,
}

fn queue_key(end_points: &[EndPoint], index: usize) -> (f64, usize) {
    (end_points[index].distance_out, index)
}

fn sift_down(end_points: &[EndPoint], queue: &mut Vec<usize>, start: usize) {
    let mut root = start;
    loop {
        let left = 2 * root + 1;
        let right = left + 1;
        let mut smallest = root;
        if left < queue.len()
            && queue_key(end_points, queue[left]) < queue_key(end_points, queue[smallest])
        {
            smallest = left;
        }
        if right < queue.len()
            && queue_key(end_points, queue[right]) < queue_key(end_points, queue[smallest])
        {
            smallest = right;
        }
        if smallest == root {
            return;
        }
        queue.swap(root, smallest);
        root = smallest;
    }
}

fn pop_top(end_points: &[EndPoint], queue: &mut Vec<usize>) -> usize {
    let top = queue[0];
    let last = queue.pop().expect("queue not empty");
    if !queue.is_empty() {
        queue[0] = last;
        sift_down(end_points, queue, 0);
    }
    top
}

fn remove_at(end_points: &[EndPoint], queue: &mut Vec<usize>, target: usize) {
    let Some(position) = queue.iter().position(|&item| item == target) else {
        return;
    };
    let last = queue.pop().expect("queue not empty");
    if position >= queue.len() {
        return;
    }
    queue[position] = last;
    let mut node = position;
    while node > 0 {
        let parent = (node - 1) / 2;
        if queue_key(end_points, queue[node]) < queue_key(end_points, queue[parent]) {
            queue.swap(node, parent);
            node = parent;
        } else {
            break;
        }
    }
    sift_down(end_points, queue, node);
}

/// Union-find over chain ids (`EquivalentChains`, `ShortestPath.cpp:135-166`).
struct EquivalentChains {
    equivalent_with: Vec<usize>,
    last_chain_id: usize,
}

impl EquivalentChains {
    fn new() -> Self {
        Self {
            equivalent_with: vec![0],
            last_chain_id: 0,
        }
    }

    fn class_of(&mut self, chain_id: usize) -> usize {
        if chain_id == 0 {
            return 0;
        }
        let mut last = chain_id;
        loop {
            let lower = self.equivalent_with[last];
            if lower == last {
                break;
            }
            last = lower;
        }
        self.equivalent_with[chain_id] = last;
        last
    }

    fn new_chain(&mut self) -> usize {
        self.last_chain_id += 1;
        self.equivalent_with.push(self.last_chain_id);
        self.last_chain_id
    }

    fn merge(&mut self, chain1: usize, chain2: usize) -> usize {
        let merged = self.class_of(chain1).min(self.class_of(chain2));
        let a = self.class_of(chain1);
        let b = self.class_of(chain2);
        self.equivalent_with[a] = merged;
        self.equivalent_with[b] = merged;
        merged
    }
}

/// Orders `centers` (print object instance centers, print-object order)
/// the way upstream `chain_print_object_instances(print_objects,
/// start_near)` does (`ShortestPath.cpp:2015-2036`) and returns the
/// permutation of instance indices in chained path order. `seed` is the
/// `start_near` point in scaled integer units.
pub(super) fn chain_instance_order(centers: &[(f64, f64)], start_near: (i64, i64)) -> Vec<usize> {
    let count = centers.len();
    if count <= 1 {
        return (0..count).collect();
    }
    // Upstream chains in scaled integer coordinates; the squared-distance
    // comparisons are scale-invariant, so only the seed keeps its raw
    // scaled units (`GCode.cpp:5115` constructs `Point wt_pos(x_mm,
    // y_mm)`, which truncates into coord_t — the configured millimetre
    // values land a few microns from the origin).
    let scaled = |point: (f64, f64)| -> (i64, i64) {
        (
            (point.0 / 1.0e-6).round() as i64,
            (point.1 / 1.0e-6).round() as i64,
        )
    };
    let scaled_centers: Vec<(i64, i64)> = centers.iter().map(|&center| scaled(center)).collect();
    let position = |endpoint: usize| scaled_centers[endpoint >> 1];
    let distance_sq = |a: usize, b: usize| {
        let (ax, ay) = position(a);
        let (bx, by) = position(b);
        let dx = ax - bx;
        let dy = ay - by;
        dx * dx + dy * dy
    };
    // Closest end point of a different instance (`(idx ^ this_idx) > 1`),
    // additionally excluding `skip` (the occupied start point); ties
    // resolve to the lowest endpoint index.
    let nearest = |this: usize, skip: Option<usize>| -> (usize, f64) {
        let mut best: Option<(usize, f64)> = None;
        for candidate in 0..count * 2 {
            if (candidate ^ this) <= 1 || Some(candidate) == skip {
                continue;
            }
            let distance = distance_sq(this, candidate) as f64;
            if best.is_none_or(|(_, best_distance)| distance < best_distance) {
                best = Some((candidate, distance));
            }
        }
        best.expect("at least two instances")
    };

    let mut chains = EquivalentChains::new();
    let mut end_points = vec![
        EndPoint {
            edge_out: None,
            distance_out: f64::INFINITY,
            chain_id: 0,
        };
        count * 2
    ];
    let distance_to_start = |endpoint: usize| {
        let (x, y) = position(endpoint);
        let (dx, dy) = (x - start_near.0, y - start_near.1);
        dx * dx + dy * dy
    };
    // `start_near` designates the start end point; it carries a chain of
    // its own and never enters the queue or the neighbor searches
    // (`ShortestPath.cpp:178-198`). Ties resolve to the lowest endpoint.
    let seed_point = (0..count * 2)
        .min_by_key(|&endpoint| distance_to_start(endpoint))
        .expect("at least one end point");
    let seed_chain = chains.new_chain();
    end_points[seed_point].distance_out = 0.0;
    end_points[seed_point].chain_id = seed_chain;
    for index in 0..count * 2 {
        if index == seed_point {
            continue;
        }
        let (next, distance) = nearest(index, Some(seed_point));
        end_points[index].edge_out = Some(next);
        end_points[index].distance_out = distance;
    }

    let mut queue: Vec<usize> = (0..count * 2)
        .filter(|&index| index != seed_point)
        .collect();
    for start in (0..queue.len() / 2).rev() {
        sift_down(&end_points, &mut queue, start);
    }

    let mut first_point = Some(seed_point);
    let mut remaining_links = count - 1;
    while !queue.is_empty() {
        let end_point1 = pop_top(&end_points, &mut queue);
        let Some(end_point2) = end_points[end_point1].edge_out else {
            break;
        };
        let chain1 = chains.class_of(end_points[end_point1 ^ 1].chain_id);
        let chain2 = chains.class_of(end_points[end_point2 ^ 1].chain_id);
        let valid = end_points[end_point2].chain_id == 0 && (chain1 != chain2 || chain1 == 0);
        if valid {
            remove_at(&end_points, &mut queue, end_point2);
            end_points[end_point2].edge_out = Some(end_point1);
            end_points[end_point2].distance_out = end_points[end_point1].distance_out;
            let chain1 = chains.class_of(end_points[end_point1 ^ 1].chain_id);
            let chain2 = chains.class_of(end_points[end_point2 ^ 1].chain_id);
            let chain_id = if chain1 == 0 {
                if chain2 == 0 {
                    chains.new_chain()
                } else {
                    chain2
                }
            } else if chain2 == 0 {
                chain1
            } else if chain1 == chain2 {
                chain1
            } else {
                chains.merge(chain1, chain2)
            };
            end_points[end_point1].chain_id = chain_id;
            end_points[end_point2].chain_id = chain_id;
            remaining_links -= 1;
            if remaining_links == 0 {
                // With a start point, exactly one end point remains
                // (`ShortestPath.cpp:322-333`); it closes the path.
                let tail = pop_top(&end_points, &mut queue);
                end_points[tail].edge_out = None;
                break;
            }
        } else {
            // Update the neighbor to the closest still-valid end point
            // (`ShortestPath.cpp:339-366`).
            let mut best: Option<(usize, f64)> = None;
            for candidate in 0..count * 2 {
                if (candidate ^ end_point1) <= 1 || end_points[candidate].chain_id != 0 {
                    continue;
                }
                let chain1 = chains.class_of(end_points[end_point1 ^ 1].chain_id);
                let chain2 = chains.class_of(end_points[candidate ^ 1].chain_id);
                if chain1 != chain2 || chain1 == 0 {
                    let distance = distance_sq(end_point1, candidate) as f64;
                    if best.is_none_or(|(_, best_distance)| distance < best_distance) {
                        best = Some((candidate, distance));
                    }
                }
            }
            match best {
                Some((candidate, distance)) => {
                    end_points[end_point1].edge_out = Some(candidate);
                    end_points[end_point1].distance_out = distance;
                    queue.push(end_point1);
                    let mut node = queue.len() - 1;
                    while node > 0 {
                        let parent = (node - 1) / 2;
                        if queue_key(&end_points, queue[node])
                            < queue_key(&end_points, queue[parent])
                        {
                            queue.swap(node, parent);
                            node = parent;
                        } else {
                            break;
                        }
                    }
                }
                None => {
                    end_points[end_point1].edge_out = None;
                    end_points[end_point1].distance_out = f64::INFINITY;
                }
            }
        }
    }
    let Some(mut walker) = first_point else {
        return (0..count).collect();
    };
    let mut order = Vec::with_capacity(count);
    loop {
        order.push(walker >> 1);
        let Some(next) = end_points[walker ^ 1].edge_out else {
            break;
        };
        walker = next;
    }
    order
}

#[cfg(test)]
mod tests;
