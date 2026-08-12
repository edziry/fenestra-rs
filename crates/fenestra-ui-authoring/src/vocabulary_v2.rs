/// Authoring frontend that supplied one format-2 compilation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthoringFrontendV2 {
    /// External UTF-8 `.fen` source.
    Fen,
    /// Rust `ui!` procedural-macro tokens.
    UiMacro,
}

impl AuthoringFrontendV2 {
    /// Every frontend in deterministic artifact order.
    pub const ALL: [Self; 2] = [Self::Fen, Self::UiMacro];
}

/// Semantic role of one logical format-2 authoring source anchor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnchorKindV2 {
    /// Authoring document header.
    Document,
    /// Schema header.
    Schema,
    /// Component declaration.
    Component,
    /// Property declaration.
    Property,
    /// Construction header.
    Construction,
    /// Template declaration.
    Template,
    /// Initial property assignment.
    InitialProperty,
    /// Static child reference.
    StaticChild,
    /// Structural region child reference.
    RegionChild,
    /// Structural region declaration.
    Region,
    /// Initial keyed member.
    InitialKey,
    /// Style header.
    Style,
    /// Exact style assignment.
    StyleAssignment,
    /// Spatial program header.
    Spatial,
    /// Spatial image-resource collection.
    Resources,
    /// Spatial image resource.
    Image,
    /// Symbolic spatial node.
    SpatialNode,
    /// Viewport or node container recipe.
    SpatialContainer,
    /// Spatial node placement recipe.
    SpatialPlacement,
    /// Spatial node transform recipe.
    SpatialTransform,
    /// Independently fallible spatial IR field.
    SpatialField,
    /// Spatial shape recipe.
    SpatialShape,
    /// Spatial path verb.
    SpatialPathVerb,
    /// Spatial polygon point.
    SpatialPolygonPoint,
    /// Spatial brush recipe.
    SpatialBrush,
    /// Spatial gradient stop.
    SpatialGradientStop,
    /// Spatial clip recipe.
    SpatialClip,
    /// Spatial paint recipe.
    SpatialPaint,
    /// Spatial hit recipe.
    SpatialHit,
    /// Spatial semantic recipe.
    SpatialSemantic,
}

impl AnchorKindV2 {
    /// Every anchor role in deterministic vocabulary order.
    pub const ALL: [Self; 30] = [
        Self::Document,
        Self::Schema,
        Self::Component,
        Self::Property,
        Self::Construction,
        Self::Template,
        Self::InitialProperty,
        Self::StaticChild,
        Self::RegionChild,
        Self::Region,
        Self::InitialKey,
        Self::Style,
        Self::StyleAssignment,
        Self::Spatial,
        Self::Resources,
        Self::Image,
        Self::SpatialNode,
        Self::SpatialContainer,
        Self::SpatialPlacement,
        Self::SpatialTransform,
        Self::SpatialField,
        Self::SpatialShape,
        Self::SpatialPathVerb,
        Self::SpatialPolygonPoint,
        Self::SpatialBrush,
        Self::SpatialGradientStop,
        Self::SpatialClip,
        Self::SpatialPaint,
        Self::SpatialHit,
        Self::SpatialSemantic,
    ];
}
