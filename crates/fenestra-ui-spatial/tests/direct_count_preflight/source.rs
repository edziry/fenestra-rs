use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const HELPER: &str = "preflight_spatial_direct_counts_v2";

#[test]
fn helper_is_the_only_prototype_surface_delta_and_adds_no_struct() {
    let baseline = read(&manifest_dir().join("tests/api_surface.rs"));
    let mut expected_exports = quoted_array(&baseline, "EXPECTED_EXPORTS", 121);
    assert!(!expected_exports.iter().any(|name| name == HELPER));
    expected_exports.push(HELPER.to_owned());
    assert_eq!(expected_exports.len(), 122);

    let source = read(&source_dir().join("lib.rs"));
    let exports = prototype_exports(&source);
    assert_eq!(exports.len(), 122);
    assert_eq!(exports, expected_exports.into_iter().collect());

    let expected_structs = quoted_array(&baseline, "EXPECTED_STRUCTS", 52)
        .into_iter()
        .collect::<BTreeSet<_>>();
    assert_eq!(public_structs(&all_source()), expected_structs);
}

#[test]
fn helper_declaration_has_the_exact_signature_and_attribute() {
    let source = all_source();
    assert_eq!(source.matches(&format!("pub fn {HELPER}")).count(), 1);
    let declaration = function_declaration(&source, &format!("pub fn {HELPER}"));
    assert_eq!(
        compact(declaration),
        concat!(
            "#[must_use=\"direct-countpreflighterrorsmustbehandled\"]",
            "pubfnpreflight_spatial_direct_counts_v2(",
            "observed:[u128;12],limits:SpatialLimitsV2,)",
            "->Result<(),SpatialResolveErrorV2>"
        )
    );
}

#[test]
fn raw_resolver_delegates_its_complete_widened_direct_count_phase() {
    let source = all_source();
    let body = function_body(&source, "fn prepare_direct_counts(");
    let body = compact(body);
    let observed = concat!(
        "letobserved=[",
        "topology.nodes().len()asu128,",
        "geometry.shapes().len()asu128,",
        "resources.brushes().len()asu128,",
        "geometry.clips().len()asu128,",
        "items.paint_items().len()asu128,",
        "items.hit_items().len()asu128,",
        "items.semantic_items().len()asu128,",
        "geometry.paths().len()asu128,",
        "geometry.path_verbs().len()asu128,",
        "geometry.polygon_points().len()asu128,",
        "resources.gradient_stops().len()asu128,",
        "resources.images().len()asu128,",
        "];"
    );
    assert!(
        body.contains(observed),
        "direct counts are not widened in order"
    );
    assert_eq!(
        body.matches("preflight_spatial_direct_counts_v2(observed,limits)?;")
            .count(),
        1,
        "raw phase one must delegate exactly once"
    );
    for duplicate in [
        "u32::MAX",
        "U32_ROW_CAPACITY",
        "caller_maximum",
        "limit_exceeded",
        "validate_direct_count",
        ".min(",
    ] {
        assert!(
            !body.contains(duplicate),
            "raw phase one duplicates effective-maximum logic: {duplicate}"
        );
    }
}

fn quoted_array(source: &str, name: &str, expected_len: usize) -> Vec<String> {
    let marker = format!("const {name}: [&str; {expected_len}] = [");
    let tail = source
        .split_once(&marker)
        .unwrap_or_else(|| panic!("missing {name} baseline"))
        .1;
    let body = tail
        .split_once("];")
        .unwrap_or_else(|| panic!("unterminated {name} baseline"))
        .0;
    let values = body
        .lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix('"'))
        .map(|line| {
            line.strip_suffix("\",")
                .unwrap_or_else(|| panic!("invalid {name} entry"))
                .to_owned()
        })
        .collect::<Vec<_>>();
    assert_eq!(values.len(), expected_len, "invalid {name} baseline");
    values
}

fn prototype_exports(source: &str) -> BTreeSet<String> {
    let marker = "pub mod prototype {";
    let start = source.find(marker).expect("prototype module") + marker.len();
    let body = &source[start..source.rfind('}').expect("prototype end")];
    assert!(!body.contains(" as "));
    assert!(!body.contains("::*"));
    let mut names = BTreeSet::new();
    for item in body.split("pub use crate::").skip(1) {
        let exported = if let Some(list_start) = item.find("::{") {
            &item[list_start + 3..item.find("};").expect("grouped reexport end")]
        } else {
            let end = item.find(';').expect("singleton reexport end");
            item[..end].rsplit("::").next().expect("singleton name")
        };
        for name in exported
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            assert!(names.insert(name.to_owned()), "duplicate export {name}");
        }
    }
    names
}

fn public_structs(source: &str) -> BTreeSet<String> {
    source
        .lines()
        .filter_map(|line| line.trim().strip_prefix("pub struct "))
        .map(|declaration| {
            declaration
                .split(['<', '(', '{'])
                .next()
                .expect("struct name")
                .trim()
                .to_owned()
        })
        .collect()
}

fn function_declaration<'a>(source: &'a str, marker: &str) -> &'a str {
    let function = source.find(marker).expect("function declaration");
    let attribute = source[..function].rfind("#[").expect("function attribute");
    let end = source[function..].find('{').expect("function body") + function;
    &source[attribute..end]
}

fn function_body<'a>(source: &'a str, marker: &str) -> &'a str {
    let start = source.find(marker).expect("function") + marker.len();
    let open = source[start..].find('{').expect("function body") + start;
    let mut depth = 0_usize;
    for (offset, character) in source[open..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[open..open + offset + 1];
                }
            }
            _ => {}
        }
    }
    panic!("unterminated function body");
}

fn compact(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn source_dir() -> PathBuf {
    manifest_dir().join("src")
}

fn all_source() -> String {
    let mut paths = Vec::new();
    collect_rust_sources(&source_dir(), &mut paths);
    paths.sort();
    paths
        .into_iter()
        .map(|path| read(&path))
        .collect::<Vec<_>>()
        .join("\n")
}

fn collect_rust_sources(directory: &Path, paths: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).expect("read source directory") {
        let path = entry.expect("source entry").path();
        if path.is_dir() {
            collect_rust_sources(&path, paths);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            paths.push(path);
        }
    }
}

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}
