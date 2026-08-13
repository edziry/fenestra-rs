use fenestra_ui_exp_0014_windows_gpu::{
    ArtifactAdapterV1, ArtifactEventV1, ArtifactPresentV1, ArtifactSurfaceV1, ArtifactTerminalV1,
    GpuBackendV1, GpuDeviceTypeV1, GpuSurfaceExtentV1, GpuTargetV1, InteractiveArtifactBuilderV1,
    InteractiveMilestoneV1, InteractiveResultV1, SurfaceAlphaV1, SurfaceFormatV1,
    SurfacePresentModeV1, verify_interactive_artifact_v1,
};

#[test]
fn typed_builder_roundtrips_a_complete_pass_without_raw_environment_text() {
    let mut builder = InteractiveArtifactBuilderV1::new(GpuTargetV1::WindowsDx12, b"11.0")
        .expect("bounded run metadata");
    builder
        .record_adapter(ArtifactAdapterV1::new(
            GpuBackendV1::Dx12,
            GpuDeviceTypeV1::Integrated,
            4098,
            5686,
            b"AMD",
            b"amd",
            b"31.0",
        ))
        .expect("admitted adapter");
    builder
        .record_surface(ArtifactSurfaceV1::new(
            SurfaceFormatV1::Bgra8Unorm,
            SurfacePresentModeV1::Fifo,
            SurfaceAlphaV1::Opaque,
        ))
        .expect("admitted surface");
    builder
        .observe(ArtifactEventV1::Adapter)
        .expect("adapter milestone");
    builder
        .observe(ArtifactEventV1::Present(ArtifactPresentV1::new(
            InteractiveMilestoneV1::InitialPresent,
            0,
            0,
            0,
            GpuSurfaceExtentV1::new(192, 128),
            0x0123_4567_89ab_cdef,
        )))
        .expect("initial present");
    builder
        .observe(ArtifactEventV1::PointerMove)
        .expect("pointer move");
    builder
        .observe(ArtifactEventV1::PointerPress)
        .expect("pointer press");
    builder
        .observe(ArtifactEventV1::Present(ArtifactPresentV1::new(
            InteractiveMilestoneV1::MutationPresent,
            1,
            1,
            1,
            GpuSurfaceExtentV1::new(192, 128),
            0xfedc_ba98_7654_3210,
        )))
        .expect("mutation present");
    builder
        .observe(ArtifactEventV1::Resize(GpuSurfaceExtentV1::new(224, 160)))
        .expect("resize");
    builder
        .observe(ArtifactEventV1::Present(ArtifactPresentV1::new(
            InteractiveMilestoneV1::ResizePresent,
            2,
            2,
            2,
            GpuSurfaceExtentV1::new(224, 160),
            0x0011_2233_4455_6677,
        )))
        .expect("resize present");
    builder.observe(ArtifactEventV1::Suspend).expect("suspend");
    builder.observe(ArtifactEventV1::Restore).expect("restore");
    builder
        .observe(ArtifactEventV1::Present(ArtifactPresentV1::new(
            InteractiveMilestoneV1::RestorePresent,
            2,
            3,
            3,
            GpuSurfaceExtentV1::new(224, 160),
            0x0011_2233_4455_6677,
        )))
        .expect("restore present");
    builder.observe(ArtifactEventV1::Close).expect("close");

    let bytes = builder
        .finish(ArtifactTerminalV1::Pass)
        .expect("complete verified artifact");
    let verified = verify_interactive_artifact_v1(&bytes).expect("fresh encoding verifies");
    assert_eq!(verified.result(), InteractiveResultV1::Pass);
    let text = std::str::from_utf8(&bytes).expect("ASCII artifact");
    assert!(text.contains("name-hex=414d44"));
    assert!(!text.contains("|name=AMD"));
}

#[test]
fn builder_is_atomic_for_out_of_order_events_and_false_passes() {
    let mut builder = InteractiveArtifactBuilderV1::new(GpuTargetV1::WindowsDx12, b"11.0")
        .expect("bounded run metadata");
    let before = builder.record_count();
    builder
        .observe(ArtifactEventV1::PointerMove)
        .expect_err("pointer cannot precede adapter");
    assert_eq!(builder.record_count(), before);
    builder
        .finish(ArtifactTerminalV1::Pass)
        .expect_err("incomplete sequence cannot pass");
}
