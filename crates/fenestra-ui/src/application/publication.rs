use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_runtime::prototype::{CommitWithError, UiTransaction};

use super::{
    Application, NamedNode, controls, hit_node, interaction::Interaction, layout, text, viewport,
};
use crate::{Error, Size, lower};

impl Application {
    pub(super) fn commit_state(&mut self, nodes: Vec<NamedNode>, size: Size) -> Result<(), Error> {
        self.commit_with_state(nodes, size, self.interaction.clone())
    }

    pub(super) fn commit_with_state(
        &mut self,
        mut nodes: Vec<NamedNode>,
        size: Size,
        mut interaction: Interaction,
    ) -> Result<(), Error> {
        controls::prepare(&mut nodes, &mut interaction);
        layout::resolve_nodes(&mut nodes, &mut self.text_engine, size, self.limits)?;
        let revision = self.revision.wrapping_add(1);
        if let Some((x, y)) = interaction.pointer {
            // Layout can move a control under a stationary pointer. Ask the
            // runtime for candidate hit geometry without publishing any state.
            let geometry_changed = size != self.size
                || self.nodes.iter().zip(&nodes).any(|(old, new)| {
                    old.resolved != new.resolved
                        || old.effective_style.padding != new.effective_style.padding
                        || old.effective_style.gap != new.effective_style.gap
                        || old.effective_style.input != new.effective_style.input
                });
            let committed = if geometry_changed && nodes.iter().any(|node| node.control.is_some()) {
                let transaction = self.stage_transaction(&nodes, size, revision)?;
                self.runtime
                    .preview(transaction)
                    .map_err(|error| Error::Runtime(format!("{:?}", error.kind())))?
            } else {
                self.runtime.committed()
            };
            interaction.hover = interaction
                .window_focused
                .then(|| hit_node(&committed, size, x, y))
                .flatten()
                .and_then(|id| nodes.iter().find(|node| node.id == id))
                .and_then(|node| node.control_owner)
                .filter(|&owner| {
                    !nodes[owner]
                        .control
                        .as_ref()
                        .expect("control owner")
                        .disabled
                });
            controls::prepare(&mut nodes, &mut interaction);
        }
        for node in &mut nodes {
            text::prepare_node(node, &mut self.text_engine, self.limits.text())?;
        }
        let transaction = self.stage_transaction(&nodes, size, revision)?;
        let (_, frame) = self
            .runtime
            .commit_with(transaction, |committed| {
                text::prepare_frame(
                    committed,
                    &nodes,
                    &interaction,
                    self.spatial_limits,
                    self.limits,
                )
            })
            .map_err(|error| match error {
                CommitWithError::Runtime(error) => Error::Runtime(format!("{:?}", error.kind())),
                CommitWithError::Preparation(error) => error,
            })?;
        self.nodes = nodes;
        self.interaction = interaction;
        self.revision = revision;
        self.size = size;
        self.text_frame = frame;
        self.raster_cache.clear();
        Ok(())
    }

    fn stage_transaction(
        &self,
        nodes: &[NamedNode],
        size: Size,
        revision: i32,
    ) -> Result<UiTransaction, Error> {
        let mut transaction = self.runtime.begin_transaction();
        for (old, new) in self.nodes.iter().zip(nodes) {
            for ((_, before), (property, after)) in old
                .effective_style
                .values(old.resolved)
                .into_iter()
                .zip(new.effective_style.values(new.resolved))
            {
                if before != after {
                    transaction
                        .set_property(new.id, property, after)
                        .map_err(|error| Error::Runtime(format!("{:?}", error.kind())))?;
                }
            }
        }
        transaction
            .set_property(
                nodes[0].id,
                lower::VIEW_REVISION,
                PropertyValue::ScalarI32(revision),
            )
            .map_err(|error| Error::Runtime(format!("{:?}", error.kind())))?;
        if size != self.size {
            transaction
                .resize_spatial(viewport(size))
                .map_err(|error| Error::Runtime(format!("{:?}", error.kind())))?;
        }
        Ok(transaction)
    }
}
