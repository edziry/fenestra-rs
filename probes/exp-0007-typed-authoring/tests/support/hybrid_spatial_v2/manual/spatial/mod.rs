mod layout;
mod nodes;
mod scene;
mod tile;

use fenestra_ui_ir::prototype::{
    SUPPORTED_SPATIAL_FORMAT, SchemaNamespace, SchemaRevision, SpatialAxisV2,
    SpatialImageDeclarationV2, SpatialProgramV2, SpatialViewportContainerV2,
};

use super::value::{field, image, span};

pub(super) fn program() -> SpatialProgramV2 {
    SpatialProgramV2::new(
        SUPPORTED_SPATIAL_FORMAT,
        SchemaNamespace::new(13_013),
        SchemaRevision::new(2),
        SpatialViewportContainerV2::new(
            SpatialAxisV2::Row,
            field(4, 53),
            field(4, 54),
            field(3, 55),
            field(3, 56),
            field(2, 57),
            span(52),
        ),
        vec![
            scene::scene(),
            nodes::stack(),
            nodes::floating(),
            nodes::floating_child(),
            tile::tile(),
            nodes::guide(),
            nodes::viewport_layer(),
        ],
        vec![image_declaration()],
        span(51),
    )
}

fn image_declaration() -> SpatialImageDeclarationV2 {
    SpatialImageDeclarationV2::new(
        image(0, 60),
        field(2, 61),
        field(2, 62),
        field(8, 63),
        vec![255, 0, 0, 255, 0, 128, 0, 128, 0, 0, 64, 64, 0, 0, 0, 0].into_boxed_slice(),
        span(59),
    )
}
