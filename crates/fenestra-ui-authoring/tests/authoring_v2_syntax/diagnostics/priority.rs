use fenestra_ui_authoring::prototype::{AnchorKindV2, AuthoringDiagnosticKindV2};

use super::super::support::{FIXTURE, assert_diagnostic, replace_once};
use super::expected;

#[test]
fn free_placement_reports_fields_before_its_later_anchor_target() {
    let before = concat!(
        "placement free\n",
        "          width property span_x height property span_y\n",
        "          self_anchor anchor(center, end)\n",
        "          target node guide",
    );
    let after = concat!(
        "placement free\n",
        "          width property missing_width height property span_y\n",
        "          self_anchor anchor(center, end)\n",
        "          target node missing_target",
    );
    let source = replace_once(FIXTURE, before, after);
    assert_diagnostic(
        &source,
        expected(
            AuthoringDiagnosticKindV2::UnknownPropertyName,
            AnchorKindV2::SpatialField,
            237,
            "missing_width",
            0,
        ),
    );
}
