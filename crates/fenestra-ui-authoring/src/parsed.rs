use fenestra_ui_ir::prototype::{InvalidationSet, PropertyValue, ValueType};

use crate::source::PhysicalOriginV1;
use crate::vocabulary::{AnchorKindV1, AuthoringFrontendV1};

#[derive(Clone)]
pub(crate) struct ParsedDocumentV1 {
    pub(crate) frontend: AuthoringFrontendV1,
    pub(crate) format: u32,
    pub(crate) document_anchor: u32,
    pub(crate) schema: ParsedSchemaV1,
    pub(crate) construction: ParsedConstructionV1,
    pub(crate) style: ParsedStyleV1,
    pub(crate) anchors: Vec<ParsedAnchorV1>,
}

#[derive(Clone)]
pub(crate) struct ParsedSchemaV1 {
    pub(crate) namespace: ParsedLiteralV1<u64>,
    pub(crate) revision: ParsedLiteralV1<u32>,
    pub(crate) components: Vec<ParsedComponentV1>,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) struct ParsedComponentV1 {
    pub(crate) name: Box<str>,
    pub(crate) id: ParsedLiteralV1<u32>,
    pub(crate) properties: Vec<ParsedPropertyV1>,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) struct ParsedPropertyV1 {
    pub(crate) name: Box<str>,
    pub(crate) id: ParsedLiteralV1<u32>,
    pub(crate) value_type: ValueType,
    pub(crate) default: ParsedLiteralV1<PropertyValue>,
    pub(crate) invalidation: InvalidationSet,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) struct ParsedConstructionV1 {
    pub(crate) templates: Vec<ParsedTemplateV1>,
    pub(crate) regions: Vec<ParsedRegionV1>,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) struct ParsedTemplateV1 {
    pub(crate) name: Box<str>,
    pub(crate) id: ParsedLiteralV1<u32>,
    pub(crate) component: SpannedV1<Box<str>>,
    pub(crate) items: Vec<ParsedTemplateItemV1>,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) enum ParsedTemplateItemV1 {
    Initial(ParsedInitialPropertyV1),
    Child(ParsedChildV1),
}

#[derive(Clone)]
pub(crate) struct ParsedInitialPropertyV1 {
    pub(crate) property: Box<str>,
    pub(crate) value: ParsedLiteralV1<PropertyValue>,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) enum ParsedChildV1 {
    Static { template: Box<str>, anchor: u32 },
    Region { region: Box<str>, anchor: u32 },
}

#[derive(Clone)]
pub(crate) struct ParsedRegionV1 {
    pub(crate) name: Box<str>,
    pub(crate) id: ParsedLiteralV1<u32>,
    pub(crate) owner: SpannedV1<Box<str>>,
    pub(crate) repeat_body: SpannedV1<Box<str>>,
    pub(crate) initial_keys: Vec<ParsedInitialKeyV1>,
    pub(crate) invalidation: InvalidationSet,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) struct ParsedInitialKeyV1 {
    pub(crate) value: ParsedLiteralV1<u64>,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) struct ParsedStyleV1 {
    pub(crate) assignments: Vec<ParsedStyleAssignmentV1>,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) struct ParsedStyleAssignmentV1 {
    pub(crate) target: SpannedV1<Box<str>>,
    pub(crate) property: Box<str>,
    pub(crate) value: ParsedLiteralV1<PropertyValue>,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) struct ParsedAnchorV1 {
    pub(crate) kind: AnchorKindV1,
    pub(crate) label: Box<str>,
    pub(crate) physical: PhysicalOriginV1,
}

#[derive(Clone)]
pub(crate) struct SpannedV1<T> {
    pub(crate) value: T,
    pub(crate) physical: PhysicalOriginV1,
}

#[derive(Clone)]
pub(crate) struct ParsedLiteralV1<T> {
    pub(crate) value: Result<T, PhysicalOriginV1>,
    pub(crate) physical: PhysicalOriginV1,
}
