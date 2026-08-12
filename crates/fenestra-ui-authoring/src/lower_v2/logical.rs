use crate::diagnostic::{AuthoringDiagnosticKindV1, AuthoringDiagnosticV1};
use crate::diagnostic_v2::{AuthoringDiagnosticKindV2, AuthoringDiagnosticV2};
use crate::limits::AuthoringLimitKindV1;
use crate::limits_v2::AuthoringLimitKindV2;
use crate::lower::resolve_logical_semantics;
use crate::parsed::{
    ParsedAnchorV1, ParsedChildV1, ParsedComponentV1, ParsedConstructionV1, ParsedDocumentV1,
    ParsedInitialKeyV1, ParsedInitialPropertyV1, ParsedLiteralV1, ParsedPropertyV1, ParsedRegionV1,
    ParsedSchemaV1, ParsedStyleAssignmentV1, ParsedStyleV1, ParsedTemplateItemV1, ParsedTemplateV1,
    SpannedV1,
};
use crate::parsed_v2::{
    ParsedChildV2, ParsedDocumentV2, ParsedLiteralV2, ParsedTemplateItemV2, SpannedV2,
};
use crate::resolved::ResolvedDocumentV1;
use crate::source::{DiagnosticLocationV1, PhysicalOriginV1};
use crate::source_v2::{DiagnosticLocationV2, PhysicalOriginV2};
use crate::vocabulary::{AnchorKindV1, AuthoringFrontendV1};
use crate::vocabulary_v2::{AnchorKindV2, AuthoringFrontendV2};

pub(crate) fn resolve_logical(
    parsed: &ParsedDocumentV2,
) -> Result<ResolvedDocumentV1, AuthoringDiagnosticV2> {
    resolve_logical_semantics(&logical_document(parsed)).map_err(v2_diagnostic)
}

fn logical_document(parsed: &ParsedDocumentV2) -> ParsedDocumentV1 {
    ParsedDocumentV1 {
        frontend: v1_frontend(parsed.frontend),
        format: parsed.format,
        document_anchor: parsed.document_anchor,
        schema: ParsedSchemaV1 {
            namespace: literal(&parsed.schema.namespace),
            revision: literal(&parsed.schema.revision),
            components: parsed
                .schema
                .components
                .iter()
                .map(|component| ParsedComponentV1 {
                    name: component.name.clone(),
                    id: literal(&component.id),
                    properties: component
                        .properties
                        .iter()
                        .map(|property| ParsedPropertyV1 {
                            name: property.name.clone(),
                            id: literal(&property.id),
                            value_type: property.value_type,
                            default: literal(&property.default),
                            invalidation: property.invalidation,
                            anchor: property.anchor,
                        })
                        .collect(),
                    anchor: component.anchor,
                })
                .collect(),
            anchor: parsed.schema.anchor,
        },
        construction: ParsedConstructionV1 {
            templates: parsed
                .construction
                .templates
                .iter()
                .map(|template| ParsedTemplateV1 {
                    name: template.name.clone(),
                    id: literal(&template.id),
                    component: spanned(&template.component),
                    items: template.items.iter().map(template_item).collect(),
                    anchor: template.anchor,
                })
                .collect(),
            regions: parsed
                .construction
                .regions
                .iter()
                .map(|region| ParsedRegionV1 {
                    name: region.name.clone(),
                    id: literal(&region.id),
                    owner: spanned(&region.owner),
                    repeat_body: spanned(&region.repeat_body),
                    initial_keys: region
                        .initial_keys
                        .iter()
                        .map(|key| ParsedInitialKeyV1 {
                            value: literal(&key.value),
                            anchor: key.anchor,
                        })
                        .collect(),
                    invalidation: region.invalidation,
                    anchor: region.anchor,
                })
                .collect(),
            anchor: parsed.construction.anchor,
        },
        style: ParsedStyleV1 {
            assignments: parsed
                .style
                .assignments
                .iter()
                .map(|assignment| ParsedStyleAssignmentV1 {
                    target: spanned(&assignment.target),
                    property: assignment.property.clone(),
                    value: literal(&assignment.value),
                    anchor: assignment.anchor,
                })
                .collect(),
            anchor: parsed.style.anchor,
        },
        anchors: parsed
            .anchors
            .iter()
            .take(parsed.spatial.anchor as usize)
            .map(|anchor| ParsedAnchorV1 {
                kind: v1_anchor(anchor.kind),
                label: anchor.label.clone(),
                physical: v1_origin(anchor.physical),
            })
            .collect(),
    }
}

fn template_item(item: &ParsedTemplateItemV2) -> ParsedTemplateItemV1 {
    match item {
        ParsedTemplateItemV2::Initial(initial) => {
            ParsedTemplateItemV1::Initial(ParsedInitialPropertyV1 {
                property: initial.property.clone(),
                value: literal(&initial.value),
                anchor: initial.anchor,
            })
        }
        ParsedTemplateItemV2::Child(ParsedChildV2::Static { template, anchor }) => {
            ParsedTemplateItemV1::Child(ParsedChildV1::Static {
                template: template.clone(),
                anchor: *anchor,
            })
        }
        ParsedTemplateItemV2::Child(ParsedChildV2::Region { region, anchor }) => {
            ParsedTemplateItemV1::Child(ParsedChildV1::Region {
                region: region.clone(),
                anchor: *anchor,
            })
        }
    }
}

fn literal<T: Clone>(value: &ParsedLiteralV2<T>) -> ParsedLiteralV1<T> {
    ParsedLiteralV1 {
        value: value.value.clone().map_err(v1_origin),
        physical: v1_origin(value.physical),
    }
}

fn spanned<T: Clone>(value: &SpannedV2<T>) -> SpannedV1<T> {
    SpannedV1 {
        value: value.value.clone(),
        physical: v1_origin(value.physical),
    }
}

fn v1_origin(origin: PhysicalOriginV2) -> PhysicalOriginV1 {
    if let (Some(source), Some((start, end))) = (origin.source_id(), origin.fen_byte_range()) {
        PhysicalOriginV1::fen_bytes(source, start, end)
    } else {
        PhysicalOriginV1::ui_token(origin.ui_span().expect("V2 origin must retain one lane"))
    }
}

fn v2_origin(origin: PhysicalOriginV1) -> PhysicalOriginV2 {
    if let (Some(source), Some((start, end))) = (origin.source_id(), origin.fen_byte_range()) {
        PhysicalOriginV2::fen_bytes(source, start, end)
    } else {
        PhysicalOriginV2::ui_token(origin.ui_span().expect("V1 origin must retain one lane"))
    }
}

fn v2_diagnostic(error: AuthoringDiagnosticV1) -> AuthoringDiagnosticV2 {
    let location = match *error.location() {
        DiagnosticLocationV1::Physical(physical) => {
            DiagnosticLocationV2::Physical(v2_origin(physical))
        }
        DiagnosticLocationV1::Anchored {
            logical,
            anchor_kind,
            physical,
        } => DiagnosticLocationV2::Anchored {
            logical,
            anchor_kind: v2_anchor(anchor_kind),
            physical: v2_origin(physical),
        },
    };
    AuthoringDiagnosticV2::new(
        v2_frontend(error.frontend()),
        v2_kind(error.kind()),
        location,
    )
}

fn v1_frontend(value: AuthoringFrontendV2) -> AuthoringFrontendV1 {
    match value {
        AuthoringFrontendV2::Fen => AuthoringFrontendV1::Fen,
        AuthoringFrontendV2::UiMacro => AuthoringFrontendV1::UiMacro,
    }
}

fn v2_frontend(value: AuthoringFrontendV1) -> AuthoringFrontendV2 {
    match value {
        AuthoringFrontendV1::Fen => AuthoringFrontendV2::Fen,
        AuthoringFrontendV1::UiMacro => AuthoringFrontendV2::UiMacro,
    }
}

fn v1_anchor(value: AnchorKindV2) -> AnchorKindV1 {
    match value {
        AnchorKindV2::Document => AnchorKindV1::Document,
        AnchorKindV2::Schema => AnchorKindV1::Schema,
        AnchorKindV2::Component => AnchorKindV1::Component,
        AnchorKindV2::Property => AnchorKindV1::Property,
        AnchorKindV2::Construction => AnchorKindV1::Construction,
        AnchorKindV2::Template => AnchorKindV1::Template,
        AnchorKindV2::InitialProperty => AnchorKindV1::InitialProperty,
        AnchorKindV2::StaticChild => AnchorKindV1::StaticChild,
        AnchorKindV2::RegionChild => AnchorKindV1::RegionChild,
        AnchorKindV2::Region => AnchorKindV1::Region,
        AnchorKindV2::InitialKey => AnchorKindV1::InitialKey,
        AnchorKindV2::Style => AnchorKindV1::Style,
        AnchorKindV2::StyleAssignment => AnchorKindV1::StyleAssignment,
        _ => unreachable!("logical resolver must not consume spatial anchors"),
    }
}

fn v2_anchor(value: AnchorKindV1) -> AnchorKindV2 {
    AnchorKindV2::ALL[value as usize]
}

fn v2_kind(value: AuthoringDiagnosticKindV1) -> AuthoringDiagnosticKindV2 {
    use AuthoringDiagnosticKindV1 as V1;
    use AuthoringDiagnosticKindV2 as V2;
    match value {
        V1::InvalidUtf8 => V2::InvalidUtf8,
        V1::UnsupportedToken => V2::UnsupportedToken,
        V1::UnsupportedAuthoringFormat => V2::UnsupportedAuthoringFormat,
        V1::UnexpectedToken => V2::UnexpectedToken,
        V1::UnexpectedEof => V2::UnexpectedEof,
        V1::InvalidIdentifier => V2::InvalidIdentifier,
        V1::InvalidLiteral => V2::InvalidLiteral,
        V1::DuplicateComponentName => V2::DuplicateComponentName,
        V1::DuplicatePropertyName => V2::DuplicatePropertyName,
        V1::DuplicateTemplateName => V2::DuplicateTemplateName,
        V1::DuplicateRegionName => V2::DuplicateRegionName,
        V1::UnknownComponentName => V2::UnknownComponentName,
        V1::UnknownPropertyName => V2::UnknownPropertyName,
        V1::UnknownTemplateName => V2::UnknownTemplateName,
        V1::UnknownRegionName => V2::UnknownRegionName,
        V1::ValueTypeMismatch => V2::ValueTypeMismatch,
        V1::LimitExceeded(kind) => V2::LimitExceeded(v2_limit(kind)),
        V1::IrValidation(kind) => V2::IrValidation(kind),
    }
}

fn v2_limit(value: AuthoringLimitKindV1) -> AuthoringLimitKindV2 {
    use AuthoringLimitKindV1 as V1;
    use AuthoringLimitKindV2 as V2;
    match value {
        V1::FenSourceBytes => V2::FenSourceBytes,
        V1::Tokens => V2::Tokens,
        V1::IdentifierBytes => V2::IdentifierBytes,
        V1::NestingDepth => V2::NestingDepth,
        V1::Components => V2::Components,
        V1::Properties => V2::Properties,
        V1::Templates => V2::Templates,
        V1::Regions => V2::Regions,
        V1::ChildSlots => V2::ChildSlots,
        V1::InitialProperties => V2::InitialProperties,
        V1::InitialKeys => V2::InitialKeys,
        V1::StyleAssignments => V2::StyleAssignments,
        V1::SourceAnchors => V2::SourceAnchors,
        V1::GeneratedRustBytes => V2::GeneratedRustBytes,
    }
}
