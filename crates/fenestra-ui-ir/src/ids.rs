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
u32_symbol!(StyleFormatVersion, "Version of the style program format.");
u32_symbol!(
    SpatialFormatVersion,
    "Version of the spatial program format."
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
u32_symbol!(
    SpatialNodeSymbolV2,
    "Node symbol local to one symbolic spatial program."
);
u32_symbol!(
    SpatialShapeSymbolV2,
    "Shape symbol local to one symbolic spatial node."
);
u32_symbol!(
    SpatialBrushSymbolV2,
    "Brush symbol local to one symbolic spatial node."
);
u32_symbol!(
    SpatialClipSymbolV2,
    "Clip symbol local to one symbolic spatial node."
);
u32_symbol!(
    SpatialImageSymbolV2,
    "Image symbol local to one symbolic spatial program."
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
/// Style program format understood by this prototype.
pub const SUPPORTED_STYLE_FORMAT: StyleFormatVersion = StyleFormatVersion::new(1);
/// Symbolic spatial program format understood by this prototype.
pub const SUPPORTED_SPATIAL_FORMAT: SpatialFormatVersion = SpatialFormatVersion::new(2);
