use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use super::{
    EvidenceMilestone, InspectorAction, InspectorErrorKind, NativeApplication, NativeInspectorError,
};

pub(super) fn requests_redraw(event: &WindowEvent) -> bool {
    match event {
        WindowEvent::CursorMoved { .. }
        | WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Left,
            ..
        }
        | WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    physical_key: PhysicalKey::Code(KeyCode::Space),
                    state: ElementState::Pressed,
                    repeat: false,
                    ..
                },
            ..
        } => true,
        WindowEvent::Resized(size) => size.width > 0 && size.height > 0,
        _ => false,
    }
}

impl NativeApplication {
    pub(super) fn resize_window(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<(), NativeInspectorError> {
        self.resize_application(width, height)?;
        if self.drawable
            && self
                .evidence
                .as_ref()
                .is_some_and(|evidence| evidence.next_required() == Some(EvidenceMilestone::Resize))
        {
            let frame = self
                .inspector
                .observe()
                .map_err(NativeInspectorError::Application)?;
            self.evidence
                .as_mut()
                .expect("evidence was checked above")
                .record_resize(&frame)
                .map_err(NativeInspectorError::Evidence)?;
        }
        Ok(())
    }

    pub(super) fn resize_application(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<(), NativeInspectorError> {
        if width == 0 || height == 0 {
            self.drawable = false;
            return Ok(());
        }
        let width = i32::try_from(width).map_err(|_| NativeInspectorError::Presenter)?;
        let height = i32::try_from(height).map_err(|_| NativeInspectorError::Presenter)?;
        self.inspector
            .dispatch(InspectorAction::Resize { width, height })
            .map_err(NativeInspectorError::Application)?;
        self.drawable = true;
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
