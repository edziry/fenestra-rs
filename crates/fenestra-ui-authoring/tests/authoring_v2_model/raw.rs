use fenestra_ui_ir::prototype::{
    PropertyId, SpatialAnchorComponentV2, SpatialAnchorTargetRecipeV2, SpatialBindingV2,
    SpatialFieldV2, SpatialNodeDeclarationV2, SpatialNodeParentV2, SpatialPlacementRecipeV2,
};

use crate::support;

const S: i64 = 65_536;

#[test]
fn frontends_lower_to_the_same_exact_raw_quadruple() {
    let (fen, ui) = support::compile_both();
    assert_eq!(fen.schema(), ui.schema());
    assert_eq!(fen.construction(), ui.construction());
    assert_eq!(fen.style(), ui.style());
    assert_eq!(fen.spatial(), ui.spatial());

    let spatial = fen.spatial();
    assert_eq!(spatial.format().get(), 2);
    assert_eq!(spatial.schema_namespace().get(), 13_013);
    assert_eq!(spatial.schema_revision().get(), 2);
    assert_eq!(spatial.nodes().len(), 7);
    assert_eq!(spatial.images().len(), 1);

    let viewport = spatial.viewport_container();
    assert_eq!(format!("{:?}", viewport.axis()), "Row");
    assert_eq!(*viewport.left().value(), 4);
    assert_eq!(*viewport.right().value(), 4);
    assert_eq!(*viewport.top().value(), 3);
    assert_eq!(*viewport.bottom().value(), 3);
    assert_eq!(*viewport.gap().value(), 2);

    let image = &spatial.images()[0];
    assert_eq!(image.symbol().value().get(), 0);
    assert_eq!(*image.width().value(), 2);
    assert_eq!(*image.height().value(), 2);
    assert_eq!(*image.stride().value(), 8);
    assert_eq!(
        image.bytes(),
        [255, 0, 0, 255, 0, 128, 0, 128, 0, 0, 64, 64, 0, 0, 0, 0]
    );
}

#[test]
fn dense_symbols_parentage_placements_and_transform_lowering_are_exact() {
    let compiled = support::compile_fen(support::FIXTURE);
    let nodes = compiled.spatial().nodes();
    let symbols = nodes
        .iter()
        .map(|node| node.symbol().value().get())
        .collect::<Vec<_>>();
    let templates = nodes
        .iter()
        .map(|node| node.template().value().get())
        .collect::<Vec<_>>();
    assert_eq!(symbols, [0, 1, 2, 3, 4, 5, 6]);
    assert_eq!(templates, [0, 1, 2, 3, 4, 5, 6]);

    assert!(matches!(nodes[0].parent(), SpatialNodeParentV2::Viewport));
    for (index, parent) in [(1, 0), (2, 1), (3, 2), (4, 2), (5, 0), (6, 5)] {
        let SpatialNodeParentV2::Node(field) = nodes[index].parent() else {
            panic!("node {index} must retain its nested parent");
        };
        assert_eq!(field.value().get(), parent);
    }

    assert_layout(&nodes[0], [lit_i32(0), prop_i32(0), lit_i32(240)]);
    assert_layout(&nodes[1], [lit_i32(0), prop_i32(0), lit_i32(120)]);
    assert_layout(&nodes[4], [lit_i32(0), prop_i32(0), lit_i32(32)]);
    assert_layout(&nodes[5], [lit_i32(0), prop_i32(0), lit_i32(100)]);

    assert_free(
        &nodes[2],
        [
            SpatialAnchorComponentV2::Center,
            SpatialAnchorComponentV2::End,
        ],
        SpatialAnchorTargetRecipeV2::Node(nodes[2].placement_target_field(5)),
        [
            SpatialAnchorComponentV2::Start,
            SpatialAnchorComponentV2::Center,
        ],
        [lit_fixed(-S), prop_fixed(3)],
    );
    assert_free(
        &nodes[3],
        [
            SpatialAnchorComponentV2::Start,
            SpatialAnchorComponentV2::Start,
        ],
        SpatialAnchorTargetRecipeV2::Parent,
        [SpatialAnchorComponentV2::End, SpatialAnchorComponentV2::End],
        [lit_fixed(S), lit_fixed(-S)],
    );
    assert_free(
        &nodes[6],
        [
            SpatialAnchorComponentV2::End,
            SpatialAnchorComponentV2::Center,
        ],
        SpatialAnchorTargetRecipeV2::Viewport,
        [
            SpatialAnchorComponentV2::Start,
            SpatialAnchorComponentV2::Start,
        ],
        [lit_fixed(2 * S), lit_fixed(3 * S)],
    );

    assert_transform(
        &nodes[0],
        [
            lit_fixed(S),
            lit_fixed(0),
            lit_fixed(0),
            lit_fixed(S),
            lit_fixed(0),
            lit_fixed(0),
        ],
        [lit_fixed(0), lit_fixed(0)],
    );
    assert_transform(
        &nodes[1],
        [
            lit_fixed(S),
            lit_fixed(0),
            lit_fixed(0),
            lit_fixed(S),
            prop_fixed(3),
            lit_fixed(S / 2),
        ],
        [lit_fixed(0), lit_fixed(0)],
    );
    assert_transform(
        &nodes[2],
        [
            prop_fixed(3),
            lit_fixed(0),
            lit_fixed(0),
            lit_fixed(S),
            lit_fixed(0),
            lit_fixed(0),
        ],
        [lit_fixed(2 * S), lit_fixed(2 * S)],
    );
    assert_transform(
        &nodes[3],
        [
            lit_fixed(0),
            lit_fixed(S),
            lit_fixed(-S),
            lit_fixed(0),
            lit_fixed(0),
            lit_fixed(0),
        ],
        [lit_fixed(6 * S), lit_fixed(5 * S)],
    );
    assert_transform(
        &nodes[4],
        [
            lit_fixed(S),
            lit_fixed(0),
            lit_fixed(0),
            lit_fixed(S),
            prop_fixed(3),
            lit_fixed(-S / 2),
        ],
        [lit_fixed(0), lit_fixed(0)],
    );
    assert_transform(
        &nodes[5],
        [
            lit_fixed(S),
            lit_fixed(0),
            lit_fixed(0),
            lit_fixed(S),
            lit_fixed(0),
            lit_fixed(0),
        ],
        [lit_fixed(0), lit_fixed(0)],
    );
    assert_transform(
        &nodes[6],
        [
            lit_fixed(S),
            lit_fixed(0),
            lit_fixed(0),
            lit_fixed(S),
            lit_fixed(S / 2),
            lit_fixed(-S / 2),
        ],
        [lit_fixed(0), lit_fixed(0)],
    );
}

fn assert_layout(node: &SpatialNodeDeclarationV2, expected_width: [SpatialBindingV2<i32>; 3]) {
    let SpatialPlacementRecipeV2::Layout(placement) = node.placement() else {
        panic!("expected layout placement");
    };
    let width = placement.width();
    assert_eq!(
        bindings_i32([width.minimum(), width.preferred(), width.maximum()]),
        expected_width
    );
}

fn assert_free(
    node: &SpatialNodeDeclarationV2,
    self_anchor: [SpatialAnchorComponentV2; 2],
    target: SpatialAnchorTargetRecipeV2,
    target_anchor: [SpatialAnchorComponentV2; 2],
    offset: [SpatialBindingV2<i64>; 2],
) {
    let SpatialPlacementRecipeV2::Free(placement) = node.placement() else {
        panic!("expected free placement");
    };
    assert_eq!(*placement.width().value(), prop_i32(0));
    assert_eq!(*placement.height().value(), prop_i32(1));
    assert_eq!(placement.self_anchor(), self_anchor);
    assert_eq!(placement.target(), target);
    assert_eq!(placement.target_anchor(), target_anchor);
    assert_eq!(
        bindings_fixed([placement.offset().x(), placement.offset().y()]),
        offset
    );
}

fn assert_transform(
    node: &SpatialNodeDeclarationV2,
    matrix: [SpatialBindingV2<i64>; 6],
    origin: [SpatialBindingV2<i64>; 2],
) {
    let transform = match node.placement() {
        SpatialPlacementRecipeV2::Layout(value) => value.transform(),
        SpatialPlacementRecipeV2::Free(value) => value.transform(),
    };
    assert_eq!(
        bindings_fixed([
            transform.a(),
            transform.b(),
            transform.c(),
            transform.d(),
            transform.tx(),
            transform.ty()
        ]),
        matrix
    );
    assert_eq!(
        bindings_fixed([transform.origin().x(), transform.origin().y()]),
        origin
    );
}

fn bindings_i32<const N: usize>(
    fields: [SpatialFieldV2<SpatialBindingV2<i32>>; N],
) -> [SpatialBindingV2<i32>; N] {
    fields.map(|field| *field.value())
}

fn bindings_fixed<const N: usize>(
    fields: [SpatialFieldV2<SpatialBindingV2<i64>>; N],
) -> [SpatialBindingV2<i64>; N] {
    fields.map(|field| *field.value())
}

const fn lit_i32(value: i32) -> SpatialBindingV2<i32> {
    SpatialBindingV2::Literal(value)
}
const fn prop_i32(value: u32) -> SpatialBindingV2<i32> {
    SpatialBindingV2::Property(PropertyId::new(value))
}
const fn lit_fixed(value: i64) -> SpatialBindingV2<i64> {
    SpatialBindingV2::Literal(value)
}
const fn prop_fixed(value: u32) -> SpatialBindingV2<i64> {
    SpatialBindingV2::Property(PropertyId::new(value))
}

trait PlacementTargetField {
    fn placement_target_field(
        &self,
        symbol: u32,
    ) -> SpatialFieldV2<fenestra_ui_ir::prototype::SpatialNodeSymbolV2>;
}

impl PlacementTargetField for SpatialNodeDeclarationV2 {
    fn placement_target_field(
        &self,
        symbol: u32,
    ) -> SpatialFieldV2<fenestra_ui_ir::prototype::SpatialNodeSymbolV2> {
        let SpatialPlacementRecipeV2::Free(placement) = self.placement() else {
            panic!("free placement");
        };
        let SpatialAnchorTargetRecipeV2::Node(field) = placement.target() else {
            panic!("node target");
        };
        assert_eq!(field.value().get(), symbol);
        field
    }
}
