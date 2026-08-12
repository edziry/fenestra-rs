use std::error::Error;
use std::fmt;

use crate::source::SourceSpan;

/// Resource category exceeded during bounded validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationLimitKind {
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
    /// Template ownership depth.
    TemplateDepth,
    /// Initially expanded template instances.
    InitialInstances,
    /// Exact style assignments.
    StyleAssignments,
    /// Spatial node declarations.
    SpatialNodes,
    /// Spatial shape declarations across all nodes.
    SpatialShapes,
    /// Spatial brush declarations across all nodes.
    SpatialBrushes,
    /// Spatial clip declarations across all nodes.
    SpatialClips,
    /// Spatial paint item recipes across all nodes.
    SpatialPaintItems,
    /// Spatial hit item recipes across all nodes.
    SpatialHitItems,
    /// Spatial semantic item recipes across all nodes.
    SpatialSemanticItems,
    /// Path-shaped spatial declarations across all nodes.
    SpatialPaths,
    /// Path verbs across all spatial paths.
    SpatialPathVerbs,
    /// Polygon points across all spatial polygons.
    SpatialPolygonPoints,
    /// Gradient stops across all spatial brushes.
    SpatialGradientStops,
    /// Spatial image declarations.
    SpatialImages,
    /// Bytes across all spatial images.
    SpatialImageBytes,
}

impl ValidationLimitKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Components => "components",
            Self::Properties => "properties",
            Self::Templates => "templates",
            Self::Regions => "regions",
            Self::ChildSlots => "child-slots",
            Self::InitialProperties => "initial-properties",
            Self::InitialKeys => "initial-keys",
            Self::TemplateDepth => "template-depth",
            Self::InitialInstances => "initial-instances",
            Self::StyleAssignments => "style-assignments",
            Self::SpatialNodes => "spatial-nodes",
            Self::SpatialShapes => "spatial-shapes",
            Self::SpatialBrushes => "spatial-brushes",
            Self::SpatialClips => "spatial-clips",
            Self::SpatialPaintItems => "spatial-paint-items",
            Self::SpatialHitItems => "spatial-hit-items",
            Self::SpatialSemanticItems => "spatial-semantic-items",
            Self::SpatialPaths => "spatial-paths",
            Self::SpatialPathVerbs => "spatial-path-verbs",
            Self::SpatialPolygonPoints => "spatial-polygon-points",
            Self::SpatialGradientStops => "spatial-gradient-stops",
            Self::SpatialImages => "spatial-images",
            Self::SpatialImageBytes => "spatial-image-bytes",
        }
    }
}

/// Exhaustive typed failures for the provisional validators.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrValidationErrorKind {
    /// Unsupported schema manifest format.
    UnsupportedSchemaFormat,
    /// Unsupported construction program format.
    UnsupportedConstructionFormat,
    /// A linked program and schema authored identities differ.
    SchemaIdentityMismatch,
    /// A concrete byte span has its bounds inverted.
    InvalidSourceSpan,
    /// A component symbol is declared more than once.
    DuplicateComponent,
    /// A property symbol is repeated within one component.
    DuplicateProperty,
    /// A property default does not match its declared type.
    PropertyDefaultTypeMismatch,
    /// A property has no declared invalidation work.
    EmptyPropertyInvalidation,
    /// A property declares an invalid structural or surface class.
    InvalidPropertyInvalidation,
    /// A template symbol is declared more than once.
    DuplicateNode,
    /// A property is assigned twice on one template.
    DuplicateInitialProperty,
    /// An initial assignment names no property on its component.
    UnknownInitialProperty,
    /// An initial value does not match its property type.
    InitialPropertyTypeMismatch,
    /// A structural region symbol is declared more than once.
    DuplicateRegion,
    /// A template names an absent component.
    MissingComponent,
    /// A static child slot names an absent template.
    MissingStaticChild,
    /// A region slot names an absent region.
    MissingRegion,
    /// A region names an absent owner template.
    MissingRegionOwner,
    /// A region is placed under a template other than its owner.
    RegionOwnerMismatch,
    /// A region appears in more than one child slot.
    DuplicateRegionPlacement,
    /// A region names an absent repeat-body template.
    MissingRegionTemplate,
    /// A declared region has no child-slot placement.
    UnplacedRegion,
    /// A template receives more than one definition owner.
    DuplicateNodeOwner,
    /// The construction program does not have exactly one root.
    InvalidRootCount,
    /// Template definition ownership contains a cycle.
    OwnershipCycle,
    /// An initial key is repeated within one region.
    DuplicateRegionKey,
    /// A region has an invalid structural invalidation declaration.
    InvalidRegionInvalidation,
    /// Unsupported exact-target style program format.
    UnsupportedStyleFormat,
    /// A style assignment names an absent construction template.
    MissingStyleTarget,
    /// A style assignment names no property on its target component.
    UnknownStyleProperty,
    /// A style value does not match its target property's type.
    StylePropertyTypeMismatch,
    /// A target-property pair is assigned more than once.
    DuplicateStyleAssignment,
    /// Unsupported symbolic spatial program format.
    UnsupportedSpatialFormat,
    /// A spatial node symbol is declared more than once.
    DuplicateSpatialNode,
    /// A construction template has more than one spatial node.
    DuplicateSpatialTemplate,
    /// A spatial node names an absent construction template.
    MissingSpatialTemplate,
    /// A spatial node names an absent spatial parent.
    MissingSpatialParent,
    /// A spatial parent is outside the node's structural context.
    SpatialParentContextMismatch,
    /// A spatial parent does not precede the child.
    SpatialParentNotEarlier,
    /// Spatial declarations do not form the required preorder.
    InvalidSpatialPreorder,
    /// A spatial binding names no property in its component scope.
    UnknownSpatialProperty,
    /// A spatial binding property has the wrong value type.
    SpatialPropertyTypeMismatch,
    /// A fixed-point literal is outside the canonical scalar domain.
    SpatialFixed16OutOfRange,
    /// A spatial shape symbol is repeated within one node.
    DuplicateSpatialShape,
    /// A spatial brush symbol is repeated within one node.
    DuplicateSpatialBrush,
    /// A spatial clip symbol is repeated within one node.
    DuplicateSpatialClip,
    /// A spatial image symbol is declared more than once.
    DuplicateSpatialImage,
    /// A spatial item names an absent shape in its owner scope.
    MissingSpatialShape,
    /// A spatial item names an absent brush in its owner scope.
    MissingSpatialBrush,
    /// A spatial item names an absent image.
    MissingSpatialImage,
    /// A qualified spatial clip address names an absent owner.
    MissingSpatialClipOwner,
    /// A qualified spatial clip address names an absent clip.
    MissingSpatialClip,
    /// A qualified clip owner is not a spatial ancestor.
    SpatialClipOwnerNotAncestor,
    /// A same-owner clip parent does not precede its child.
    SpatialClipParentNotEarlier,
    /// A spatial anchor names an absent target.
    MissingSpatialAnchorTarget,
    /// A spatial node anchors to itself.
    SelfAnchorTarget,
    /// A spatial anchor target is outside the node's structural context.
    SpatialAnchorContextMismatch,
    /// A bounded validation resource was exceeded.
    LimitExceeded(ValidationLimitKind),
}

impl IrValidationErrorKind {
    /// Every concrete error kind, including each typed limit category.
    pub const ALL: [Self; 80] = [
        Self::UnsupportedSchemaFormat,
        Self::UnsupportedConstructionFormat,
        Self::SchemaIdentityMismatch,
        Self::InvalidSourceSpan,
        Self::DuplicateComponent,
        Self::DuplicateProperty,
        Self::PropertyDefaultTypeMismatch,
        Self::EmptyPropertyInvalidation,
        Self::InvalidPropertyInvalidation,
        Self::DuplicateNode,
        Self::DuplicateInitialProperty,
        Self::UnknownInitialProperty,
        Self::InitialPropertyTypeMismatch,
        Self::DuplicateRegion,
        Self::MissingComponent,
        Self::MissingStaticChild,
        Self::MissingRegion,
        Self::MissingRegionOwner,
        Self::RegionOwnerMismatch,
        Self::DuplicateRegionPlacement,
        Self::MissingRegionTemplate,
        Self::UnplacedRegion,
        Self::DuplicateNodeOwner,
        Self::InvalidRootCount,
        Self::OwnershipCycle,
        Self::DuplicateRegionKey,
        Self::InvalidRegionInvalidation,
        Self::LimitExceeded(ValidationLimitKind::Components),
        Self::LimitExceeded(ValidationLimitKind::Properties),
        Self::LimitExceeded(ValidationLimitKind::Templates),
        Self::LimitExceeded(ValidationLimitKind::Regions),
        Self::LimitExceeded(ValidationLimitKind::ChildSlots),
        Self::LimitExceeded(ValidationLimitKind::InitialProperties),
        Self::LimitExceeded(ValidationLimitKind::InitialKeys),
        Self::LimitExceeded(ValidationLimitKind::TemplateDepth),
        Self::LimitExceeded(ValidationLimitKind::InitialInstances),
        Self::UnsupportedStyleFormat,
        Self::MissingStyleTarget,
        Self::UnknownStyleProperty,
        Self::StylePropertyTypeMismatch,
        Self::DuplicateStyleAssignment,
        Self::LimitExceeded(ValidationLimitKind::StyleAssignments),
        Self::UnsupportedSpatialFormat,
        Self::LimitExceeded(ValidationLimitKind::SpatialNodes),
        Self::LimitExceeded(ValidationLimitKind::SpatialShapes),
        Self::LimitExceeded(ValidationLimitKind::SpatialBrushes),
        Self::LimitExceeded(ValidationLimitKind::SpatialClips),
        Self::LimitExceeded(ValidationLimitKind::SpatialPaintItems),
        Self::LimitExceeded(ValidationLimitKind::SpatialHitItems),
        Self::LimitExceeded(ValidationLimitKind::SpatialSemanticItems),
        Self::LimitExceeded(ValidationLimitKind::SpatialPaths),
        Self::LimitExceeded(ValidationLimitKind::SpatialPathVerbs),
        Self::LimitExceeded(ValidationLimitKind::SpatialPolygonPoints),
        Self::LimitExceeded(ValidationLimitKind::SpatialGradientStops),
        Self::LimitExceeded(ValidationLimitKind::SpatialImages),
        Self::LimitExceeded(ValidationLimitKind::SpatialImageBytes),
        Self::DuplicateSpatialNode,
        Self::DuplicateSpatialTemplate,
        Self::MissingSpatialTemplate,
        Self::MissingSpatialParent,
        Self::SpatialParentContextMismatch,
        Self::SpatialParentNotEarlier,
        Self::InvalidSpatialPreorder,
        Self::UnknownSpatialProperty,
        Self::SpatialPropertyTypeMismatch,
        Self::SpatialFixed16OutOfRange,
        Self::DuplicateSpatialShape,
        Self::DuplicateSpatialBrush,
        Self::DuplicateSpatialClip,
        Self::DuplicateSpatialImage,
        Self::MissingSpatialShape,
        Self::MissingSpatialBrush,
        Self::MissingSpatialImage,
        Self::MissingSpatialClipOwner,
        Self::MissingSpatialClip,
        Self::SpatialClipOwnerNotAncestor,
        Self::SpatialClipParentNotEarlier,
        Self::MissingSpatialAnchorTarget,
        Self::SelfAnchorTarget,
        Self::SpatialAnchorContextMismatch,
    ];

    const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedSchemaFormat => "unsupported-schema-format",
            Self::UnsupportedConstructionFormat => "unsupported-construction-format",
            Self::SchemaIdentityMismatch => "schema-identity-mismatch",
            Self::InvalidSourceSpan => "invalid-source-span",
            Self::DuplicateComponent => "duplicate-component",
            Self::DuplicateProperty => "duplicate-property",
            Self::PropertyDefaultTypeMismatch => "property-default-type-mismatch",
            Self::EmptyPropertyInvalidation => "empty-property-invalidation",
            Self::InvalidPropertyInvalidation => "invalid-property-invalidation",
            Self::DuplicateNode => "duplicate-node",
            Self::DuplicateInitialProperty => "duplicate-initial-property",
            Self::UnknownInitialProperty => "unknown-initial-property",
            Self::InitialPropertyTypeMismatch => "initial-property-type-mismatch",
            Self::DuplicateRegion => "duplicate-region",
            Self::MissingComponent => "missing-component",
            Self::MissingStaticChild => "missing-static-child",
            Self::MissingRegion => "missing-region",
            Self::MissingRegionOwner => "missing-region-owner",
            Self::RegionOwnerMismatch => "region-owner-mismatch",
            Self::DuplicateRegionPlacement => "duplicate-region-placement",
            Self::MissingRegionTemplate => "missing-region-template",
            Self::UnplacedRegion => "unplaced-region",
            Self::DuplicateNodeOwner => "duplicate-node-owner",
            Self::InvalidRootCount => "invalid-root-count",
            Self::OwnershipCycle => "ownership-cycle",
            Self::DuplicateRegionKey => "duplicate-region-key",
            Self::InvalidRegionInvalidation => "invalid-region-invalidation",
            Self::UnsupportedStyleFormat => "unsupported-style-format",
            Self::MissingStyleTarget => "missing-style-target",
            Self::UnknownStyleProperty => "unknown-style-property",
            Self::StylePropertyTypeMismatch => "style-property-type-mismatch",
            Self::DuplicateStyleAssignment => "duplicate-style-assignment",
            Self::UnsupportedSpatialFormat => "unsupported-spatial-format",
            Self::DuplicateSpatialNode => "duplicate-spatial-node",
            Self::DuplicateSpatialTemplate => "duplicate-spatial-template",
            Self::MissingSpatialTemplate => "missing-spatial-template",
            Self::MissingSpatialParent => "missing-spatial-parent",
            Self::SpatialParentContextMismatch => "spatial-parent-context-mismatch",
            Self::SpatialParentNotEarlier => "spatial-parent-not-earlier",
            Self::InvalidSpatialPreorder => "invalid-spatial-preorder",
            Self::UnknownSpatialProperty => "unknown-spatial-property",
            Self::SpatialPropertyTypeMismatch => "spatial-property-type-mismatch",
            Self::SpatialFixed16OutOfRange => "spatial-fixed16-out-of-range",
            Self::DuplicateSpatialShape => "duplicate-spatial-shape",
            Self::DuplicateSpatialBrush => "duplicate-spatial-brush",
            Self::DuplicateSpatialClip => "duplicate-spatial-clip",
            Self::DuplicateSpatialImage => "duplicate-spatial-image",
            Self::MissingSpatialShape => "missing-spatial-shape",
            Self::MissingSpatialBrush => "missing-spatial-brush",
            Self::MissingSpatialImage => "missing-spatial-image",
            Self::MissingSpatialClipOwner => "missing-spatial-clip-owner",
            Self::MissingSpatialClip => "missing-spatial-clip",
            Self::SpatialClipOwnerNotAncestor => "spatial-clip-owner-not-ancestor",
            Self::SpatialClipParentNotEarlier => "spatial-clip-parent-not-earlier",
            Self::MissingSpatialAnchorTarget => "missing-spatial-anchor-target",
            Self::SelfAnchorTarget => "self-anchor-target",
            Self::SpatialAnchorContextMismatch => "spatial-anchor-context-mismatch",
            Self::LimitExceeded(_) => "limit-exceeded",
        }
    }
}

/// Validation failure with a typed kind and opaque source anchor.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct IrValidationError {
    kind: IrValidationErrorKind,
    span: SourceSpan,
}

impl IrValidationError {
    pub(crate) const fn new(kind: IrValidationErrorKind, span: SourceSpan) -> Self {
        Self { kind, span }
    }

    /// Returns the typed failure category.
    #[must_use]
    pub const fn kind(self) -> IrValidationErrorKind {
        self.kind
    }

    /// Returns the opaque authored source anchor.
    #[must_use]
    pub const fn span(self) -> SourceSpan {
        self.span
    }
}

impl fmt::Debug for IrValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "IrValidationError({self})")
    }
}

impl fmt::Display for IrValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.kind.as_str())?;
        if let IrValidationErrorKind::LimitExceeded(limit) = self.kind {
            write!(formatter, "({})", limit.as_str())?;
        }
        write!(formatter, " at {:?}", self.span)
    }
}

impl Error for IrValidationError {}
