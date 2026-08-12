use crate::*;
use fenestra_ui_ir::prototype::{PropertyId, SourceSpan};

use super::source::all_source;
use super::surface_support::{assert_enum_body, public_constants};

#[test]
fn format_symbols_fields_and_generic_bindings_round_trip_exact_values() {
    assert_eq!(SUPPORTED_SPATIAL_FORMAT, SpatialFormatVersion::new(2));
    assert_eq!(SpatialFormatVersion::new(u32::MAX).get(), u32::MAX);
    assert_eq!(SpatialNodeSymbolV2::new(1).get(), 1);
    assert_eq!(SpatialShapeSymbolV2::new(2).get(), 2);
    assert_eq!(SpatialBrushSymbolV2::new(3).get(), 3);
    assert_eq!(SpatialClipSymbolV2::new(4).get(), 4);
    assert_eq!(SpatialImageSymbolV2::new(5).get(), 5);

    let span = SourceSpan::synthetic();
    let field = SpatialFieldV2::new(SpatialBindingV2::Literal(-7_i64), span);
    assert_eq!(field.value(), &SpatialBindingV2::Literal(-7));
    assert_eq!(field.span(), span);

    let unconstrained = SpatialBindingV2::Literal(String::from("generic"));
    assert_eq!(
        unconstrained,
        SpatialBindingV2::Literal(String::from("generic"))
    );
    let property: SpatialBindingV2<String> = SpatialBindingV2::Property(PropertyId::new(9));
    assert_eq!(property, SpatialBindingV2::Property(PropertyId::new(9)));
}

#[test]
fn closed_spatial_vocabularies_have_exact_all_arrays_and_order() {
    assert_eq!(
        SpatialAxisV2::ALL,
        [SpatialAxisV2::Row, SpatialAxisV2::Column]
    );
    assert_eq!(
        SpatialAnchorComponentV2::ALL,
        [
            SpatialAnchorComponentV2::Start,
            SpatialAnchorComponentV2::Center,
            SpatialAnchorComponentV2::End,
        ]
    );
    assert_eq!(
        SpatialFillRuleV2::ALL,
        [SpatialFillRuleV2::NonZero, SpatialFillRuleV2::EvenOdd]
    );
    assert_eq!(axis_ordinal(SpatialAxisV2::Column), 1);
    assert_eq!(anchor_ordinal(SpatialAnchorComponentV2::End), 2);
    assert_eq!(fill_ordinal(SpatialFillRuleV2::EvenOdd), 1);

    let source = all_source();
    assert_enum_body(&source, "SpatialAxisV2", "Row,Column,");
    assert_enum_body(&source, "SpatialAnchorComponentV2", "Start,Center,End,");
    assert_enum_body(&source, "SpatialFillRuleV2", "NonZero,EvenOdd,");
}

#[test]
fn validation_limit_vocabulary_has_exact_variant_order_and_no_all() {
    let source = all_source();
    assert_enum_body(
        &source,
        "ValidationLimitKind",
        concat!(
            "Components,Properties,Templates,Regions,ChildSlots,InitialProperties,",
            "InitialKeys,TemplateDepth,InitialInstances,StyleAssignments,",
            "SpatialNodes,SpatialShapes,SpatialBrushes,SpatialClips,SpatialPaintItems,",
            "SpatialHitItems,SpatialSemanticItems,SpatialPaths,SpatialPathVerbs,",
            "SpatialPolygonPoints,SpatialGradientStops,SpatialImages,SpatialImageBytes,"
        ),
    );
    assert!(public_constants(&source, "ValidationLimitKind").is_empty());
}

#[test]
fn ir_validation_error_vocabulary_and_all_registry_are_exact() {
    let source = all_source();
    assert_enum_body(
        &source,
        "IrValidationErrorKind",
        concat!(
            "UnsupportedSchemaFormat,UnsupportedConstructionFormat,SchemaIdentityMismatch,",
            "InvalidSourceSpan,DuplicateComponent,DuplicateProperty,",
            "PropertyDefaultTypeMismatch,EmptyPropertyInvalidation,",
            "InvalidPropertyInvalidation,DuplicateNode,DuplicateInitialProperty,",
            "UnknownInitialProperty,InitialPropertyTypeMismatch,DuplicateRegion,",
            "MissingComponent,MissingStaticChild,MissingRegion,MissingRegionOwner,",
            "RegionOwnerMismatch,DuplicateRegionPlacement,MissingRegionTemplate,",
            "UnplacedRegion,DuplicateNodeOwner,InvalidRootCount,OwnershipCycle,",
            "DuplicateRegionKey,InvalidRegionInvalidation,UnsupportedStyleFormat,",
            "MissingStyleTarget,UnknownStyleProperty,StylePropertyTypeMismatch,",
            "DuplicateStyleAssignment,UnsupportedSpatialFormat,DuplicateSpatialNode,",
            "DuplicateSpatialTemplate,MissingSpatialTemplate,MissingSpatialParent,",
            "SpatialParentContextMismatch,SpatialParentNotEarlier,InvalidSpatialPreorder,",
            "UnknownSpatialProperty,SpatialPropertyTypeMismatch,SpatialFixed16OutOfRange,",
            "DuplicateSpatialShape,DuplicateSpatialBrush,DuplicateSpatialClip,",
            "DuplicateSpatialImage,MissingSpatialShape,MissingSpatialBrush,",
            "MissingSpatialImage,MissingSpatialClipOwner,MissingSpatialClip,",
            "SpatialClipOwnerNotAncestor,SpatialClipParentNotEarlier,",
            "MissingSpatialAnchorTarget,SelfAnchorTarget,SpatialAnchorContextMismatch,",
            "LimitExceeded(ValidationLimitKind),"
        ),
    );
    let compact = compact(&source);
    assert!(
        compact.contains(&expected_error_all()),
        "IrValidationErrorKind::ALL must have its exact 80 values"
    );
}

const _: SpatialFormatVersion = SpatialFormatVersion::new(2);
const _: SpatialFormatVersion = SUPPORTED_SPATIAL_FORMAT;
const _: u32 = SpatialFormatVersion::new(2).get();
const _: SpatialNodeSymbolV2 = SpatialNodeSymbolV2::new(0);
const _: SpatialShapeSymbolV2 = SpatialShapeSymbolV2::new(0);
const _: SpatialBrushSymbolV2 = SpatialBrushSymbolV2::new(0);
const _: SpatialClipSymbolV2 = SpatialClipSymbolV2::new(0);
const _: SpatialImageSymbolV2 = SpatialImageSymbolV2::new(0);

fn expected_error_all() -> String {
    compact(concat!(
        "pub const ALL: [Self; 80] = [",
        "Self::UnsupportedSchemaFormat,Self::UnsupportedConstructionFormat,",
        "Self::SchemaIdentityMismatch,Self::InvalidSourceSpan,Self::DuplicateComponent,",
        "Self::DuplicateProperty,Self::PropertyDefaultTypeMismatch,",
        "Self::EmptyPropertyInvalidation,Self::InvalidPropertyInvalidation,",
        "Self::DuplicateNode,Self::DuplicateInitialProperty,Self::UnknownInitialProperty,",
        "Self::InitialPropertyTypeMismatch,Self::DuplicateRegion,Self::MissingComponent,",
        "Self::MissingStaticChild,Self::MissingRegion,Self::MissingRegionOwner,",
        "Self::RegionOwnerMismatch,Self::DuplicateRegionPlacement,",
        "Self::MissingRegionTemplate,Self::UnplacedRegion,Self::DuplicateNodeOwner,",
        "Self::InvalidRootCount,Self::OwnershipCycle,Self::DuplicateRegionKey,",
        "Self::InvalidRegionInvalidation,",
        "Self::LimitExceeded(ValidationLimitKind::Components),",
        "Self::LimitExceeded(ValidationLimitKind::Properties),",
        "Self::LimitExceeded(ValidationLimitKind::Templates),",
        "Self::LimitExceeded(ValidationLimitKind::Regions),",
        "Self::LimitExceeded(ValidationLimitKind::ChildSlots),",
        "Self::LimitExceeded(ValidationLimitKind::InitialProperties),",
        "Self::LimitExceeded(ValidationLimitKind::InitialKeys),",
        "Self::LimitExceeded(ValidationLimitKind::TemplateDepth),",
        "Self::LimitExceeded(ValidationLimitKind::InitialInstances),",
        "Self::UnsupportedStyleFormat,Self::MissingStyleTarget,Self::UnknownStyleProperty,",
        "Self::StylePropertyTypeMismatch,Self::DuplicateStyleAssignment,",
        "Self::LimitExceeded(ValidationLimitKind::StyleAssignments),",
        "Self::UnsupportedSpatialFormat,",
        "Self::LimitExceeded(ValidationLimitKind::SpatialNodes),",
        "Self::LimitExceeded(ValidationLimitKind::SpatialShapes),",
        "Self::LimitExceeded(ValidationLimitKind::SpatialBrushes),",
        "Self::LimitExceeded(ValidationLimitKind::SpatialClips),",
        "Self::LimitExceeded(ValidationLimitKind::SpatialPaintItems),",
        "Self::LimitExceeded(ValidationLimitKind::SpatialHitItems),",
        "Self::LimitExceeded(ValidationLimitKind::SpatialSemanticItems),",
        "Self::LimitExceeded(ValidationLimitKind::SpatialPaths),",
        "Self::LimitExceeded(ValidationLimitKind::SpatialPathVerbs),",
        "Self::LimitExceeded(ValidationLimitKind::SpatialPolygonPoints),",
        "Self::LimitExceeded(ValidationLimitKind::SpatialGradientStops),",
        "Self::LimitExceeded(ValidationLimitKind::SpatialImages),",
        "Self::LimitExceeded(ValidationLimitKind::SpatialImageBytes),",
        "Self::DuplicateSpatialNode,Self::DuplicateSpatialTemplate,",
        "Self::MissingSpatialTemplate,Self::MissingSpatialParent,",
        "Self::SpatialParentContextMismatch,Self::SpatialParentNotEarlier,",
        "Self::InvalidSpatialPreorder,Self::UnknownSpatialProperty,",
        "Self::SpatialPropertyTypeMismatch,Self::SpatialFixed16OutOfRange,",
        "Self::DuplicateSpatialShape,Self::DuplicateSpatialBrush,",
        "Self::DuplicateSpatialClip,Self::DuplicateSpatialImage,",
        "Self::MissingSpatialShape,Self::MissingSpatialBrush,Self::MissingSpatialImage,",
        "Self::MissingSpatialClipOwner,Self::MissingSpatialClip,",
        "Self::SpatialClipOwnerNotAncestor,Self::SpatialClipParentNotEarlier,",
        "Self::MissingSpatialAnchorTarget,Self::SelfAnchorTarget,",
        "Self::SpatialAnchorContextMismatch,];"
    ))
}

const fn axis_ordinal(value: SpatialAxisV2) -> usize {
    match value {
        SpatialAxisV2::Row => 0,
        SpatialAxisV2::Column => 1,
    }
}

const fn anchor_ordinal(value: SpatialAnchorComponentV2) -> usize {
    match value {
        SpatialAnchorComponentV2::Start => 0,
        SpatialAnchorComponentV2::Center => 1,
        SpatialAnchorComponentV2::End => 2,
    }
}

const fn fill_ordinal(value: SpatialFillRuleV2) -> usize {
    match value {
        SpatialFillRuleV2::NonZero => 0,
        SpatialFillRuleV2::EvenOdd => 1,
    }
}

fn compact(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}
