use super::*;
use support::*;

#[test]
fn error_registry_preserves_the_old_prefix_and_appends_the_frozen_suffix() {
    let old_prefix = [
        IrValidationErrorKind::UnsupportedSchemaFormat,
        IrValidationErrorKind::UnsupportedConstructionFormat,
        IrValidationErrorKind::SchemaIdentityMismatch,
        IrValidationErrorKind::InvalidSourceSpan,
        IrValidationErrorKind::DuplicateComponent,
        IrValidationErrorKind::DuplicateProperty,
        IrValidationErrorKind::PropertyDefaultTypeMismatch,
        IrValidationErrorKind::EmptyPropertyInvalidation,
        IrValidationErrorKind::InvalidPropertyInvalidation,
        IrValidationErrorKind::DuplicateNode,
        IrValidationErrorKind::DuplicateInitialProperty,
        IrValidationErrorKind::UnknownInitialProperty,
        IrValidationErrorKind::InitialPropertyTypeMismatch,
        IrValidationErrorKind::DuplicateRegion,
        IrValidationErrorKind::MissingComponent,
        IrValidationErrorKind::MissingStaticChild,
        IrValidationErrorKind::MissingRegion,
        IrValidationErrorKind::MissingRegionOwner,
        IrValidationErrorKind::RegionOwnerMismatch,
        IrValidationErrorKind::DuplicateRegionPlacement,
        IrValidationErrorKind::MissingRegionTemplate,
        IrValidationErrorKind::UnplacedRegion,
        IrValidationErrorKind::DuplicateNodeOwner,
        IrValidationErrorKind::InvalidRootCount,
        IrValidationErrorKind::OwnershipCycle,
        IrValidationErrorKind::DuplicateRegionKey,
        IrValidationErrorKind::InvalidRegionInvalidation,
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::Components),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::Properties),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::Templates),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::Regions),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::ChildSlots),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::InitialProperties),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::InitialKeys),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::TemplateDepth),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::InitialInstances),
        IrValidationErrorKind::UnsupportedStyleFormat,
        IrValidationErrorKind::MissingStyleTarget,
        IrValidationErrorKind::UnknownStyleProperty,
        IrValidationErrorKind::StylePropertyTypeMismatch,
        IrValidationErrorKind::DuplicateStyleAssignment,
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::StyleAssignments),
    ];
    let suffix = [
        IrValidationErrorKind::UnsupportedSpatialFormat,
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::SpatialNodes),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::SpatialShapes),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::SpatialBrushes),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::SpatialClips),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::SpatialPaintItems),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::SpatialHitItems),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::SpatialSemanticItems),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::SpatialPaths),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::SpatialPathVerbs),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::SpatialPolygonPoints),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::SpatialGradientStops),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::SpatialImages),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::SpatialImageBytes),
        IrValidationErrorKind::DuplicateSpatialNode,
        IrValidationErrorKind::DuplicateSpatialTemplate,
        IrValidationErrorKind::MissingSpatialTemplate,
        IrValidationErrorKind::MissingSpatialParent,
        IrValidationErrorKind::SpatialParentContextMismatch,
        IrValidationErrorKind::SpatialParentNotEarlier,
        IrValidationErrorKind::InvalidSpatialPreorder,
        IrValidationErrorKind::UnknownSpatialProperty,
        IrValidationErrorKind::SpatialPropertyTypeMismatch,
        IrValidationErrorKind::SpatialFixed16OutOfRange,
        IrValidationErrorKind::DuplicateSpatialShape,
        IrValidationErrorKind::DuplicateSpatialBrush,
        IrValidationErrorKind::DuplicateSpatialClip,
        IrValidationErrorKind::DuplicateSpatialImage,
        IrValidationErrorKind::MissingSpatialShape,
        IrValidationErrorKind::MissingSpatialBrush,
        IrValidationErrorKind::MissingSpatialImage,
        IrValidationErrorKind::MissingSpatialClipOwner,
        IrValidationErrorKind::MissingSpatialClip,
        IrValidationErrorKind::SpatialClipOwnerNotAncestor,
        IrValidationErrorKind::SpatialClipParentNotEarlier,
        IrValidationErrorKind::MissingSpatialAnchorTarget,
        IrValidationErrorKind::SelfAnchorTarget,
        IrValidationErrorKind::SpatialAnchorContextMismatch,
    ];

    assert_eq!(IrValidationErrorKind::ALL.len(), 80);
    assert_eq!(&IrValidationErrorKind::ALL[..42], old_prefix.as_slice());
    assert_eq!(&IrValidationErrorKind::ALL[42..], suffix.as_slice());
}

#[test]
fn new_diagnostics_keep_typed_labels_spans_and_redaction() {
    let style = style();
    let unsupported = program_with(
        SpatialFormatVersion::new(3),
        NS,
        REV,
        viewport(200),
        Vec::new(),
        Vec::new(),
        span(201),
    );
    let error = validate(&style, unsupported).expect_err("format should fail");
    assert_eq!(
        error.to_string(),
        format!("unsupported-spatial-format at {:?}", span(201))
    );
    assert_eq!(format!("{error:?}"), format!("IrValidationError({error})"));

    let error = validate_spatial(
        &style,
        program(vec![node(0, ROOT, SpatialNodeParentV2::Viewport, 202)]),
        SpatialValidationLimitsV2::new([0; 13]),
    )
    .expect_err("node limit should fail");
    assert_eq!(
        error.to_string(),
        format!("limit-exceeded(spatial-nodes) at {:?}", span(210))
    );
}
