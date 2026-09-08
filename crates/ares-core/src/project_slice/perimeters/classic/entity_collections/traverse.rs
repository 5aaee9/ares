use crate::ProcessWallDirection;

use super::{
    super::{
        chained_loops::{ChainedLoopNode, ExtrusionLoop},
        shortest_path::chain_extrusion_loops,
        traversal::TraversalSeed,
    },
    orientation::orient_loop,
    types::{ExtrusionEntity, ExtrusionEntityCollection, OrderedExtrusionLoop},
};

struct PendingParent {
    extrusion_loop: ExtrusionLoop,
    is_contour: bool,
    depth: u16,
}

struct GroupFrame<'a> {
    nodes: Vec<ChainedLoopNode>,
    seeds: &'a [TraversalSeed],
    entities: Vec<Option<ExtrusionLoop>>,
    chain: Vec<(usize, bool)>,
    next: usize,
    reverse_thin_wall_hole: bool,
    pending_parent: Option<PendingParent>,
    output: ExtrusionEntityCollection,
}

impl<'a> GroupFrame<'a> {
    fn new(
        mut nodes: Vec<ChainedLoopNode>,
        seeds: &'a [TraversalSeed],
        reverse_thin_wall_hole: bool,
    ) -> Self {
        assert_eq!(nodes.len(), seeds.len());
        let compact: Vec<_> = nodes
            .iter_mut()
            .filter_map(|node| node.extrusion_loop.take())
            .collect();
        let chain = chain_extrusion_loops(&compact);
        Self {
            nodes,
            seeds,
            entities: compact.into_iter().map(Some).collect(),
            chain,
            next: 0,
            reverse_thin_wall_hole,
            pending_parent: None,
            output: ExtrusionEntityCollection::default(),
        }
    }

    fn begin_next(&mut self) -> Option<(Vec<ChainedLoopNode>, &'a [TraversalSeed], bool)> {
        let &(index, reverse) = self.chain.get(self.next)?;
        debug_assert!(!reverse);
        self.next += 1;
        let extrusion_loop = self.entities[index]
            .take()
            .expect("entity chain indices are unique");
        let seed = &self.seeds[index];
        let reverse_children_thin_wall_hole = self.seeds.len() == 1
            && seed.is_contour
            && seed.children.len() == 1
            && !seed.children[0].is_contour
            && seed.children[0].children.is_empty();
        let children = std::mem::take(&mut self.nodes[index].children);
        self.pending_parent = Some(PendingParent {
            extrusion_loop,
            is_contour: seed.is_contour,
            depth: seed.depth,
        });
        Some((
            children,
            seed.children.as_slice(),
            reverse_children_thin_wall_hole,
        ))
    }
}

pub(super) fn traverse_loops(
    roots: Vec<ChainedLoopNode>,
    seeds: &[TraversalSeed],
    wall_direction: ProcessWallDirection,
) -> ExtrusionEntityCollection {
    let mut frames = vec![GroupFrame::new(roots, seeds, false)];
    let mut completed: Option<ExtrusionEntityCollection> = None;
    loop {
        if let Some(mut children) = completed.take() {
            let frame = frames.last_mut().expect("a child has a parent frame");
            let mut parent = frame
                .pending_parent
                .take()
                .expect("a child result has a pending parent");
            orient_loop(
                &mut parent.extrusion_loop,
                wall_direction,
                parent.is_contour,
                frame.reverse_thin_wall_hole,
            );
            if frame.reverse_thin_wall_hole {
                frame.output.entities.reverse();
            }
            let is_contour = parent.is_contour;
            let parent = ExtrusionEntity::Loop(OrderedExtrusionLoop {
                extrusion_loop: parent.extrusion_loop,
                inset_idx: i32::from(parent.depth),
            });
            if is_contour {
                frame.output.entities.append(&mut children.entities);
                frame.output.entities.push(parent);
            } else {
                frame.output.entities.push(parent);
                frame.output.entities.append(&mut children.entities);
            }
            continue;
        }

        if let Some((children, child_seeds, reverse_children_thin_wall_hole)) = frames
            .last_mut()
            .expect("the root frame remains until return")
            .begin_next()
        {
            frames.push(GroupFrame::new(
                children,
                child_seeds,
                reverse_children_thin_wall_hole,
            ));
            continue;
        }

        let frame = frames.pop().expect("a frame exists");
        drain_nodes(frame.nodes);
        let result = frame.output;
        if frames.is_empty() {
            return result;
        }
        completed = Some(result);
    }
}

fn drain_nodes(mut nodes: Vec<ChainedLoopNode>) {
    while let Some(mut node) = nodes.pop() {
        nodes.append(&mut node.children);
    }
}
