pub(crate) mod beading;
mod extrusion_line;
pub(crate) mod skeletal;
mod trapezoidation;
pub(crate) mod wall_toolpaths;

#[cfg(test)]
pub(crate) use extrusion_line::ExtrusionJunction;
pub(crate) use extrusion_line::ExtrusionLine;

#[cfg(test)]
mod tests;
