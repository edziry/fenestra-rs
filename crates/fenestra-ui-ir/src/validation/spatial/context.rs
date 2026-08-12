use std::collections::{HashMap, HashSet};

use crate::ids::{
    SpatialBrushSymbolV2, SpatialClipSymbolV2, SpatialImageSymbolV2, SpatialNodeSymbolV2,
    SpatialShapeSymbolV2, StructuralRegionId, TemplateNodeId,
};
use crate::spatial::{SpatialNodeParentV2, SpatialProgramV2};
use crate::validated::{ChildFactory, ValidatedStyleProgram};

pub(super) struct SpatialContext {
    pub(super) node_indexes: HashMap<SpatialNodeSymbolV2, usize>,
    pub(super) template_indexes: HashMap<TemplateNodeId, usize>,
    pub(super) signatures: HashMap<SpatialNodeSymbolV2, Vec<StructuralRegionId>>,
    pub(super) shapes: Vec<HashSet<SpatialShapeSymbolV2>>,
    pub(super) brushes: Vec<HashSet<SpatialBrushSymbolV2>>,
    pub(super) clips: Vec<HashMap<SpatialClipSymbolV2, usize>>,
    pub(super) images: HashSet<SpatialImageSymbolV2>,
}

pub(super) fn build_context(
    style: &ValidatedStyleProgram,
    program: &SpatialProgramV2,
) -> SpatialContext {
    let requested_templates = program
        .nodes()
        .iter()
        .map(|node| *node.template().value())
        .collect();
    let template_signatures = template_signatures(style, &requested_templates);
    let mut node_indexes = HashMap::with_capacity(program.nodes().len());
    let mut template_indexes = HashMap::with_capacity(program.nodes().len());
    let mut signatures = HashMap::with_capacity(program.nodes().len());
    let mut shapes = Vec::with_capacity(program.nodes().len());
    let mut brushes = Vec::with_capacity(program.nodes().len());
    let mut clips = Vec::with_capacity(program.nodes().len());
    for (index, node) in program.nodes().iter().enumerate() {
        let symbol = *node.symbol().value();
        node_indexes.entry(symbol).or_insert(index);
        template_indexes
            .entry(*node.template().value())
            .or_insert(index);
        signatures.entry(symbol).or_insert_with(|| {
            template_signatures
                .get(node.template().value())
                .cloned()
                .unwrap_or_default()
        });
        shapes.push(
            node.shapes()
                .iter()
                .map(|item| *item.symbol().value())
                .collect(),
        );
        brushes.push(
            node.brushes()
                .iter()
                .map(|item| *item.symbol().value())
                .collect(),
        );
        let mut clip_indexes = HashMap::with_capacity(node.clips().len());
        for (clip_index, item) in node.clips().iter().enumerate() {
            clip_indexes
                .entry(*item.symbol().value())
                .or_insert(clip_index);
        }
        clips.push(clip_indexes);
    }
    let images = program
        .images()
        .iter()
        .map(|item| *item.symbol().value())
        .collect();
    SpatialContext {
        node_indexes,
        template_indexes,
        signatures,
        shapes,
        brushes,
        clips,
        images,
    }
}

fn template_signatures(
    style: &ValidatedStyleProgram,
    requested: &HashSet<TemplateNodeId>,
) -> HashMap<TemplateNodeId, Vec<StructuralRegionId>> {
    let mut signatures = HashMap::new();
    if requested.is_empty() {
        return signatures;
    }
    let mut visited = HashSet::new();
    let mut pending = vec![(style.construction().root_factory(), Vec::new())];
    while let Some((template, active)) = pending.pop() {
        if !visited.insert(template.id()) {
            continue;
        }
        if requested.contains(&template.id()) {
            signatures.insert(template.id(), active.clone());
            if signatures.len() == requested.len() {
                break;
            }
        }
        for child in template.children() {
            match child {
                ChildFactory::Static { template, .. } => {
                    pending.push((template, active.clone()));
                }
                ChildFactory::Region { region, .. } => {
                    let mut nested = active.clone();
                    nested.push(region.id());
                    pending.push((region.repeat_body(), nested));
                }
            }
        }
    }
    signatures
}

pub(super) fn signature_prefix(
    target: &[StructuralRegionId],
    source: &[StructuralRegionId],
) -> bool {
    source.starts_with(target)
}

pub(super) fn is_ancestor(
    program: &SpatialProgramV2,
    context: &SpatialContext,
    owner: SpatialNodeSymbolV2,
    candidate: SpatialNodeSymbolV2,
) -> bool {
    let mut current = owner;
    loop {
        if current == candidate {
            return true;
        }
        let Some(node) = context
            .node_indexes
            .get(&current)
            .map(|index| &program.nodes()[*index])
        else {
            return false;
        };
        match node.parent() {
            SpatialNodeParentV2::Viewport => return false,
            SpatialNodeParentV2::Node(parent) => current = *parent.value(),
        }
    }
}
