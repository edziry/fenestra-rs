use super::DraftApplication;
use crate::runtime::error::{TransactionError, TransactionErrorKind};
use crate::runtime::headless::HeadlessSurface;
use crate::runtime::mutation::SpatialViewportChange;
use crate::runtime::mutation::{HeadlessSurfaceChange, MutationRecord};
use fenestra_ui_spatial::prototype::SpatialViewportV2;

impl DraftApplication<'_> {
    pub(super) fn resize(
        &mut self,
        surface: HeadlessSurface,
        operation_index: usize,
    ) -> Result<(), TransactionError> {
        let old_surface = self.candidate_surface.ok_or_else(|| {
            TransactionError::new(
                TransactionErrorKind::HeadlessUnavailable,
                Some(operation_index),
            )
        })?;
        if old_surface == surface {
            return Ok(());
        }
        if let Some(record_index) = self.surface_record {
            let MutationRecord::HeadlessSurfaceChanged(change) = &mut self.records[record_index]
            else {
                return Err(TransactionError::new(
                    TransactionErrorKind::InvariantViolation,
                    None,
                ));
            };
            change.new_surface = surface;
        } else {
            self.surface_record = Some(self.records.len());
            self.records.push(MutationRecord::HeadlessSurfaceChanged(
                HeadlessSurfaceChange {
                    old_surface,
                    new_surface: surface,
                },
            ));
        }
        self.candidate_surface = Some(surface);
        self.surface_source = Some(operation_index);
        Ok(())
    }

    pub(super) fn resize_spatial(
        &mut self,
        viewport: SpatialViewportV2,
        operation_index: usize,
    ) -> Result<(), TransactionError> {
        let old_viewport = self.candidate_spatial_viewport.ok_or_else(|| {
            TransactionError::new(
                TransactionErrorKind::SpatialUnavailable,
                Some(operation_index),
            )
        })?;
        if old_viewport == viewport {
            return Ok(());
        }
        if let Some(record_index) = self.spatial_viewport_record {
            let MutationRecord::SpatialViewportChanged(change) = &mut self.records[record_index]
            else {
                return Err(TransactionError::new(
                    TransactionErrorKind::InvariantViolation,
                    None,
                ));
            };
            change.new_viewport = viewport;
        } else {
            self.spatial_viewport_record = Some(self.records.len());
            self.records.push(MutationRecord::SpatialViewportChanged(
                SpatialViewportChange {
                    old_viewport,
                    new_viewport: viewport,
                },
            ));
        }
        self.candidate_spatial_viewport = Some(viewport);
        Ok(())
    }
}
