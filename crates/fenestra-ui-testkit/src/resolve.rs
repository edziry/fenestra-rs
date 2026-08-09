use fenestra_ui_ir::prototype::{PropertyId, PropertyValue};
use fenestra_ui_runtime::prototype::{FragmentId, NodeId, TransactionError, UiTransaction};

use crate::case::{OperationV1, SemanticOperationV1, TransactionIdV1};
use crate::desired::DesiredStateV1;
use crate::error::{HarnessError, HarnessErrorKind};
use crate::identity::IdentityIndexV1;
use crate::semantic::{FragmentPathV1, NodePathV1};

pub(crate) struct ResolvedBaseV1<'a> {
    identities: &'a IdentityIndexV1,
    desired: &'a DesiredStateV1,
}

pub(crate) enum ResolvedOperationV1 {
    SetProperty {
        node: NodeId,
        property: PropertyId,
        value: PropertyValue,
    },
    InsertKeyed {
        fragment: FragmentId,
        key: u64,
        final_index: usize,
    },
    MoveKeyed {
        fragment: FragmentId,
        key: u64,
        final_index: usize,
    },
    UpdateKeyed {
        fragment: FragmentId,
        key: u64,
        property: PropertyId,
        value: PropertyValue,
    },
    RemoveKeyed {
        fragment: FragmentId,
        key: u64,
    },
}

impl ResolvedOperationV1 {
    pub(crate) fn stage(self, transaction: &mut UiTransaction) -> Result<(), TransactionError> {
        match self {
            Self::SetProperty {
                node,
                property,
                value,
            } => transaction.set_property(node, property, value),
            Self::InsertKeyed {
                fragment,
                key,
                final_index,
            } => transaction.insert_keyed(fragment, key, final_index),
            Self::MoveKeyed {
                fragment,
                key,
                final_index,
            } => transaction.move_keyed(fragment, key, final_index),
            Self::UpdateKeyed {
                fragment,
                key,
                property,
                value,
            } => transaction.update_keyed(fragment, key, property, value),
            Self::RemoveKeyed { fragment, key } => transaction.remove_keyed(fragment, key),
        }
    }
}

impl<'a> ResolvedBaseV1<'a> {
    pub(crate) const fn new(identities: &'a IdentityIndexV1, desired: &'a DesiredStateV1) -> Self {
        Self {
            identities,
            desired,
        }
    }

    pub(crate) fn resolve(
        &self,
        transaction_id: TransactionIdV1,
        operation: &OperationV1,
        draft: &DesiredStateV1,
    ) -> Result<ResolvedOperationV1, HarnessError> {
        match operation.operation() {
            SemanticOperationV1::SetProperty {
                node,
                property,
                value,
            } => Ok(ResolvedOperationV1::SetProperty {
                node: self.node(node, draft, transaction_id, operation)?,
                property: *property,
                value: value.clone(),
            }),
            SemanticOperationV1::InsertKeyed {
                fragment,
                key,
                final_index,
            } => Ok(ResolvedOperationV1::InsertKeyed {
                fragment: self.fragment(fragment, draft, transaction_id, operation)?,
                key: *key,
                final_index: usize::try_from(*final_index)
                    .map_err(|_| arithmetic_error().at_operation(transaction_id, operation.id()))?,
            }),
            SemanticOperationV1::MoveKeyed {
                fragment,
                key,
                final_index,
            } => Ok(ResolvedOperationV1::MoveKeyed {
                fragment: self.fragment(fragment, draft, transaction_id, operation)?,
                key: *key,
                final_index: usize::try_from(*final_index)
                    .map_err(|_| arithmetic_error().at_operation(transaction_id, operation.id()))?,
            }),
            SemanticOperationV1::UpdateKeyed {
                fragment,
                key,
                property,
                value,
            } => Ok(ResolvedOperationV1::UpdateKeyed {
                fragment: self.fragment(fragment, draft, transaction_id, operation)?,
                key: *key,
                property: *property,
                value: value.clone(),
            }),
            SemanticOperationV1::RemoveKeyed { fragment, key } => {
                Ok(ResolvedOperationV1::RemoveKeyed {
                    fragment: self.fragment(fragment, draft, transaction_id, operation)?,
                    key: *key,
                })
            }
        }
    }

    fn node(
        &self,
        path: &NodePathV1,
        draft: &DesiredStateV1,
        transaction_id: TransactionIdV1,
        operation: &OperationV1,
    ) -> Result<NodeId, HarnessError> {
        let node = self
            .identities
            .node(path)
            .ok_or_else(|| invalid_operation().at_operation(transaction_id, operation.id()))?;
        if !self.desired.preserves_incarnation(draft, path) {
            return Err(invalid_operation().at_operation(transaction_id, operation.id()));
        }
        Ok(node)
    }

    fn fragment(
        &self,
        path: &FragmentPathV1,
        draft: &DesiredStateV1,
        transaction_id: TransactionIdV1,
        operation: &OperationV1,
    ) -> Result<FragmentId, HarnessError> {
        let fragment = self
            .identities
            .fragment(path)
            .ok_or_else(|| invalid_operation().at_operation(transaction_id, operation.id()))?;
        if !self.desired.preserves_incarnation(draft, path.owner()) {
            return Err(invalid_operation().at_operation(transaction_id, operation.id()));
        }
        Ok(fragment)
    }
}

fn invalid_operation() -> HarnessError {
    HarnessError::new(HarnessErrorKind::InvalidOperation)
}

fn arithmetic_error() -> HarnessError {
    HarnessError::new(HarnessErrorKind::ArithmeticExhausted)
}
