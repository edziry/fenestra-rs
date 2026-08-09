use crate::error::{HarnessError, HarnessErrorKind};
use crate::semantic::NodePathV1;

use super::types::{
    HeadlessMismatchFieldV1 as Field, HeadlessMismatchKindV1 as Kind,
    HeadlessMismatchLocationV1 as Location, HeadlessMismatchV1,
    NormalizedHeadlessComputedStyleV1 as Computed, NormalizedHeadlessGeometryV1 as Geometry,
    NormalizedHeadlessHitRegionV1 as Hit, NormalizedHeadlessProjectionV1 as Projection,
    NormalizedHeadlessSceneRectangleV1 as Scene, NormalizedHeadlessSemanticV1 as Semantic,
};

/// Returns the first deterministic mismatch after validating the surface.
pub fn compare_headless_projection_v1(
    expected: &Projection,
    observed: &Projection,
) -> Result<Option<HeadlessMismatchV1>, HarnessError> {
    if expected.surface != observed.surface {
        return Err(HarnessError::new(HarnessErrorKind::StateMismatch));
    }
    if let Some(found) = compare_records(
        Kind::ComputedStyle,
        &expected.computed_styles,
        &observed.computed_styles,
        computed_field,
    ) {
        return Ok(Some(found));
    }
    if let Some(found) = compare_records(
        Kind::Geometry,
        &expected.geometries,
        &observed.geometries,
        geometry_field,
    ) {
        return Ok(Some(found));
    }
    if let Some(found) = compare_records(
        Kind::Semantics,
        &expected.semantics,
        &observed.semantics,
        semantic_field,
    ) {
        return Ok(Some(found));
    }
    if let Some(found) = compare_records(
        Kind::HitRegions,
        &expected.hit_regions,
        &observed.hit_regions,
        hit_field,
    ) {
        return Ok(Some(found));
    }
    Ok(compare_records(
        Kind::SceneRectangles,
        &expected.scene_rectangles,
        &observed.scene_rectangles,
        scene_field,
    ))
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

has_path!(Computed, Geometry, Semantic, Hit, Scene);

fn compare_records<T: HasPath>(
    kind: Kind,
    expected: &[T],
    observed: &[T],
    compare_fields: fn(&T, &T) -> Option<Field>,
) -> Option<HeadlessMismatchV1> {
    let count = expected.len().max(observed.len());
    for index in 0..count {
        let left = expected.get(index);
        let right = observed.get(index);
        let field = match (left, right) {
            (Some(left), Some(right)) if left.path() != right.path() => Some(Field::Path),
            (Some(left), Some(right)) => compare_fields(left, right),
            (Some(_), None) | (None, Some(_)) => Some(Field::Path),
            (None, None) => None,
        };
        if let Some(field) = field {
            let location = left
                .map(HasPath::path)
                .or_else(|| right.map(HasPath::path))
                .cloned()
                .map_or(Location::End, Location::Path);
            return Some(HeadlessMismatchV1 {
                kind,
                index,
                field,
                location,
            });
        }
    }
    None
}

fn computed_field(left: &Computed, right: &Computed) -> Option<Field> {
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

fn geometry_field(left: &Geometry, right: &Geometry) -> Option<Field> {
    if left.bounds != right.bounds {
        Some(Field::Bounds)
    } else if left.clip != right.clip {
        Some(Field::Clip)
    } else {
        None
    }
}

fn semantic_field(left: &Semantic, right: &Semantic) -> Option<Field> {
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

fn hit_field(left: &Hit, right: &Hit) -> Option<Field> {
    (left.clip != right.clip).then_some(Field::Clip)
}

fn scene_field(left: &Scene, right: &Scene) -> Option<Field> {
    if left.rectangle != right.rectangle {
        Some(Field::Rectangle)
    } else if left.color != right.color {
        Some(Field::Color)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::headless::oracle::types::rect;

    fn geometry(path: NodePathV1, bounds: [i32; 4], clip: [i32; 4]) -> Geometry {
        Geometry {
            path,
            bounds: rect(bounds[0], bounds[1], bounds[2], bounds[3]),
            clip: rect(clip[0], clip[1], clip[2], clip[3]),
        }
    }

    #[test]
    fn geometry_bounds_precede_clip() {
        let left = geometry(NodePathV1::root(), [0, 0, 10, 10], [0, 0, 8, 8]);
        let right = geometry(NodePathV1::root(), [0, 0, 9, 9], [0, 0, 7, 7]);

        let mismatch = compare_records(Kind::Geometry, &[left], &[right], geometry_field)
            .expect("both geometry fields differ");

        assert_eq!(mismatch.field, Field::Bounds);
    }

    #[test]
    fn geometry_path_precedes_payload() {
        let left = geometry(NodePathV1::root(), [0, 0, 10, 10], [0, 0, 8, 8]);
        let right = geometry(
            NodePathV1::root().static_child(0),
            [0, 0, 9, 9],
            [0, 0, 7, 7],
        );

        let mismatch = compare_records(Kind::Geometry, &[left], &[right], geometry_field)
            .expect("path and payload differ");

        assert_eq!(mismatch.field, Field::Path);
    }

    #[test]
    fn geometry_clip_is_reported_when_it_is_the_only_difference() {
        let left = geometry(NodePathV1::root(), [0, 0, 10, 10], [0, 0, 8, 8]);
        let right = geometry(NodePathV1::root(), [0, 0, 10, 10], [0, 0, 7, 7]);

        let mismatch = compare_records(Kind::Geometry, &[left], &[right], geometry_field)
            .expect("clip differs");

        assert_eq!(mismatch.field, Field::Clip);
    }
}
