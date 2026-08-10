use fenestra_ui_testkit::prototype::inject_headless_projection_fault_v1;

use super::{
    RuntimeArtifactEncodeErrorV1, RuntimeArtifactFaultV1, RuntimeArtifactModelV1, invalid_log,
};

pub(super) fn inject(
    model: &RuntimeArtifactModelV1,
    fault: RuntimeArtifactFaultV1,
) -> Result<RuntimeArtifactModelV1, RuntimeArtifactEncodeErrorV1> {
    let mut faulted = model.clone();
    match fault {
        RuntimeArtifactFaultV1::ReceiptGeneration => replace_receipt(
            &mut faulted,
            1,
            "receipt|begin|",
            "|generation=1|",
            "|generation=9|",
        )?,
        RuntimeArtifactFaultV1::ReceiptInvalidation => replace_receipt(
            &mut faulted,
            1,
            "receipt|begin|",
            "|invalidates=paint",
            "|invalidates=layout",
        )?,
        RuntimeArtifactFaultV1::MutationPath => replace_receipt(
            &mut faulted,
            1,
            "mutation|i=0|kind=property|",
            "|node=root/s:0/s:0|",
            "|node=root/s:0/s:9|",
        )?,
        RuntimeArtifactFaultV1::MutationProperty => replace_receipt(
            &mut faulted,
            1,
            "mutation|i=0|kind=property|",
            "|property=2|",
            "|property=9|",
        )?,
        RuntimeArtifactFaultV1::MutationValue => replace_receipt(
            &mut faulted,
            1,
            "mutation|i=0|kind=property|",
            "|new=rgba8:20,30,40,255",
            "|new=rgba8:21,30,40,255",
        )?,
        RuntimeArtifactFaultV1::MutationKey => replace_receipt(
            &mut faulted,
            2,
            "mutation|i=0|kind=insert|",
            "|key=30|",
            "|key=31|",
        )?,
        RuntimeArtifactFaultV1::MutationRoot => replace_receipt(
            &mut faulted,
            2,
            "mutation|i=0|kind=insert|",
            "|root=root/s:0/m:1:30|",
            "|root=root/s:0/m:1:31|",
        )?,
        RuntimeArtifactFaultV1::MutationIndices => replace_receipt(
            &mut faulted,
            3,
            "mutation|i=0|kind=move|",
            "|old=1|final=2",
            "|old=2|final=1",
        )?,
        RuntimeArtifactFaultV1::CreatedManifest => replace_receipt(
            &mut faulted,
            2,
            "manifest|mutation=0|kind=created|",
            "node=root/s:0/m:1:30",
            "node=root/s:0/m:1:31",
        )?,
        RuntimeArtifactFaultV1::RetiredManifest => replace_receipt(
            &mut faulted,
            5,
            "manifest|mutation=0|kind=retired|",
            "node=root/s:0/m:1:20",
            "node=root/s:0/m:1:21",
        )?,
        RuntimeArtifactFaultV1::StateNodeParent => replace_state(
            &mut faulted,
            "node|i=1|path=root/s:0|",
            "|parent=root|",
            "|parent=root/s:9|",
        )?,
        RuntimeArtifactFaultV1::StateNodeTemplate => replace_state(
            &mut faulted,
            "node|i=1|path=root/s:0|",
            "|template=1|",
            "|template=9|",
        )?,
        RuntimeArtifactFaultV1::StateNodeComponent => replace_state(
            &mut faulted,
            "node|i=1|path=root/s:0|",
            "|component=0",
            "|component=9",
        )?,
        RuntimeArtifactFaultV1::StateNodeOrder => swap_state(
            &mut faulted,
            "node|i=0|path=root|",
            "node|i=1|path=root/s:0|",
        )?,
        RuntimeArtifactFaultV1::StatePropertyId => {
            replace_state(&mut faulted, "property|node=root|i=0|", "|id=0|", "|id=9|")?
        }
        RuntimeArtifactFaultV1::StatePropertyValue => replace_state(
            &mut faulted,
            "property|node=root|i=0|",
            "|value=scalar-i32:100",
            "|value=scalar-i32:101",
        )?,
        RuntimeArtifactFaultV1::StateChildKind => replace_state(
            &mut faulted,
            "child|node=root/s:0|i=1|",
            "|kind=region|",
            "|kind=static|",
        )?,
        RuntimeArtifactFaultV1::StateChildTarget => replace_state(
            &mut faulted,
            "child|node=root|i=0|",
            "|target=root/s:0",
            "|target=root/s:9",
        )?,
        RuntimeArtifactFaultV1::StateFragmentDescriptor => replace_state(
            &mut faulted,
            "fragment|i=0|path=root/s:0/r:1|",
            "|descriptor=0",
            "|descriptor=9",
        )?,
        RuntimeArtifactFaultV1::StateMemberKey => replace_state(
            &mut faulted,
            "member|fragment=root/s:0/r:1|i=0|",
            "|key=10|",
            "|key=11|",
        )?,
        RuntimeArtifactFaultV1::StateMemberPath => replace_state(
            &mut faulted,
            "member|fragment=root/s:0/r:1|i=0|",
            "|node=root/s:0/m:1:10",
            "|node=root/s:0/m:1:11",
        )?,
        RuntimeArtifactFaultV1::StateMemberOrder => swap_state(
            &mut faulted,
            "member|fragment=root/s:0/r:1|i=0|",
            "member|fragment=root/s:0/r:1|i=1|",
        )?,
        RuntimeArtifactFaultV1::Surface => {
            replace_projection(&mut faulted, "surface|", "width=120", "width=121")?
        }
        RuntimeArtifactFaultV1::Projection(projection_fault) => {
            let source = faulted.projection_sources.first().ok_or_else(invalid_log)?;
            let projection = inject_headless_projection_fault_v1(source, projection_fault)
                .map_err(|_| invalid_log())?;
            let lines = super::encode::collect_projection(&projection)?;
            faulted
                .generations
                .first_mut()
                .ok_or_else(invalid_log)?
                .projection = lines;
            faulted.projection_sources[0] = projection;
        }
    }
    Ok(faulted)
}

fn replace_receipt(
    model: &mut RuntimeArtifactModelV1,
    generation: usize,
    prefix: &str,
    needle: &str,
    replacement: &str,
) -> Result<(), RuntimeArtifactEncodeErrorV1> {
    let lines = &mut model
        .generations
        .get_mut(generation)
        .ok_or_else(invalid_log)?
        .receipt;
    replace_one(lines, prefix, needle, replacement)
}

fn replace_state(
    model: &mut RuntimeArtifactModelV1,
    prefix: &str,
    needle: &str,
    replacement: &str,
) -> Result<(), RuntimeArtifactEncodeErrorV1> {
    let lines = &mut model.generations.first_mut().ok_or_else(invalid_log)?.state;
    replace_one(lines, prefix, needle, replacement)
}

fn replace_projection(
    model: &mut RuntimeArtifactModelV1,
    prefix: &str,
    needle: &str,
    replacement: &str,
) -> Result<(), RuntimeArtifactEncodeErrorV1> {
    let lines = &mut model
        .generations
        .first_mut()
        .ok_or_else(invalid_log)?
        .projection;
    replace_one(lines, prefix, needle, replacement)
}

fn swap_state(
    model: &mut RuntimeArtifactModelV1,
    first: &str,
    second: &str,
) -> Result<(), RuntimeArtifactEncodeErrorV1> {
    let lines = &mut model.generations.first_mut().ok_or_else(invalid_log)?.state;
    let first = unique_index(lines, first, "")?;
    let second = unique_index(lines, second, "")?;
    lines.swap(first, second);
    Ok(())
}

fn replace_one(
    lines: &mut [String],
    prefix: &str,
    needle: &str,
    replacement: &str,
) -> Result<(), RuntimeArtifactEncodeErrorV1> {
    let index = unique_index(lines, prefix, needle)?;
    if lines[index].matches(needle).count() != 1 {
        return Err(invalid_log());
    }
    lines[index] = lines[index].replacen(needle, replacement, 1);
    Ok(())
}

fn unique_index(
    lines: &[String],
    prefix: &str,
    needle: &str,
) -> Result<usize, RuntimeArtifactEncodeErrorV1> {
    let mut matches = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.starts_with(prefix) && line.contains(needle));
    let (index, _) = matches.next().ok_or_else(invalid_log)?;
    if matches.next().is_some() {
        return Err(invalid_log());
    }
    Ok(index)
}
