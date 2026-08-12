mod logical;
mod spatial;
mod value;

use fenestra_ui_ir::prototype::{
    ConstructionProgram, SchemaManifest, SpatialProgramV2, StyleProgram,
};

pub fn manual_hybrid_spatial_v2() -> (
    SchemaManifest,
    ConstructionProgram,
    StyleProgram,
    SpatialProgramV2,
) {
    (
        logical::schema(),
        logical::construction(),
        logical::style(),
        spatial::program(),
    )
}
