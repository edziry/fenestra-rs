use crate::headless::oracle::HeadlessMismatchFieldV1 as Field;
use crate::semantic::NodePathV1;

use super::super::error::{
    HeadlessArtifactVerificationErrorKindV1 as Kind, HeadlessArtifactVerificationErrorV1 as Error,
};
use super::super::model::HeadlessArtifactV1;
use super::super::record::{
    ComputedRecordV1, GeometryRecordV1, RectangleRecordV1, SceneRecordV1, SemanticRecordV1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ProjectionDifferenceV1 {
    pub(super) kind: Kind,
    pub(super) index: usize,
    pub(super) field: Field,
}

pub(super) fn verify_projection_v1(
    stored: &HeadlessArtifactV1,
    expected: &HeadlessArtifactV1,
) -> Result<(), Error> {
    match first_projection_difference_v1(stored, expected) {
        Some(difference) => Err(Error::at(difference.kind, difference.index)),
        None => Ok(()),
    }
}

pub(super) fn first_projection_difference_v1(
    stored: &HeadlessArtifactV1,
    expected: &HeadlessArtifactV1,
) -> Option<ProjectionDifferenceV1> {
    compare_records(
        Kind::ComputedStyleMismatch,
        &stored.projection.computed,
        &expected.projection.computed,
        computed_field,
    )
    .or_else(|| {
        compare_records(
            Kind::GeometryMismatch,
            &stored.projection.geometry,
            &expected.projection.geometry,
            geometry_field,
        )
    })
    .or_else(|| {
        compare_records(
            Kind::SemanticsMismatch,
            &stored.projection.semantics,
            &expected.projection.semantics,
            semantic_field,
        )
    })
    .or_else(|| {
        compare_records(
            Kind::HitMismatch,
            &stored.projection.hits,
            &expected.projection.hits,
            hit_field,
        )
    })
    .or_else(|| {
        compare_records(
            Kind::SceneMismatch,
            &stored.projection.scene,
            &expected.projection.scene,
            scene_field,
        )
    })
}

trait HasPath {
    fn path(&self) -> &NodePathV1;
}

macro_rules! has_path {
    ($($record:ty),+ $(,)?) => {
        $(impl HasPath for $record {
            fn path(&self) -> &NodePathV1 {
                &self.path
            }
        })+
    };
}

has_path!(
    ComputedRecordV1,
    GeometryRecordV1,
    SemanticRecordV1,
    RectangleRecordV1,
    SceneRecordV1,
);

fn compare_records<T: HasPath>(
    kind: Kind,
    stored: &[T],
    expected: &[T],
    compare_fields: fn(&T, &T) -> Option<Field>,
) -> Option<ProjectionDifferenceV1> {
    let count = stored.len().max(expected.len());
    for index in 0..count {
        let field = match (stored.get(index), expected.get(index)) {
            (Some(left), Some(right)) if left.path() != right.path() => Some(Field::Path),
            (Some(left), Some(right)) => compare_fields(left, right),
            (Some(_), None) | (None, Some(_)) => Some(Field::Path),
            (None, None) => None,
        };
        if let Some(field) = field {
            return Some(ProjectionDifferenceV1 { kind, index, field });
        }
    }
    None
}

fn computed_field(left: &ComputedRecordV1, right: &ComputedRecordV1) -> Option<Field> {
    if left.width != right.width {
        Some(Field::Width)
    } else if left.height != right.height {
        Some(Field::Height)
    } else if left.color != right.color {
        Some(Field::Color)
    } else if left.visible != right.visible {
        Some(Field::Visible)
    } else if left.input != right.input {
        Some(Field::Input)
    } else {
        None
    }
}

fn geometry_field(left: &GeometryRecordV1, right: &GeometryRecordV1) -> Option<Field> {
    if left.bounds != right.bounds {
        Some(Field::Bounds)
    } else if left.clip != right.clip {
        Some(Field::Clip)
    } else {
        None
    }
}

fn semantic_field(left: &SemanticRecordV1, right: &SemanticRecordV1) -> Option<Field> {
    if left.role != right.role {
        Some(Field::Role)
    } else if left.label != right.label {
        Some(Field::Label)
    } else if left.action != right.action {
        Some(Field::Action)
    } else {
        None
    }
}

fn hit_field(left: &RectangleRecordV1, right: &RectangleRecordV1) -> Option<Field> {
    (left.rectangle != right.rectangle).then_some(Field::Clip)
}

fn scene_field(left: &SceneRecordV1, right: &SceneRecordV1) -> Option<Field> {
    if left.rectangle != right.rectangle {
        Some(Field::Rectangle)
    } else if left.color != right.color {
        Some(Field::Color)
    } else {
        None
    }
}
