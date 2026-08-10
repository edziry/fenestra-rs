use fenestra_ui_ir::prototype::{
    InputPolicy, InvalidationClass, InvalidationSet, PropertyValue, ValueType,
};

use crate::resolved::{
    ResolvedChildV1, ResolvedComponentV1, ResolvedConstructionV1, ResolvedDocumentV1,
    ResolvedInitialKeyV1, ResolvedInitialPropertyV1, ResolvedPropertyV1, ResolvedRegionV1,
    ResolvedSchemaV1, ResolvedStyleAssignmentV1, ResolvedStyleV1, ResolvedTemplateV1,
};

pub(super) fn document() -> ResolvedDocumentV1 {
    ResolvedDocumentV1 {
        format: 1,
        document_anchor: 0,
        schema: schema(),
        construction: construction(),
        style: style(),
    }
}

fn schema() -> ResolvedSchemaV1 {
    ResolvedSchemaV1 {
        namespace: 8_001,
        revision: 1,
        components: vec![ResolvedComponentV1 {
            name: "widget".into(),
            id: 0,
            properties: vec![
                ResolvedPropertyV1 {
                    name: "width".into(),
                    id: 0,
                    value_type: ValueType::ScalarI32,
                    default: PropertyValue::ScalarI32(10),
                    invalidation: invalidation(&[InvalidationClass::Layout]),
                    anchor: 3,
                },
                ResolvedPropertyV1 {
                    name: "color".into(),
                    id: 1,
                    value_type: ValueType::Rgba8,
                    default: PropertyValue::Rgba8([1, 2, 3, 255]),
                    invalidation: invalidation(&[InvalidationClass::Paint]),
                    anchor: 4,
                },
            ],
            anchor: 2,
        }],
        anchor: 1,
    }
}

fn construction() -> ResolvedConstructionV1 {
    ResolvedConstructionV1 {
        templates: vec![
            ResolvedTemplateV1 {
                name: "root".into(),
                id: 0,
                component: 0,
                initial_properties: vec![ResolvedInitialPropertyV1 {
                    property: 0,
                    value: PropertyValue::ScalarI32(20),
                    anchor: 7,
                }],
                children: vec![
                    ResolvedChildV1::Static {
                        template: 1,
                        anchor: 8,
                    },
                    ResolvedChildV1::Region {
                        region: 0,
                        anchor: 9,
                    },
                ],
                anchor: 6,
            },
            ResolvedTemplateV1 {
                name: "leaf".into(),
                id: 1,
                component: 0,
                initial_properties: Vec::new(),
                children: Vec::new(),
                anchor: 10,
            },
        ],
        regions: vec![ResolvedRegionV1 {
            name: "items".into(),
            id: 0,
            owner: 0,
            repeat_body: 1,
            initial_keys: vec![ResolvedInitialKeyV1 {
                value: 7,
                anchor: 12,
            }],
            invalidation: invalidation(&[InvalidationClass::Structure, InvalidationClass::Layout]),
            anchor: 11,
        }],
        anchor: 5,
    }
}

fn style() -> ResolvedStyleV1 {
    ResolvedStyleV1 {
        assignments: vec![ResolvedStyleAssignmentV1 {
            target: 1,
            property: 1,
            value: PropertyValue::Rgba8([4, 5, 6, 255]),
            anchor: 14,
        }],
        anchor: 13,
    }
}

pub(super) fn invalidation(classes: &[InvalidationClass]) -> InvalidationSet {
    classes
        .iter()
        .copied()
        .fold(InvalidationSet::NONE, |set, class| {
            set.union(InvalidationSet::from_class(class))
        })
}

pub(super) fn alternate_policy() -> PropertyValue {
    PropertyValue::InputPolicy(InputPolicy::Accept)
}
