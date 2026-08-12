use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn prepared_and_snapshot_values_have_only_the_staged_public_surface() {
    let source = all_source();
    assert_private_fields(&source, "PreparedSpatialV2");
    assert_private_fields(&source, "SpatialResolvedSnapshotV2");
    assert_nongeneric_struct(&source, "SpatialResolvedSnapshotV2");
    assert!(
        implementation_blocks(&source, "PreparedSpatialV2")
            .iter()
            .all(|block| !block.lines().map(str::trim).any(is_public_associated_item))
    );

    let function = public_function(&source, "prepare_spatial_v2");
    assert!(has_must_use(&source, function.start));
    let function = public_function(&source, "materialize_reference_spatial_v2");
    assert!(has_must_use(&source, function.start));
    let one_shot = public_function(&source, "resolve_spatial_v2");
    assert!(has_must_use(&source, one_shot.start));
    assert_one_shot_delegation(one_shot._body);

    let mut methods = public_method_declarations(&source, "SpatialResolvedSnapshotV2");
    methods.sort_unstable();
    assert_eq!(
        methods,
        vec![
            "pub const fn viewport",
            "pub fn effective_clip_aabbs",
            "pub fn output",
        ]
    );
    assert!(source.contains("pub const fn viewport(&self) -> SpatialViewportV2"));
    assert!(source.contains("pub fn output(&self) -> SpatialOutputV2<'_>"));
    assert!(source.contains("pub fn effective_clip_aabbs(&self) -> &[SpatialAabbV2]"));
    for method in ["viewport", "output", "effective_clip_aabbs"] {
        let item = public_method(&source, "SpatialResolvedSnapshotV2", method);
        assert!(has_must_use(&source, item.start));
    }

    let forbidden = "validate_spatial_output_v2";
    assert!(!source.contains(&format!("pub struct {forbidden}")));
    assert!(!source.contains(&format!("pub fn {forbidden}")));
}

fn assert_one_shot_delegation(body: &str) {
    assert_eq!(body.matches("prepare_spatial_v2").count(), 1);
    assert_eq!(body.matches("materialize_reference_spatial_v2").count(), 1);
    let prepare = body.find("prepare_spatial_v2").expect("preparation call");
    let materialize = body
        .find("materialize_reference_spatial_v2")
        .expect("materialization call");
    assert!(prepare < materialize);
    for forbidden in [
        "prepare_phase_10",
        "materialize_tables",
        "validate_spatial_output_v2",
        "as_input(",
        "Arc::clone",
    ] {
        assert!(!body.contains(forbidden), "one-shot duplicated {forbidden}");
    }
}

fn public_method<'a>(source: &'a str, owner: &str, name: &str) -> SourceItem<'a> {
    implementation_blocks(source, owner)
        .into_iter()
        .find_map(|block| {
            let plain = format!("pub fn {name}");
            let constant = format!("pub const fn {name}");
            let relative = block.find(&plain).or_else(|| block.find(&constant))?;
            let absolute = block.as_ptr() as usize - source.as_ptr() as usize + relative;
            Some(SourceItem {
                start: absolute,
                _body: block,
            })
        })
        .unwrap_or_else(|| panic!("missing {owner}::{name}"))
}

fn public_method_declarations<'a>(source: &'a str, owner: &str) -> Vec<&'a str> {
    implementation_blocks(source, owner)
        .into_iter()
        .flat_map(str::lines)
        .map(str::trim)
        .filter(|line| line.starts_with("pub "))
        .map(|line| line.split('(').next().expect("method declaration"))
        .collect()
}

fn assert_nongeneric_struct(source: &str, name: &str) {
    let marker = format!("pub struct {name}");
    let declaration = &source[source.find(&marker).expect("snapshot struct") + marker.len()..];
    assert!(
        matches!(
            declaration.trim_start().chars().next(),
            Some('{') | Some('(')
        ),
        "{name} must not have lifetime or type parameters"
    );
}

struct SourceItem<'a> {
    start: usize,
    _body: &'a str,
}

fn public_function<'a>(source: &'a str, name: &str) -> SourceItem<'a> {
    let marker = format!("pub fn {name}");
    let start = source
        .find(&marker)
        .unwrap_or_else(|| panic!("missing {name}"));
    let declaration = &source[start..];
    let brace = declaration.find('{').expect("function body");
    let end = matching_brace(declaration, brace);
    SourceItem {
        start,
        _body: &declaration[..=end],
    }
}

fn assert_private_fields(source: &str, name: &str) {
    let marker = format!("pub struct {name}");
    let declaration = &source[source.find(&marker).expect("prepared struct")..];
    let brace = declaration.find('{');
    let tuple = declaration.find('(');
    match (brace, tuple) {
        (Some(brace), None) => assert_braced_fields_private(declaration, brace),
        (Some(brace), Some(tuple)) if brace < tuple => {
            assert_braced_fields_private(declaration, brace);
        }
        (_, Some(tuple)) => {
            let end = declaration[tuple..].find(';').expect("tuple struct end") + tuple;
            let fields = &declaration[tuple + 1..end];
            assert!(!fields.contains("pub ") && !fields.contains("pub("));
        }
        _ => panic!("unsupported prepared struct"),
    }
}

fn assert_braced_fields_private(declaration: &str, brace: usize) {
    let end = matching_brace(declaration, brace);
    assert!(
        !declaration[brace + 1..end]
            .lines()
            .any(|line| line.trim_start().starts_with("pub"))
    );
}

fn implementation_blocks<'a>(source: &'a str, name: &str) -> Vec<&'a str> {
    let mut blocks = Vec::new();
    let mut remaining = source;
    while let Some(start) = remaining
        .lines()
        .scan(0_usize, |offset, line| {
            let start = *offset;
            *offset += line.len() + 1;
            Some((start, line.trim_start()))
        })
        .find_map(|(offset, line)| {
            (line.starts_with("impl") && line.contains(name) && !line.contains(" for "))
                .then_some(offset)
        })
    {
        let implementation = &remaining[start..];
        let brace = implementation.find('{').expect("impl body");
        let end = matching_brace(implementation, brace);
        blocks.push(&implementation[..=end]);
        remaining = &implementation[end + 1..];
    }
    blocks
}

fn is_public_associated_item(line: &str) -> bool {
    line.starts_with("pub ")
}

fn has_must_use(source: &str, item: usize) -> bool {
    let mut found = false;
    for line in source[..item].lines().rev() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("///") {
            continue;
        }
        if line.starts_with("#[") {
            found |= line.starts_with("#[must_use");
            continue;
        }
        break;
    }
    found
}

fn matching_brace(source: &str, open: usize) -> usize {
    let mut depth = 0_usize;
    for (offset, character) in source[open..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return open + offset;
                }
            }
            _ => {}
        }
    }
    panic!("unterminated source item")
}

fn all_source() -> String {
    let mut paths = Vec::new();
    collect_sources(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut paths,
    );
    paths.sort();
    paths
        .into_iter()
        .map(|path| fs::read_to_string(path).expect("read spatial source"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn collect_sources(directory: &Path, paths: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).expect("read spatial source directory") {
        let path = entry.expect("spatial source entry").path();
        if path.is_dir() {
            collect_sources(&path, paths);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            paths.push(path);
        }
    }
}
