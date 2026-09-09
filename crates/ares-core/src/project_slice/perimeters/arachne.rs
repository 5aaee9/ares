// Arachne integration seam 3: variable-width wall-entity conversion from
// OrcaSlicer v2.4.2 `PerimeterGenerator.cpp:370-574::traverse_extrusions`
// (called at `PerimeterGenerator.cpp:2471` from `process_arachne`), turning
// ordered Arachne `ExtrusionLine` walls into the typed
// `ExtrusionEntityCollection` vocabulary of seam 2. Arachne dispatch stays
// typed-rejected; wall generation and overhang clipping are later seams.

mod traverse;
pub(in crate::project_slice) mod variable_width;

// Constructed only by the seam tests until the arachne materialization
// transaction (`PerimeterGenerator.cpp:2093-2519`) dispatches here.
#[cfg_attr(not(test), allow(unused_imports))]
pub(in crate::project_slice) use traverse::{
    PerimeterGeneratorArachneExtrusion, TraverseExtrusionsContext, traverse_extrusions,
};
