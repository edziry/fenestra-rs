use std::error::Error;
use std::fmt;

use fenestra_ui_runtime::prototype::FrameWork;

use crate::scheduler::{SyntheticResourceIdV1, SyntheticResourceUseV1};

const HEADLESS_RESOURCE_BYTES: usize = 64;

/// Closed failures produced while deriving a headless frame resource.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessRendererErrorKindV1 {
    /// The frame snapshot has no headless projection.
    HeadlessUnavailable,
    /// The frame and its headless projection name different generations.
    IdentityMismatch,
}

/// Privacy-safe failure from headless renderer resource derivation.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct HeadlessRendererErrorV1 {
    kind: HeadlessRendererErrorKindV1,
}

impl HeadlessRendererErrorV1 {
    const fn new(kind: HeadlessRendererErrorKindV1) -> Self {
        Self { kind }
    }

    /// Returns the closed headless renderer failure category.
    #[must_use]
    pub const fn kind(self) -> HeadlessRendererErrorKindV1 {
        self.kind
    }
}

impl fmt::Debug for HeadlessRendererErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HeadlessRendererErrorV1")
            .field("kind", &self.kind)
            .finish()
    }
}

impl fmt::Display for HeadlessRendererErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "headless renderer resource failed: {:?}",
            self.kind
        )
    }
}

impl Error for HeadlessRendererErrorV1 {}

/// Derives the fixed synthetic resource owned by one headless frame generation.
pub fn headless_frame_resource_v1(
    frame: &FrameWork,
) -> Result<SyntheticResourceUseV1, HeadlessRendererErrorV1> {
    let projection = frame
        .snapshot()
        .headless_projection()
        .ok_or_else(headless_unavailable_error)?;
    if projection.generation() != frame.generation() {
        return Err(HeadlessRendererErrorV1::new(
            HeadlessRendererErrorKindV1::IdentityMismatch,
        ));
    }
    Ok(SyntheticResourceUseV1::new(
        SyntheticResourceIdV1::new(frame.generation().get()),
        HEADLESS_RESOURCE_BYTES,
    ))
}

const fn headless_unavailable_error() -> HeadlessRendererErrorV1 {
    HeadlessRendererErrorV1::new(HeadlessRendererErrorKindV1::HeadlessUnavailable)
}
