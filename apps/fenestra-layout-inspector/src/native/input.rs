use super::{EvidenceMilestone, InspectorAction, NativeApplication, NativeInspectorError};
use crate::InspectorErrorKind;

impl NativeApplication {
    pub(super) fn resize_window(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<(), NativeInspectorError> {
        if width == 0 || height == 0 {
            return Ok(());
        }
        let width = i32::try_from(width).map_err(|_| NativeInspectorError::Presenter)?;
        let height = i32::try_from(height).map_err(|_| NativeInspectorError::Presenter)?;
        self.inspector
            .dispatch(InspectorAction::Resize { width, height })
            .map_err(NativeInspectorError::Application)?;
        if let Some(evidence) = &mut self.evidence
            && evidence.next_required() == Some(EvidenceMilestone::Resize)
        {
            let frame = self
                .inspector
                .observe()
                .map_err(NativeInspectorError::Application)?;
            evidence
                .record_resize(&frame)
                .map_err(NativeInspectorError::Evidence)?;
        }
        Ok(())
    }

    pub(super) fn pointer_move(&mut self, x: i32, y: i32) -> Result<(), NativeInspectorError> {
        self.inspector
            .dispatch(InspectorAction::PointerMove { x, y })
            .map_err(NativeInspectorError::Application)?;
        if let Some(evidence) = &mut self.evidence
            && evidence.next_required() == Some(EvidenceMilestone::PointerMove)
        {
            let frame = self
                .inspector
                .observe()
                .map_err(NativeInspectorError::Application)?;
            evidence
                .record_pointer_move(x, y, &frame)
                .map_err(NativeInspectorError::Evidence)?;
        }
        Ok(())
    }

    pub(super) fn pointer_press(&mut self) -> Result<(), NativeInspectorError> {
        self.inspector
            .dispatch(InspectorAction::PointerPress)
            .map_err(NativeInspectorError::Application)?;
        if let Some(evidence) = &mut self.evidence
            && evidence.next_required() == Some(EvidenceMilestone::PointerPress)
        {
            let frame = self
                .inspector
                .observe()
                .map_err(NativeInspectorError::Application)?;
            evidence
                .record_pointer_press(&frame)
                .map_err(NativeInspectorError::Evidence)?;
        }
        Ok(())
    }

    pub(super) fn insert_tile(&mut self) -> Result<(), NativeInspectorError> {
        let current = self
            .inspector
            .observe()
            .map_err(NativeInspectorError::Application)?;
        let mut key = 30_u64;
        while current.keyed_keys().contains(&key) {
            key = key
                .checked_add(10)
                .ok_or(NativeInspectorError::Application(
                    InspectorErrorKind::Transaction,
                ))?;
        }
        self.inspector
            .dispatch(InspectorAction::InsertTile { key })
            .map_err(NativeInspectorError::Application)?;
        if self.evidence.as_ref().is_some_and(|evidence| {
            evidence.next_required() == Some(EvidenceMilestone::KeyedInsert)
        }) {
            let frame = self
                .inspector
                .observe()
                .map_err(NativeInspectorError::Application)?;
            self.evidence
                .as_mut()
                .expect("evidence was checked above")
                .record_keyed_insert(key, &frame)
                .map_err(NativeInspectorError::Evidence)?;
        }
        Ok(())
    }
}
