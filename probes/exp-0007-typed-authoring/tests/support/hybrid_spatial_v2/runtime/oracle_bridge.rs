use fenestra_ui_ir::prototype::{InputPolicy, InvalidationClass, PropertyValue, SourceSpan};
use fenestra_ui_spatial::prototype::{
    SpatialErrorLocationV2, SpatialResolveErrorKindV2, SpatialTransformErrorKindV2,
};

use super::path::{FragmentPath, NodePath};
use super::types::{
    AuthoredSpatialLaneLog, NormalizedAabb, NormalizedChild, NormalizedManifestEntry,
    NormalizedMutation, NormalizedObservation, NormalizedPaintReference,
};
use crate::oracle::types as expected;

impl AuthoredSpatialLaneLog {
    pub fn oracle_log(&self) -> crate::oracle::Log {
        expected::Log {
            observations: self.observations.iter().map(observation).collect(),
            final_keys: self.final_keys.clone(),
            noop: expected::Noop {
                empty_preserved: self.noop_checks().empty_preserved(),
                same_value_preserved: self.noop_checks().same_value_preserved(),
                round_trip_preserved: self.noop_checks().round_trip_preserved(),
            },
            failure: failure(self),
        }
    }
}

fn failure(value: &AuthoredSpatialLaneLog) -> expected::Failure {
    let failure = value.failure();
    let kind = match failure.resolve_kind() {
        SpatialResolveErrorKindV2::Transform(SpatialTransformErrorKindV2::SingularTransform) => {
            expected::FailureKind::SingularTransform
        }
        SpatialResolveErrorKindV2::Transform(SpatialTransformErrorKindV2::ScalarOutOfDomain(_)) => {
            expected::FailureKind::ScalarOutOfDomain
        }
        other => panic!("unexpected authored failure kind: {other:?}"),
    };
    let location = match failure.resolve_location() {
        SpatialErrorLocationV2::Input => expected::FailureLocation::Input,
        SpatialErrorLocationV2::Node { index } => expected::FailureLocation::Node { index },
        other => panic!("unexpected authored failure location: {other:?}"),
    };
    expected::Failure {
        kind,
        location,
        ir_span: span(failure.ir_span()),
        operation_index: failure.operation_index(),
        outer_state_preserved: failure.outer_state_preserved(),
        spatial_snapshot_preserved: failure.spatial_snapshot_preserved(),
        complete_observation_preserved: failure.complete_observation_preserved(),
        authored_factor_span: span(value.authored_factor_span()),
    }
}

fn span(value: SourceSpan) -> expected::Span {
    match value {
        SourceSpan::Synthetic => expected::Span::Synthetic,
        SourceSpan::Bytes { source, start, end } => expected::Span::Bytes {
            source: source.get(),
            start,
            end,
        },
    }
}

fn observation(value: &NormalizedObservation) -> expected::Observation {
    expected::Observation {
        generation: value.generation,
        viewport: [value.viewport.width(), value.viewport.height()],
        receipt: expected::Receipt {
            generation: value.receipt.generation,
            invalidation: value
                .receipt
                .invalidation
                .iter()
                .map(invalidation)
                .collect(),
            mutations: value.receipt.mutations.iter().map(mutation).collect(),
        },
        state: expected::State {
            nodes: value
                .state
                .nodes
                .iter()
                .map(|node| expected::Node {
                    path: node.path.render(),
                    parent: node.parent.as_ref().map(NodePath::render),
                    template: node.template,
                    component: node.component,
                    properties: node
                        .properties
                        .iter()
                        .map(|(property, value)| (*property, property_value(value)))
                        .collect(),
                    children: node
                        .children
                        .iter()
                        .map(|child| match child {
                            NormalizedChild::Static(path) => expected::Child::Static(path.render()),
                            NormalizedChild::Region(path) => expected::Child::Region(path.render()),
                        })
                        .collect(),
                })
                .collect(),
            fragments: value
                .state
                .fragments
                .iter()
                .map(|fragment| expected::Fragment {
                    path: fragment.path.render(),
                    descriptor: fragment.descriptor,
                    members: fragment
                        .members
                        .iter()
                        .map(|(key, path)| (*key, path.render()))
                        .collect(),
                })
                .collect(),
        },
        projection: expected::Projection {
            mapping: value
                .projection
                .mapping
                .iter()
                .map(|(key, path)| (*key, path.as_ref().map(NodePath::render)))
                .collect(),
            geometry: value
                .projection
                .geometry
                .iter()
                .map(|row| expected::Geometry {
                    key: row.key,
                    path: row.path.as_ref().map(NodePath::render),
                    base: row.base,
                    affine: row.affine.0,
                    determinant: row.determinant,
                    aabb: aabb(row.aabb),
                })
                .collect(),
            clips: value
                .projection
                .clips
                .iter()
                .map(|row| expected::Clip {
                    key: row.key,
                    owner: row.owner,
                    path: row.path.render(),
                    parent: row.parent,
                    shape: row.shape,
                    affine: row.affine.0,
                    determinant: row.determinant,
                    primitive: aabb(row.primitive),
                    effective: aabb(row.effective),
                })
                .collect(),
            paints: value
                .projection
                .paints
                .iter()
                .map(|row| expected::Paint {
                    key: row.key,
                    owner: row.owner,
                    path: row.path.render(),
                    affine: row.affine.0,
                    determinant: row.determinant,
                    aabb: aabb(row.aabb),
                    reference: match row.reference {
                        NormalizedPaintReference::Coverage { shape, brush } => {
                            expected::PaintReference::Coverage { shape, brush }
                        }
                        NormalizedPaintReference::Image { image } => {
                            expected::PaintReference::Image { image }
                        }
                    },
                    clip: row.clip,
                    stack: row.stack,
                    item: row.item,
                })
                .collect(),
            hits: value.projection.hits.iter().map(item).collect(),
            semantics: value.projection.semantics.iter().map(item).collect(),
        },
        hit_queries: value
            .hit_queries
            .iter()
            .map(|query| expected::HitQuery {
                scene: query.scene,
                result: query.result.as_ref().map(|hit| expected::Hit {
                    key: hit.key,
                    owner: hit.owner,
                    path: hit.path.render(),
                    item: hit.item,
                    local: hit.local,
                }),
            })
            .collect(),
        raster: expected::Raster {
            width: value.raster.width,
            height: value.raster.height,
            stride: value.raster.stride,
            bytes: value.raster.bytes.clone(),
        },
    }
}

fn mutation(value: &NormalizedMutation) -> expected::Mutation {
    match value {
        NormalizedMutation::Property {
            node,
            property,
            old,
            new,
        } => expected::Mutation::Property {
            node: node.render(),
            property: *property,
            old: property_value(old),
            new: property_value(new),
        },
        NormalizedMutation::Insert {
            fragment,
            key,
            root,
            final_index,
            created,
        } => expected::Mutation::Insert {
            fragment: fragment.render(),
            key: *key,
            root: root.render(),
            final_index: *final_index,
            created: created.iter().map(manifest).collect(),
        },
        NormalizedMutation::Move {
            fragment,
            key,
            root,
            old_index,
            final_index,
        } => expected::Mutation::Move {
            fragment: fragment.render(),
            key: *key,
            root: root.render(),
            old_index: *old_index,
            final_index: *final_index,
        },
        NormalizedMutation::Remove {
            fragment,
            key,
            root,
            old_index,
            retired,
        } => expected::Mutation::Remove {
            fragment: fragment.render(),
            key: *key,
            root: root.render(),
            old_index: *old_index,
            retired: retired.iter().map(manifest).collect(),
        },
        NormalizedMutation::Viewport { old, new } => expected::Mutation::Viewport {
            old: [old.width(), old.height()],
            new: [new.width(), new.height()],
        },
    }
}

fn manifest(value: &NormalizedManifestEntry) -> expected::ManifestEntry {
    match value {
        NormalizedManifestEntry::Node(path) => expected::ManifestEntry::Node(path.render()),
        NormalizedManifestEntry::Fragment(path) => expected::ManifestEntry::Fragment(path.render()),
    }
}

fn property_value(value: &PropertyValue) -> expected::Value {
    match value {
        PropertyValue::Bool(value) => expected::Value::Bool(*value),
        PropertyValue::ScalarI32(value) => expected::Value::I32(*value),
        PropertyValue::Rgba8(value) => expected::Value::Rgba(*value),
        PropertyValue::InputPolicy(value) => {
            expected::Value::Policy(matches!(value, InputPolicy::Accept))
        }
    }
}

fn invalidation(value: InvalidationClass) -> u8 {
    match value {
        InvalidationClass::Structure => 0,
        InvalidationClass::StyleMatch => 1,
        InvalidationClass::Intrinsic => 2,
        InvalidationClass::Layout => 3,
        InvalidationClass::Semantics => 4,
        InvalidationClass::HitTest => 5,
        InvalidationClass::Paint => 6,
        InvalidationClass::Composition => 7,
        InvalidationClass::Surface => 8,
    }
}

fn aabb(value: NormalizedAabb) -> expected::Aabb {
    expected::Aabb {
        empty: value.empty,
        edges: value.edges,
    }
}

fn item(value: &super::types::NormalizedItem) -> expected::Item {
    expected::Item {
        key: value.key,
        owner: value.owner,
        path: value.path.render(),
        affine: value.affine.0,
        determinant: value.determinant,
        aabb: aabb(value.aabb),
        shape: value.shape,
        clip: value.clip,
        stack: value.stack,
        item: value.item,
    }
}

#[allow(dead_code)]
fn _paths_are_distinct(_: &NodePath, _: &FragmentPath) {}
