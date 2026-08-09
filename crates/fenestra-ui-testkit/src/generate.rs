use crate::case::{
    GeneratedCaseV1, GeneratorConfigV1, OperationIdV1, OperationV1, REGISTERED_FIXTURE_REVISION_V1,
    SeedV1, SemanticOperationV1, TransactionIdV1, TransactionV1,
};
use crate::desired::DesiredStateV1;
use crate::error::{GeneratorError, GeneratorErrorKind, HarnessErrorKind, HarnessLimitKind};
use crate::fixture::{HarnessLimitsV1, RuntimeOracleFixtureV1};
use crate::model::clean_rebuild_v1;
use crate::semantic::{FragmentPathV1, NodePathV1, NormalizedStateV1};

mod catalog;
#[cfg(test)]
mod tests;

use catalog::{WordStreamV1, directed_prefix, value_catalog};

const DIRECTED_TRANSACTIONS: u32 = 8;
const DIRECTED_OPERATIONS: u32 = 10;
const MIN_MAX_OPERATIONS: u32 = 2;
const MIN_LIVE_MEMBERSHIPS: u32 = 8;

/// Generates the exact bounded semantic case for one V1 seed and configuration.
pub fn generate_case_v1(
    fixture: &RuntimeOracleFixtureV1,
    seed: SeedV1,
    config: GeneratorConfigV1,
) -> Result<GeneratedCaseV1, GeneratorError> {
    validate_config(config, fixture.harness_limits())?;
    let mut desired =
        DesiredStateV1::from_construction(fixture.construction(), fixture.harness_limits())
            .map_err(map_harness_error)?;
    let mut transactions = Vec::new();
    let mut next_operation = 0_u32;

    for operations in directed_prefix() {
        for operation in &operations {
            desired
                .apply_operation(operation, fixture.harness_limits())
                .map_err(map_harness_error)?;
        }
        append_transaction(&mut transactions, &mut next_operation, operations)?;
    }
    debug_assert_eq!(next_operation, DIRECTED_OPERATIONS);

    let mut words = WordStreamV1::new(seed);
    while transactions.len() < usize_from(config.transaction_count())? {
        let count = 1 + words.next() % u64::from(config.max_operations_per_transaction());
        let count = usize::try_from(count).map_err(|_| arithmetic_error())?;
        let base = clean_rebuild_v1(fixture.construction(), &desired, fixture.harness_limits())
            .map_err(map_harness_error)?;
        let base_nodes: Vec<_> = base
            .nodes()
            .iter()
            .map(|node| {
                let path = node.path().clone();
                desired
                    .incarnation_token(&path)
                    .map(|token| (path, token))
                    .ok_or_else(no_action_error)
            })
            .collect::<Result<_, _>>()?;
        let base_fragments: Vec<_> = base
            .fragments()
            .iter()
            .map(|fragment| {
                let path = fragment.path().clone();
                desired
                    .incarnation_token(path.owner())
                    .map(|token| (path, token))
                    .ok_or_else(no_action_error)
            })
            .collect::<Result<_, _>>()?;
        let mut draft = desired.clone();
        let mut selected = Vec::with_capacity(count);

        for _ in 0..count {
            let actions =
                applicable_actions(fixture, &draft, &base_nodes, &base_fragments, config)?;
            if actions.is_empty() {
                return Err(GeneratorError::new(GeneratorErrorKind::NoApplicableAction));
            }
            let action_count = u64::try_from(actions.len()).map_err(|_| arithmetic_error())?;
            let index =
                usize::try_from(words.next() % action_count).map_err(|_| arithmetic_error())?;
            let operation = actions[index].clone();
            draft
                .apply_operation(&operation, fixture.harness_limits())
                .map_err(map_harness_error)?;
            selected.push(operation);
        }

        append_transaction(&mut transactions, &mut next_operation, selected)?;
        desired = draft;
    }

    let operation_count = transactions.iter().try_fold(0_usize, |count, transaction| {
        count.checked_add(transaction.operations().len())
    });
    if operation_count.ok_or_else(arithmetic_error)? > fixture.harness_limits().operations() {
        return Err(GeneratorError::limit(HarnessLimitKind::Operations));
    }
    Ok(GeneratedCaseV1::new(
        REGISTERED_FIXTURE_REVISION_V1,
        config,
        seed,
        transactions,
    ))
}

fn validate_config(
    config: GeneratorConfigV1,
    limits: HarnessLimitsV1,
) -> Result<(), GeneratorError> {
    validate_field(
        config.transaction_count(),
        DIRECTED_TRANSACTIONS,
        limits.transactions(),
        HarnessLimitKind::Transactions,
    )?;
    validate_field(
        config.max_operations_per_transaction(),
        MIN_MAX_OPERATIONS,
        limits.operations_per_transaction(),
        HarnessLimitKind::OperationsPerTransaction,
    )?;
    validate_field(
        config.max_live_memberships(),
        MIN_LIVE_MEMBERSHIPS,
        limits.live_memberships(),
        HarnessLimitKind::LiveMemberships,
    )
}

fn validate_field(
    value: u32,
    minimum: u32,
    maximum: usize,
    limit: HarnessLimitKind,
) -> Result<(), GeneratorError> {
    if usize_from(value)? > maximum {
        Err(GeneratorError::limit(limit))
    } else if value < minimum {
        Err(GeneratorError::new(GeneratorErrorKind::InvalidConfig))
    } else {
        Ok(())
    }
}

fn append_transaction(
    transactions: &mut Vec<TransactionV1>,
    next_operation: &mut u32,
    operations: Vec<SemanticOperationV1>,
) -> Result<(), GeneratorError> {
    let transaction_id = u32::try_from(transactions.len()).map_err(|_| arithmetic_error())?;
    let mut identified = Vec::with_capacity(operations.len());
    for operation in operations {
        identified.push(OperationV1::new(
            OperationIdV1::new(*next_operation),
            operation,
        ));
        *next_operation = next_operation.checked_add(1).ok_or_else(arithmetic_error)?;
    }
    transactions.push(TransactionV1::new(
        TransactionIdV1::new(transaction_id),
        identified,
    ));
    Ok(())
}

fn applicable_actions(
    fixture: &RuntimeOracleFixtureV1,
    desired: &DesiredStateV1,
    base_nodes: &[(NodePathV1, Vec<u64>)],
    base_fragments: &[(FragmentPathV1, Vec<u64>)],
    config: GeneratorConfigV1,
) -> Result<Vec<SemanticOperationV1>, GeneratorError> {
    let state = clean_rebuild_v1(fixture.construction(), desired, fixture.harness_limits())
        .map_err(map_harness_error)?;
    let mut actions = Vec::new();

    for (path, incarnation) in base_nodes {
        if desired.incarnation_token(path).as_ref() != Some(incarnation) {
            continue;
        }
        let Some(node) = state.node(path) else {
            continue;
        };
        for property in node.properties() {
            for value in value_catalog(property.value().value_type()) {
                push_action(
                    &mut actions,
                    SemanticOperationV1::SetProperty {
                        node: path.clone(),
                        property: property.property(),
                        value,
                    },
                    fixture.harness_limits(),
                )?;
            }
        }
    }

    for (path, incarnation) in base_fragments {
        if desired.incarnation_token(path.owner()).as_ref() != Some(incarnation) {
            continue;
        }
        let Some(fragment) = state.fragment(path) else {
            continue;
        };
        let free_keys: Vec<_> = (0..=31_u64)
            .filter(|key| !fragment.members().iter().any(|member| member.key() == *key))
            .collect();
        let Some(&probe_key) = free_keys.first() else {
            continue;
        };
        let probe = SemanticOperationV1::InsertKeyed {
            fragment: path.clone(),
            key: probe_key,
            final_index: 0,
        };
        if !insertion_fits(fixture, desired, &probe, config)? {
            continue;
        }
        for key in free_keys {
            for index in 0..=fragment.members().len() {
                let operation = SemanticOperationV1::InsertKeyed {
                    fragment: path.clone(),
                    key,
                    final_index: u32::try_from(index).map_err(|_| arithmetic_error())?,
                };
                push_action(&mut actions, operation, fixture.harness_limits())?;
            }
        }
    }

    for (path, incarnation) in base_fragments {
        if desired.incarnation_token(path.owner()).as_ref() != Some(incarnation) {
            continue;
        }
        let Some(fragment) = state.fragment(path) else {
            continue;
        };
        for member in fragment.members() {
            for index in 0..fragment.members().len() {
                push_action(
                    &mut actions,
                    SemanticOperationV1::MoveKeyed {
                        fragment: path.clone(),
                        key: member.key(),
                        final_index: u32::try_from(index).map_err(|_| arithmetic_error())?,
                    },
                    fixture.harness_limits(),
                )?;
            }
        }
    }

    append_update_actions(
        &mut actions,
        &state,
        desired,
        base_fragments,
        fixture.harness_limits(),
    )?;
    for (path, incarnation) in base_fragments {
        if desired.incarnation_token(path.owner()).as_ref() != Some(incarnation) {
            continue;
        }
        let Some(fragment) = state.fragment(path) else {
            continue;
        };
        for member in fragment.members() {
            push_action(
                &mut actions,
                SemanticOperationV1::RemoveKeyed {
                    fragment: path.clone(),
                    key: member.key(),
                },
                fixture.harness_limits(),
            )?;
        }
    }
    Ok(actions)
}

fn append_update_actions(
    actions: &mut Vec<SemanticOperationV1>,
    state: &NormalizedStateV1,
    desired: &DesiredStateV1,
    base_fragments: &[(FragmentPathV1, Vec<u64>)],
    limits: HarnessLimitsV1,
) -> Result<(), GeneratorError> {
    for (path, incarnation) in base_fragments {
        if desired.incarnation_token(path.owner()).as_ref() != Some(incarnation) {
            continue;
        }
        let Some(fragment) = state.fragment(path) else {
            continue;
        };
        for member in fragment.members() {
            let node = state.node(member.node()).ok_or_else(no_action_error)?;
            for property in node.properties() {
                for value in value_catalog(property.value().value_type()) {
                    push_action(
                        actions,
                        SemanticOperationV1::UpdateKeyed {
                            fragment: path.clone(),
                            key: member.key(),
                            property: property.property(),
                            value,
                        },
                        limits,
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn insertion_fits(
    fixture: &RuntimeOracleFixtureV1,
    desired: &DesiredStateV1,
    operation: &SemanticOperationV1,
    config: GeneratorConfigV1,
) -> Result<bool, GeneratorError> {
    let mut candidate = desired.clone();
    match candidate.apply_operation(operation, fixture.harness_limits()) {
        Ok(()) => {}
        Err(error) if matches!(error.kind(), HarnessErrorKind::LimitExceeded(_)) => {
            return Ok(false);
        }
        Err(error) => return Err(map_harness_error(error)),
    }
    let rebuilt =
        match clean_rebuild_v1(fixture.construction(), &candidate, fixture.harness_limits()) {
            Ok(rebuilt) => rebuilt,
            Err(error) if matches!(error.kind(), HarnessErrorKind::LimitExceeded(_)) => {
                return Ok(false);
            }
            Err(error) => return Err(map_harness_error(error)),
        };
    let memberships = rebuilt
        .fragments()
        .iter()
        .try_fold(0_usize, |count, fragment| {
            count.checked_add(fragment.members().len())
        })
        .ok_or_else(arithmetic_error)?;
    Ok(memberships <= usize_from(config.max_live_memberships())?)
}

fn push_action(
    actions: &mut Vec<SemanticOperationV1>,
    operation: SemanticOperationV1,
    limits: HarnessLimitsV1,
) -> Result<(), GeneratorError> {
    if actions.len() >= limits.applicable_actions() {
        return Err(GeneratorError::limit(HarnessLimitKind::ApplicableActions));
    }
    actions.push(operation);
    Ok(())
}

fn usize_from(value: u32) -> Result<usize, GeneratorError> {
    usize::try_from(value).map_err(|_| arithmetic_error())
}

fn map_harness_error(error: crate::error::HarnessError) -> GeneratorError {
    match error.kind() {
        HarnessErrorKind::LimitExceeded(kind) => GeneratorError::limit(kind),
        HarnessErrorKind::ArithmeticExhausted => arithmetic_error(),
        _ => no_action_error(),
    }
}

fn no_action_error() -> GeneratorError {
    GeneratorError::new(GeneratorErrorKind::NoApplicableAction)
}

fn arithmetic_error() -> GeneratorError {
    GeneratorError::new(GeneratorErrorKind::ArithmeticExhausted)
}
