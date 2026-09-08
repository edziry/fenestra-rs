use super::*;

#[test]
fn preview_completes_candidate_geometry_without_replacing_the_accepted_projection() {
    let (mut runtime, program, engine) = runtime(4);
    let before = runtime.committed();
    let resized = SpatialViewportV2::new(100, 90);
    let resize = |runtime: &UiRuntime| {
        let mut transaction = runtime.begin_transaction();
        transaction.resize_spatial(resized).unwrap();
        transaction
    };
    let candidate = runtime.preview(resize(&runtime)).unwrap();
    let snapshot = candidate.spatial().unwrap().snapshot();
    assert_eq!(candidate.generation().get(), 1);
    assert_eq!(snapshot.viewport(), resized);
    assert_eq!(
        snapshot.output().geometry()[0].base_width(),
        SpatialScalarV2::new(100 * 65_536)
    );
    assert_eq!(
        snapshot.output().geometry()[0].base_height(),
        SpatialScalarV2::new(90 * 65_536)
    );
    assert_eq!(before.spatial().unwrap().snapshot().viewport(), VIEWPORT);
    assert!(before.shares_state_with(&runtime.committed()));
    assert_eq!(program.calls.load(Ordering::SeqCst), 2);
    assert_eq!(engine.load(Ordering::SeqCst), 2);
    runtime.commit(resize(&runtime)).unwrap();
    assert_eq!(runtime.committed().generation(), candidate.generation());
    assert!(!candidate.shares_state_with(&runtime.committed()));
    assert_eq!(
        runtime
            .committed()
            .spatial()
            .unwrap()
            .snapshot()
            .output()
            .geometry(),
        snapshot.output().geometry()
    );
    assert_eq!(program.calls.load(Ordering::SeqCst), 3);
    drop(candidate);
    assert!(program.sources.lock().unwrap()[1].upgrade().is_none());
}

#[test]
fn noop_preview_skips_rebuild_and_late_rejection_releases_candidate_source() {
    let (mut runtime, program, engine) = runtime(4);
    runtime.set_generation_for_test(u64::MAX);
    let before = runtime.committed();
    let candidate = runtime.preview(runtime.begin_transaction()).unwrap();
    assert!(candidate.shares_state_with(&before));
    assert_eq!(program.calls.load(Ordering::SeqCst), 1);
    let error = runtime
        .preview(changed_transaction(&runtime, 10))
        .unwrap_err();
    assert_eq!(error.kind(), TransactionErrorKind::GenerationExhausted);
    assert!(before.shares_state_with(&runtime.committed()));
    assert_eq!(engine.load(Ordering::SeqCst), 2);
    assert!(program.sources.lock().unwrap()[1].upgrade().is_none());
}

#[test]
fn invalid_projection_preview_matches_commit_rejection_and_keeps_retry_valid() {
    let (mut runtime, program, _) = runtime(4);
    let before = runtime.committed();
    let rejected = |runtime: &UiRuntime| {
        let mut transaction = runtime.begin_transaction();
        transaction
            .resize_spatial(SpatialViewportV2::new(-1, 60))
            .unwrap();
        transaction
    };
    let preview_error = runtime.preview(rejected(&runtime)).unwrap_err();
    assert!(matches!(
        preview_error.kind(),
        TransactionErrorKind::Spatial(_)
    ));
    assert_eq!(
        preview_error,
        runtime.commit(rejected(&runtime)).unwrap_err()
    );
    assert!(before.shares_state_with(&runtime.committed()));
    let sources = program.sources.lock().unwrap();
    assert!(sources[1].upgrade().is_none());
    assert!(sources[2].upgrade().is_none());
    drop(sources);
    runtime.commit(changed_transaction(&runtime, 10)).unwrap();
    assert_eq!(runtime.committed().generation().get(), 1);
}
