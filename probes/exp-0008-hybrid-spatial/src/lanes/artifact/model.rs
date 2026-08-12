#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LaneArtifactV2 {
    pub(crate) name: &'static str,
    pub(crate) bytes: Vec<u8>,
}

#[derive(Clone, Copy)]
pub(super) struct CandidateEvidenceV2 {
    pub(super) lane: &'static str,
    pub(super) name: &'static str,
    pub(super) versions: &'static str,
    pub(super) features: &'static str,
    pub(super) target: &'static str,
    pub(super) roots: &'static [(&'static str, &'static str)],
    pub(super) outcome: &'static str,
    pub(super) reason: &'static str,
}

pub(super) struct LaneEvidenceV2 {
    pub(super) file: &'static str,
    pub(super) candidates: Vec<CandidateEvidenceV2>,
}

pub(super) const LINUX: &str = "x86_64-unknown-linux-gnu";
pub(super) const WINDOWS: &str = "x86_64-pc-windows-msvc";
pub(super) const VELLO_LINUX: &str = "x86_64-unknown-linux-gnu:vulkan-wayland";
pub(super) const VELLO_WINDOWS: &str = "x86_64-pc-windows-msvc:dx12-win32";
