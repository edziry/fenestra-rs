macro_rules! accessors {
    ($(($getter:ident, $with:ident, $field:ident, $docs:literal)),+ $(,)?) => {
        $(
            pub(crate) const fn $getter(self) -> usize {
                self.$field
            }

            #[doc = $docs]
            #[must_use]
            pub const fn $with(mut self, value: usize) -> Self {
                self.$field = value;
                self
            }
        )+
    };
}

/// Inclusive resource limits applied while validating provisional IR.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidationLimits {
    components: usize,
    properties: usize,
    templates: usize,
    regions: usize,
    child_slots: usize,
    initial_properties: usize,
    initial_keys: usize,
    template_depth: usize,
    initial_instances: usize,
}

impl ValidationLimits {
    /// Creates a complete set of explicit inclusive limits.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        components: usize,
        properties: usize,
        templates: usize,
        regions: usize,
        child_slots: usize,
        initial_properties: usize,
        initial_keys: usize,
        template_depth: usize,
        initial_instances: usize,
    ) -> Self {
        Self {
            components,
            properties,
            templates,
            regions,
            child_slots,
            initial_properties,
            initial_keys,
            template_depth,
            initial_instances,
        }
    }

    accessors!(
        (
            components,
            with_components,
            components,
            "Returns limits with a new component bound."
        ),
        (
            properties,
            with_properties,
            properties,
            "Returns limits with a new property bound."
        ),
        (
            templates,
            with_templates,
            templates,
            "Returns limits with a new template bound."
        ),
        (
            regions,
            with_regions,
            regions,
            "Returns limits with a new region bound."
        ),
        (
            child_slots,
            with_child_slots,
            child_slots,
            "Returns limits with a new child-slot bound."
        ),
        (
            initial_properties,
            with_initial_properties,
            initial_properties,
            "Returns limits with a new initial-property bound."
        ),
        (
            initial_keys,
            with_initial_keys,
            initial_keys,
            "Returns limits with a new initial-key bound."
        ),
        (
            template_depth,
            with_template_depth,
            template_depth,
            "Returns limits with a new template-depth bound."
        ),
        (
            initial_instances,
            with_initial_instances,
            initial_instances,
            "Returns limits with a new expanded-instance bound."
        ),
    );
}
