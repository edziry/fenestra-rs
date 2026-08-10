/// Authoring frontend that supplied one compilation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthoringFrontendV1 {
    /// External UTF-8 `.fen` source.
    Fen,
    /// Rust `ui!` procedural-macro tokens.
    UiMacro,
}

impl AuthoringFrontendV1 {
    /// Every frontend in deterministic artifact order.
    pub const ALL: [Self; 2] = [Self::Fen, Self::UiMacro];
}

/// Semantic role of one logical authoring source anchor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnchorKindV1 {
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
}

impl AnchorKindV1 {
    /// Every anchor role in deterministic vocabulary order.
    pub const ALL: [Self; 13] = [
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
    ];
}
