use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::ids::{PropertyId, TemplateNodeId};
use crate::invalidation::InvalidationSet;
use crate::source::SourceSpan;
use crate::style::{StyleAssignment, StyleProgram};
use crate::value::PropertyValue;

use super::{PropertySchemaView, TemplateFactory, ValidatedConstruction};

struct StyleData {
    program: StyleProgram,
    assignments: HashMap<(TemplateNodeId, PropertyId), usize>,
}

/// Immutable style program linked to an exact validated construction.
#[derive(Clone)]
pub struct ValidatedStyleProgram {
    construction: ValidatedConstruction,
    data: Arc<StyleData>,
}

impl ValidatedStyleProgram {
    pub(crate) fn new(
        construction: ValidatedConstruction,
        program: StyleProgram,
        assignments: HashMap<(TemplateNodeId, PropertyId), usize>,
    ) -> Self {
        Self {
            construction,
            data: Arc::new(StyleData {
                program,
                assignments,
            }),
        }
    }

    /// Returns the exact validated construction retained by this style program.
    #[must_use]
    pub const fn construction(&self) -> &ValidatedConstruction {
        &self.construction
    }

    /// Iterates exact assignments in authored declaration order.
    pub fn assignments(&self) -> StyleAssignmentIter<'_> {
        StyleAssignmentIter {
            style: self,
            next: 0,
        }
    }

    /// Resolves one exact target-property assignment.
    #[must_use]
    pub fn assignment(
        &self,
        target: TemplateNodeId,
        property: PropertyId,
    ) -> Option<StyleAssignmentView<'_>> {
        let index = *self.data.assignments.get(&(target, property))?;
        Some(self.assignment_view(index))
    }

    /// Returns the exact assignment or schema default for a resolved property.
    #[must_use]
    pub fn linked_value(
        &self,
        target: TemplateNodeId,
        property: PropertyId,
    ) -> Option<LinkedStyleValueView<'_>> {
        let target = self.construction.template(target)?;
        let declared = target.component().property(property)?;
        let (value, origin) = self.assignment(target.id(), property).map_or_else(
            || (declared.default(), StyleValueOrigin::SchemaDefault),
            |assignment| (assignment.replacement(), StyleValueOrigin::ExactAssignment),
        );
        Some(LinkedStyleValueView {
            value,
            invalidation: declared.invalidation(),
            origin,
        })
    }

    /// Returns whether another clone shares this exact style validation domain.
    #[must_use]
    pub fn shares_domain_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.data, &other.data)
    }

    fn assignment_view(&self, index: usize) -> StyleAssignmentView<'_> {
        let assignment = &self.data.program.assignments[index];
        let target = self
            .construction
            .template(assignment.target)
            .expect("validated style target must resolve");
        let property = target
            .component()
            .property(assignment.property)
            .expect("validated style property must resolve");
        StyleAssignmentView {
            target,
            property,
            assignment,
        }
    }
}

impl fmt::Debug for ValidatedStyleProgram {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ValidatedStyleProgram(..)")
    }
}

/// Origin of one value resolved from the provisional style program.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StyleValueOrigin {
    /// The value comes from an authored exact assignment.
    ExactAssignment,
    /// The value comes from the linked component schema default.
    SchemaDefault,
}

/// Receiver-scoped view of one validated exact style assignment.
#[derive(Clone, Copy)]
pub struct StyleAssignmentView<'a> {
    target: TemplateFactory<'a>,
    property: PropertySchemaView<'a>,
    assignment: &'a StyleAssignment,
}

impl<'a> StyleAssignmentView<'a> {
    /// Returns the resolved target template factory.
    #[must_use]
    pub const fn target(self) -> TemplateFactory<'a> {
        self.target
    }

    /// Returns the resolved component-local property declaration.
    #[must_use]
    pub const fn property(self) -> PropertySchemaView<'a> {
        self.property
    }

    /// Returns the immutable linked schema default.
    #[must_use]
    pub fn schema_default(self) -> &'a PropertyValue {
        self.property.default()
    }

    /// Returns the immutable authored replacement value.
    #[must_use]
    pub const fn replacement(self) -> &'a PropertyValue {
        &self.assignment.value
    }

    /// Returns the exact-assignment value origin.
    #[must_use]
    pub const fn origin(self) -> StyleValueOrigin {
        StyleValueOrigin::ExactAssignment
    }

    /// Returns the property's validated invalidation declaration.
    #[must_use]
    pub fn invalidation(self) -> InvalidationSet {
        self.property.invalidation()
    }

    /// Returns the authored assignment source anchor.
    #[must_use]
    pub const fn span(self) -> SourceSpan {
        self.assignment.span
    }
}

/// Iterator over validated exact style assignments in authored order.
pub struct StyleAssignmentIter<'a> {
    style: &'a ValidatedStyleProgram,
    next: usize,
}

impl<'a> Iterator for StyleAssignmentIter<'a> {
    type Item = StyleAssignmentView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        (self.next < self.style.data.program.assignments.len()).then(|| {
            let assignment = self.style.assignment_view(self.next);
            self.next += 1;
            assignment
        })
    }
}

/// Immutable value selected by exact style lookup or schema fallback.
#[derive(Clone, Copy)]
pub struct LinkedStyleValueView<'a> {
    value: &'a PropertyValue,
    invalidation: InvalidationSet,
    origin: StyleValueOrigin,
}

impl<'a> LinkedStyleValueView<'a> {
    /// Returns the immutable linked value.
    #[must_use]
    pub const fn value(self) -> &'a PropertyValue {
        self.value
    }

    /// Returns the property's validated invalidation declaration.
    #[must_use]
    pub const fn invalidation(self) -> InvalidationSet {
        self.invalidation
    }

    /// Returns whether the value came from an assignment or schema default.
    #[must_use]
    pub const fn origin(self) -> StyleValueOrigin {
        self.origin
    }
}
