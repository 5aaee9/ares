// Source boundary: OrcaSlicer v2.4.2 `GCode.cpp:4982-5000,5010-5068,8220-8249`
// island membership and sortable collection flattening.

use crate::{
    geometry::{Coord, ExPolygon, Point, fixed_gcc_sort_by},
    project_slice::{
        fill_entities::{
            FillExtrusionCollection, FillExtrusionEntity, LayerFillEntities,
            PreparedPostFillEntities,
        },
        perimeters::classic::{
            entity_collections::ExtrusionEntityCollection, gap_extrusion::GapFillEntity,
        },
    },
};

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) enum IslandInfillEntity {
    Fill(FillExtrusionEntity),
    FillCollection(FillExtrusionCollection),
    Thin(GapFillEntity),
}

#[derive(Debug, Default, PartialEq)]
pub(in crate::project_slice) struct ExtrusionIsland {
    pub(in crate::project_slice) infills: Vec<IslandInfillEntity>,
    pub(in crate::project_slice) perimeters: Vec<ExtrusionEntityCollection>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct LayerExtrusionIslands {
    pub(in crate::project_slice) islands: Vec<ExtrusionIsland>,
}

pub(in crate::project_slice) struct PreparedPostExtrusionIslands {
    pub(in crate::project_slice) predecessor: PreparedPostFillEntities,
    pub(in crate::project_slice) objects: Vec<Vec<LayerExtrusionIslands>>,
}

pub(in crate::project_slice) fn prepare(
    mut predecessor: PreparedPostFillEntities,
) -> PreparedPostExtrusionIslands {
    let objects = {
        let traversal = &predecessor
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .predecessor;
        traversal
            .objects
            .iter()
            .zip(&mut predecessor.objects)
            .map(|(source, layers)| {
                let prelude = &source
                    .predecessor
                    .predecessor
                    .predecessor
                    .predecessor
                    .object;
                let (compensated, _) = prelude.as_parts();
                let (_, layer_slices) = compensated.as_parts();
                layers
                    .iter_mut()
                    .zip(layer_slices)
                    .map(|(layer, slices)| assign_layer(layer, slices))
                    .collect()
            })
            .collect()
    };
    PreparedPostExtrusionIslands {
        predecessor,
        objects,
    }
}

fn assign_layer(layer: &mut LayerFillEntities, slices: &[ExPolygon]) -> LayerExtrusionIslands {
    let bounds = slices.iter().map(contour_bounds).collect::<Vec<_>>();
    let mut order = (0..slices.len()).collect::<Vec<_>>();
    fixed_gcc_sort_by(&mut order, |left, right| {
        bounds[*left].area() < bounds[*right].area()
    });
    let mut islands = (0..=slices.len())
        .map(|_| ExtrusionIsland::default())
        .collect::<Vec<_>>();

    for collection in std::mem::take(&mut layer.collections) {
        let island = island_index(collection.first_point(), slices, &bounds, &order);
        if let Ok(path) = std::env::var("ARES_DUMP_IORDER") {
            use std::io::Write;
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                let _ = writeln!(
                    file,
                    "ASSIGN island={} n={} kinds={}",
                    island,
                    collection.entities.len(),
                    collection
                        .entities
                        .iter()
                        .map(|entity| match entity {
                            FillExtrusionEntity::Path(path) => format!("P{:.3}", path.width),
                            FillExtrusionEntity::VariableWidth(_) => "V".to_owned(),
                        })
                        .collect::<Vec<_>>()
                        .join(",")
                );
            }
        }
        if collection.no_sort {
            islands[island]
                .infills
                .push(IslandInfillEntity::FillCollection(collection));
        } else {
            islands[island].infills.extend(
                collection
                    .entities
                    .into_iter()
                    .map(IslandInfillEntity::Fill),
            );
        }
    }
    let perimeters = std::mem::take(&mut layer.perimeters);
    for (source_order, mut perimeter) in perimeters.into_iter().enumerate() {
        perimeter.source_order = source_order;
        let first = *perimeter.entities[0].first_point3();
        let island = island_index(Point::new(first.x, first.y), slices, &bounds, &order);
        islands[island].perimeters.push(perimeter);
    }
    for thin in std::mem::take(&mut layer.thin_fills) {
        let island = island_index(thin_first_point(&thin), slices, &bounds, &order);
        islands[island].infills.push(IslandInfillEntity::Thin(thin));
    }
    LayerExtrusionIslands { islands }
}

fn thin_first_point(entity: &GapFillEntity) -> Point {
    let point = match entity {
        GapFillEntity::Path(path) => path.polyline.points[0],
        GapFillEntity::Loop(paths) => paths[0].polyline.points[0],
    };
    Point::new(point.x, point.y)
}

fn island_index(point: Point, slices: &[ExPolygon], bounds: &[Bounds], order: &[usize]) -> usize {
    order
        .iter()
        .copied()
        .find(|&index| bounds[index].contains(point) && slices[index].contour().contains(&point))
        .unwrap_or(slices.len())
}

#[derive(Clone, Copy)]
struct Bounds {
    min_x: Coord,
    min_y: Coord,
    max_x: Coord,
    max_y: Coord,
}

impl Bounds {
    fn contains(self, point: Point) -> bool {
        point.x() >= self.min_x
            && point.x() < self.max_x
            && point.y() >= self.min_y
            && point.y() < self.max_y
    }

    fn area(self) -> i128 {
        i128::from(self.max_x - self.min_x) * i128::from(self.max_y - self.min_y)
    }
}

fn contour_bounds(expolygon: &ExPolygon) -> Bounds {
    let first = expolygon.contour().points()[0];
    expolygon.contour().points().iter().copied().fold(
        Bounds {
            min_x: first.x(),
            min_y: first.y(),
            max_x: first.x(),
            max_y: first.y(),
        },
        |bounds, point| Bounds {
            min_x: bounds.min_x.min(point.x()),
            min_y: bounds.min_y.min(point.y()),
            max_x: bounds.max_x.max(point.x()),
            max_y: bounds.max_y.max(point.y()),
        },
    )
}

pub(in crate::project_slice) fn dispose(prepared: PreparedPostExtrusionIslands) {
    super::fill_entities::dispose(prepared.predecessor);
}
#[cfg(test)]
mod tests;
