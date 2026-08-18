use fenestra_ui_exp_0014_windows_gpu::inspect_registered_scene_pair_v1;

#[test]
fn registered_fen_frame_becomes_one_exact_vello_image_scene() {
    let [initial, mutated] =
        inspect_registered_scene_pair_v1().expect("registered scene pair should build");

    assert_eq!(initial.generation(), 0);
    assert_eq!(initial.width(), 192);
    assert_eq!(initial.height(), 128);
    assert_eq!(initial.rgba_bytes(), 192 * 128 * 4);
    assert_eq!(initial.scene_commands(), 1);
    assert!(initial.used_vello_scene());
    assert!(!initial.executed_gpu());

    assert_eq!(mutated.generation(), 1);
    assert_eq!(mutated.width(), initial.width());
    assert_eq!(mutated.height(), initial.height());
    assert_eq!(mutated.rgba_bytes(), initial.rgba_bytes());
    assert_eq!(mutated.scene_commands(), 1);
    assert!(mutated.used_vello_scene());
    assert!(!mutated.executed_gpu());
    assert_ne!(mutated.raster_digest(), initial.raster_digest());
    assert_ne!(mutated.scene_fingerprint(), initial.scene_fingerprint());
}

#[test]
fn two_fresh_scene_pairs_are_identical() {
    let first = inspect_registered_scene_pair_v1().expect("first scene pair should build");
    let second = inspect_registered_scene_pair_v1().expect("second scene pair should build");
    assert_eq!(first, second);
}
