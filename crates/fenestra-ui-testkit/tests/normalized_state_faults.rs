use fenestra_ui_testkit::prototype::{
    DesiredStateV1, HeadlessFixtureV1, HeadlessOracleV1, NormalizedChildGroupV1,
    NormalizedHeadlessProjectionV1, NormalizedStateFaultV1, NormalizedStateV1,
    RuntimeOracleFixtureV1, clean_rebuild_v1, inject_headless_surface_fault_v1,
    inject_normalized_state_fault_v1,
};

fn initial_state() -> NormalizedStateV1 {
    let fixture = RuntimeOracleFixtureV1::build().expect("registered fixture should validate");
    let desired =
        DesiredStateV1::from_construction(fixture.construction(), fixture.harness_limits())
            .expect("registered desired state should fit");
    clean_rebuild_v1(fixture.construction(), &desired, fixture.harness_limits())
        .expect("registered state should normalize")
}

fn initial_projection() -> NormalizedHeadlessProjectionV1 {
    let fixture = HeadlessFixtureV1::build().expect("registered fixture should validate");
    HeadlessOracleV1::new(&fixture)
        .expect("registered oracle should initialize")
        .rebuild()
        .expect("registered projection should rebuild")
}

#[test]
fn normalized_state_fault_registry_is_closed_and_typed() {
    assert_eq!(
        NormalizedStateFaultV1::ALL,
        [
            NormalizedStateFaultV1::NodePath,
            NormalizedStateFaultV1::NodeParent,
            NormalizedStateFaultV1::NodeTemplate,
            NormalizedStateFaultV1::NodeComponent,
            NormalizedStateFaultV1::NodeOrder,
            NormalizedStateFaultV1::PropertyOrder,
            NormalizedStateFaultV1::PropertyId,
            NormalizedStateFaultV1::PropertyValue,
            NormalizedStateFaultV1::ChildOrder,
            NormalizedStateFaultV1::ChildKind,
            NormalizedStateFaultV1::ChildTarget,
            NormalizedStateFaultV1::FragmentPath,
            NormalizedStateFaultV1::FragmentDescriptor,
            NormalizedStateFaultV1::MemberOrder,
            NormalizedStateFaultV1::MemberKey,
            NormalizedStateFaultV1::MemberPath,
        ]
    );

    let baseline = initial_state();
    for fault in NormalizedStateFaultV1::ALL {
        let faulted = inject_normalized_state_fault_v1(&baseline, fault)
            .expect("registered typed state fault should apply");
        assert_ne!(faulted, baseline, "missed {fault:?}");
        assert_typed_field_changed(&baseline, &faulted, fault);
    }
}

#[test]
fn surface_fault_is_separate_and_preserves_all_five_projection_families() {
    let baseline = initial_projection();
    let faulted = inject_headless_surface_fault_v1(&baseline)
        .expect("registered typed surface fault should apply");

    assert_ne!(faulted.surface(), baseline.surface());
    assert_eq!(faulted.computed_styles(), baseline.computed_styles());
    assert_eq!(faulted.geometries(), baseline.geometries());
    assert_eq!(faulted.semantics(), baseline.semantics());
    assert_eq!(faulted.hit_regions(), baseline.hit_regions());
    assert_eq!(faulted.scene_rectangles(), baseline.scene_rectangles());
}

fn assert_typed_field_changed(
    baseline: &NormalizedStateV1,
    faulted: &NormalizedStateV1,
    fault: NormalizedStateFaultV1,
) {
    match fault {
        NormalizedStateFaultV1::NodePath => {
            assert_ne!(faulted.nodes()[0].path(), baseline.nodes()[0].path());
            assert_eq!(faulted.nodes()[0].parent(), baseline.nodes()[0].parent());
        }
        NormalizedStateFaultV1::NodeParent => {
            assert_ne!(faulted.nodes()[1].parent(), baseline.nodes()[1].parent());
            assert_eq!(faulted.nodes()[1].path(), baseline.nodes()[1].path());
        }
        NormalizedStateFaultV1::NodeTemplate => {
            assert_ne!(
                faulted.nodes()[0].template(),
                baseline.nodes()[0].template()
            );
            assert_eq!(
                faulted.nodes()[0].component(),
                baseline.nodes()[0].component()
            );
        }
        NormalizedStateFaultV1::NodeComponent => {
            assert_ne!(
                faulted.nodes()[0].component(),
                baseline.nodes()[0].component()
            );
            assert_eq!(
                faulted.nodes()[0].template(),
                baseline.nodes()[0].template()
            );
        }
        NormalizedStateFaultV1::NodeOrder => {
            assert_eq!(faulted.nodes()[0], baseline.nodes()[1]);
            assert_eq!(faulted.nodes()[1], baseline.nodes()[0]);
        }
        NormalizedStateFaultV1::PropertyOrder => {
            assert_eq!(
                faulted.nodes()[0].properties()[0],
                baseline.nodes()[0].properties()[1]
            );
            assert_eq!(
                faulted.nodes()[0].properties()[1],
                baseline.nodes()[0].properties()[0]
            );
        }
        NormalizedStateFaultV1::PropertyId => {
            assert_ne!(
                faulted.nodes()[0].properties()[0].property(),
                baseline.nodes()[0].properties()[0].property()
            );
            assert_eq!(
                faulted.nodes()[0].properties()[0].value(),
                baseline.nodes()[0].properties()[0].value()
            );
        }
        NormalizedStateFaultV1::PropertyValue => {
            assert_eq!(
                faulted.nodes()[0].properties()[0].property(),
                baseline.nodes()[0].properties()[0].property()
            );
            assert_ne!(
                faulted.nodes()[0].properties()[0].value(),
                baseline.nodes()[0].properties()[0].value()
            );
        }
        NormalizedStateFaultV1::ChildOrder => {
            assert_eq!(
                faulted.nodes()[0].child_groups()[0],
                baseline.nodes()[0].child_groups()[1]
            );
            assert_eq!(
                faulted.nodes()[0].child_groups()[1],
                baseline.nodes()[0].child_groups()[0]
            );
        }
        NormalizedStateFaultV1::ChildKind => {
            assert_ne!(
                child_kind(&faulted.nodes()[0].child_groups()[0]),
                child_kind(&baseline.nodes()[0].child_groups()[0])
            );
        }
        NormalizedStateFaultV1::ChildTarget => {
            assert_eq!(
                child_kind(&faulted.nodes()[0].child_groups()[0]),
                child_kind(&baseline.nodes()[0].child_groups()[0])
            );
            assert_ne!(
                faulted.nodes()[0].child_groups()[0],
                baseline.nodes()[0].child_groups()[0]
            );
        }
        NormalizedStateFaultV1::FragmentPath => {
            assert_ne!(
                faulted.fragments()[0].path(),
                baseline.fragments()[0].path()
            );
            assert_eq!(
                faulted.fragments()[0].descriptor(),
                baseline.fragments()[0].descriptor()
            );
        }
        NormalizedStateFaultV1::FragmentDescriptor => {
            assert_eq!(
                faulted.fragments()[0].path(),
                baseline.fragments()[0].path()
            );
            assert_ne!(
                faulted.fragments()[0].descriptor(),
                baseline.fragments()[0].descriptor()
            );
        }
        NormalizedStateFaultV1::MemberOrder => {
            assert_eq!(
                faulted.fragments()[0].members()[0],
                baseline.fragments()[0].members()[1]
            );
            assert_eq!(
                faulted.fragments()[0].members()[1],
                baseline.fragments()[0].members()[0]
            );
        }
        NormalizedStateFaultV1::MemberKey => {
            assert_ne!(
                faulted.fragments()[0].members()[0].key(),
                baseline.fragments()[0].members()[0].key()
            );
            assert_eq!(
                faulted.fragments()[0].members()[0].node(),
                baseline.fragments()[0].members()[0].node()
            );
        }
        NormalizedStateFaultV1::MemberPath => {
            assert_eq!(
                faulted.fragments()[0].members()[0].key(),
                baseline.fragments()[0].members()[0].key()
            );
            assert_ne!(
                faulted.fragments()[0].members()[0].node(),
                baseline.fragments()[0].members()[0].node()
            );
        }
    }
}

const fn child_kind(child: &NormalizedChildGroupV1) -> u8 {
    match child {
        NormalizedChildGroupV1::Static(_) => 0,
        NormalizedChildGroupV1::Region(_) => 1,
    }
}
