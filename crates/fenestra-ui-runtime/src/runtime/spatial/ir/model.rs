use std::collections::HashMap;

use fenestra_ui_ir::prototype::{
    SpatialNodeDeclarationV2, SpatialNodeParentV2, SpatialNodeSymbolV2, StructuralRegionId,
    ValidatedSpatialProgramV2,
};

use crate::logical_tree::NodeId;

use super::super::error::{RuntimeSpatialIrErrorKindV2, RuntimeSpatialIrErrorV2};

pub(super) struct ProgramIndex<'a> {
    program: &'a ValidatedSpatialProgramV2,
    symbols: HashMap<SpatialNodeSymbolV2, usize>,
    parents: Vec<Option<usize>>,
    signatures: Vec<Vec<StructuralRegionId>>,
}

impl<'a> ProgramIndex<'a> {
    pub(super) fn new(
        program: &'a ValidatedSpatialProgramV2,
    ) -> Result<Self, RuntimeSpatialIrErrorV2> {
        let declarations = program.program().nodes();
        let mut symbols = HashMap::with_capacity(declarations.len());
        let mut templates = HashMap::with_capacity(declarations.len());
        for (index, declaration) in declarations.iter().enumerate() {
            if symbols
                .insert(*declaration.symbol().value(), index)
                .is_some()
                || templates
                    .insert(*declaration.template().value(), index)
                    .is_some()
            {
                return Err(invariant(program));
            }
        }

        let mut parents = Vec::with_capacity(declarations.len());
        let mut signatures = Vec::with_capacity(declarations.len());
        for declaration in declarations {
            let parent = match declaration.parent() {
                SpatialNodeParentV2::Viewport => None,
                SpatialNodeParentV2::Node(field) => {
                    let Some(parent) = symbols.get(field.value()).copied() else {
                        return Err(invariant(program));
                    };
                    Some(parent)
                }
            };
            let Some(signature) = program.region_signature(*declaration.symbol().value()) else {
                return Err(invariant(program));
            };
            parents.push(parent);
            signatures.push(signature.to_vec());
        }

        Ok(Self {
            program,
            symbols,
            parents,
            signatures,
        })
    }

    pub(super) fn program(&self) -> &'a ValidatedSpatialProgramV2 {
        self.program
    }

    pub(super) fn declaration(&self, index: usize) -> &'a SpatialNodeDeclarationV2 {
        &self.program.program().nodes()[index]
    }

    pub(super) fn declaration_index(&self, symbol: SpatialNodeSymbolV2) -> Option<usize> {
        self.symbols.get(&symbol).copied()
    }

    pub(super) fn children(&self, parent: Option<usize>) -> impl Iterator<Item = usize> + '_ {
        self.parents
            .iter()
            .enumerate()
            .filter_map(move |(index, candidate)| (*candidate == parent).then_some(index))
    }

    pub(super) fn signature(&self, index: usize) -> &[StructuralRegionId] {
        &self.signatures[index]
    }
}

pub(super) struct ExpandedSpatialNode<'a> {
    logical: NodeId,
    declaration_index: usize,
    declaration: &'a SpatialNodeDeclarationV2,
    context: Vec<u64>,
    ordinal: u128,
    parent_ordinal: u128,
}

impl<'a> ExpandedSpatialNode<'a> {
    pub(super) fn new(
        logical: NodeId,
        declaration_index: usize,
        declaration: &'a SpatialNodeDeclarationV2,
        context: Vec<u64>,
        ordinal: u128,
        parent_ordinal: u128,
    ) -> Self {
        Self {
            logical,
            declaration_index,
            declaration,
            context,
            ordinal,
            parent_ordinal,
        }
    }

    pub(super) const fn logical(&self) -> NodeId {
        self.logical
    }

    pub(super) const fn declaration(&self) -> &'a SpatialNodeDeclarationV2 {
        self.declaration
    }

    pub(super) fn context(&self) -> &[u64] {
        &self.context
    }

    pub(super) fn key(&self) -> u32 {
        u32::try_from(self.ordinal).expect("preflighted spatial node key must fit u32")
    }

    pub(super) fn parent_key(&self) -> u32 {
        u32::try_from(self.parent_ordinal).expect("preflighted spatial parent key must fit u32")
    }

    pub(super) const fn ordinal(&self) -> u128 {
        self.ordinal
    }

    pub(super) const fn parent_ordinal(&self) -> u128 {
        self.parent_ordinal
    }
}

pub(super) struct LiveProgram<'a> {
    index: ProgramIndex<'a>,
    expanded: Vec<ExpandedSpatialNode<'a>>,
}

impl<'a> LiveProgram<'a> {
    pub(super) fn new(index: ProgramIndex<'a>, expanded: Vec<ExpandedSpatialNode<'a>>) -> Self {
        Self { index, expanded }
    }

    pub(super) fn expanded(&self) -> &[ExpandedSpatialNode<'a>] {
        &self.expanded
    }

    pub(super) fn resolve_node(
        &self,
        source_context: &[u64],
        target_symbol: SpatialNodeSymbolV2,
    ) -> Option<&ExpandedSpatialNode<'a>> {
        let declaration = self.index.declaration_index(target_symbol)?;
        let signature_length = self.index.signature(declaration).len();
        let target_context = source_context.get(..signature_length)?;
        self.expanded.iter().find(|node| {
            node.declaration_index == declaration && node.context.as_slice() == target_context
        })
    }

    pub(super) fn logical_mapping(&self) -> Box<[NodeId]> {
        self.expanded
            .iter()
            .map(ExpandedSpatialNode::logical)
            .collect()
    }
}

fn invariant(program: &ValidatedSpatialProgramV2) -> RuntimeSpatialIrErrorV2 {
    RuntimeSpatialIrErrorV2::new(
        RuntimeSpatialIrErrorKindV2::InvariantViolation,
        program.program().span(),
    )
}
