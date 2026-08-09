macro_rules! u32_symbol {
    ($name:ident, $docs:literal) => {
        #[doc = $docs]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(u32);

        impl $name {
            /// Creates a local symbol. Zero is valid.
            #[must_use]
            pub const fn new(value: u32) -> Self {
                Self(value)
            }

            /// Returns the local numeric symbol.
            #[must_use]
            pub const fn get(self) -> u32 {
                self.0
            }
        }
    };
}

u32_symbol!(
    SchemaFormatVersion,
    "Version of the schema manifest format."
);
u32_symbol!(
    SchemaRevision,
    "Authored revision within a schema namespace."
);
u32_symbol!(
    ConstructionFormatVersion,
    "Version of the construction program format."
);
u32_symbol!(
    ComponentTypeId,
    "Component symbol local to one validated schema."
);
u32_symbol!(PropertyId, "Property symbol local to one component.");
u32_symbol!(
    TemplateNodeId,
    "Template symbol local to one construction program."
);
u32_symbol!(
    StructuralRegionId,
    "Region symbol local to one construction program."
);
u32_symbol!(SourceId, "Opaque source namespace local to one fixture.");

/// Authored namespace of a schema.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SchemaNamespace(u64);

impl SchemaNamespace {
    /// Creates an authored schema namespace. Zero is valid.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the authored numeric namespace.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Schema format understood by this prototype.
pub const SUPPORTED_SCHEMA_FORMAT: SchemaFormatVersion = SchemaFormatVersion::new(1);
/// Construction format understood by this prototype.
pub const SUPPORTED_CONSTRUCTION_FORMAT: ConstructionFormatVersion =
    ConstructionFormatVersion::new(1);
