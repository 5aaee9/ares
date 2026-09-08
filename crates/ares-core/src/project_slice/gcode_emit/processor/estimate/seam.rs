//! Reached planar `GCodeProcessor::process_G1` / `SeamsDetector` lifecycle.

#[derive(Default)]
pub(super) struct SeamDetector {
    active: bool,
    first_vertex: Option<[f32; 3]>,
    last_vertex: [f32; 3],
    external: bool,
    overhang: bool,
}

impl SeamDetector {
    pub(super) fn role(&mut self, line: &str) {
        let Some(role) = line
            .strip_prefix(";TYPE:")
            .or_else(|| line.strip_prefix("; FEATURE: "))
        else {
            return;
        };
        self.external = role == "Outer wall";
        self.overhang = role == "Overhang wall";
        if self.external && !self.active {
            self.active = true;
            self.first_vertex = None;
        }
    }

    pub(super) fn associate(&mut self, extruding: bool, end: [f64; 3]) -> bool {
        let mut seam_vertex = false;
        if self.active {
            if extruding && self.external {
                self.first_vertex.get_or_insert(self.last_vertex);
            } else if (!extruding || !self.overhang)
                && let Some(first) = self.first_vertex
            {
                // Upstream tests the previous stored vertex, not this move's
                // endpoint, after assigning the block's next-vertex index.
                let delta = std::array::from_fn::<_, 3, _>(|i| self.last_vertex[i] - first[i]);
                seam_vertex =
                    delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2] < 0.0625_f32;
                self.active = false;
            }
        } else if extruding && self.external {
            self.active = true;
            self.first_vertex = Some(self.last_vertex);
        }
        self.last_vertex = end.map(|value| value as f32);
        seam_vertex
    }

    pub(super) fn advance(&mut self, end: [f64; 3]) {
        self.last_vertex = end.map(|value| value as f32);
    }
}
