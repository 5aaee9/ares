use crate::{
    ProjectVolumeType,
    geometry::Coord,
    mesh_slicer::{LoopedLayer, make_loops},
};

use super::{chained_intersections::ChainedPrintObject, layers::PlannedPrintObject};

pub(super) struct LoopedVolumeIntersections {
    source_volume_index: usize,
    volume_ordinal: u32,
    volume_type: ProjectVolumeType,
    layers: Vec<LoopedLayer>,
}

impl LoopedVolumeIntersections {
    pub(super) fn into_parts(self) -> (usize, u32, ProjectVolumeType, Vec<LoopedLayer>) {
        (
            self.source_volume_index,
            self.volume_ordinal,
            self.volume_type,
            self.layers,
        )
    }

    #[cfg(test)]
    pub(super) const fn source_volume_index(&self) -> usize {
        self.source_volume_index
    }

    #[cfg(test)]
    pub(super) const fn ordinal(&self) -> u32 {
        self.volume_ordinal
    }

    #[cfg(test)]
    pub(super) const fn volume_type(&self) -> ProjectVolumeType {
        self.volume_type
    }

    #[cfg(test)]
    pub(super) fn layers(&self) -> &[LoopedLayer] {
        &self.layers
    }
}

pub(super) struct LoopedPrintObject {
    plan: PlannedPrintObject,
    volumes: Vec<LoopedVolumeIntersections>,
}

impl LoopedPrintObject {
    pub(super) fn into_parts(self) -> (PlannedPrintObject, Vec<LoopedVolumeIntersections>) {
        (self.plan, self.volumes)
    }

    #[cfg(test)]
    pub(super) fn plan(&self) -> &PlannedPrintObject {
        &self.plan
    }

    #[cfg(test)]
    pub(super) fn volumes(&self) -> &[LoopedVolumeIntersections] {
        &self.volumes
    }
}

pub(super) fn loop_project_intersections(
    objects: Vec<ChainedPrintObject>,
    max_gap_scaled: Coord,
) -> Vec<LoopedPrintObject> {
    objects
        .into_iter()
        .map(|object| {
            let (plan, volumes) = object.into_parts();
            let volumes = volumes
                .into_iter()
                .map(|volume| {
                    let (source_volume_index, volume_ordinal, volume_type, layers) =
                        volume.into_parts();
                    let layers = layers
                        .into_iter()
                        .enumerate()
                        .map(|(layer_index, layer)| {
                            let looped = make_loops(layer, max_gap_scaled);
                            if let Ok(path) = std::env::var("ARES_DUMP_SLICE") {
                                use std::io::Write;
                                if let Ok(mut file) = std::fs::OpenOptions::new()
                                    .create(true)
                                    .append(true)
                                    .open(path)
                                {
                                    for polygon in looped.polygons() {
                                        let _ = write!(file, "SLICE {layer_index}:");
                                        for point in polygon.points() {
                                            let _ = write!(file, " ({},{})", point.x(), point.y());
                                        }
                                        let _ = writeln!(file);
                                    }
                                }
                            }
                            looped
                        })
                        .collect();
                    LoopedVolumeIntersections {
                        source_volume_index,
                        volume_ordinal,
                        volume_type,
                        layers,
                    }
                })
                .collect();
            LoopedPrintObject { plan, volumes }
        })
        .collect()
}
