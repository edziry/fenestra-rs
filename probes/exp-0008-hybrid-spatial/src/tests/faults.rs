use fenestra_ui_spatial::prototype::{
    ReferenceRasterErrorKindV2, ReferenceRasterLimitKindV2, SpatialBrushFieldV2,
    SpatialClipErrorV2, SpatialClipFieldV2, SpatialContentErrorKindV2, SpatialContentReferenceV2,
    SpatialDependencyErrorKindV2, SpatialErrorLocationV2, SpatialHitFieldV2, SpatialImageErrorV2,
    SpatialImageFieldV2, SpatialLimitKindV2, SpatialOutputErrorKindV2, SpatialPaintFieldV2,
    SpatialPathGrammarErrorV2, SpatialPathVerbFieldV2, SpatialResolveErrorKindV2,
    SpatialShapeErrorV2, SpatialShapeFieldV2, SpatialTransformErrorKindV2,
};

use crate::baseline::raw_fault_evidence_v2;

#[test]
fn raw_content_faults_retain_existing_typed_kinds_and_locations() {
    let report = raw_fault_evidence_v2();
    let expected = [
        (
            "shape-negative-extent",
            SpatialResolveErrorKindV2::Content(SpatialContentErrorKindV2::InvalidShape(
                SpatialShapeErrorV2::NegativeExtent,
            )),
            SpatialErrorLocationV2::Shape {
                index: 0,
                field: SpatialShapeFieldV2::RectWidth,
            },
        ),
        (
            "path-first-not-move",
            SpatialResolveErrorKindV2::Content(SpatialContentErrorKindV2::InvalidPathGrammar(
                SpatialPathGrammarErrorV2::FirstNotMove,
            )),
            SpatialErrorLocationV2::PathVerb {
                path: 0,
                verb: 0,
                field: SpatialPathVerbFieldV2::Kind,
            },
        ),
        (
            "brush-too-few-stops",
            SpatialResolveErrorKindV2::Content(SpatialContentErrorKindV2::InvalidGradient(
                fenestra_ui_spatial::prototype::SpatialGradientErrorV2::TooFewStops,
            )),
            SpatialErrorLocationV2::Brush {
                index: 0,
                field: SpatialBrushFieldV2::GradientStopLength,
            },
        ),
        (
            "image-stride-mismatch",
            SpatialResolveErrorKindV2::Content(SpatialContentErrorKindV2::InvalidImage(
                SpatialImageErrorV2::StrideMismatch,
            )),
            SpatialErrorLocationV2::Image {
                index: 0,
                field: SpatialImageFieldV2::Stride,
            },
        ),
        (
            "clip-forward-parent",
            SpatialResolveErrorKindV2::Content(SpatialContentErrorKindV2::InvalidClip(
                SpatialClipErrorV2::ForwardParent,
            )),
            SpatialErrorLocationV2::Clip {
                index: 0,
                field: SpatialClipFieldV2::Parent,
            },
        ),
        (
            "paint-missing-brush",
            SpatialResolveErrorKindV2::Content(SpatialContentErrorKindV2::InvalidReference(
                SpatialContentReferenceV2::Brush,
            )),
            SpatialErrorLocationV2::Paint {
                index: 0,
                field: SpatialPaintFieldV2::Brush,
            },
        ),
        (
            "hit-missing-shape",
            SpatialResolveErrorKindV2::Content(SpatialContentErrorKindV2::InvalidReference(
                SpatialContentReferenceV2::Shape,
            )),
            SpatialErrorLocationV2::Hit {
                index: 0,
                field: SpatialHitFieldV2::Shape,
            },
        ),
    ];
    assert_eq!(report.raw_inputs.len(), expected.len());
    for (actual, (label, kind, location)) in report.raw_inputs.iter().zip(expected) {
        assert_eq!(actual.label, label);
        assert_eq!(actual.kind, kind);
        assert_eq!(actual.location, location);
        assert_eq!(actual.observed, None);
        assert_eq!(actual.maximum, None);
    }
}

#[test]
fn all_raw_limits_and_structural_output_faults_are_exercised() {
    let report = raw_fault_evidence_v2();
    assert_eq!(report.limits.len(), SpatialLimitKindV2::ALL.len());
    for (boundary, kind) in report.limits.iter().zip(SpatialLimitKindV2::ALL) {
        assert_eq!(boundary.kind, kind);
        assert!(boundary.equality_passes);
        assert_eq!(
            boundary.one_over_kind,
            SpatialResolveErrorKindV2::LimitExceeded(kind)
        );
        assert_eq!(boundary.location, SpatialErrorLocationV2::Input);
        assert_eq!(boundary.observed, boundary.maximum + 1);
    }
    assert_eq!(report.output_faults, SpatialOutputErrorKindV2::ALL);
}

#[test]
fn dependency_singular_raster_and_exact_rollback_seams_are_distinct() {
    let report = raw_fault_evidence_v2();
    assert_eq!(
        report.dependency_cycle.kind,
        SpatialResolveErrorKindV2::Dependency(SpatialDependencyErrorKindV2::Cycle)
    );
    assert_eq!(
        report.dependency_cycle.location,
        SpatialErrorLocationV2::Dependency { ordinal: 1 }
    );
    assert_eq!(
        report.singular.kind,
        SpatialResolveErrorKindV2::Transform(SpatialTransformErrorKindV2::SingularTransform)
    );
    assert_eq!(
        report.singular.location,
        SpatialErrorLocationV2::Node { index: 3 }
    );
    assert_eq!(
        report.raster.kind,
        ReferenceRasterErrorKindV2::LimitExceeded(ReferenceRasterLimitKindV2::Pixels)
    );
    assert_eq!(report.raster.location, SpatialErrorLocationV2::Input);
    assert_eq!(report.raster.observed, 4_194_305);
    assert_eq!(report.raster.maximum, 4_194_304);

    assert_eq!(report.rollback.attempted_generation, 9);
    assert_eq!(report.rollback.retained_generation, 8);
    assert_eq!(report.rollback.before_digest, report.rollback.after_digest);
    assert_eq!(
        report.rollback.before_allocation,
        report.rollback.after_allocation
    );
    assert_eq!(report.rollback.before_state, report.rollback.after_state);
}

#[test]
fn baseline_contains_no_renderer_shaped_or_invented_core_error() {
    let report = raw_fault_evidence_v2();
    assert_eq!(report.native_faults, 0);
    assert_eq!(report.native_presenter_rows, 0);
    assert_eq!(report.candidate_faults, 0);
    assert!(report
        .raw_inputs
        .iter()
        .all(|fault| !fault.label.contains("hit-error") && !fault.label.contains("paint-error")));
}
