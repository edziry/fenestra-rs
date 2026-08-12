use std::error::Error;
use std::fmt;

use fenestra_ui_ir::prototype::{IrValidationErrorKind, ValidationLimitKind};

use crate::limits::AuthoringLimitKindV1;
use crate::source::DiagnosticLocationV1;
use crate::vocabulary::AuthoringFrontendV1;

/// Closed failure categories for version-1 authoring compilation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthoringDiagnosticKindV1 {
    /// `.fen` input is not valid UTF-8.
    InvalidUtf8,
    /// A frontend token has no format-1 abstract-token representation.
    UnsupportedToken,
    /// The authoring document declares an unsupported format.
    UnsupportedAuthoringFormat,
    /// The shared grammar encountered an unexpected token.
    UnexpectedToken,
    /// The shared grammar reached the end of incomplete input.
    UnexpectedEof,
    /// An identifier violates the closed format-1 spelling.
    InvalidIdentifier,
    /// A literal violates its closed spelling or range.
    InvalidLiteral,
    /// A component name is declared more than once.
    DuplicateComponentName,
    /// A property name is repeated within one component.
    DuplicatePropertyName,
    /// A template name is declared more than once.
    DuplicateTemplateName,
    /// A region name is declared more than once.
    DuplicateRegionName,
    /// A component reference does not resolve.
    UnknownComponentName,
    /// A property reference does not resolve in its component scope.
    UnknownPropertyName,
    /// A template reference does not resolve.
    UnknownTemplateName,
    /// A region reference does not resolve.
    UnknownRegionName,
    /// An authored value does not match its resolved property type.
    ValueTypeMismatch,
    /// One explicit authoring resource bound was exceeded.
    LimitExceeded(AuthoringLimitKindV1),
    /// Existing typed IR validation rejected the lowered programs.
    IrValidation(IrValidationErrorKind),
}

impl AuthoringDiagnosticKindV1 {
    /// Every concrete diagnostic outcome in deterministic vocabulary order.
    pub const ALL: [Self; 110] = [
        Self::InvalidUtf8,
        Self::UnsupportedToken,
        Self::UnsupportedAuthoringFormat,
        Self::UnexpectedToken,
        Self::UnexpectedEof,
        Self::InvalidIdentifier,
        Self::InvalidLiteral,
        Self::DuplicateComponentName,
        Self::DuplicatePropertyName,
        Self::DuplicateTemplateName,
        Self::DuplicateRegionName,
        Self::UnknownComponentName,
        Self::UnknownPropertyName,
        Self::UnknownTemplateName,
        Self::UnknownRegionName,
        Self::ValueTypeMismatch,
        Self::LimitExceeded(AuthoringLimitKindV1::FenSourceBytes),
        Self::LimitExceeded(AuthoringLimitKindV1::Tokens),
        Self::LimitExceeded(AuthoringLimitKindV1::IdentifierBytes),
        Self::LimitExceeded(AuthoringLimitKindV1::NestingDepth),
        Self::LimitExceeded(AuthoringLimitKindV1::Components),
        Self::LimitExceeded(AuthoringLimitKindV1::Properties),
        Self::LimitExceeded(AuthoringLimitKindV1::Templates),
        Self::LimitExceeded(AuthoringLimitKindV1::Regions),
        Self::LimitExceeded(AuthoringLimitKindV1::ChildSlots),
        Self::LimitExceeded(AuthoringLimitKindV1::InitialProperties),
        Self::LimitExceeded(AuthoringLimitKindV1::InitialKeys),
        Self::LimitExceeded(AuthoringLimitKindV1::StyleAssignments),
        Self::LimitExceeded(AuthoringLimitKindV1::SourceAnchors),
        Self::LimitExceeded(AuthoringLimitKindV1::GeneratedRustBytes),
        Self::IrValidation(IrValidationErrorKind::UnsupportedSchemaFormat),
        Self::IrValidation(IrValidationErrorKind::UnsupportedConstructionFormat),
        Self::IrValidation(IrValidationErrorKind::SchemaIdentityMismatch),
        Self::IrValidation(IrValidationErrorKind::InvalidSourceSpan),
        Self::IrValidation(IrValidationErrorKind::DuplicateComponent),
        Self::IrValidation(IrValidationErrorKind::DuplicateProperty),
        Self::IrValidation(IrValidationErrorKind::PropertyDefaultTypeMismatch),
        Self::IrValidation(IrValidationErrorKind::EmptyPropertyInvalidation),
        Self::IrValidation(IrValidationErrorKind::InvalidPropertyInvalidation),
        Self::IrValidation(IrValidationErrorKind::DuplicateNode),
        Self::IrValidation(IrValidationErrorKind::DuplicateInitialProperty),
        Self::IrValidation(IrValidationErrorKind::UnknownInitialProperty),
        Self::IrValidation(IrValidationErrorKind::InitialPropertyTypeMismatch),
        Self::IrValidation(IrValidationErrorKind::DuplicateRegion),
        Self::IrValidation(IrValidationErrorKind::MissingComponent),
        Self::IrValidation(IrValidationErrorKind::MissingStaticChild),
        Self::IrValidation(IrValidationErrorKind::MissingRegion),
        Self::IrValidation(IrValidationErrorKind::MissingRegionOwner),
        Self::IrValidation(IrValidationErrorKind::RegionOwnerMismatch),
        Self::IrValidation(IrValidationErrorKind::DuplicateRegionPlacement),
        Self::IrValidation(IrValidationErrorKind::MissingRegionTemplate),
        Self::IrValidation(IrValidationErrorKind::UnplacedRegion),
        Self::IrValidation(IrValidationErrorKind::DuplicateNodeOwner),
        Self::IrValidation(IrValidationErrorKind::InvalidRootCount),
        Self::IrValidation(IrValidationErrorKind::OwnershipCycle),
        Self::IrValidation(IrValidationErrorKind::DuplicateRegionKey),
        Self::IrValidation(IrValidationErrorKind::InvalidRegionInvalidation),
        Self::IrValidation(IrValidationErrorKind::LimitExceeded(
            ValidationLimitKind::Components,
        )),
        Self::IrValidation(IrValidationErrorKind::LimitExceeded(
            ValidationLimitKind::Properties,
        )),
        Self::IrValidation(IrValidationErrorKind::LimitExceeded(
            ValidationLimitKind::Templates,
        )),
        Self::IrValidation(IrValidationErrorKind::LimitExceeded(
            ValidationLimitKind::Regions,
        )),
        Self::IrValidation(IrValidationErrorKind::LimitExceeded(
            ValidationLimitKind::ChildSlots,
        )),
        Self::IrValidation(IrValidationErrorKind::LimitExceeded(
            ValidationLimitKind::InitialProperties,
        )),
        Self::IrValidation(IrValidationErrorKind::LimitExceeded(
            ValidationLimitKind::InitialKeys,
        )),
        Self::IrValidation(IrValidationErrorKind::LimitExceeded(
            ValidationLimitKind::TemplateDepth,
        )),
        Self::IrValidation(IrValidationErrorKind::LimitExceeded(
            ValidationLimitKind::InitialInstances,
        )),
        Self::IrValidation(IrValidationErrorKind::UnsupportedStyleFormat),
        Self::IrValidation(IrValidationErrorKind::MissingStyleTarget),
        Self::IrValidation(IrValidationErrorKind::UnknownStyleProperty),
        Self::IrValidation(IrValidationErrorKind::StylePropertyTypeMismatch),
        Self::IrValidation(IrValidationErrorKind::DuplicateStyleAssignment),
        Self::IrValidation(IrValidationErrorKind::LimitExceeded(
            ValidationLimitKind::StyleAssignments,
        )),
        Self::IrValidation(IrValidationErrorKind::UnsupportedSpatialFormat),
        Self::IrValidation(IrValidationErrorKind::LimitExceeded(
            ValidationLimitKind::SpatialNodes,
        )),
        Self::IrValidation(IrValidationErrorKind::LimitExceeded(
            ValidationLimitKind::SpatialShapes,
        )),
        Self::IrValidation(IrValidationErrorKind::LimitExceeded(
            ValidationLimitKind::SpatialBrushes,
        )),
        Self::IrValidation(IrValidationErrorKind::LimitExceeded(
            ValidationLimitKind::SpatialClips,
        )),
        Self::IrValidation(IrValidationErrorKind::LimitExceeded(
            ValidationLimitKind::SpatialPaintItems,
        )),
        Self::IrValidation(IrValidationErrorKind::LimitExceeded(
            ValidationLimitKind::SpatialHitItems,
        )),
        Self::IrValidation(IrValidationErrorKind::LimitExceeded(
            ValidationLimitKind::SpatialSemanticItems,
        )),
        Self::IrValidation(IrValidationErrorKind::LimitExceeded(
            ValidationLimitKind::SpatialPaths,
        )),
        Self::IrValidation(IrValidationErrorKind::LimitExceeded(
            ValidationLimitKind::SpatialPathVerbs,
        )),
        Self::IrValidation(IrValidationErrorKind::LimitExceeded(
            ValidationLimitKind::SpatialPolygonPoints,
        )),
        Self::IrValidation(IrValidationErrorKind::LimitExceeded(
            ValidationLimitKind::SpatialGradientStops,
        )),
        Self::IrValidation(IrValidationErrorKind::LimitExceeded(
            ValidationLimitKind::SpatialImages,
        )),
        Self::IrValidation(IrValidationErrorKind::LimitExceeded(
            ValidationLimitKind::SpatialImageBytes,
        )),
        Self::IrValidation(IrValidationErrorKind::DuplicateSpatialNode),
        Self::IrValidation(IrValidationErrorKind::DuplicateSpatialTemplate),
        Self::IrValidation(IrValidationErrorKind::MissingSpatialTemplate),
        Self::IrValidation(IrValidationErrorKind::MissingSpatialParent),
        Self::IrValidation(IrValidationErrorKind::SpatialParentContextMismatch),
        Self::IrValidation(IrValidationErrorKind::SpatialParentNotEarlier),
        Self::IrValidation(IrValidationErrorKind::InvalidSpatialPreorder),
        Self::IrValidation(IrValidationErrorKind::UnknownSpatialProperty),
        Self::IrValidation(IrValidationErrorKind::SpatialPropertyTypeMismatch),
        Self::IrValidation(IrValidationErrorKind::SpatialFixed16OutOfRange),
        Self::IrValidation(IrValidationErrorKind::DuplicateSpatialShape),
        Self::IrValidation(IrValidationErrorKind::DuplicateSpatialBrush),
        Self::IrValidation(IrValidationErrorKind::DuplicateSpatialClip),
        Self::IrValidation(IrValidationErrorKind::DuplicateSpatialImage),
        Self::IrValidation(IrValidationErrorKind::MissingSpatialShape),
        Self::IrValidation(IrValidationErrorKind::MissingSpatialBrush),
        Self::IrValidation(IrValidationErrorKind::MissingSpatialImage),
        Self::IrValidation(IrValidationErrorKind::MissingSpatialClipOwner),
        Self::IrValidation(IrValidationErrorKind::MissingSpatialClip),
        Self::IrValidation(IrValidationErrorKind::SpatialClipOwnerNotAncestor),
        Self::IrValidation(IrValidationErrorKind::SpatialClipParentNotEarlier),
        Self::IrValidation(IrValidationErrorKind::MissingSpatialAnchorTarget),
        Self::IrValidation(IrValidationErrorKind::SelfAnchorTarget),
        Self::IrValidation(IrValidationErrorKind::SpatialAnchorContextMismatch),
    ];
}

/// Typed authoring failure with a closed kind and source location.
#[derive(Clone, Copy)]
pub struct AuthoringDiagnosticV1 {
    frontend: AuthoringFrontendV1,
    kind: AuthoringDiagnosticKindV1,
    location: DiagnosticLocationV1,
}

impl AuthoringDiagnosticV1 {
    pub(crate) const fn new(
        frontend: AuthoringFrontendV1,
        kind: AuthoringDiagnosticKindV1,
        location: DiagnosticLocationV1,
    ) -> Self {
        Self {
            frontend,
            kind,
            location,
        }
    }

    /// Returns the frontend that produced this failure.
    #[must_use]
    pub const fn frontend(&self) -> AuthoringFrontendV1 {
        self.frontend
    }

    /// Returns the closed failure category.
    #[must_use]
    pub const fn kind(&self) -> AuthoringDiagnosticKindV1 {
        self.kind
    }

    /// Returns the typed physical or anchored source location.
    #[must_use]
    pub const fn location(&self) -> &DiagnosticLocationV1 {
        &self.location
    }
}

impl fmt::Display for AuthoringDiagnosticV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            AuthoringDiagnosticKindV1::InvalidUtf8 => formatter.write_str("invalid-utf8"),
            AuthoringDiagnosticKindV1::UnsupportedToken => formatter.write_str("unsupported-token"),
            AuthoringDiagnosticKindV1::UnsupportedAuthoringFormat => {
                formatter.write_str("unsupported-authoring-format")
            }
            AuthoringDiagnosticKindV1::UnexpectedToken => formatter.write_str("unexpected-token"),
            AuthoringDiagnosticKindV1::UnexpectedEof => formatter.write_str("unexpected-eof"),
            AuthoringDiagnosticKindV1::InvalidIdentifier => {
                formatter.write_str("invalid-identifier")
            }
            AuthoringDiagnosticKindV1::InvalidLiteral => formatter.write_str("invalid-literal"),
            AuthoringDiagnosticKindV1::DuplicateComponentName => {
                formatter.write_str("duplicate-component-name")
            }
            AuthoringDiagnosticKindV1::DuplicatePropertyName => {
                formatter.write_str("duplicate-property-name")
            }
            AuthoringDiagnosticKindV1::DuplicateTemplateName => {
                formatter.write_str("duplicate-template-name")
            }
            AuthoringDiagnosticKindV1::DuplicateRegionName => {
                formatter.write_str("duplicate-region-name")
            }
            AuthoringDiagnosticKindV1::UnknownComponentName => {
                formatter.write_str("unknown-component-name")
            }
            AuthoringDiagnosticKindV1::UnknownPropertyName => {
                formatter.write_str("unknown-property-name")
            }
            AuthoringDiagnosticKindV1::UnknownTemplateName => {
                formatter.write_str("unknown-template-name")
            }
            AuthoringDiagnosticKindV1::UnknownRegionName => {
                formatter.write_str("unknown-region-name")
            }
            AuthoringDiagnosticKindV1::ValueTypeMismatch => {
                formatter.write_str("value-type-mismatch")
            }
            AuthoringDiagnosticKindV1::LimitExceeded(limit) => {
                write!(formatter, "limit-exceeded({})", limit.as_str())
            }
            AuthoringDiagnosticKindV1::IrValidation(_) => formatter.write_str("ir-validation"),
        }
    }
}

impl fmt::Debug for AuthoringDiagnosticV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "AuthoringDiagnosticV1({self})")
    }
}

impl Error for AuthoringDiagnosticV1 {}
