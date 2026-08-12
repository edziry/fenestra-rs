use crate::baseline::{CaseKindV2, CorpusObligationV2, QuerySetV2, registered_corpus_v2};

use super::support::expected::{CASE_NAMES, CASES, OBSERVATION_COUNTS};

#[test]
fn corpus_has_the_exact_registered_order_and_typed_values() {
    let corpus = registered_corpus_v2();
    assert_eq!(corpus.len(), CASE_NAMES.len());
    for (ordinal, ((case, expected), name)) in
        corpus.iter().zip(CASES.iter()).zip(CASE_NAMES).enumerate()
    {
        assert_eq!(usize::from(case.ordinal), ordinal);
        assert_eq!(case.name, name);
        assert_eq!(case.kind, expected.kind);
        assert_eq!(
            case.node_keys,
            (0..case.node_keys.len() as u32).collect::<Vec<_>>()
        );
        assert_eq!(case.placements, expected.placements);
        assert_eq!(case.operations, expected.operations);
        assert_eq!(case.obligations, expected.obligations);
        assert_eq!(case.typed_scalars, expected.scalars);
        assert_eq!(case.query_set, expected.query_set);
        assert_eq!(case.observation_count, OBSERVATION_COUNTS[ordinal]);
        assert_ne!(case.authored_order_digest, 0);
        assert_ne!(case.table_digest, 0);
    }
}

#[test]
fn direct_case_obligations_are_proved_by_the_typed_registry() {
    let corpus = registered_corpus_v2();
    for ordinal in [0, 1] {
        assert!(corpus[ordinal].node_keys.len() >= 4);
        assert!(corpus[ordinal].nested_depth >= 3);
        assert!(corpus[ordinal].has(CorpusObligationV2::Resize));
    }
    assert!(corpus[2].has(CorpusObligationV2::OuterFreeInnerLayout));
    assert!(corpus[3].has(CorpusObligationV2::OuterLayoutInnerFree));
    assert!(corpus[4].has(CorpusObligationV2::FreeConsumesNoLayout));
    assert!(corpus[5].has(CorpusObligationV2::TransparentNoPaintHit));
    for obligation in [
        CorpusObligationV2::SplitLayoutExtent,
        CorpusObligationV2::PaintOverflow,
        CorpusObligationV2::CircularHit,
        CorpusObligationV2::IndependentSemantics,
    ] {
        assert!(corpus[6].has(obligation));
    }
    assert!(corpus[7].has(CorpusObligationV2::ThreeLevelTransforms));
    assert!(corpus[7].has(CorpusObligationV2::TwoLinkClip));
    assert_eq!(corpus[8].obligations.len(), 10);
    assert_eq!(corpus[9].obligations.len(), 6);
    assert!(corpus[10].has(CorpusObligationV2::DependencyCycleControl));
    assert_eq!(corpus[11].obligations.len(), 3);
}

#[test]
fn runtime_cases_copy_the_fixed_script_without_importing_authoring_support() {
    let corpus = registered_corpus_v2();
    assert_eq!(corpus[12].kind, CaseKindV2::RuntimeMutation);
    assert_eq!(corpus[12].observation_count, 9);
    assert_eq!(corpus[12].operations.len(), 9);
    assert_eq!(corpus[12].initial_viewport, (192, 128));
    assert_eq!(corpus[12].final_viewport, (224, 160));
    assert_eq!(corpus[13].kind, CaseKindV2::RuntimeRollback);
    assert_eq!(corpus[13].observation_count, 1);
    assert!(corpus[13].has(CorpusObligationV2::ExactRetainedState));
}

#[test]
fn query_registries_are_complete_ordered_and_duplicate_preserving() {
    let corpus = registered_corpus_v2();
    for case in &corpus[..12] {
        assert_eq!(case.query_set, QuerySetV2::DirectComplete);
        assert_eq!(case.query_inventory.viewport_outside, 4);
        assert!(case.query_inventory.duplicates_retained);
        assert!(case.query_inventory.authored_boundaries > 0);
        assert_eq!(case.query_inventory.logical_pixel_centers, 0);
    }
    for case in &corpus[12..] {
        assert_eq!(case.query_set, QuerySetV2::RuntimePixelCenters);
        assert_eq!(case.query_inventory.viewport_outside, 4);
        assert!(case.query_inventory.logical_pixel_centers > 0);
        assert!(case.query_inventory.duplicates_retained);
    }
}
