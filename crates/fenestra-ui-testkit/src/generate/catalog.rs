use fenestra_ui_ir::prototype::{InputPolicy, PropertyId, PropertyValue, ValueType};

use crate::case::{SeedV1, SemanticOperationV1};
use crate::semantic::{FragmentPathV1, NodePathV1};

pub(super) fn value_catalog(value_type: ValueType) -> Vec<PropertyValue> {
    match value_type {
        ValueType::Bool => vec![PropertyValue::Bool(false), PropertyValue::Bool(true)],
        ValueType::ScalarI32 => [-1024, -1, 0, 1, 1024]
            .into_iter()
            .map(PropertyValue::ScalarI32)
            .collect(),
        ValueType::Rgba8 => [
            [0, 0, 0, 0],
            [0, 0, 0, 255],
            [255, 255, 255, 255],
            [51, 102, 153, 255],
        ]
        .into_iter()
        .map(PropertyValue::Rgba8)
        .collect(),
        ValueType::InputPolicy => [InputPolicy::Accept, InputPolicy::Ignore]
            .into_iter()
            .map(PropertyValue::InputPolicy)
            .collect(),
    }
}

pub(super) fn directed_prefix() -> [Vec<SemanticOperationV1>; 8] {
    let root = NodePathV1::root();
    let primary = FragmentPathV1::new(root.clone(), 1);
    let secondary = FragmentPathV1::new(root.clone(), 2);
    let nested = FragmentPathV1::new(root.clone().member(1, 9), 1);
    [
        vec![set_width(root.clone(), 320), set_width(root.clone(), 480)],
        vec![set_width(root, 480)],
        vec![
            SemanticOperationV1::InsertKeyed {
                fragment: primary.clone(),
                key: 9,
                final_index: 2,
            },
            SemanticOperationV1::MoveKeyed {
                fragment: primary.clone(),
                key: 9,
                final_index: 0,
            },
        ],
        vec![SemanticOperationV1::UpdateKeyed {
            fragment: primary.clone(),
            key: 9,
            property: PropertyId::new(0),
            value: PropertyValue::ScalarI32(90),
        }],
        vec![SemanticOperationV1::UpdateKeyed {
            fragment: secondary,
            key: 7,
            property: PropertyId::new(0),
            value: PropertyValue::ScalarI32(70),
        }],
        vec![SemanticOperationV1::InsertKeyed {
            fragment: nested,
            key: 2,
            final_index: 1,
        }],
        vec![SemanticOperationV1::RemoveKeyed {
            fragment: primary.clone(),
            key: 9,
        }],
        vec![SemanticOperationV1::InsertKeyed {
            fragment: primary,
            key: 9,
            final_index: 2,
        }],
    ]
}

fn set_width(node: NodePathV1, value: i32) -> SemanticOperationV1 {
    SemanticOperationV1::SetProperty {
        node,
        property: PropertyId::new(0),
        value: PropertyValue::ScalarI32(value),
    }
}

pub(super) struct WordStreamV1 {
    state: u64,
}

impl WordStreamV1 {
    pub(super) const fn new(seed: SeedV1) -> Self {
        Self {
            state: seed.get() ^ 0xa076_1d64_78bd_642f,
        }
    }

    pub(super) fn next(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(0xd134_2543_de82_ef95)
            .wrapping_add(0x9e37_79b9_7f4a_7c15);
        self.state ^ self.state.rotate_right(29)
    }
}
