use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::ids::{SpatialNodeSymbolV2, StructuralRegionId, TemplateNodeId};
use crate::spatial::{SpatialNodeDeclarationV2, SpatialProgramV2};

use super::ValidatedStyleProgram;

struct SpatialValidationData {
    program: SpatialProgramV2,
    nodes: HashMap<SpatialNodeSymbolV2, usize>,
    templates: HashMap<TemplateNodeId, usize>,
    signatures: HashMap<SpatialNodeSymbolV2, Vec<StructuralRegionId>>,
}

/// Immutable symbolic spatial program linked to one exact style domain.
#[derive(Clone)]
pub struct ValidatedSpatialProgramV2 {
    style: ValidatedStyleProgram,
    data: Arc<SpatialValidationData>,
}

impl ValidatedSpatialProgramV2 {
    pub(crate) fn new(
        style: ValidatedStyleProgram,
        program: SpatialProgramV2,
        nodes: HashMap<SpatialNodeSymbolV2, usize>,
        templates: HashMap<TemplateNodeId, usize>,
        signatures: HashMap<SpatialNodeSymbolV2, Vec<StructuralRegionId>>,
    ) -> Self {
        Self {
            style,
            data: Arc::new(SpatialValidationData {
                program,
                nodes,
                templates,
                signatures,
            }),
        }
    }

    /// Returns the exact retained symbolic program.
    #[must_use]
    pub fn program(&self) -> &SpatialProgramV2 {
        &self.data.program
    }

    /// Returns the exact retained validated style domain.
    #[must_use]
    pub fn style(&self) -> &ValidatedStyleProgram {
        &self.style
    }

    /// Resolves a spatial node symbol within this validation domain.
    #[must_use]
    pub fn node(&self, symbol: SpatialNodeSymbolV2) -> Option<&SpatialNodeDeclarationV2> {
        self.data
            .nodes
            .get(&symbol)
            .map(|index| &self.data.program.nodes()[*index])
    }

    /// Resolves the spatial declaration for one construction template.
    #[must_use]
    pub fn node_for_template(&self, template: TemplateNodeId) -> Option<&SpatialNodeDeclarationV2> {
        self.data
            .templates
            .get(&template)
            .map(|index| &self.data.program.nodes()[*index])
    }

    /// Returns the construction repeat-region signature for one spatial node.
    #[must_use]
    pub fn region_signature(&self, symbol: SpatialNodeSymbolV2) -> Option<&[StructuralRegionId]> {
        self.data.signatures.get(&symbol).map(Vec::as_slice)
    }

    /// Returns whether another clone shares this exact validation domain.
    #[must_use]
    pub fn shares_domain_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.data, &other.data)
    }
}

impl fmt::Debug for ValidatedSpatialProgramV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ValidatedSpatialProgramV2(..)")
    }
}
