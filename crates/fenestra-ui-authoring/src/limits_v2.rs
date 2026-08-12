/// Inclusive resource category bounded by the format-2 authoring compiler.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthoringLimitKindV2 {
    /// Bytes in one `.fen` source.
    FenSourceBytes,
    /// Flattened abstract tokens.
    Tokens,
    /// Bytes in one identifier.
    IdentifierBytes,
    /// Delimiter nesting depth.
    NestingDepth,
    /// Component declarations.
    Components,
    /// Property declarations across all components.
    Properties,
    /// Template declarations.
    Templates,
    /// Structural region declarations.
    Regions,
    /// Child slots across all templates.
    ChildSlots,
    /// Initial property assignments across all templates.
    InitialProperties,
    /// Initial keys across all regions.
    InitialKeys,
    /// Exact style assignments.
    StyleAssignments,
    /// Spatial image declarations.
    Images,
    /// Bytes across all spatial images.
    ImageBytes,
    /// Symbolic spatial node declarations.
    SpatialNodes,
    /// Emitted symbolic spatial fields.
    SpatialFields,
    /// Spatial shape declarations.
    Shapes,
    /// Spatial path declarations.
    Paths,
    /// Verbs across all spatial paths.
    PathVerbs,
    /// Points across all spatial polygons.
    PolygonPoints,
    /// Spatial brush declarations.
    Brushes,
    /// Stops across all spatial gradients.
    GradientStops,
    /// Spatial clip declarations.
    Clips,
    /// Spatial paint recipes.
    PaintItems,
    /// Spatial hit recipes.
    HitItems,
    /// Spatial semantic recipes.
    SemanticItems,
    /// Logical source anchors.
    SourceAnchors,
    /// Bytes in generated Rust, including its final line feed.
    GeneratedRustBytes,
}

impl AuthoringLimitKindV2 {
    /// Every bounded resource in deterministic validation order.
    pub const ALL: [Self; 28] = [
        Self::FenSourceBytes,
        Self::Tokens,
        Self::IdentifierBytes,
        Self::NestingDepth,
        Self::Components,
        Self::Properties,
        Self::Templates,
        Self::Regions,
        Self::ChildSlots,
        Self::InitialProperties,
        Self::InitialKeys,
        Self::StyleAssignments,
        Self::Images,
        Self::ImageBytes,
        Self::SpatialNodes,
        Self::SpatialFields,
        Self::Shapes,
        Self::Paths,
        Self::PathVerbs,
        Self::PolygonPoints,
        Self::Brushes,
        Self::GradientStops,
        Self::Clips,
        Self::PaintItems,
        Self::HitItems,
        Self::SemanticItems,
        Self::SourceAnchors,
        Self::GeneratedRustBytes,
    ];

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::FenSourceBytes => "fen-source-bytes",
            Self::Tokens => "tokens",
            Self::IdentifierBytes => "identifier-bytes",
            Self::NestingDepth => "nesting-depth",
            Self::Components => "components",
            Self::Properties => "properties",
            Self::Templates => "templates",
            Self::Regions => "regions",
            Self::ChildSlots => "child-slots",
            Self::InitialProperties => "initial-properties",
            Self::InitialKeys => "initial-keys",
            Self::StyleAssignments => "style-assignments",
            Self::Images => "images",
            Self::ImageBytes => "image-bytes",
            Self::SpatialNodes => "spatial-nodes",
            Self::SpatialFields => "spatial-fields",
            Self::Shapes => "shapes",
            Self::Paths => "paths",
            Self::PathVerbs => "path-verbs",
            Self::PolygonPoints => "polygon-points",
            Self::Brushes => "brushes",
            Self::GradientStops => "gradient-stops",
            Self::Clips => "clips",
            Self::PaintItems => "paint-items",
            Self::HitItems => "hit-items",
            Self::SemanticItems => "semantic-items",
            Self::SourceAnchors => "source-anchors",
            Self::GeneratedRustBytes => "generated-rust-bytes",
        }
    }
}

/// Complete inclusive limits for one format-2 authoring compilation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthoringLimitsV2 {
    values: [usize; 28],
}

impl AuthoringLimitsV2 {
    /// Creates a complete explicit authoring limit set.
    #[must_use]
    pub const fn new(values: [usize; 28]) -> Self {
        Self { values }
    }

    /// Returns the inclusive bound for one resource category.
    #[must_use]
    pub const fn limit(self, kind: AuthoringLimitKindV2) -> usize {
        self.values[kind as usize]
    }
}
