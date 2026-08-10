use std::collections::HashSet;

use fenestra_ui_ir::prototype::{
    ChildFactory, PropertyId, PropertyValue, TemplateFactory, TemplateNodeId,
    ValidatedConstruction, ValidatedStyleProgram, ValueType,
};
use fenestra_ui_layout::prototype::LayoutEngineV1;

use super::types::HeadlessRuntimeConfig;

/// Bounded headless projection collections in deterministic diagnostic order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessProjectionLimitKind {
    /// Materialized computed-style records.
    ComputedStyles,
    /// Materialized geometry records.
    Geometry,
    /// Materialized semantic records.
    Semantics,
    /// Materialized hit regions.
    HitRegions,
    /// Materialized scene rectangles.
    SceneRectangles,
}

impl HeadlessProjectionLimitKind {
    /// All projection limits in deterministic tie-break order.
    pub const ALL: [Self; 5] = [
        Self::ComputedStyles,
        Self::Geometry,
        Self::Semantics,
        Self::HitRegions,
        Self::SceneRectangles,
    ];
}

/// Closed failure taxonomy for provisional headless projection work.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessProjectionErrorKind {
    /// A named template or component-local property does not exist.
    MissingSpecificationTargetOrProperty,
    /// A named property does not have the required closed value type.
    PropertyTypeMismatch,
    /// The semantic template is repeated or belongs to a structural region.
    InvalidSemanticTemplate,
    /// A surface extent is negative.
    InvalidSurface,
    /// A materialized projection collection would exceed its bound.
    CapacityExceeded(HeadlessProjectionLimitKind),
    /// Materialized geometry contains a negative dimension.
    NegativeGeometry,
    /// Checked projection arithmetic cannot produce a result.
    ArithmeticExhausted,
    /// Projection records violate an internal invariant.
    InvariantViolation,
}

/// Inclusive bounds for one provisional headless projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HeadlessProjectionCapacity {
    computed_styles: usize,
    geometry: usize,
    semantics: usize,
    hit_regions: usize,
    scene_rectangles: usize,
}

impl HeadlessProjectionCapacity {
    /// Creates the complete set of projection bounds.
    #[must_use]
    pub const fn new(
        computed_styles: usize,
        geometry: usize,
        semantics: usize,
        hit_regions: usize,
        scene_rectangles: usize,
    ) -> Self {
        Self {
            computed_styles,
            geometry,
            semantics,
            hit_regions,
            scene_rectangles,
        }
    }

    /// Returns the computed-style record ceiling.
    #[must_use]
    pub const fn computed_styles(self) -> usize {
        self.computed_styles
    }

    /// Returns the geometry record ceiling.
    #[must_use]
    pub const fn geometry(self) -> usize {
        self.geometry
    }

    /// Returns the semantic record ceiling.
    #[must_use]
    pub const fn semantics(self) -> usize {
        self.semantics
    }

    /// Returns the hit-region ceiling.
    #[must_use]
    pub const fn hit_regions(self) -> usize {
        self.hit_regions
    }

    /// Returns the scene-rectangle ceiling.
    #[must_use]
    pub const fn scene_rectangles(self) -> usize {
        self.scene_rectangles
    }
}

/// Runtime-owned symbols and bounds for the provisional headless projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HeadlessProjectionSpec {
    width: PropertyId,
    height: PropertyId,
    color: PropertyId,
    visible: PropertyId,
    input: PropertyId,
    semantic_template: TemplateNodeId,
    semantic_label: u32,
    capacity: HeadlessProjectionCapacity,
}

impl HeadlessProjectionSpec {
    /// Creates a complete provisional projection specification.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        width: PropertyId,
        height: PropertyId,
        color: PropertyId,
        visible: PropertyId,
        input: PropertyId,
        semantic_template: TemplateNodeId,
        semantic_label: u32,
        capacity: HeadlessProjectionCapacity,
    ) -> Self {
        Self {
            width,
            height,
            color,
            visible,
            input,
            semantic_template,
            semantic_label,
            capacity,
        }
    }

    /// Returns the component-local width property symbol.
    #[must_use]
    pub const fn width(self) -> PropertyId {
        self.width
    }

    /// Returns the component-local height property symbol.
    #[must_use]
    pub const fn height(self) -> PropertyId {
        self.height
    }

    /// Returns the component-local color property symbol.
    #[must_use]
    pub const fn color(self) -> PropertyId {
        self.color
    }

    /// Returns the component-local visibility property symbol.
    #[must_use]
    pub const fn visible(self) -> PropertyId {
        self.visible
    }

    /// Returns the component-local input-policy property symbol.
    #[must_use]
    pub const fn input(self) -> PropertyId {
        self.input
    }

    /// Returns the construction-local semantic template symbol.
    #[must_use]
    pub const fn semantic_template(self) -> TemplateNodeId {
        self.semantic_template
    }

    /// Returns the closed semantic label symbol.
    #[must_use]
    pub const fn semantic_label(self) -> u32 {
        self.semantic_label
    }

    /// Returns the explicit projection bounds.
    #[must_use]
    pub const fn capacity(self) -> HeadlessProjectionCapacity {
        self.capacity
    }
}

/// Logical extents of one provisional headless surface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HeadlessSurface {
    width: i32,
    height: i32,
}

impl HeadlessSurface {
    /// Creates a logical surface extent.
    #[must_use]
    pub const fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }

    /// Returns the logical surface width.
    #[must_use]
    pub const fn width(self) -> i32 {
        self.width
    }

    /// Returns the logical surface height.
    #[must_use]
    pub const fn height(self) -> i32 {
        self.height
    }

    pub(super) fn validate(self) -> Result<(), HeadlessProjectionErrorKind> {
        if self.width < 0 || self.height < 0 {
            return Err(HeadlessProjectionErrorKind::InvalidSurface);
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct AuthoredTemplate<'a> {
    factory: TemplateFactory<'a>,
    under_region: bool,
}

impl HeadlessRuntimeConfig {
    pub(crate) fn new(
        style: ValidatedStyleProgram,
        spec: HeadlessProjectionSpec,
        surface: HeadlessSurface,
        layout_engine: Box<dyn LayoutEngineV1>,
    ) -> Result<Self, HeadlessProjectionErrorKind> {
        validate_specification(style.construction(), spec, surface)?;
        Ok(Self {
            style,
            spec,
            layout_engine,
        })
    }

    pub(crate) fn preflight_fixed_records(
        &self,
        projected_nodes: Option<usize>,
    ) -> Result<(), HeadlessProjectionErrorKind> {
        let Some(projected_nodes) = projected_nodes else {
            return Err(HeadlessProjectionErrorKind::CapacityExceeded(
                HeadlessProjectionLimitKind::ComputedStyles,
            ));
        };
        if projected_nodes > self.spec.capacity().computed_styles() {
            return Err(HeadlessProjectionErrorKind::CapacityExceeded(
                HeadlessProjectionLimitKind::ComputedStyles,
            ));
        }
        if projected_nodes > self.spec.capacity().geometry() {
            return Err(HeadlessProjectionErrorKind::CapacityExceeded(
                HeadlessProjectionLimitKind::Geometry,
            ));
        }
        Ok(())
    }

    pub(crate) fn materialized_value(
        &self,
        template: TemplateFactory<'_>,
        property: PropertyId,
    ) -> Option<PropertyValue> {
        self.style
            .assignment(template.id(), property)
            .map(|assignment| assignment.replacement().clone())
            .or_else(|| template.effective_value(property).cloned())
    }
}

fn validate_specification(
    construction: &ValidatedConstruction,
    spec: HeadlessProjectionSpec,
    surface: HeadlessSurface,
) -> Result<(), HeadlessProjectionErrorKind> {
    if construction.template(spec.semantic_template()).is_none() {
        return Err(HeadlessProjectionErrorKind::MissingSpecificationTargetOrProperty);
    }
    let authored = collect_authored_templates(construction)?;
    let properties = [
        spec.width(),
        spec.height(),
        spec.color(),
        spec.visible(),
        spec.input(),
    ];
    for entry in &authored {
        for property in properties {
            if entry.factory.component().property(property).is_none() {
                return Err(HeadlessProjectionErrorKind::MissingSpecificationTargetOrProperty);
            }
        }
    }

    let typed_properties = [
        (spec.width(), ValueType::ScalarI32),
        (spec.height(), ValueType::ScalarI32),
        (spec.color(), ValueType::Rgba8),
        (spec.visible(), ValueType::Bool),
        (spec.input(), ValueType::InputPolicy),
    ];
    for entry in &authored {
        for (property, expected) in typed_properties {
            if entry
                .factory
                .component()
                .property(property)
                .is_none_or(|declared| declared.value_type() != expected)
            {
                return Err(HeadlessProjectionErrorKind::PropertyTypeMismatch);
            }
        }
    }

    let mut semantic = authored
        .iter()
        .filter(|entry| entry.factory.id() == spec.semantic_template());
    let Some(target) = semantic.next() else {
        return Err(HeadlessProjectionErrorKind::InvalidSemanticTemplate);
    };
    if target.under_region || semantic.next().is_some() {
        return Err(HeadlessProjectionErrorKind::InvalidSemanticTemplate);
    }
    surface.validate()?;
    Ok(())
}

fn collect_authored_templates(
    construction: &ValidatedConstruction,
) -> Result<Vec<AuthoredTemplate<'_>>, HeadlessProjectionErrorKind> {
    let mut authored = Vec::new();
    let mut seen = HashSet::new();
    let mut pending = vec![AuthoredTemplate {
        factory: construction.root_factory(),
        under_region: false,
    }];
    while let Some(entry) = pending.pop() {
        if !seen.insert(entry.factory.id()) {
            return Err(HeadlessProjectionErrorKind::InvariantViolation);
        }
        let children = entry.factory.children().collect::<Vec<_>>();
        authored.push(entry);
        for child in children.into_iter().rev() {
            pending.push(match child {
                ChildFactory::Static { template, .. } => AuthoredTemplate {
                    factory: template,
                    under_region: entry.under_region,
                },
                ChildFactory::Region { region, .. } => AuthoredTemplate {
                    factory: region.repeat_body(),
                    under_region: true,
                },
            });
        }
    }
    Ok(authored)
}
