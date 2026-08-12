use std::collections::BTreeSet;

use super::source::{all_source, read, source_dir};

const EXPECTED_EXPORTS: [&str; 99] = [
    "ChildSlot",
    "ConstructionProgram",
    "InitialKey",
    "InitialProperty",
    "StructuralRegion",
    "TemplateNode",
    "IrValidationError",
    "IrValidationErrorKind",
    "ValidationLimitKind",
    "ComponentTypeId",
    "ConstructionFormatVersion",
    "PropertyId",
    "SUPPORTED_CONSTRUCTION_FORMAT",
    "SUPPORTED_SCHEMA_FORMAT",
    "SUPPORTED_STYLE_FORMAT",
    "SchemaFormatVersion",
    "SchemaNamespace",
    "SchemaRevision",
    "SourceId",
    "StructuralRegionId",
    "StyleFormatVersion",
    "TemplateNodeId",
    "InvalidationClass",
    "InvalidationIter",
    "InvalidationSet",
    "StyleValidationLimits",
    "ValidationLimits",
    "ComponentSchema",
    "PropertySchema",
    "SchemaManifest",
    "SourceSpan",
    "StyleAssignment",
    "StyleProgram",
    "ChildFactory",
    "ChildFactoryIter",
    "ComponentPropertiesIter",
    "ComponentSchemaView",
    "InitialKeyIter",
    "InitialKeyView",
    "InitialPropertyIter",
    "InitialPropertyView",
    "LinkedStyleValueView",
    "PropertySchemaView",
    "RegionFactory",
    "StyleAssignmentIter",
    "StyleAssignmentView",
    "StyleValueOrigin",
    "TemplateFactory",
    "ValidatedConstruction",
    "ValidatedSchema",
    "ValidatedStyleProgram",
    "validate_construction",
    "validate_schema",
    "validate_style",
    "InputPolicy",
    "PropertyValue",
    "ValueType",
    "SpatialFormatVersion",
    "SUPPORTED_SPATIAL_FORMAT",
    "SpatialNodeSymbolV2",
    "SpatialShapeSymbolV2",
    "SpatialBrushSymbolV2",
    "SpatialClipSymbolV2",
    "SpatialImageSymbolV2",
    "SpatialFieldV2",
    "SpatialBindingV2",
    "SpatialAxisV2",
    "SpatialAnchorComponentV2",
    "SpatialFillRuleV2",
    "SpatialNodeParentV2",
    "SpatialAnchorTargetRecipeV2",
    "SpatialClipAddressV2",
    "SpatialPointRecipeV2",
    "SpatialPaddingRecipeV2",
    "SpatialDimensionRecipeV2",
    "SpatialTransformRecipeV2",
    "SpatialViewportContainerV2",
    "SpatialContainerRecipeV2",
    "SpatialLayoutPlacementRecipeV2",
    "SpatialFreePlacementRecipeV2",
    "SpatialPlacementRecipeV2",
    "SpatialPathVerbRecipeV2",
    "SpatialPolygonPointV2",
    "SpatialShapeGeometryV2",
    "SpatialShapeDeclarationV2",
    "SpatialGradientStopV2",
    "SpatialBrushContentV2",
    "SpatialBrushDeclarationV2",
    "SpatialCoverageRecipeV2",
    "SpatialClipDeclarationV2",
    "SpatialPaintRecipeV2",
    "SpatialHitRecipeV2",
    "SpatialSemanticRecipeV2",
    "SpatialImageDeclarationV2",
    "SpatialNodeDeclarationV2",
    "SpatialProgramV2",
    "SpatialValidationLimitsV2",
    "ValidatedSpatialProgramV2",
    "validate_spatial",
];

const EXPECTED_STRUCTS: [&str; 70] = [
    "ChildFactoryIter",
    "ChildSlot",
    "ComponentPropertiesIter",
    "ComponentSchema",
    "ComponentSchemaView",
    "ComponentTypeId",
    "ConstructionFormatVersion",
    "ConstructionProgram",
    "InitialKey",
    "InitialKeyIter",
    "InitialKeyView",
    "InitialProperty",
    "InitialPropertyIter",
    "InitialPropertyView",
    "InvalidationIter",
    "InvalidationSet",
    "IrValidationError",
    "LinkedStyleValueView",
    "PropertyId",
    "PropertySchema",
    "PropertySchemaView",
    "RegionFactory",
    "SchemaFormatVersion",
    "SchemaManifest",
    "SchemaNamespace",
    "SchemaRevision",
    "SourceId",
    "SpatialBrushDeclarationV2",
    "SpatialBrushSymbolV2",
    "SpatialClipAddressV2",
    "SpatialClipDeclarationV2",
    "SpatialClipSymbolV2",
    "SpatialContainerRecipeV2",
    "SpatialDimensionRecipeV2",
    "SpatialFieldV2",
    "SpatialFormatVersion",
    "SpatialFreePlacementRecipeV2",
    "SpatialGradientStopV2",
    "SpatialHitRecipeV2",
    "SpatialImageDeclarationV2",
    "SpatialImageSymbolV2",
    "SpatialLayoutPlacementRecipeV2",
    "SpatialNodeDeclarationV2",
    "SpatialNodeSymbolV2",
    "SpatialPaddingRecipeV2",
    "SpatialPointRecipeV2",
    "SpatialPolygonPointV2",
    "SpatialProgramV2",
    "SpatialSemanticRecipeV2",
    "SpatialShapeDeclarationV2",
    "SpatialShapeSymbolV2",
    "SpatialTransformRecipeV2",
    "SpatialValidationLimitsV2",
    "SpatialViewportContainerV2",
    "StructuralRegion",
    "StructuralRegionId",
    "StyleAssignment",
    "StyleAssignmentIter",
    "StyleAssignmentView",
    "StyleFormatVersion",
    "StyleProgram",
    "StyleValidationLimits",
    "TemplateFactory",
    "TemplateNode",
    "TemplateNodeId",
    "ValidatedConstruction",
    "ValidatedSchema",
    "ValidatedSpatialProgramV2",
    "ValidatedStyleProgram",
    "ValidationLimits",
];

#[test]
fn prototype_has_the_exact_ordered_99_item_registry() {
    let lib = read(&source_dir().join("lib.rs"));
    let all = all_source();
    for forbidden in ["include!", "#[macro_export]"] {
        assert!(!all.contains(forbidden), "unexpected API form {forbidden}");
    }

    let marker = "pub mod prototype {";
    assert!(lib.contains("#[doc(hidden)]\npub mod prototype {"));
    let offset = lib.find(marker).expect("prototype module");
    assert!(!lib[..offset].lines().any(is_public_line));
    let prototype = &lib[offset + marker.len()..lib.rfind('}').expect("prototype end")];
    for forbidden in [" as ", "::*"] {
        assert!(
            !prototype.contains(forbidden),
            "unexpected API form {forbidden}"
        );
    }
    assert!(
        prototype
            .lines()
            .filter(|line| is_public_line(line))
            .all(|line| line.trim_start().starts_with("pub use crate::"))
    );

    assert_eq!(prototype_exports(prototype), EXPECTED_EXPORTS);
}

#[test]
fn crate_has_the_exact_70_public_struct_registry() {
    let source = all_source();
    let observed = public_structs(&source);
    assert_eq!(observed, EXPECTED_STRUCTS.into_iter().collect());
}

fn prototype_exports(prototype: &str) -> Vec<&str> {
    let mut observed = Vec::new();
    for item in prototype.split("pub use crate::").skip(1) {
        let names = if let Some(list_start) = item.find("::{") {
            let list_end = item.find("};").expect("terminated grouped reexport");
            &item[list_start + 3..list_end]
        } else {
            let item_end = item.find(';').expect("terminated singleton reexport");
            item[..item_end]
                .rsplit("::")
                .next()
                .expect("singleton export")
        };
        observed.extend(
            names
                .split(',')
                .map(str::trim)
                .filter(|name| !name.is_empty()),
        );
    }
    observed
}

fn public_structs(source: &str) -> BTreeSet<&str> {
    let mut observed = BTreeSet::new();
    for declaration in source
        .lines()
        .filter_map(|line| line.trim().strip_prefix("pub struct "))
    {
        let name = declaration
            .split(['<', '(', '{'])
            .next()
            .expect("struct name")
            .trim();
        if name != "$name" {
            assert!(observed.insert(name), "duplicate struct {name}");
        }
    }
    for invocation in source.split("u32_symbol!(").skip(1) {
        let name = invocation
            .trim_start()
            .split([',', '\n'])
            .next()
            .expect("symbol name")
            .trim();
        assert!(observed.insert(name), "duplicate symbol struct {name}");
    }
    observed
}

fn is_public_line(line: &str) -> bool {
    let line = line.trim_start();
    line.starts_with("pub ") || line.starts_with("pub(")
}
