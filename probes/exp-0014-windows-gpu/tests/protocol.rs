use fenestra_ui_exp_0014_windows_gpu::{
    GpuAdapterObservationV1, GpuAdmissionErrorKindV1, GpuBackendV1, GpuDeviceTypeV1, GpuTargetV1,
    InteractiveEvidenceErrorKindV1, InteractiveEvidenceV1, InteractiveMilestoneV1,
    InteractiveObservationV1, InteractiveResultV1, admit_adapter_v1,
};

#[test]
fn target_requires_its_exact_backend_and_a_hardware_device() {
    let dx12 = adapter(
        GpuBackendV1::Dx12,
        GpuDeviceTypeV1::Discrete,
        0x10de,
        0x2684,
    );
    let vulkan = adapter(
        GpuBackendV1::Vulkan,
        GpuDeviceTypeV1::Integrated,
        0x1002,
        0x164c,
    );

    assert_eq!(admit_adapter_v1(GpuTargetV1::WindowsDx12, dx12), Ok(()));
    assert_eq!(admit_adapter_v1(GpuTargetV1::LinuxVulkan, vulkan), Ok(()));
    assert_eq!(
        admit_adapter_v1(GpuTargetV1::WindowsDx12, vulkan),
        Err(GpuAdmissionErrorKindV1::Backend)
    );
}

#[test]
fn backend_failure_precedes_device_and_identity_failures() {
    let wrong = adapter(GpuBackendV1::Vulkan, GpuDeviceTypeV1::Cpu, 0, 0);
    assert_eq!(
        admit_adapter_v1(GpuTargetV1::WindowsDx12, wrong),
        Err(GpuAdmissionErrorKindV1::Backend)
    );

    for kind in [
        GpuDeviceTypeV1::Other,
        GpuDeviceTypeV1::Virtual,
        GpuDeviceTypeV1::Cpu,
    ] {
        let adapter = adapter(GpuBackendV1::Dx12, kind, 1, 1);
        assert_eq!(
            admit_adapter_v1(GpuTargetV1::WindowsDx12, adapter),
            Err(GpuAdmissionErrorKindV1::DeviceType)
        );
    }

    let missing_identity = adapter(GpuBackendV1::Dx12, GpuDeviceTypeV1::Discrete, 0, 0);
    assert_eq!(
        admit_adapter_v1(GpuTargetV1::WindowsDx12, missing_identity),
        Err(GpuAdmissionErrorKindV1::Identity)
    );
}

#[test]
fn exact_interactive_sequence_reaches_pass() {
    let mut evidence = InteractiveEvidenceV1::new();
    for observation in complete_sequence() {
        evidence
            .observe(observation)
            .expect("the exact sequence should be accepted");
    }

    assert_eq!(evidence.result(), Some(InteractiveResultV1::Pass));
    assert_eq!(evidence.next_required(), None);
    assert_eq!(
        evidence.milestones(),
        InteractiveMilestoneV1::ALL.as_slice()
    );
}

#[test]
fn mutation_requires_a_newer_generation() {
    let mut evidence = InteractiveEvidenceV1::new();
    evidence
        .observe(InteractiveObservationV1::Adapter)
        .expect("adapter should start the sequence");
    evidence
        .observe(InteractiveObservationV1::InitialPresent { generation: 4 })
        .expect("initial present should retain its generation");
    evidence
        .observe(InteractiveObservationV1::PointerMove)
        .expect("pointer move should advance");
    evidence
        .observe(InteractiveObservationV1::PointerPress)
        .expect("pointer press should advance");

    assert_eq!(
        evidence.observe(InteractiveObservationV1::MutationPresent { generation: 4 }),
        Err(InteractiveEvidenceErrorKindV1::Generation)
    );
    assert_eq!(
        evidence.next_required(),
        Some(InteractiveMilestoneV1::MutationPresent)
    );
}

#[test]
fn unexpected_or_duplicate_milestone_is_typed_and_atomic() {
    let mut evidence = InteractiveEvidenceV1::new();
    assert_eq!(
        evidence.observe(InteractiveObservationV1::PointerMove),
        Err(InteractiveEvidenceErrorKindV1::Order)
    );
    assert!(evidence.milestones().is_empty());

    evidence
        .observe(InteractiveObservationV1::Adapter)
        .expect("adapter should start the sequence");
    assert_eq!(
        evidence.observe(InteractiveObservationV1::Adapter),
        Err(InteractiveEvidenceErrorKindV1::Order)
    );
    assert_eq!(evidence.milestones(), &[InteractiveMilestoneV1::Adapter]);
}

#[test]
fn early_close_is_a_stop_with_the_first_missing_milestone() {
    let mut evidence = InteractiveEvidenceV1::new();
    evidence
        .observe(InteractiveObservationV1::Adapter)
        .expect("adapter should start the sequence");
    evidence
        .observe(InteractiveObservationV1::InitialPresent { generation: 0 })
        .expect("initial present should advance");
    evidence
        .observe(InteractiveObservationV1::Close)
        .expect("early close should form a terminal stop");

    assert_eq!(evidence.result(), Some(InteractiveResultV1::Stop));
    assert_eq!(
        evidence.next_required(),
        Some(InteractiveMilestoneV1::PointerMove)
    );
    assert_eq!(
        evidence.observe(InteractiveObservationV1::PointerMove),
        Err(InteractiveEvidenceErrorKindV1::Terminal)
    );
}

fn adapter(
    backend: GpuBackendV1,
    device_type: GpuDeviceTypeV1,
    vendor: u32,
    device: u32,
) -> GpuAdapterObservationV1 {
    GpuAdapterObservationV1::new(backend, device_type, vendor, device)
}

fn complete_sequence() -> [InteractiveObservationV1; 11] {
    [
        InteractiveObservationV1::Adapter,
        InteractiveObservationV1::InitialPresent { generation: 4 },
        InteractiveObservationV1::PointerMove,
        InteractiveObservationV1::PointerPress,
        InteractiveObservationV1::MutationPresent { generation: 5 },
        InteractiveObservationV1::Resize,
        InteractiveObservationV1::ResizePresent { generation: 6 },
        InteractiveObservationV1::Suspend,
        InteractiveObservationV1::Restore,
        InteractiveObservationV1::RestorePresent { generation: 6 },
        InteractiveObservationV1::Close,
    ]
}
