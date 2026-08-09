#[path = "headless/artifact_decode_support.rs"]
mod support;

use fenestra_ui_testkit::prototype::HeadlessArtifactDecodeErrorKindV1 as Kind;

#[test]
fn both_trace_sequences_are_dense_and_share_the_registered_domain() {
    let canonical = support::canonical_bytes();
    for (bytes, line) in [
        (support::set_field(&canonical, "h-event|1|0|", 2, "1"), 16),
        (
            support::set_field(&canonical, "h-event|1|0|", 3, "8002"),
            16,
        ),
        (support::set_field(&canonical, "s-event|1|0|", 2, "1"), 73),
        (
            support::set_field(&canonical, "s-event|1|0|", 3, "8002"),
            73,
        ),
    ] {
        support::assert_decode_error(&bytes, Kind::InvalidReference, Some(line));
    }
}

#[test]
fn scheduler_events_correlate_tick_generation_and_lane_state_to_headless_events() {
    let canonical = support::canonical_bytes();
    let cases = [
        support::set_field(&canonical, "h-event|1|4|", 4, "2"),
        support::set_field(&canonical, "s-event|1|0|", 11, "1"),
    ];
    for bytes in cases {
        support::assert_decode_error(&bytes, Kind::InvalidReference, Some(73));
    }
}

#[test]
fn scheduler_step_shape_and_every_accounted_lane_field_match_the_headless_event() {
    let canonical = support::canonical_bytes();
    let changed_stage = support::set_field(&canonical, "h-event|1|4|", 5, "scheduler");
    let changed_stage = support::set_field(&changed_stage, "h-event|1|4|", 7, "action");
    support::assert_decode_error(&changed_stage, Kind::InvalidReference, Some(73));

    let changed_outcome = support::set_field(&canonical, "h-event|1|4|", 7, "failed:scheduler");
    support::assert_decode_error(&changed_outcome, Kind::InvalidReference, Some(73));

    for (items_field, items, bytes_field, bytes) in [
        (20, "0", 21, "0"),
        (22, "1", 23, "32"),
        (24, "1", 25, "40"),
        (26, "1", 27, "40"),
        (28, "1", 29, "96"),
    ] {
        let changed = support::set_field(&canonical, "h-event|1|4|", items_field, items);
        let changed = support::set_field(&changed, "h-event|1|4|", bytes_field, bytes);
        support::assert_decode_error(&changed, Kind::InvalidReference, Some(73));
    }
}

#[test]
fn ordinary_mapped_frame_and_control_fields_match_between_traces() {
    let canonical = support::canonical_bytes();
    let wrong_frame = support::set_field(&canonical, "h-event|1|27|", 11, "1");
    support::assert_decode_error(&wrong_frame, Kind::InvalidReference, Some(90));

    let wrong_control = support::set_field(&canonical, "h-event|1|41|", 12, "1");
    support::assert_decode_error(&wrong_control, Kind::InvalidReference, Some(101));
}

#[test]
fn scheduler_step_rejects_common_fields_that_are_not_applicable() {
    let canonical = support::canonical_bytes();
    for (headless, scheduler, field) in [
        ("h-event|1|4|", "s-event|1|0|", 11),
        ("h-event|1|4|", "s-event|1|0|", 12),
        ("h-event|1|7|", "s-event|1|2|", 11),
        ("h-event|1|7|", "s-event|1|2|", 12),
        ("h-event|1|9|", "s-event|1|3|", 11),
        ("h-event|1|9|", "s-event|1|3|", 12),
        ("h-event|1|5|", "s-event|1|1|", 11),
        ("h-event|1|5|", "s-event|1|1|", 12),
    ] {
        let scheduler_field = field + 1;
        let changed = mapped_common(&canonical, headless, scheduler, field, scheduler_field, "0");
        support::assert_decode_error(
            &changed,
            Kind::MalformedRecord,
            Some(support::line_number(&canonical, scheduler)),
        );
    }
}

#[test]
fn scheduler_step_requires_common_fields_that_it_derives() {
    let canonical = support::canonical_bytes();
    for (headless, scheduler, headless_field, scheduler_field) in [
        ("h-event|1|27|", "s-event|1|17|", 11, 12),
        ("h-event|1|52|", "s-event|1|38|", 12, 13),
        ("h-event|1|28|", "s-event|1|18|", 11, 12),
        ("h-event|1|38|", "s-event|1|25|", 11, 12),
        ("h-event|1|41|", "s-event|1|28|", 12, 13),
    ] {
        let changed = mapped_common(
            &canonical,
            headless,
            scheduler,
            headless_field,
            scheduler_field,
            "-",
        );
        support::assert_decode_error(
            &changed,
            Kind::MalformedRecord,
            Some(support::line_number(&canonical, scheduler)),
        );
    }
}

#[test]
fn frame_control_submission_and_watermark_references_are_coherent() {
    let canonical = support::canonical_bytes();
    let cases = [
        (support::set_field(&canonical, "s-event|1|17|", 12, "1"), 90),
        (support::set_field(&canonical, "s-event|1|18|", 12, "1"), 91),
        (support::set_field(&canonical, "s-event|1|18|", 7, "2"), 91),
        (support::set_field(&canonical, "s-event|1|28|", 7, "2"), 101),
        (
            support::set_field(&canonical, "s-event|1|28|", 13, "1"),
            101,
        ),
        (
            support::set_field(&canonical, "s-event|1|40|", 30, "0:2"),
            113,
        ),
    ];
    for (bytes, line) in cases {
        support::assert_decode_error(&bytes, Kind::InvalidReference, Some(line));
    }
}

#[test]
fn targets_paths_and_final_projection_resolve_against_recorded_state() {
    let canonical = support::canonical_bytes();
    let cases = [
        (
            support::set_field(&canonical, "h-event|1|7|", 10, "key:31"),
            23,
        ),
        (
            support::replace_once(&canonical, "computed|root|84|80", "computed|root/s:9|84|80"),
            117,
        ),
        (
            support::set_field(&canonical, "projection-begin|", 1, "8"),
            115,
        ),
        (
            support::set_field(&canonical, "projection-begin|", 2, "89"),
            115,
        ),
    ];
    for (bytes, line) in cases {
        support::assert_decode_error(&bytes, Kind::InvalidReference, Some(line));
    }
}

#[test]
fn semantically_wrong_but_resolved_trace_values_are_preserved_for_verification() {
    let canonical = support::canonical_bytes();
    let changed_input = support::set_field(&canonical, "h-event|1|4|", 6, "resize");
    let changed_input = support::set_field(&changed_input, "h-event|1|4|", 10, "none");
    let changed_target = support::set_field(&changed_input, "h-event|1|18|", 10, "key:30");
    let decoded = support::decode(&changed_target);

    assert_eq!(
        fenestra_ui_testkit::prototype::encode_headless_artifact_v1(&decoded)
            .expect("resolved trace variants should re-encode"),
        changed_target
    );
}

#[test]
fn fixture_valid_but_finally_absent_key_path_is_preserved_for_verification() {
    let canonical = support::canonical_bytes();
    let text = std::str::from_utf8(&canonical).expect("canonical artifact should be ASCII");
    assert_eq!(text.matches("root/s:0/m:1:30").count(), 4);
    let changed = text
        .replace("root/s:0/m:1:30", "root/s:0/m:1:20")
        .into_bytes();
    let decoded = support::decode(&changed);

    assert_eq!(
        fenestra_ui_testkit::prototype::encode_headless_artifact_v1(&decoded)
            .expect("fixture-valid projection paths should re-encode"),
        changed
    );
}

#[test]
fn duplicate_and_derived_projection_paths_must_resolve() {
    let canonical = support::canonical_bytes();
    let duplicate =
        support::replace_once(&canonical, "computed|root/s:0|80|50", "computed|root|80|50");
    support::assert_decode_error(&duplicate, Kind::InvalidReference, Some(118));

    let absent_hit = support::replace_once(
        &canonical,
        "hit|root/s:0/m:1:10|0|10|40|12",
        "hit|root/s:0/m:1:20|0|10|40|12",
    );
    support::assert_decode_error(&absent_hit, Kind::InvalidReference, Some(133));
}

#[test]
fn every_multi_record_derived_family_rejects_duplicate_paths() {
    let canonical = support::canonical_bytes();
    for (prefix, duplicate_path) in [
        ("geometry|root/s:0|", "root"),
        ("hit|root/s:0/m:1:30|", "root/s:0/m:1:10"),
        ("scene|root/s:0|", "root"),
    ] {
        let duplicate = support::set_field(&canonical, prefix, 1, duplicate_path);
        support::assert_decode_error(
            &duplicate,
            Kind::InvalidReference,
            Some(support::line_number(&canonical, prefix)),
        );
    }
}

#[test]
fn renderer_loss_is_the_only_mapped_event_with_a_headless_only_frame() {
    let canonical = support::canonical_bytes();
    let _ = support::decode(&canonical);

    let wrong_loss_frame = support::set_field(&canonical, "h-event|1|48|", 11, "2");
    support::assert_decode_error(&wrong_loss_frame, Kind::InvalidReference, Some(64));

    let scheduler_claims_frame = support::set_field(&canonical, "s-event|1|34|", 12, "3");
    support::assert_decode_error(&scheduler_claims_frame, Kind::MalformedRecord, Some(107));

    let shutdown_claims_frame = support::set_field(&canonical, "h-event|1|49|", 11, "3");
    support::assert_decode_error(&shutdown_claims_frame, Kind::InvalidReference, Some(65));
}

#[test]
fn invalid_references_precede_private_trailing_data() {
    let canonical = support::canonical_bytes();
    let mut bytes = support::set_field(&canonical, "h-event|1|0|", 2, "1");
    bytes.extend_from_slice(b"private|payload\n");
    support::assert_decode_error(&bytes, Kind::InvalidReference, Some(16));
}

#[test]
fn coherent_nonregistered_generation_space_roundtrips_for_later_verification() {
    let canonical = support::canonical_bytes();
    let shifted = shift_all_generations(&canonical);
    let decoded = support::decode(&shifted);

    assert_eq!(decoded.final_generation(), 10);
    assert_eq!(
        fenestra_ui_testkit::prototype::encode_headless_artifact_v1(&decoded)
            .expect("coherent shifted generations should encode"),
        shifted
    );
}

#[test]
fn coherent_nonregistered_surface_space_roundtrips_for_later_verification() {
    let canonical = support::canonical_bytes();
    let shifted = shift_all_surfaces(&canonical);
    let decoded = support::decode(&shifted);

    assert_eq!(
        [
            decoded.final_surface().width(),
            decoded.final_surface().height()
        ],
        [91, 71]
    );
    assert_eq!(
        fenestra_ui_testkit::prototype::encode_headless_artifact_v1(&decoded)
            .expect("coherent shifted surfaces should encode"),
        shifted
    );
}

fn shift_all_generations(bytes: &[u8]) -> Vec<u8> {
    let mut output = String::new();
    for line in support::lines(bytes) {
        let mut fields = line.split('|').map(str::to_owned).collect::<Vec<_>>();
        match fields.first().map(String::as_str) {
            Some("h-event") => {
                shift_optional(&mut fields[8]);
                shift_optional(&mut fields[9]);
            }
            Some("s-event") => shift_number(&mut fields[11]),
            Some("projection-begin") => shift_number(&mut fields[1]),
            _ => {}
        }
        output.push_str(&fields.join("|"));
        output.push('\n');
    }
    output.into_bytes()
}

fn mapped_common(
    bytes: &[u8],
    headless: &str,
    scheduler: &str,
    headless_field: usize,
    scheduler_field: usize,
    value: &str,
) -> Vec<u8> {
    let changed = support::set_field(bytes, headless, headless_field, value);
    support::set_field(&changed, scheduler, scheduler_field, value)
}

fn shift_all_surfaces(bytes: &[u8]) -> Vec<u8> {
    let mut output = String::new();
    for line in support::lines(bytes) {
        let mut fields = line.split('|').map(str::to_owned).collect::<Vec<_>>();
        match fields.first().map(String::as_str) {
            Some("h-event") => {
                shift_number(&mut fields[13]);
                shift_number(&mut fields[14]);
            }
            Some("projection-begin") => {
                shift_number(&mut fields[2]);
                shift_number(&mut fields[3]);
            }
            _ => {}
        }
        output.push_str(&fields.join("|"));
        output.push('\n');
    }
    output.into_bytes()
}

fn shift_optional(value: &mut String) {
    if value != "-" {
        shift_number(value);
    }
}

fn shift_number(value: &mut String) {
    let number = value
        .parse::<u64>()
        .expect("canonical generation is numeric");
    *value = (number + 1).to_string();
}
