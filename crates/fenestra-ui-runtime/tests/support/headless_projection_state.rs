use fenestra_ui_ir::prototype::{InputPolicy, PropertyValue};
use fenestra_ui_runtime::prototype::{
    CommittedRuntimeSnapshot, HeadlessRect, HeadlessSemanticAction, HeadlessSemanticRole,
    HeadlessSurface, NodeId, RuntimeGeneration,
};

use super::headless::{COLOR, HEIGHT, INPUT, VISIBLE, WIDTH};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComputedRecord {
    pub node: NodeId,
    pub width: i32,
    pub height: i32,
    pub color: [u8; 4],
    pub visible: bool,
    pub input: InputPolicy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GeometryRecord {
    pub node: NodeId,
    pub bounds: HeadlessRect,
    pub clip: HeadlessRect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SemanticRecord {
    pub node: NodeId,
    pub role: HeadlessSemanticRole,
    pub label: u32,
    pub action: HeadlessSemanticAction,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HitRecord {
    pub node: NodeId,
    pub clip: HeadlessRect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneRecord {
    pub node: NodeId,
    pub rectangle: HeadlessRect,
    pub color: [u8; 4],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionRecords {
    pub generation: RuntimeGeneration,
    pub surface: HeadlessSurface,
    pub computed: Vec<ComputedRecord>,
    pub geometry: Vec<GeometryRecord>,
    pub semantics: Vec<SemanticRecord>,
    pub hits: Vec<HitRecord>,
    pub scenes: Vec<SceneRecord>,
}

impl ProjectionRecords {
    pub fn computed_mut(&mut self, node: NodeId) -> &mut ComputedRecord {
        self.computed
            .iter_mut()
            .find(|record| record.node == node)
            .expect("expected computed record should exist")
    }

    pub fn geometry_mut(&mut self, node: NodeId) -> &mut GeometryRecord {
        self.geometry
            .iter_mut()
            .find(|record| record.node == node)
            .expect("expected geometry record should exist")
    }

    pub fn hit_mut(&mut self, node: NodeId) -> &mut HitRecord {
        self.hits
            .iter_mut()
            .find(|record| record.node == node)
            .expect("expected hit record should exist")
    }

    pub fn scene_mut(&mut self, node: NodeId) -> &mut SceneRecord {
        self.scenes
            .iter_mut()
            .find(|record| record.node == node)
            .expect("expected scene record should exist")
    }

    pub fn remove_node(&mut self, node: NodeId) {
        self.computed.retain(|record| record.node != node);
        self.geometry.retain(|record| record.node != node);
        self.semantics.retain(|record| record.node != node);
        self.hits.retain(|record| record.node != node);
        self.scenes.retain(|record| record.node != node);
    }
}

pub fn capture_projection(snapshot: &CommittedRuntimeSnapshot) -> ProjectionRecords {
    let projection = snapshot
        .headless_projection()
        .expect("headless projection should exist");
    ProjectionRecords {
        generation: projection.generation(),
        surface: projection.surface(),
        computed: projection
            .computed_styles()
            .map(|record| ComputedRecord {
                node: record.node(),
                width: scalar(record.property(WIDTH)),
                height: scalar(record.property(HEIGHT)),
                color: color(record.property(COLOR)),
                visible: boolean(record.property(VISIBLE)),
                input: input(record.property(INPUT)),
            })
            .collect(),
        geometry: projection
            .geometries()
            .map(|record| GeometryRecord {
                node: record.node(),
                bounds: record.bounds(),
                clip: record.clip(),
            })
            .collect(),
        semantics: projection
            .semantics()
            .map(|record| SemanticRecord {
                node: record.node(),
                role: record.role(),
                label: record.label(),
                action: record.action(),
            })
            .collect(),
        hits: projection
            .hit_regions()
            .map(|record| HitRecord {
                node: record.node(),
                clip: record.clip(),
            })
            .collect(),
        scenes: projection
            .scene_rectangles()
            .map(|record| SceneRecord {
                node: record.node(),
                rectangle: record.rectangle(),
                color: record.color(),
            })
            .collect(),
    }
}

fn scalar(value: Option<&PropertyValue>) -> i32 {
    let Some(PropertyValue::ScalarI32(value)) = value else {
        panic!("expected scalar projection property");
    };
    *value
}

fn color(value: Option<&PropertyValue>) -> [u8; 4] {
    let Some(PropertyValue::Rgba8(value)) = value else {
        panic!("expected color projection property");
    };
    *value
}

fn boolean(value: Option<&PropertyValue>) -> bool {
    let Some(PropertyValue::Bool(value)) = value else {
        panic!("expected boolean projection property");
    };
    *value
}

fn input(value: Option<&PropertyValue>) -> InputPolicy {
    let Some(PropertyValue::InputPolicy(value)) = value else {
        panic!("expected input projection property");
    };
    *value
}
