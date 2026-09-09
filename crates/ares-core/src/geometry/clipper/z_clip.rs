// `clip_extrusion` port from OrcaSlicer v2.4.2
// `PerimeterGenerator.cpp:311-357`: clips one open variable-width wall
// polyline (width as the Z coordinate) against closed polygons, keeping a
// width on every output vertex. The upstream ZFill callback interpolates the
// subject width at true edge crossings; this engine records those crossings
// as sentinel values instead, so callers must reject them (`z > 0` guard).

use super::{
    ClipOperation, Clipper, ClipperError, ClipperOptions, FillRule, PathRole, Polygon,
    z::{KernelPoint, ZPath},
};

/// One clipped wall polyline vertex: `(x, y, width)` with the width in
/// scaled coordinates.
pub(crate) type ZWidthPath = Vec<(i64, i64, i64)>;

pub(crate) fn clip_open_path_widths(
    subject: &ZWidthPath,
    clip: &[Polygon],
    operation: ClipOperation,
) -> Result<Vec<ZWidthPath>, ClipperError> {
    let subject_path = subject
        .iter()
        .map(|&(x, y, width)| KernelPoint::new(x, y, width))
        .collect::<ZPath>();
    let mut clipper = Clipper::new(ClipperOptions::default());
    for polygon in clip {
        clipper.add_z_closed_path(
            &polygon
                .points()
                .iter()
                .map(|&point| KernelPoint::new(point.x(), point.y(), 0))
                .collect::<ZPath>(),
            PathRole::Clip,
        )?;
    }
    clipper.add_z_open_path(&subject_path, PathRole::Subject)?;
    let (paths, _) = clipper.execute_z_paths(operation, FillRule::NonZero, FillRule::NonZero);
    Ok(paths
        .into_iter()
        .map(|path| {
            path.into_iter()
                .map(|point| (point.x(), point.y(), point.z))
                .collect::<ZWidthPath>()
        })
        .collect())
}
