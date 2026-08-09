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
    /// A bounded validation resource was exceeded.
    LimitExceeded(ValidationLimitKind),
}

impl IrValidationErrorKind {
    /// Every concrete error kind, including each typed limit category.
    pub const ALL: [Self; 42] = [
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
