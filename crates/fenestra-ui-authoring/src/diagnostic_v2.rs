use std::error::Error;
use std::fmt;

use fenestra_ui_ir::prototype::IrValidationErrorKind;

use crate::limits_v2::AuthoringLimitKindV2;
use crate::source_v2::DiagnosticLocationV2;
use crate::vocabulary_v2::AuthoringFrontendV2;

/// Closed failure categories for format-2 authoring compilation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthoringDiagnosticKindV2 {
    /// `.fen` input is not valid UTF-8.
    InvalidUtf8,
    /// A frontend token has no format-2 abstract-token representation.
    UnsupportedToken,
    /// The authoring document declares an unsupported format.
    UnsupportedAuthoringFormat,
    /// The shared grammar encountered an unexpected token.
    UnexpectedToken,
    /// The shared grammar reached the end of incomplete input.
    UnexpectedEof,
    /// An identifier violates the closed format-2 spelling.
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
    /// A spatial node name is declared more than once.
    DuplicateSpatialNodeName,
    /// A spatial image name is declared more than once.
    DuplicateSpatialImageName,
    /// A spatial shape name is repeated within one node.
    DuplicateSpatialShapeName,
    /// A spatial brush name is repeated within one node.
    DuplicateSpatialBrushName,
    /// A spatial clip name is repeated within one node.
    DuplicateSpatialClipName,
    /// A spatial node reference does not resolve.
    UnknownSpatialNodeName,
    /// A spatial image reference does not resolve.
    UnknownSpatialImageName,
    /// A spatial shape reference does not resolve.
    UnknownSpatialShapeName,
    /// A spatial brush reference does not resolve.
    UnknownSpatialBrushName,
    /// A spatial clip reference does not resolve.
    UnknownSpatialClipName,
    /// One explicit authoring resource bound was exceeded.
    LimitExceeded(AuthoringLimitKindV2),
    /// Existing typed IR validation rejected the lowered programs.
    IrValidation(IrValidationErrorKind),
}

impl AuthoringDiagnosticKindV2 {
    /// Every concrete diagnostic outcome in deterministic vocabulary order.
    pub const ALL: [Self; 134] = all_authoring_diagnostic_kinds_v2();
}

const fn all_authoring_diagnostic_kinds_v2() -> [AuthoringDiagnosticKindV2; 134] {
    use AuthoringDiagnosticKindV2 as Kind;

    let prefix = [
        Kind::InvalidUtf8,
        Kind::UnsupportedToken,
        Kind::UnsupportedAuthoringFormat,
        Kind::UnexpectedToken,
        Kind::UnexpectedEof,
        Kind::InvalidIdentifier,
        Kind::InvalidLiteral,
        Kind::DuplicateComponentName,
        Kind::DuplicatePropertyName,
        Kind::DuplicateTemplateName,
        Kind::DuplicateRegionName,
        Kind::UnknownComponentName,
        Kind::UnknownPropertyName,
        Kind::UnknownTemplateName,
        Kind::UnknownRegionName,
        Kind::ValueTypeMismatch,
        Kind::DuplicateSpatialNodeName,
        Kind::DuplicateSpatialImageName,
        Kind::DuplicateSpatialShapeName,
        Kind::DuplicateSpatialBrushName,
        Kind::DuplicateSpatialClipName,
        Kind::UnknownSpatialNodeName,
        Kind::UnknownSpatialImageName,
        Kind::UnknownSpatialShapeName,
        Kind::UnknownSpatialBrushName,
        Kind::UnknownSpatialClipName,
    ];
    let mut result = [Kind::InvalidUtf8; 134];
    let mut result_index = 0;
    while result_index < prefix.len() {
        result[result_index] = prefix[result_index];
        result_index += 1;
    }

    let mut limit_index = 0;
    while limit_index < AuthoringLimitKindV2::ALL.len() {
        result[result_index] = Kind::LimitExceeded(AuthoringLimitKindV2::ALL[limit_index]);
        result_index += 1;
        limit_index += 1;
    }

    let mut validation_index = 0;
    while validation_index < IrValidationErrorKind::ALL.len() {
        result[result_index] = Kind::IrValidation(IrValidationErrorKind::ALL[validation_index]);
        result_index += 1;
        validation_index += 1;
    }
    result
}

/// Typed format-2 authoring failure with a closed kind and source location.
#[derive(Clone, Copy)]
pub struct AuthoringDiagnosticV2 {
    frontend: AuthoringFrontendV2,
    kind: AuthoringDiagnosticKindV2,
    location: DiagnosticLocationV2,
}

impl AuthoringDiagnosticV2 {
    pub(crate) const fn new(
        frontend: AuthoringFrontendV2,
        kind: AuthoringDiagnosticKindV2,
        location: DiagnosticLocationV2,
    ) -> Self {
        Self {
            frontend,
            kind,
            location,
        }
    }

    /// Returns the frontend that produced this failure.
    #[must_use]
    pub const fn frontend(&self) -> AuthoringFrontendV2 {
        self.frontend
    }

    /// Returns the closed failure category.
    #[must_use]
    pub const fn kind(&self) -> AuthoringDiagnosticKindV2 {
        self.kind
    }

    /// Returns the typed physical or anchored source location.
    #[must_use]
    pub const fn location(&self) -> &DiagnosticLocationV2 {
        &self.location
    }
}

impl fmt::Display for AuthoringDiagnosticV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            AuthoringDiagnosticKindV2::InvalidUtf8 => formatter.write_str("invalid-utf8"),
            AuthoringDiagnosticKindV2::UnsupportedToken => formatter.write_str("unsupported-token"),
            AuthoringDiagnosticKindV2::UnsupportedAuthoringFormat => {
                formatter.write_str("unsupported-authoring-format")
            }
            AuthoringDiagnosticKindV2::UnexpectedToken => formatter.write_str("unexpected-token"),
            AuthoringDiagnosticKindV2::UnexpectedEof => formatter.write_str("unexpected-eof"),
            AuthoringDiagnosticKindV2::InvalidIdentifier => {
                formatter.write_str("invalid-identifier")
            }
            AuthoringDiagnosticKindV2::InvalidLiteral => formatter.write_str("invalid-literal"),
            AuthoringDiagnosticKindV2::DuplicateComponentName => {
                formatter.write_str("duplicate-component-name")
            }
            AuthoringDiagnosticKindV2::DuplicatePropertyName => {
                formatter.write_str("duplicate-property-name")
            }
            AuthoringDiagnosticKindV2::DuplicateTemplateName => {
                formatter.write_str("duplicate-template-name")
            }
            AuthoringDiagnosticKindV2::DuplicateRegionName => {
                formatter.write_str("duplicate-region-name")
            }
            AuthoringDiagnosticKindV2::UnknownComponentName => {
                formatter.write_str("unknown-component-name")
            }
            AuthoringDiagnosticKindV2::UnknownPropertyName => {
                formatter.write_str("unknown-property-name")
            }
            AuthoringDiagnosticKindV2::UnknownTemplateName => {
                formatter.write_str("unknown-template-name")
            }
            AuthoringDiagnosticKindV2::UnknownRegionName => {
                formatter.write_str("unknown-region-name")
            }
            AuthoringDiagnosticKindV2::ValueTypeMismatch => {
                formatter.write_str("value-type-mismatch")
            }
            AuthoringDiagnosticKindV2::DuplicateSpatialNodeName => {
                formatter.write_str("duplicate-spatial-node-name")
            }
            AuthoringDiagnosticKindV2::DuplicateSpatialImageName => {
                formatter.write_str("duplicate-spatial-image-name")
            }
            AuthoringDiagnosticKindV2::DuplicateSpatialShapeName => {
                formatter.write_str("duplicate-spatial-shape-name")
            }
            AuthoringDiagnosticKindV2::DuplicateSpatialBrushName => {
                formatter.write_str("duplicate-spatial-brush-name")
            }
            AuthoringDiagnosticKindV2::DuplicateSpatialClipName => {
                formatter.write_str("duplicate-spatial-clip-name")
            }
            AuthoringDiagnosticKindV2::UnknownSpatialNodeName => {
                formatter.write_str("unknown-spatial-node-name")
            }
            AuthoringDiagnosticKindV2::UnknownSpatialImageName => {
                formatter.write_str("unknown-spatial-image-name")
            }
            AuthoringDiagnosticKindV2::UnknownSpatialShapeName => {
                formatter.write_str("unknown-spatial-shape-name")
            }
            AuthoringDiagnosticKindV2::UnknownSpatialBrushName => {
                formatter.write_str("unknown-spatial-brush-name")
            }
            AuthoringDiagnosticKindV2::UnknownSpatialClipName => {
                formatter.write_str("unknown-spatial-clip-name")
            }
            AuthoringDiagnosticKindV2::LimitExceeded(limit) => {
                write!(formatter, "limit-exceeded({})", limit.as_str())
            }
            AuthoringDiagnosticKindV2::IrValidation(_) => formatter.write_str("ir-validation"),
        }
    }
}

impl fmt::Debug for AuthoringDiagnosticV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "AuthoringDiagnosticV2({self})")
    }
}

impl Error for AuthoringDiagnosticV2 {}
