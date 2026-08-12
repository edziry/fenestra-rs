use crate::resolved::ResolvedDocumentV1;

pub(crate) struct ResolvedDocumentV2 {
    pub(crate) core: ResolvedDocumentV1,
    pub(crate) spatial_anchor: u32,
    pub(crate) resources_anchor: u32,
}

impl ResolvedDocumentV2 {
    pub(crate) const fn new(
        core: ResolvedDocumentV1,
        spatial_anchor: u32,
        resources_anchor: u32,
    ) -> Self {
        Self {
            core,
            spatial_anchor,
            resources_anchor,
        }
    }

    pub(crate) const fn authoring_format(&self) -> u32 {
        self.core.authoring_format()
    }

    pub(crate) const fn document_anchor(&self) -> u32 {
        self.core.document_anchor()
    }
}
