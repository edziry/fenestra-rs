use fenestra_ui_authoring::prototype::canonical_semantics_v2;

use crate::support;

const IMAGE_LIST: &str = "255, 0, 0, 255, 0, 128, 0, 128, 0, 0, 64, 64, 0, 0, 0, 0";

#[test]
fn every_binding_family_and_literal_payload_changes_retained_semantics() {
    let cases = [
        (
            "i32 literal/property",
            support::replace_occurrence(support::FIXTURE, "gap property pad;", "gap 7;", 0),
        ),
        (
            "fixed16 literal/property",
            support::replace_once(
                support::FIXTURE,
                "      width property span_x;",
                "      width fixed(589824);",
            ),
        ),
        (
            "rgba8 literal/property",
            support::replace_occurrence(
                support::FIXTURE,
                "      color property tone;",
                "      color rgba8(1, 2, 3, 255);",
                0,
            ),
        ),
        (
            "input-policy literal/property",
            support::replace_occurrence(
                support::FIXTURE,
                "      input property policy;",
                "      input accept;",
                0,
            ),
        ),
        (
            "opacity",
            support::replace_once(support::FIXTURE, "      opacity 200;", "      opacity 199;"),
        ),
        (
            "gradient color",
            support::replace_once(
                support::FIXTURE,
                "      stop 32768 rgba8(128, 64, 32, 255);",
                "      stop 32768 rgba8(127, 64, 32, 255);",
            ),
        ),
    ];
    for (name, source) in cases {
        assert_semantics_change(name, &source);
    }
}

#[test]
fn every_authored_transform_form_and_affine_coefficient_is_observed() {
    let cases = [
        (
            "identity",
            support::replace_once(
                support::FIXTURE,
                "transform identity origin",
                "transform quarter_turn(2) origin",
            ),
        ),
        (
            "translate",
            support::replace_once(
                support::FIXTURE,
                "transform translate(point(property factor, fixed(32768)))",
                "transform translate(point(property factor, fixed(32769)))",
            ),
        ),
        (
            "scale",
            support::replace_once(
                support::FIXTURE,
                "transform scale(property factor, fixed(65536))",
                "transform scale(property factor, fixed(65535))",
            ),
        ),
        (
            "quarter turn",
            support::replace_once(
                support::FIXTURE,
                "transform quarter_turn(1)",
                "transform quarter_turn(2)",
            ),
        ),
        (
            "affine",
            support::replace_once(
                support::FIXTURE,
                "transform affine(fixed(65536), fixed(0), fixed(0), fixed(65536), property factor, fixed(-32768))",
                "transform affine(fixed(65535), fixed(1), fixed(-1), fixed(65537), property factor, fixed(-32767))",
            ),
        ),
    ];
    for (name, source) in cases {
        assert_semantics_change(name, &source);
    }
}

#[test]
fn every_image_byte_position_insertion_removal_and_table_order_are_observed() {
    let baseline = semantic_bytes(support::FIXTURE);
    let bytes = [255u8, 0, 0, 255, 0, 128, 0, 128, 0, 0, 64, 64, 0, 0, 0, 0];
    for index in 0..bytes.len() {
        let mut changed = bytes.to_vec();
        changed[index] = if changed[index] == 255 {
            254
        } else {
            changed[index] + 1
        };
        let source = with_image_bytes(&changed);
        assert_ne!(
            semantic_bytes(&source),
            baseline,
            "missed image byte {index}"
        );
    }
    let mut inserted = bytes.to_vec();
    inserted.insert(3, 17);
    assert_ne!(semantic_bytes(&with_image_bytes(&inserted)), baseline);
    let mut removed = bytes.to_vec();
    removed.remove(3);
    assert_ne!(semantic_bytes(&with_image_bytes(&removed)), baseline);

    let extra = concat!(
        "    image extra {\n",
        "      width 1;\n      height 1;\n      stride 4;\n",
        "      bytes [0, 0, 0, 0];\n    }\n"
    );
    let before = support::replace_once(
        support::FIXTURE,
        "    image checker {",
        &format!("{extra}    image checker {{"),
    );
    let after = support::replace_once(
        support::FIXTURE,
        "    }\n  }\n  node scene : root {",
        &format!("    }}\n{extra}  }}\n  node scene : root {{"),
    );
    assert_ne!(semantic_bytes(&before), semantic_bytes(&after));
}

#[test]
fn ordered_geometry_and_gradient_tables_are_observed() {
    let polygon = swap_unique(
        support::FIXTURE,
        "      point(fixed(0), fixed(0));",
        "      point(fixed(327680), fixed(524288));",
    );
    assert_semantics_change("polygon order", &polygon);

    let path = swap_unique(
        support::FIXTURE,
        "      line_to point(fixed(655360), fixed(0));",
        "      quadratic_to point(fixed(983040), fixed(327680)) point(fixed(655360), fixed(655360));",
    );
    assert_semantics_change("path order", &path);

    let stops = swap_unique(
        support::FIXTURE,
        "      stop 0 property accent_tone;",
        "      stop 32768 rgba8(128, 64, 32, 255);",
    );
    assert_semantics_change("gradient stop order", &stops);
}

fn assert_semantics_change(name: &str, source: &str) {
    assert_ne!(
        semantic_bytes(source),
        semantic_bytes(support::FIXTURE),
        "semantic mutation was invisible: {name}"
    );
}

fn semantic_bytes(source: &str) -> Vec<u8> {
    let compiled = support::compile_fen(source);
    canonical_semantics_v2(&compiled, support::semantic_limits())
        .expect("mutated source should produce a semantic artifact")
        .as_bytes()
        .to_vec()
}

fn with_image_bytes(bytes: &[u8]) -> String {
    let encoded = bytes
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    support::replace_once(support::FIXTURE, IMAGE_LIST, &encoded)
}

fn swap_unique(source: &str, first: &str, second: &str) -> String {
    assert_eq!(source.match_indices(first).count(), 1, "first swap item");
    assert_eq!(source.match_indices(second).count(), 1, "second swap item");
    let marker = "__fenestra_authoring_v2_swap_marker__";
    assert!(!source.contains(marker));
    source
        .replacen(first, marker, 1)
        .replacen(second, first, 1)
        .replacen(marker, second, 1)
}
