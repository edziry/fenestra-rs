mod geometry;
mod items;
mod layout;
mod value;

use std::cell::Cell;
use std::collections::BTreeMap;

use fenestra_ui_ir::prototype::{
    SchemaNamespace, SchemaRevision, SpatialFieldV2, SpatialFormatVersion, SpatialImageSymbolV2,
    SpatialNodeDeclarationV2, SpatialNodeParentV2, SpatialNodeSymbolV2, SpatialProgramV2,
    TemplateNodeId,
};

use crate::diagnostic_v2::{AuthoringDiagnosticKindV2, AuthoringDiagnosticV2};
use crate::limits_v2::AuthoringLimitKindV2;
use crate::parsed_v2::{ParsedDocumentV2, ParsedNodeV2};
use crate::resolved::{ResolvedDocumentV1, ResolvedTemplateV1, logical_span};

pub(crate) fn resolve_spatial(
    parsed: &ParsedDocumentV2,
    core: &ResolvedDocumentV1,
) -> Result<SpatialProgramV2, AuthoringDiagnosticV2> {
    SpatialLowerer::new(parsed, core).lower()
}

pub(super) struct SpatialLowerer<'a> {
    pub(super) parsed: &'a ParsedDocumentV2,
    pub(super) core: &'a ResolvedDocumentV1,
    nodes: Vec<FlatNode<'a>>,
    pub(super) node_symbols: SymbolSet,
    pub(super) image_symbols: SymbolSet,
    pub(super) owner_symbols: Vec<OwnerSymbols>,
    pub(super) emitted_fields: Cell<usize>,
}

#[derive(Clone, Copy)]
struct FlatNode<'a> {
    node: &'a ParsedNodeV2,
    parent: Option<usize>,
}

pub(super) struct OwnerSymbols {
    pub(super) shapes: SymbolSet,
    pub(super) brushes: SymbolSet,
    pub(super) clips: SymbolSet,
}

pub(super) struct SymbolSet {
    by_name: BTreeMap<Box<str>, usize>,
    duplicates: Vec<bool>,
}

impl SymbolSet {
    fn new<'a>(names: impl IntoIterator<Item = &'a str>) -> Self {
        let mut by_name = BTreeMap::new();
        let mut duplicates = Vec::new();
        for (index, name) in names.into_iter().enumerate() {
            duplicates.push(by_name.contains_key(name));
            by_name.entry(Box::<str>::from(name)).or_insert(index);
        }
        Self {
            by_name,
            duplicates,
        }
    }

    pub(super) fn get(&self, name: &str) -> Option<usize> {
        self.by_name.get(name).copied()
    }

    pub(super) fn is_duplicate(&self, index: usize) -> bool {
        self.duplicates[index]
    }
}

impl OwnerSymbols {
    fn new(node: &ParsedNodeV2) -> Self {
        Self {
            shapes: SymbolSet::new(node.shapes.iter().map(|shape| shape.name.as_ref())),
            brushes: SymbolSet::new(node.brushes.iter().map(|brush| brush.name.as_ref())),
            clips: SymbolSet::new(node.clips.iter().map(|clip| clip.name.as_ref())),
        }
    }
}

impl<'a> SpatialLowerer<'a> {
    fn new(parsed: &'a ParsedDocumentV2, core: &'a ResolvedDocumentV1) -> Self {
        let mut nodes = Vec::new();
        flatten_nodes(&parsed.spatial.nodes, None, &mut nodes);
        let node_symbols = SymbolSet::new(nodes.iter().map(|entry| entry.node.name.as_ref()));
        let image_symbols = SymbolSet::new(
            parsed
                .spatial
                .images
                .iter()
                .map(|image| image.name.as_ref()),
        );
        let owner_symbols = nodes
            .iter()
            .map(|entry| OwnerSymbols::new(entry.node))
            .collect();
        Self {
            parsed,
            core,
            nodes,
            node_symbols,
            image_symbols,
            owner_symbols,
            emitted_fields: Cell::new(0),
        }
    }

    fn lower(&self) -> Result<SpatialProgramV2, AuthoringDiagnosticV2> {
        let viewport = self.lower_viewport()?;
        let images = self.lower_images()?;
        if let Some(index) =
            (0..self.nodes.len()).find(|index| self.node_symbols.is_duplicate(*index))
        {
            return Err(self.error(
                AuthoringDiagnosticKindV2::DuplicateSpatialNodeName,
                self.nodes[index].node.symbol.anchor,
            ));
        }
        let nodes = (0..self.nodes.len())
            .map(|index| self.lower_node(index))
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(
            self.emitted_fields.get(),
            self.parsed.spatial.field_count,
            "parsed and lowered spatial field counts must agree"
        );
        Ok(SpatialProgramV2::new(
            SpatialFormatVersion::new(self.parsed.spatial.format),
            SchemaNamespace::new(self.core.schema.namespace),
            SchemaRevision::new(self.core.schema.revision),
            viewport,
            nodes,
            images,
            logical_span(self.parsed.spatial.anchor),
        ))
    }

    fn lower_node(&self, index: usize) -> Result<SpatialNodeDeclarationV2, AuthoringDiagnosticV2> {
        let flat = self.nodes[index];
        let node = flat.node;
        if self.node_symbols.is_duplicate(index) {
            return Err(self.error(
                AuthoringDiagnosticKindV2::DuplicateSpatialNodeName,
                node.symbol.anchor,
            ));
        }
        let symbol = self.node_symbol_field(&node.symbol, index)?;
        let template = self.template(node)?;
        let template_field =
            self.field_value(node.template.anchor, TemplateNodeId::new(template.id));
        let parent = match flat.parent {
            None => SpatialNodeParentV2::Viewport,
            Some(parent) => {
                let field = node
                    .parent
                    .as_ref()
                    .expect("nested parsed nodes must retain a parent field");
                SpatialNodeParentV2::Node(self.node_symbol_field(field, parent)?)
            }
        };

        let component = template.component;
        let container = self.lower_container(node, component)?;
        let placement = self.lower_placement(node, component)?;
        let shapes = self.lower_shapes(index, component)?;
        let brushes = self.lower_brushes(index, component)?;
        let clips = self.lower_clips(index)?;
        let paint = self.lower_paint_items(index, component)?;
        let hit = self.lower_hit_items(index, component)?;
        let semantics = self.lower_semantic_items(index)?;
        Ok(SpatialNodeDeclarationV2::new(
            symbol,
            template_field,
            parent,
            placement,
            container,
            shapes,
            brushes,
            clips,
            paint,
            hit,
            semantics,
            logical_span(node.anchor),
        ))
    }

    fn template(&self, node: &ParsedNodeV2) -> Result<&ResolvedTemplateV1, AuthoringDiagnosticV2> {
        self.core
            .construction
            .templates
            .iter()
            .find(|template| template.name.as_ref() == node.template.value.as_ref())
            .ok_or_else(|| {
                self.error(
                    AuthoringDiagnosticKindV2::UnknownTemplateName,
                    node.template.anchor,
                )
            })
    }

    pub(super) fn node_symbol_field(
        &self,
        field: &crate::parsed_v2::ParsedNameFieldV2,
        index: usize,
    ) -> Result<SpatialFieldV2<SpatialNodeSymbolV2>, AuthoringDiagnosticV2> {
        let value = self.dense_symbol(index, AuthoringLimitKindV2::SpatialNodes, field.anchor)?;
        Ok(self.field_value(field.anchor, SpatialNodeSymbolV2::new(value)))
    }

    pub(super) fn image_symbol_field(
        &self,
        field: &crate::parsed_v2::ParsedNameFieldV2,
        index: usize,
    ) -> Result<SpatialFieldV2<SpatialImageSymbolV2>, AuthoringDiagnosticV2> {
        let value = self.dense_symbol(index, AuthoringLimitKindV2::Images, field.anchor)?;
        Ok(self.field_value(field.anchor, SpatialImageSymbolV2::new(value)))
    }
}

fn flatten_nodes<'a>(
    nodes: &'a [ParsedNodeV2],
    parent: Option<usize>,
    output: &mut Vec<FlatNode<'a>>,
) {
    for node in nodes {
        let index = output.len();
        output.push(FlatNode { node, parent });
        flatten_nodes(&node.children, Some(index), output);
    }
}
