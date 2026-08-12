use std::sync::Arc;

use fenestra_ui_layout::prototype::ReferenceStackEngineV1;
use fenestra_ui_spatial::prototype::{
    SpatialLimitKindV2, SpatialResolveErrorV2, resolve_spatial_v2,
};

use super::behavior::expect_limit;
use super::preflight_spatial_direct_counts_v2;
use super::support::{DIRECT_COUNT, limits_with_direct, owned_input};

#[test]
fn raw_resolver_phase_one_matches_the_shared_helper_for_every_direct_table() {
    for (index, kind) in SpatialLimitKindV2::DIRECT_ALL.into_iter().enumerate() {
        let mut counts = [0; DIRECT_COUNT];
        counts[index] = 2;
        let mut maxima = [usize::MAX; DIRECT_COUNT];
        maxima[index] = 1;
        let limits = limits_with_direct(maxima);
        let observed = counts.map(|count| count as u128);

        let helper = expect_limit(
            preflight_spatial_direct_counts_v2(observed, limits),
            kind,
            2,
            1,
        );
        let resolver = resolver_error(resolve_spatial_v2(
            &ReferenceStackEngineV1::new(),
            Arc::new(owned_input(counts)),
            limits,
        ));

        assert_eq!(resolver, helper, "phase-one mismatch for {kind:?}");
    }
}

fn resolver_error<T>(result: Result<T, SpatialResolveErrorV2>) -> SpatialResolveErrorV2 {
    match result {
        Ok(_) => panic!("expected raw resolver direct-count failure"),
        Err(error) => error,
    }
}
