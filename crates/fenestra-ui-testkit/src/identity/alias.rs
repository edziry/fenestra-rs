use std::collections::HashSet;

use super::{IdentityIndexV1, identity_mismatch};
use crate::error::HarnessError;
use crate::fingerprint::{FailureFingerprintV1, FingerprintLocationV1, FingerprintSummaryV1};

pub(super) fn first_alias_v1(
    identities: &IdentityIndexV1,
) -> Result<Option<FailureFingerprintV1>, HarnessError> {
    let mut nodes = HashSet::new();
    for (path, node) in identities.nodes_in_authored_order() {
        if !nodes.insert(node) {
            return identity_mismatch(
                FingerprintLocationV1::Node(path.clone()),
                FingerprintSummaryV1::LifecycleDistinct,
                FingerprintSummaryV1::LifecycleAliased,
            );
        }
    }

    let mut fragments = HashSet::new();
    for (path, fragment) in identities.fragments_in_authored_order() {
        if !fragments.insert(fragment) {
            return identity_mismatch(
                FingerprintLocationV1::Fragment(path.clone()),
                FingerprintSummaryV1::LifecycleDistinct,
                FingerprintSummaryV1::LifecycleAliased,
            );
        }
    }
    Ok(None)
}
