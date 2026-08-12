use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const VALIDATION_ROOT_ENV: &str = "FENESTRA_SPATIAL_VALIDATION_ROOT";

fn validation_root() -> PathBuf {
    env::var_os(VALIDATION_ROOT_ENV).map_or_else(
        || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/validation"),
        PathBuf::from,
    )
}

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn rust_sources(directory: &Path, sources: &mut Vec<(PathBuf, String)>) {
    for entry in fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", directory.display()))
    {
        let path = entry.expect("validation source entry").path();
        if path.is_dir() {
            rust_sources(&path, sources);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            sources.push((path.clone(), read(&path)));
        }
    }
}

fn function_body(source: &str, entry: usize) -> &str {
    let open = source[entry..]
        .find('{')
        .map(|offset| entry + offset)
        .expect("validate_spatial should have a body");
    let mut depth = 0usize;
    for (offset, byte) in source.as_bytes()[open..].iter().copied().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[open..=open + offset];
                }
            }
            _ => {}
        }
    }
    panic!("validate_spatial body should close")
}

fn compact(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn ordered(source: &str, anchors: &[&str]) {
    let mut cursor = 0usize;
    for anchor in anchors {
        let offset = source[cursor..]
            .find(anchor)
            .unwrap_or_else(|| panic!("missing ordered validation prefix anchor {anchor}"));
        cursor += offset + anchor.len();
    }
}

fn function_names(source: &str) -> Vec<&str> {
    let mut names = Vec::new();
    let mut remainder = source;
    while let Some(offset) = remainder.find("fn ") {
        remainder = &remainder[offset + 3..];
        let end = remainder
            .find(|character: char| !(character == '_' || character.is_ascii_alphanumeric()))
            .expect("function name should terminate");
        names.push(&remainder[..end]);
        remainder = &remainder[end..];
    }
    names
}

fn identifier_count(source: &str, target: &str) -> usize {
    let mut count = 0usize;
    let bytes = source.as_bytes();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        if bytes[cursor] == b'_' || bytes[cursor].is_ascii_alphabetic() {
            let start = cursor;
            cursor += 1;
            while cursor < bytes.len()
                && (bytes[cursor] == b'_' || bytes[cursor].is_ascii_alphanumeric())
            {
                cursor += 1;
            }
            count += usize::from(&source[start..cursor] == target);
        } else {
            cursor += 1;
        }
    }
    count
}

fn calls(source: &str) -> Vec<String> {
    let bytes = source.as_bytes();
    let mut calls = Vec::new();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        if bytes[cursor] != b'_' && !bytes[cursor].is_ascii_alphabetic() {
            cursor += 1;
            continue;
        }
        let start = cursor;
        cursor += 1;
        while cursor < bytes.len()
            && (bytes[cursor] == b'_' || bytes[cursor].is_ascii_alphanumeric())
        {
            cursor += 1;
        }
        let name = &source[start..cursor];
        let mut after = cursor;
        while after < bytes.len() && bytes[after].is_ascii_whitespace() {
            after += 1;
        }
        if after == bytes.len() || bytes[after] != b'(' {
            continue;
        }
        let mut before = start;
        while before > 0 && bytes[before - 1].is_ascii_whitespace() {
            before -= 1;
        }
        let prefix = if before > 0 && bytes[before - 1] == b'.' {
            "."
        } else if before > 1 && &bytes[before - 2..before] == b"::" {
            "::"
        } else {
            ""
        };
        if !matches!(name, "if" | "for" | "while" | "match" | "pub") {
            calls.push(format!("{prefix}{name}"));
        }
    }
    calls
}

#[test]
fn count_preflight_has_a_dedicated_nonallocating_module() {
    let path = validation_root().join("spatial/counts.rs");
    assert!(
        path.is_file(),
        "count preflight must be exactly {}",
        path.display()
    );
    let source = read(&path);

    for forbidden in [
        "std::alloc",
        "std::collections",
        "alloc::",
        "HashMap",
        "HashSet",
        "BinaryHeap",
        "LinkedList",
        "VecDeque",
        "Vec<",
        "Vec::",
        "vec![",
        "Box<",
        "Box::",
        "String",
        "Cow<",
        "Arc<",
        "Rc<",
        "with_capacity",
        ".reserve(",
        ".push(",
        ".insert(",
        ".entry(",
        ".extend(",
        ".collect(",
        ".to_vec(",
        ".to_owned(",
        ".to_string(",
        ".into_boxed_",
        ".clone(",
        "format!(",
        "include!(",
        "mod ",
    ] {
        assert!(
            !source.contains(forbidden),
            "count preflight contains allocation-capable form {forbidden}"
        );
    }

    let functions = function_names(&source);
    assert!(functions.contains(&"preflight_spatial_counts"));
    assert!(
        functions.iter().all(|name| matches!(
            *name,
            "preflight_spatial_counts" | "add_count" | "add_amount"
        )),
        "counts.rs may delegate only to local add_count/add_amount helpers: {functions:?}"
    );
    let calls = calls(&source);
    for call in &calls {
        let name = call.trim_start_matches(['.', ':']);
        let local = functions.contains(&name);
        let allowed_free = matches!(
            name,
            "add_count" | "add_amount" | "failure" | "limit_failure"
        ) || matches!(call.as_str(), "Err" | "Ok" | "::LimitExceeded");
        let allowed_method = matches!(
            call.as_str(),
            ".nodes"
                | ".shapes"
                | ".brushes"
                | ".clips"
                | ".paint_items"
                | ".hit_items"
                | ".semantic_items"
                | ".images"
                | ".geometry"
                | ".content"
                | ".points"
                | ".verbs"
                | ".stops"
                | ".bytes"
                | ".span"
                | ".len"
                | ".iter"
                | ".as_ref"
                | ".get"
                | ".value"
                | ".spatial_nodes"
                | ".spatial_shapes"
                | ".spatial_brushes"
                | ".spatial_clips"
                | ".spatial_paint_items"
                | ".spatial_hit_items"
                | ".spatial_semantic_items"
                | ".spatial_paths"
                | ".spatial_path_verbs"
                | ".spatial_polygon_points"
                | ".spatial_gradient_stops"
                | ".spatial_images"
                | ".spatial_image_bytes"
                | ".checked_add"
                | ".ok_or_else"
        );
        assert!(
            local || allowed_free || allowed_method,
            "counts.rs delegates through nonlocal call {call}: {calls:?}"
        );
    }
    let entry = source.find("fn add_amount").expect("missing add_amount");
    let add_amount = compact(function_body(&source, entry));
    ordered(
        &add_amount,
        &["checked_add(", "ok_or_else(", "LimitExceeded(kind)", "span"],
    );
    assert_eq!(
        add_amount.matches("checked_add(").count(),
        1,
        "add_amount must use one checked accumulation"
    );
    assert!(source.contains("SpatialValidationLimitsV2"));
    assert!(source.contains("LimitExceeded"));
}

#[test]
fn validate_spatial_has_inline_phase_one_then_count_preflight() {
    let root = validation_root();
    let mut sources = Vec::new();
    rust_sources(&root, &mut sources);
    let matches = sources
        .into_iter()
        .filter(|(_, source)| source.contains("pub fn validate_spatial"))
        .collect::<Vec<_>>();
    assert_eq!(
        matches.len(),
        1,
        "validate_spatial must have one definition"
    );
    let (path, source) = &matches[0];
    let entry = source.find("pub fn validate_spatial").expect("entry point");
    let body_source = function_body(source, entry);
    let body = compact(body_source);
    let preflight = "counts::preflight_spatial_counts(&program,limits)?;";
    let preflight_at = body
        .find(preflight)
        .unwrap_or_else(|| panic!("{} must call exact count preflight", path.display()));
    let prefix = &body[..preflight_at + preflight.len()];

    ordered(
        prefix,
        &[
            "if!program.span().is_valid()",
            "IrValidationErrorKind::InvalidSourceSpan",
            "program.format()!=SUPPORTED_SPATIAL_FORMAT",
            "IrValidationErrorKind::UnsupportedSpatialFormat",
            "letmanifest=&style.construction().schema().data.manifest;",
            "program.schema_namespace()!=manifest.namespace",
            "program.schema_revision()!=manifest.revision",
            "IrValidationErrorKind::SchemaIdentityMismatch",
            preflight,
        ],
    );

    let raw_preflight_at = body_source
        .find("counts::preflight_spatial_counts")
        .expect("raw preflight call");
    let raw_preflight_end = body_source[raw_preflight_at..]
        .find(';')
        .map(|offset| raw_preflight_at + offset + 1)
        .expect("raw preflight call should end with a semicolon");
    let prefix_source = &body_source[..raw_preflight_at];
    assert_eq!(identifier_count(prefix_source, "if"), 3);
    assert_eq!(identifier_count(prefix_source, "return"), 3);
    assert_eq!(identifier_count(prefix_source, "let"), 1);
    assert_eq!(prefix.matches("failure(").count(), 3);
    assert_eq!(
        prefix.matches('?').count(),
        1,
        "preflight must be first fallible work"
    );
    assert_eq!(prefix.matches('{').count(), 4);
    assert_eq!(prefix.matches('}').count(), 3);
    assert!(
        prefix.ends_with(&format!("}}{preflight}")),
        "count preflight must immediately follow schema identity validation"
    );
    assert_eq!(
        calls(&body_source[..raw_preflight_end]),
        [
            ".span",
            ".is_valid",
            "Err",
            "failure",
            ".span",
            ".format",
            "Err",
            "failure",
            ".span",
            ".construction",
            ".schema",
            ".schema_namespace",
            ".schema_revision",
            "Err",
            "failure",
            ".span",
            "::preflight_spatial_counts",
        ],
        "phase-one prefix may not delegate or allocate before count preflight"
    );

    for forbidden in [
        "validate_header(",
        "build_context(",
        "build_indexes(",
        "build_index(",
        "HashMap",
        "HashSet",
        "Vec::",
        "Box::",
        "with_capacity(",
        ".collect(",
        ".to_vec(",
        ".to_owned(",
        ".clone(",
        "format!(",
    ] {
        assert!(
            !prefix.contains(forbidden),
            "validate_spatial performs {forbidden} before count preflight"
        );
    }

    let later_work = [
        "build_context(",
        "build_indexes(",
        "build_index(",
        "HashMap::",
        "HashSet::",
    ]
    .into_iter()
    .filter_map(|needle| body.find(needle))
    .min()
    .unwrap_or_else(|| panic!("{} must visibly build validation indexes", path.display()));
    assert!(
        preflight_at < later_work,
        "indexes are built before count preflight"
    );
}
