use std::error::Error;
use std::panic::{RefUnwindSafe, UnwindSafe};

use fenestra_ui_layout::prototype::{
    LayoutArithmeticOperationV1, LayoutAxisV1, LayoutConstraintFieldV1, LayoutDimensionV1,
    LayoutEngineErrorKindV1, LayoutEngineErrorV1, LayoutEngineV1, LayoutErrorKindV1,
    LayoutErrorLocationV1, LayoutExtentV1, LayoutInputErrorKindV1, LayoutInputV1,
    LayoutLimitKindV1, LayoutLimitsV1, LayoutNodeKeyV1, LayoutNodeV1, LayoutOutputErrorKindV1,
    LayoutOutputFieldV1, LayoutOutputV1, LayoutPaddingSideV1, LayoutPaddingV1, LayoutRecordV1,
    LayoutRectV1, LayoutStyleV1, LayoutViewportV1, REGISTERED_LAYOUT_LIMITS_V1,
    ReferenceStackEngineV1, ValidatedLayoutInputV1, compute_layout_v1,
};

#[test]
fn values_round_trip_without_candidate_or_runtime_types() {
    let key = LayoutNodeKeyV1::new(7);
    let parent = LayoutNodeKeyV1::new(2);
    let width = LayoutDimensionV1::new(3, 5, 8);
    let height = LayoutDimensionV1::new(4, 6, 9);
    let padding = LayoutPaddingV1::new(1, 2, 3, 4);
    let style = LayoutStyleV1::new(LayoutAxisV1::Row, width, height, padding, 5);
    let node = LayoutNodeV1::new(key, Some(parent), style);
    let viewport = LayoutViewportV1::new(80, 60);
    let nodes = [node];
    let input = LayoutInputV1::new(viewport, &nodes);

    assert_eq!(key.get(), 7);
    assert_eq!(width.minimum(), 3);
    assert_eq!(width.preferred(), 5);
    assert_eq!(width.maximum(), 8);
    assert_eq!(padding.left(), 1);
    assert_eq!(padding.right(), 2);
    assert_eq!(padding.top(), 3);
    assert_eq!(padding.bottom(), 4);
    assert_eq!(style.axis(), LayoutAxisV1::Row);
    assert_eq!(style.width(), width);
    assert_eq!(style.height(), height);
    assert_eq!(style.padding(), padding);
    assert_eq!(style.gap(), 5);
    assert_eq!(node.key(), key);
    assert_eq!(node.parent(), Some(parent));
    assert_eq!(node.style(), style);
    assert_eq!(viewport.width(), 80);
    assert_eq!(viewport.height(), 60);
    assert_eq!(input.viewport(), viewport);
    assert_eq!(input.nodes(), nodes.as_slice());

    let bounds = LayoutRectV1::new(10, 11, 12, 13);
    let record = LayoutRecordV1::new(key, bounds);
    let output = LayoutOutputV1::new(vec![record]);
    assert_eq!(bounds.x(), 10);
    assert_eq!(bounds.y(), 11);
    assert_eq!(bounds.width(), 12);
    assert_eq!(bounds.height(), 13);
    assert_eq!(record.key(), key);
    assert_eq!(record.bounds(), bounds);
    assert_eq!(output.records(), &[record]);

    let _ = ReferenceStackEngineV1::new();
}

#[test]
fn finite_vocabularies_and_registered_limits_are_exact() {
    assert_eq!(LayoutAxisV1::ALL, [LayoutAxisV1::Row, LayoutAxisV1::Column]);
    assert_eq!(
        LayoutExtentV1::ALL,
        [LayoutExtentV1::Width, LayoutExtentV1::Height]
    );
    assert_eq!(
        LayoutConstraintFieldV1::ALL,
        [
            LayoutConstraintFieldV1::Minimum,
            LayoutConstraintFieldV1::Preferred,
            LayoutConstraintFieldV1::Maximum,
        ]
    );
    assert_eq!(
        LayoutPaddingSideV1::ALL,
        [
            LayoutPaddingSideV1::Left,
            LayoutPaddingSideV1::Right,
            LayoutPaddingSideV1::Top,
            LayoutPaddingSideV1::Bottom,
        ]
    );
    assert_eq!(
        LayoutArithmeticOperationV1::ALL,
        [
            LayoutArithmeticOperationV1::FarEdge,
            LayoutArithmeticOperationV1::ContentOrigin,
            LayoutArithmeticOperationV1::GapAdvance,
        ]
    );
    assert_eq!(
        LayoutOutputFieldV1::ALL,
        [
            LayoutOutputFieldV1::X,
            LayoutOutputFieldV1::Y,
            LayoutOutputFieldV1::Width,
            LayoutOutputFieldV1::Height,
        ]
    );
    assert_eq!(
        LayoutLimitKindV1::ALL,
        [
            LayoutLimitKindV1::Nodes,
            LayoutLimitKindV1::Depth,
            LayoutLimitKindV1::ChildrenPerNode,
        ]
    );

    let explicit = LayoutLimitsV1::new(32, 8, 16);
    assert_eq!(REGISTERED_LAYOUT_LIMITS_V1, explicit);
    assert_eq!(
        REGISTERED_LAYOUT_LIMITS_V1.limit(LayoutLimitKindV1::Nodes),
        32
    );
    assert_eq!(
        REGISTERED_LAYOUT_LIMITS_V1.limit(LayoutLimitKindV1::Depth),
        8
    );
    assert_eq!(
        REGISTERED_LAYOUT_LIMITS_V1.limit(LayoutLimitKindV1::ChildrenPerNode),
        16
    );
}

#[test]
fn closed_failure_vocabularies_are_exact() {
    let input_kinds = expected_input_error_kinds();
    assert_eq!(LayoutInputErrorKindV1::ALL.as_slice(), input_kinds);

    let engine_kinds = expected_engine_error_kinds();
    assert_eq!(LayoutEngineErrorKindV1::ALL.as_slice(), engine_kinds);

    let output_kinds = expected_output_error_kinds();
    assert_eq!(LayoutOutputErrorKindV1::ALL.as_slice(), output_kinds);

    let mut all = Vec::new();
    all.extend(input_kinds.iter().copied().map(LayoutErrorKindV1::Input));
    all.extend(engine_kinds.iter().copied().map(LayoutErrorKindV1::Engine));
    all.extend(output_kinds.iter().copied().map(LayoutErrorKindV1::Output));
    assert_eq!(LayoutErrorKindV1::ALL.as_slice(), all);
}

#[test]
fn errors_are_typed_and_do_not_render_input_values() {
    let nodes = [root_node()];
    let input = LayoutInputV1::new(LayoutViewportV1::new(-713, 29), &nodes);
    let error = compute_layout_v1(&NeverEngine, input, REGISTERED_LAYOUT_LIMITS_V1)
        .expect_err("negative viewport width should fail before engine invocation");

    assert_eq!(
        error.kind(),
        LayoutErrorKindV1::Input(LayoutInputErrorKindV1::NegativeViewport(
            LayoutExtentV1::Width,
        ))
    );
    assert_eq!(error.location(), LayoutErrorLocationV1::Viewport);
    let _: &dyn Error = &error;
    let rendered = format!("{error:?} {error}");
    for forbidden in ["-713", "LayoutInputV1", "LayoutNodeV1"] {
        assert!(
            !rendered.contains(forbidden),
            "leaked {forbidden}: {rendered}"
        );
    }

    let engine = LayoutEngineErrorV1::new(
        LayoutEngineErrorKindV1::UnrepresentableOutput,
        LayoutErrorLocationV1::OutputRecord { index: 4 },
    );
    assert_eq!(
        engine.kind(),
        LayoutEngineErrorKindV1::UnrepresentableOutput
    );
    assert_eq!(
        engine.location(),
        LayoutErrorLocationV1::OutputRecord { index: 4 }
    );
    let _: &dyn Error = &engine;
}

#[test]
fn engines_preserve_the_runtime_auto_trait_set() {
    assert_auto_traits::<ReferenceStackEngineV1>();
    assert_auto_traits::<Box<dyn LayoutEngineV1>>();
}

struct NeverEngine;

impl LayoutEngineV1 for NeverEngine {
    fn compute(
        &self,
        input: ValidatedLayoutInputV1<'_>,
    ) -> Result<LayoutOutputV1, LayoutEngineErrorV1> {
        let _ = input.viewport();
        let _ = input.nodes();
        panic!("invalid input reached the engine")
    }
}

fn assert_auto_traits<T>()
where
    T: Send + Sync + Unpin + UnwindSafe + RefUnwindSafe + 'static,
{
}

fn root_node() -> LayoutNodeV1 {
    LayoutNodeV1::new(
        LayoutNodeKeyV1::new(0),
        None,
        LayoutStyleV1::new(
            LayoutAxisV1::Column,
            LayoutDimensionV1::new(0, 10, 10),
            LayoutDimensionV1::new(0, 10, 10),
            LayoutPaddingV1::new(0, 0, 0, 0),
            0,
        ),
    )
}

fn expected_input_error_kinds() -> &'static [LayoutInputErrorKindV1] {
    use LayoutConstraintFieldV1::{Maximum, Minimum, Preferred};
    use LayoutExtentV1::{Height, Width};
    use LayoutInputErrorKindV1::{
        EmptyInput, ForwardParent, InvalidPreorder, InvalidRootKey, InvertedConstraint,
        LimitExceeded, MissingParent, NegativeConstraint, NegativeGap, NegativePadding,
        NegativeViewport, NonDenseKey, PaddingExceedsExtent, RootHasParent,
    };

    &[
        LimitExceeded(LayoutLimitKindV1::Nodes),
        EmptyInput,
        InvalidRootKey,
        RootHasParent,
        NonDenseKey,
        MissingParent,
        ForwardParent,
        InvalidPreorder,
        LimitExceeded(LayoutLimitKindV1::Depth),
        LimitExceeded(LayoutLimitKindV1::ChildrenPerNode),
        NegativeViewport(Width),
        NegativeViewport(Height),
        NegativeConstraint {
            extent: Width,
            field: Minimum,
        },
        NegativeConstraint {
            extent: Width,
            field: Preferred,
        },
        NegativeConstraint {
            extent: Width,
            field: Maximum,
        },
        InvertedConstraint(Width),
        NegativeConstraint {
            extent: Height,
            field: Minimum,
        },
        NegativeConstraint {
            extent: Height,
            field: Preferred,
        },
        NegativeConstraint {
            extent: Height,
            field: Maximum,
        },
        InvertedConstraint(Height),
        NegativePadding(LayoutPaddingSideV1::Left),
        NegativePadding(LayoutPaddingSideV1::Right),
        NegativePadding(LayoutPaddingSideV1::Top),
        NegativePadding(LayoutPaddingSideV1::Bottom),
        PaddingExceedsExtent(Width),
        PaddingExceedsExtent(Height),
        NegativeGap,
    ]
}

fn expected_engine_error_kinds() -> &'static [LayoutEngineErrorKindV1] {
    use LayoutArithmeticOperationV1::{ContentOrigin, FarEdge, GapAdvance};
    use LayoutEngineErrorKindV1::{
        ArithmeticExhausted, InvariantViolation, RejectedInput, UnrepresentableOutput,
    };
    use LayoutExtentV1::{Height, Width};

    &[
        ArithmeticExhausted {
            operation: FarEdge,
            extent: Width,
        },
        ArithmeticExhausted {
            operation: FarEdge,
            extent: Height,
        },
        ArithmeticExhausted {
            operation: ContentOrigin,
            extent: Width,
        },
        ArithmeticExhausted {
            operation: ContentOrigin,
            extent: Height,
        },
        ArithmeticExhausted {
            operation: GapAdvance,
            extent: Width,
        },
        ArithmeticExhausted {
            operation: GapAdvance,
            extent: Height,
        },
        RejectedInput,
        UnrepresentableOutput,
        InvariantViolation,
    ]
}

fn expected_output_error_kinds() -> &'static [LayoutOutputErrorKindV1] {
    use LayoutExtentV1::{Height, Width};
    use LayoutOutputErrorKindV1::{FarEdgeArithmetic, KeyMismatch, Negative, RecordCountMismatch};

    &[
        RecordCountMismatch,
        KeyMismatch,
        Negative(LayoutOutputFieldV1::X),
        Negative(LayoutOutputFieldV1::Y),
        Negative(LayoutOutputFieldV1::Width),
        Negative(LayoutOutputFieldV1::Height),
        FarEdgeArithmetic(Width),
        FarEdgeArithmetic(Height),
    ]
}
