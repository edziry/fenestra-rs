use fenestra_ui_runtime::prototype::{RuntimePaintFrameV2, SubmissionId};

use super::{SpatialPresentErrorKindV2, SpatialSurfaceTupleV2};

pub(crate) trait SpatialPresenterPortV2 {
    fn present_offer<A>(
        &mut self,
        frame: RuntimePaintFrameV2<'_>,
        surface: SpatialSurfaceTupleV2,
        accept_once: A,
    ) -> Result<u64, SpatialPresentErrorKindV2>
    where
        A: FnOnce() -> Result<SubmissionId, SpatialPresentErrorKindV2>;
}
