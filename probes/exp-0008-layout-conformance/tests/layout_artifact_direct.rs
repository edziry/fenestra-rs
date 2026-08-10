#![forbid(unsafe_code)]

#[path = "layout_artifact/direct.rs"]
mod direct;

#[test]
fn direct_artifact_model_requires_the_canonical_renderer() {
    direct::assert_direct_artifact_contract(|model| {
        let mut artifact = direct::direct_lines_v1(model).join("\n");
        artifact.push('\n');
        artifact.into_bytes()
    });
}
