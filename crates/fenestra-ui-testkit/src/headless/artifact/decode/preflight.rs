mod canonical;
mod limits;

use super::super::error::HeadlessArtifactDecodeErrorV1;
use super::super::model::{ArtifactCapacitiesV1, ArtifactMetadataV1};
use super::scan::ScannedArtifactV1;
use super::state::LayoutV1;

pub(super) struct PreflightV1 {
    pub(super) metadata: ArtifactMetadataV1,
    pub(super) capacities: ArtifactCapacitiesV1,
}

pub(super) fn preflight_v1(
    scanned: &ScannedArtifactV1<'_>,
    layout: LayoutV1,
) -> Result<PreflightV1, HeadlessArtifactDecodeErrorV1> {
    canonical::validate_versions_v1(scanned, layout)?;
    let inspected = canonical::inspect_canonical_v1(scanned, layout)?;
    limits::validate_limits_and_counts_v1(scanned, layout, &inspected)?;
    Ok(PreflightV1 {
        metadata: inspected.metadata,
        capacities: inspected.capacities,
    })
}
