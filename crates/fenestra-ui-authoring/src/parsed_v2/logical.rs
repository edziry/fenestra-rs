use fenestra_ui_ir::prototype::{InvalidationSet, PropertyValue, ValueType};

use crate::source_v2::PhysicalOriginV2;
use crate::vocabulary_v2::{AnchorKindV2, AuthoringFrontendV2};

use super::ParsedSpatialV2;

#[derive(Clone)]
pub(crate) struct ParsedDocumentV2 {
    pub(crate) frontend: AuthoringFrontendV2,
    pub(crate) format: u32,
    pub(crate) document_anchor: u32,
    pub(crate) schema: ParsedSchemaV2,
    pub(crate) construction: ParsedConstructionV2,
    pub(crate) style: ParsedStyleV2,
    pub(crate) spatial: ParsedSpatialV2,
    pub(crate) anchors: Vec<ParsedAnchorV2>,
}

#[derive(Clone)]
pub(crate) struct ParsedSchemaV2 {
    pub(crate) namespace: ParsedLiteralV2<u64>,
    pub(crate) revision: ParsedLiteralV2<u32>,
    pub(crate) components: Vec<ParsedComponentV2>,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) struct ParsedComponentV2 {
    pub(crate) name: Box<str>,
    pub(crate) id: ParsedLiteralV2<u32>,
    pub(crate) properties: Vec<ParsedPropertyV2>,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) struct ParsedPropertyV2 {
    pub(crate) name: Box<str>,
    pub(crate) id: ParsedLiteralV2<u32>,
    pub(crate) value_type: ValueType,
    pub(crate) default: ParsedLiteralV2<PropertyValue>,
    pub(crate) invalidation: InvalidationSet,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) struct ParsedConstructionV2 {
    pub(crate) templates: Vec<ParsedTemplateV2>,
    pub(crate) regions: Vec<ParsedRegionV2>,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) struct ParsedTemplateV2 {
    pub(crate) name: Box<str>,
    pub(crate) id: ParsedLiteralV2<u32>,
    pub(crate) component: SpannedV2<Box<str>>,
    pub(crate) items: Vec<ParsedTemplateItemV2>,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) enum ParsedTemplateItemV2 {
    Initial(ParsedInitialPropertyV2),
    Child(ParsedChildV2),
}

#[derive(Clone)]
pub(crate) struct ParsedInitialPropertyV2 {
    pub(crate) property: Box<str>,
    pub(crate) value: ParsedLiteralV2<PropertyValue>,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) enum ParsedChildV2 {
    Static { template: Box<str>, anchor: u32 },
    Region { region: Box<str>, anchor: u32 },
}

#[derive(Clone)]
pub(crate) struct ParsedRegionV2 {
    pub(crate) name: Box<str>,
    pub(crate) id: ParsedLiteralV2<u32>,
    pub(crate) owner: SpannedV2<Box<str>>,
    pub(crate) repeat_body: SpannedV2<Box<str>>,
    pub(crate) initial_keys: Vec<ParsedInitialKeyV2>,
    pub(crate) invalidation: InvalidationSet,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) struct ParsedInitialKeyV2 {
    pub(crate) value: ParsedLiteralV2<u64>,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) struct ParsedStyleV2 {
    pub(crate) assignments: Vec<ParsedStyleAssignmentV2>,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) struct ParsedStyleAssignmentV2 {
    pub(crate) target: SpannedV2<Box<str>>,
    pub(crate) property: Box<str>,
    pub(crate) value: ParsedLiteralV2<PropertyValue>,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) struct ParsedAnchorV2 {
    pub(crate) kind: AnchorKindV2,
    pub(crate) label: Box<str>,
    pub(crate) physical: PhysicalOriginV2,
}

#[derive(Clone)]
pub(crate) struct SpannedV2<T> {
    pub(crate) value: T,
    pub(crate) physical: PhysicalOriginV2,
}

#[derive(Clone)]
pub(crate) struct ParsedLiteralV2<T> {
    pub(crate) value: Result<T, PhysicalOriginV2>,
    pub(crate) physical: PhysicalOriginV2,
}
