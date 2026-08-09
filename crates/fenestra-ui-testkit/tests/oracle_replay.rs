use fenestra_ui_testkit::prototype::{
    GeneratorConfigV1, RuntimeOracleFixtureV1, SeedV1, generate_case_v1, replay_case_v1,
};

#[test]
fn directed_replay_reports_publication_and_identity_lifecycle() {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let generated = generate_case_v1(&fixture, SeedV1::new(0), GeneratorConfigV1::new(8, 2, 8))
        .expect("directed case should generate");

    let report = replay_case_v1(&fixture, &generated).expect("directed case should replay");

    assert_eq!(report.transaction_count(), 8);
    assert_eq!(report.operation_count(), 10);
    assert_eq!(report.verified_step_count(), 8);
    assert_eq!(report.publication_count(), 7);
    assert_eq!(report.noop_count(), 1);
    assert_eq!(report.final_generation(), 7);
    assert_eq!(report.identity().preserved(), 116);
    assert_eq!(report.identity().retired(), 5);
    assert_eq!(report.identity().fresh(), 4);
    assert_eq!(report.identity().alias_free_snapshots(), 9);
}

#[test]
fn known_case_verifies_every_replay_step() {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let generated = generate_case_v1(
        &fixture,
        SeedV1::new(1_592_614_637),
        GeneratorConfigV1::new(16, 4, 12),
    )
    .expect("known case should generate");

    let report = replay_case_v1(&fixture, &generated).expect("known case should replay");

    assert_eq!(report.transaction_count(), 16);
    assert_eq!(report.operation_count(), generated.operation_count());
    assert_eq!(report.verified_step_count(), 16);
    assert_eq!(report.publication_count() + report.noop_count(), 16);
    assert_eq!(report.final_generation(), report.publication_count() as u64);
    assert_eq!(report.identity().alias_free_snapshots(), 17);
}

#[test]
fn fixed_seed_corpus_replays_with_one_retained_generation() {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let config = GeneratorConfigV1::new(64, 4, 12);

    for seed in 0..=31_u64 {
        let generated = generate_case_v1(&fixture, SeedV1::new(seed), config)
            .expect("corpus case should generate");
        let report = replay_case_v1(&fixture, &generated).expect("corpus case should replay");

        assert_eq!(report.transaction_count(), 64, "seed {seed}");
        assert_eq!(
            report.operation_count(),
            generated.operation_count(),
            "seed {seed}"
        );
        assert!(report.operation_count() <= 256, "seed {seed}");
        assert_eq!(report.verified_step_count(), 64, "seed {seed}");
        assert_eq!(
            report.publication_count() + report.noop_count(),
            64,
            "seed {seed}"
        );
        assert_eq!(
            report.final_generation(),
            report.publication_count() as u64,
            "seed {seed}"
        );
        assert_eq!(report.identity().alias_free_snapshots(), 65, "seed {seed}");
    }
}
