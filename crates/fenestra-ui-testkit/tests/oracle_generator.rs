use fenestra_ui_ir::prototype::{PropertyId, PropertyValue};
use fenestra_ui_testkit::prototype::{
    FragmentPathV1, GeneratorConfigV1, GeneratorErrorKind, HarnessLimitKind, NodePathV1,
    RuntimeOracleFixtureV1, SeedV1, SemanticOperationV1, generate_case_v1,
};

#[test]
fn directed_prefix_has_exact_transactions_operations_and_semantics() {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let generated = generate_case_v1(&fixture, SeedV1::new(0), GeneratorConfigV1::new(8, 2, 8))
        .expect("minimum config should generate the directed prefix");

    assert_eq!(generated.transactions().len(), 8);
    assert_eq!(generated.operation_count(), 10);
    assert_eq!(
        generated
            .transactions()
            .iter()
            .map(|transaction| transaction.id().get())
            .collect::<Vec<_>>(),
        (0..8).collect::<Vec<_>>()
    );

    let root = NodePathV1::root();
    let primary = FragmentPathV1::new(root.clone(), 1);
    let secondary = FragmentPathV1::new(root.clone(), 2);
    let nested = FragmentPathV1::new(root.clone().member(1, 9), 1);
    let expected = vec![
        expected_operation(
            0,
            0,
            SemanticOperationV1::SetProperty {
                node: root.clone(),
                property: PropertyId::new(0),
                value: PropertyValue::ScalarI32(320),
            },
        ),
        expected_operation(
            0,
            1,
            SemanticOperationV1::SetProperty {
                node: root.clone(),
                property: PropertyId::new(0),
                value: PropertyValue::ScalarI32(480),
            },
        ),
        expected_operation(
            1,
            2,
            SemanticOperationV1::SetProperty {
                node: root,
                property: PropertyId::new(0),
                value: PropertyValue::ScalarI32(480),
            },
        ),
        expected_operation(
            2,
            3,
            SemanticOperationV1::InsertKeyed {
                fragment: primary.clone(),
                key: 9,
                final_index: 2,
            },
        ),
        expected_operation(
            2,
            4,
            SemanticOperationV1::MoveKeyed {
                fragment: primary.clone(),
                key: 9,
                final_index: 0,
            },
        ),
        expected_operation(
            3,
            5,
            SemanticOperationV1::UpdateKeyed {
                fragment: primary.clone(),
                key: 9,
                property: PropertyId::new(0),
                value: PropertyValue::ScalarI32(90),
            },
        ),
        expected_operation(
            4,
            6,
            SemanticOperationV1::UpdateKeyed {
                fragment: secondary,
                key: 7,
                property: PropertyId::new(0),
                value: PropertyValue::ScalarI32(70),
            },
        ),
        expected_operation(
            5,
            7,
            SemanticOperationV1::InsertKeyed {
                fragment: nested,
                key: 2,
                final_index: 1,
            },
        ),
        expected_operation(
            6,
            8,
            SemanticOperationV1::RemoveKeyed {
                fragment: primary.clone(),
                key: 9,
            },
        ),
        expected_operation(
            7,
            9,
            SemanticOperationV1::InsertKeyed {
                fragment: primary,
                key: 9,
                final_index: 2,
            },
        ),
    ];

    assert_eq!(flatten_operations(&generated), expected);
}

#[test]
fn equal_seed_fixture_and_config_generate_equal_cases() {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let config = GeneratorConfigV1::new(16, 4, 12);
    let seed = SeedV1::new(1_592_614_637);

    let first = generate_case_v1(&fixture, seed, config).expect("case should generate");
    let second = generate_case_v1(&fixture, seed, config).expect("case should regenerate");

    assert_eq!(seed.get(), 1_592_614_637);
    assert_eq!(first.fixture_revision(), 1);
    assert_eq!(first.config(), config);
    assert_eq!(first.seed(), seed);
    assert_eq!(first, second);
}

#[test]
fn selected_unequal_seeds_have_different_seeded_tails() {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let config = GeneratorConfigV1::new(16, 4, 12);

    let seed_zero =
        generate_case_v1(&fixture, SeedV1::new(0), config).expect("seed zero should generate");
    let seed_one =
        generate_case_v1(&fixture, SeedV1::new(1), config).expect("seed one should generate");

    assert_eq!(
        &seed_zero.transactions()[..8],
        &seed_one.transactions()[..8]
    );
    assert_ne!(
        &seed_zero.transactions()[8..],
        &seed_one.transactions()[8..]
    );
}

#[test]
fn selected_seeds_pin_the_first_seeded_transaction() {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let config = GeneratorConfigV1::new(16, 4, 12);
    let root = NodePathV1::root();

    let seed_zero =
        generate_case_v1(&fixture, SeedV1::new(0), config).expect("seed zero should generate");
    assert_eq!(
        flatten_transaction(&seed_zero.transactions()[8]),
        vec![
            expected_operation(
                8,
                10,
                SemanticOperationV1::UpdateKeyed {
                    fragment: FragmentPathV1::new(root.clone().member(1, 7), 1),
                    key: 1,
                    property: PropertyId::new(0),
                    value: PropertyValue::ScalarI32(1),
                },
            ),
            expected_operation(
                8,
                11,
                SemanticOperationV1::InsertKeyed {
                    fragment: FragmentPathV1::new(root.clone().member(1, 8), 1),
                    key: 23,
                    final_index: 1,
                },
            ),
        ]
    );

    let seed_one =
        generate_case_v1(&fixture, SeedV1::new(1), config).expect("seed one should generate");
    assert_eq!(
        flatten_transaction(&seed_one.transactions()[8]),
        vec![
            expected_operation(
                8,
                10,
                SemanticOperationV1::InsertKeyed {
                    fragment: FragmentPathV1::new(root.clone().member(1, 8), 1),
                    key: 2,
                    final_index: 1,
                },
            ),
            expected_operation(
                8,
                11,
                SemanticOperationV1::SetProperty {
                    node: root.member(1, 9),
                    property: PropertyId::new(1),
                    value: PropertyValue::Bool(true),
                },
            ),
        ]
    );
}

#[test]
fn generator_config_accepts_inclusive_minimum_and_maximum() {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let minimum = generate_case_v1(&fixture, SeedV1::new(0), GeneratorConfigV1::new(8, 2, 8))
        .expect("minimum config should be valid");
    let maximum = generate_case_v1(&fixture, SeedV1::new(0), GeneratorConfigV1::new(64, 4, 12))
        .expect("maximum config should be valid");

    assert_eq!(minimum.transactions().len(), 8);
    assert_eq!(minimum.operation_count(), 10);
    assert_eq!(maximum.transactions().len(), 64);
    assert!(maximum.operation_count() <= 256);
    assert!(
        maximum
            .transactions()
            .iter()
            .all(|transaction| transaction.operations().len() <= 4)
    );
}

#[test]
fn generator_config_rejects_fields_below_their_minimum() {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");

    for config in [
        GeneratorConfigV1::new(7, 2, 8),
        GeneratorConfigV1::new(8, 1, 8),
        GeneratorConfigV1::new(8, 2, 7),
    ] {
        let error = generate_case_v1(&fixture, SeedV1::new(0), config)
            .expect_err("config below a minimum should fail");
        assert_eq!(error.kind(), GeneratorErrorKind::InvalidConfig);
    }
}

#[test]
fn generator_config_reports_above_maximum_fields_in_declaration_order() {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let cases = [
        (
            GeneratorConfigV1::new(65, 5, 13),
            HarnessLimitKind::Transactions,
        ),
        (
            GeneratorConfigV1::new(64, 5, 13),
            HarnessLimitKind::OperationsPerTransaction,
        ),
        (
            GeneratorConfigV1::new(64, 4, 13),
            HarnessLimitKind::LiveMemberships,
        ),
    ];

    for (config, limit) in cases {
        let error = generate_case_v1(&fixture, SeedV1::new(0), config)
            .expect_err("config above a fixed ceiling should fail");
        assert_eq!(error.kind(), GeneratorErrorKind::LimitExceeded(limit));
    }
}

fn expected_operation(
    transaction: u32,
    operation: u32,
    semantic: SemanticOperationV1,
) -> (u32, u32, SemanticOperationV1) {
    (transaction, operation, semantic)
}

fn flatten_operations(
    generated: &fenestra_ui_testkit::prototype::GeneratedCaseV1,
) -> Vec<(u32, u32, SemanticOperationV1)> {
    generated
        .transactions()
        .iter()
        .flat_map(|transaction| {
            transaction.operations().iter().map(|operation| {
                (
                    transaction.id().get(),
                    operation.id().get(),
                    operation.operation().clone(),
                )
            })
        })
        .collect()
}

fn flatten_transaction(
    transaction: &fenestra_ui_testkit::prototype::TransactionV1,
) -> Vec<(u32, u32, SemanticOperationV1)> {
    transaction
        .operations()
        .iter()
        .map(|operation| {
            (
                transaction.id().get(),
                operation.id().get(),
                operation.operation().clone(),
            )
        })
        .collect()
}
