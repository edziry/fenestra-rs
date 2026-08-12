use std::fmt::Write;

use fenestra_ui_ir::prototype::{
    SpatialAnchorComponentV2, SpatialAnchorTargetRecipeV2, SpatialAxisV2, SpatialNodeDeclarationV2,
    SpatialNodeParentV2, SpatialPlacementRecipeV2, SpatialProgramV2,
};

use crate::resolved_v2::ResolvedDocumentV2;
use crate::semantic::{InvalidRecord, Record};
use crate::vocabulary_v2::AnchorKindV2;

use super::catalog::SourceCatalog;
use super::{content, field};

pub(super) fn collect(
    resolved: &ResolvedDocumentV2,
    spatial: &SpatialProgramV2,
    records: &mut Vec<Record>,
    catalog: &SourceCatalog<'_>,
) -> Result<(), InvalidRecord> {
    let spatial_anchor = catalog.anchor(spatial.span(), AnchorKindV2::Spatial)?;
    let resources_anchor = catalog.nth_anchor(AnchorKindV2::Resources, 0)?;
    if spatial_anchor != resolved.spatial_anchor || resources_anchor != resolved.resources_anchor {
        return Err(InvalidRecord);
    }
    records.push(Record::new(
        spatial_anchor,
        "spatial",
        format!(
            "format={}|namespace={}|revision={}|nodes={}|images={}",
            spatial.format().get(),
            spatial.schema_namespace().get(),
            spatial.schema_revision().get(),
            spatial.nodes().len(),
            spatial.images().len(),
        ),
    )?);
    collect_viewport(spatial, records, catalog)?;
    records.push(Record::new(
        resources_anchor,
        "resources",
        format!("images={}", spatial.images().len()),
    )?);
    collect_images(spatial, records, catalog)?;
    for (order, node) in spatial.nodes().iter().enumerate() {
        collect_node(node, order, records, catalog)?;
    }
    Ok(())
}

fn collect_viewport(
    spatial: &SpatialProgramV2,
    records: &mut Vec<Record>,
    catalog: &SourceCatalog<'_>,
) -> Result<(), InvalidRecord> {
    let viewport = spatial.viewport_container();
    let anchor = catalog.anchor(viewport.span(), AnchorKindV2::SpatialContainer)?;
    records.push(Record::new(
        anchor,
        "spatial-container",
        format!("owner=viewport|axis={}", axis(viewport.axis())),
    )?);
    field::push(records, catalog, anchor, "left", viewport.left())?;
    field::push(records, catalog, anchor, "right", viewport.right())?;
    field::push(records, catalog, anchor, "top", viewport.top())?;
    field::push(records, catalog, anchor, "bottom", viewport.bottom())?;
    field::push(records, catalog, anchor, "gap", viewport.gap())
}

fn collect_images(
    spatial: &SpatialProgramV2,
    records: &mut Vec<Record>,
    catalog: &SourceCatalog<'_>,
) -> Result<(), InvalidRecord> {
    for (order, image) in spatial.images().iter().enumerate() {
        let (anchor, name) = catalog.named_anchor(image.span(), AnchorKindV2::Image)?;
        if image.symbol().value().get() != u32::try_from(order).map_err(|_| InvalidRecord)? {
            return Err(InvalidRecord);
        }
        let mut bytes = String::new();
        write!(&mut bytes, "hex:{}:", image.bytes().len()).map_err(|_| InvalidRecord)?;
        for byte in image.bytes() {
            write!(&mut bytes, "{byte:02x}").map_err(|_| InvalidRecord)?;
        }
        records.push(Record::new(
            anchor,
            "image",
            format!("order={order}|name={name}|bytes={bytes}"),
        )?);
        field::push(records, catalog, anchor, "symbol", image.symbol())?;
        field::push(records, catalog, anchor, "width", image.width())?;
        field::push(records, catalog, anchor, "height", image.height())?;
        field::push(records, catalog, anchor, "stride", image.stride())?;
    }
    Ok(())
}

fn collect_node(
    node: &SpatialNodeDeclarationV2,
    order: usize,
    records: &mut Vec<Record>,
    catalog: &SourceCatalog<'_>,
) -> Result<(), InvalidRecord> {
    let (anchor, name) = catalog.named_anchor(node.span(), AnchorKindV2::SpatialNode)?;
    let symbol = node.symbol().value().get();
    if symbol != u32::try_from(order).map_err(|_| InvalidRecord)? {
        return Err(InvalidRecord);
    }
    let parent = match node.parent() {
        SpatialNodeParentV2::Viewport => "viewport",
        SpatialNodeParentV2::Node(_) => "node",
    };
    records.push(Record::new(
        anchor,
        "spatial-node",
        format!("order={order}|name={name}|parent={parent}"),
    )?);
    field::push(records, catalog, anchor, "symbol", node.symbol())?;
    field::push(records, catalog, anchor, "template", node.template())?;
    if let SpatialNodeParentV2::Node(parent) = node.parent() {
        field::push(records, catalog, anchor, "parent", parent)?;
    }
    collect_container(node, order, records, catalog)?;
    collect_placement(node, order, records, catalog)?;
    content::collect(node, records, catalog)
}

fn collect_container(
    node: &SpatialNodeDeclarationV2,
    node_order: usize,
    records: &mut Vec<Record>,
    catalog: &SourceCatalog<'_>,
) -> Result<(), InvalidRecord> {
    let anchor = catalog.nth_anchor(AnchorKindV2::SpatialContainer, node_order + 1)?;
    let node_symbol = node.symbol().value().get();
    let container = node.container();
    records.push(Record::new(
        anchor,
        "spatial-container",
        format!("owner=node:{node_symbol}|axis={}", axis(container.axis())),
    )?);
    let padding = container.padding();
    field::push(records, catalog, anchor, "left", padding.left())?;
    field::push(records, catalog, anchor, "right", padding.right())?;
    field::push(records, catalog, anchor, "top", padding.top())?;
    field::push(records, catalog, anchor, "bottom", padding.bottom())?;
    field::push(records, catalog, anchor, "gap", container.gap())
}

fn collect_placement(
    node: &SpatialNodeDeclarationV2,
    node_order: usize,
    records: &mut Vec<Record>,
    catalog: &SourceCatalog<'_>,
) -> Result<(), InvalidRecord> {
    let placement_anchor = catalog.nth_anchor(AnchorKindV2::SpatialPlacement, node_order)?;
    let transform_anchor = catalog.nth_anchor(AnchorKindV2::SpatialTransform, node_order)?;
    let symbol = node.symbol().value().get();
    let transform = match node.placement() {
        SpatialPlacementRecipeV2::Layout(layout) => {
            records.push(Record::new(
                placement_anchor,
                "spatial-placement",
                format!("node={symbol}|kind=layout"),
            )?);
            let width = layout.width();
            let height = layout.height();
            field::push(
                records,
                catalog,
                placement_anchor,
                "width-minimum",
                width.minimum(),
            )?;
            field::push(
                records,
                catalog,
                placement_anchor,
                "width-preferred",
                width.preferred(),
            )?;
            field::push(
                records,
                catalog,
                placement_anchor,
                "width-maximum",
                width.maximum(),
            )?;
            field::push(
                records,
                catalog,
                placement_anchor,
                "height-minimum",
                height.minimum(),
            )?;
            field::push(
                records,
                catalog,
                placement_anchor,
                "height-preferred",
                height.preferred(),
            )?;
            field::push(
                records,
                catalog,
                placement_anchor,
                "height-maximum",
                height.maximum(),
            )?;
            layout.transform()
        }
        SpatialPlacementRecipeV2::Free(free) => {
            let [self_x, self_y] = free.self_anchor();
            let [target_x, target_y] = free.target_anchor();
            let target = match free.target() {
                SpatialAnchorTargetRecipeV2::Viewport => "viewport",
                SpatialAnchorTargetRecipeV2::Parent => "parent",
                SpatialAnchorTargetRecipeV2::Node(_) => "node",
            };
            records.push(Record::new(
                placement_anchor,
                "spatial-placement",
                format!(
                    "node={symbol}|kind=free|self-anchor={},{}|target={target}|target-anchor={},{}",
                    anchor_component(self_x),
                    anchor_component(self_y),
                    anchor_component(target_x),
                    anchor_component(target_y),
                ),
            )?);
            field::push(records, catalog, placement_anchor, "width", free.width())?;
            field::push(records, catalog, placement_anchor, "height", free.height())?;
            if let SpatialAnchorTargetRecipeV2::Node(target) = free.target() {
                field::push(records, catalog, placement_anchor, "target", target)?;
            }
            field::point(
                records,
                catalog,
                placement_anchor,
                "offset-x",
                "offset-y",
                free.offset(),
            )?;
            free.transform()
        }
    };
    records.push(Record::new(
        transform_anchor,
        "spatial-transform",
        format!("node={symbol}"),
    )?);
    field::transform(records, catalog, transform_anchor, transform)
}

const fn axis(axis: SpatialAxisV2) -> &'static str {
    match axis {
        SpatialAxisV2::Row => "row",
        SpatialAxisV2::Column => "column",
    }
}

const fn anchor_component(anchor: SpatialAnchorComponentV2) -> &'static str {
    match anchor {
        SpatialAnchorComponentV2::Start => "start",
        SpatialAnchorComponentV2::Center => "center",
        SpatialAnchorComponentV2::End => "end",
    }
}
