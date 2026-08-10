/// Inclusive resource category bounded by the authoring compiler.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthoringLimitKindV1 {
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
    /// Logical source anchors.
    SourceAnchors,
    /// Bytes in generated Rust, including its final line feed.
    GeneratedRustBytes,
}

impl AuthoringLimitKindV1 {
    /// Every bounded resource in deterministic validation order.
    pub const ALL: [Self; 14] = [
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
            Self::SourceAnchors => "source-anchors",
            Self::GeneratedRustBytes => "generated-rust-bytes",
        }
    }
}

/// Complete inclusive limits for one authoring compilation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthoringLimitsV1 {
    values: [usize; 14],
}

/// Exact bounded profile shared by the disposable WU-0010 reference lanes.
///
/// This experiment profile is not an unbounded default or a product budget.
pub const REFERENCE_AUTHORING_LIMITS_V1: AuthoringLimitsV1 =
    AuthoringLimitsV1::new(8_192, 1_024, 32, 8, 1, 5, 4, 1, 3, 12, 2, 2, 34, 32_768);

impl AuthoringLimitsV1 {
    /// Creates a complete explicit authoring limit set.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        fen_source_bytes: usize,
        tokens: usize,
        identifier_bytes: usize,
        nesting_depth: usize,
        components: usize,
        properties: usize,
        templates: usize,
        regions: usize,
        child_slots: usize,
        initial_properties: usize,
        initial_keys: usize,
        style_assignments: usize,
        source_anchors: usize,
        generated_rust_bytes: usize,
    ) -> Self {
        Self {
            values: [
                fen_source_bytes,
                tokens,
                identifier_bytes,
                nesting_depth,
                components,
                properties,
                templates,
                regions,
                child_slots,
                initial_properties,
                initial_keys,
                style_assignments,
                source_anchors,
                generated_rust_bytes,
            ],
        }
    }

    /// Returns the inclusive bound for one resource category.
    #[must_use]
    pub const fn limit(self, kind: AuthoringLimitKindV1) -> usize {
        self.values[kind as usize]
    }
}
