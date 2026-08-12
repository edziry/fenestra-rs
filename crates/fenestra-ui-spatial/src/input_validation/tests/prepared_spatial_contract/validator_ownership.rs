use std::sync::Arc;

use super::support::{requested_limits, rich_engine, rich_owned};
use super::validator_support::*;
use super::*;
use crate::output_field::{SpatialOutputFieldV2 as Field, SpatialOutputTableV2 as Table};
use crate::resolve_error::SpatialOutputErrorKindV2 as Kind;

#[test]
fn early_and_late_failures_drop_prepared_owner_and_leave_candidate_buffers_unchanged() {
    for late in [false, true] {
        let source = rich_owned();
        let weak = Arc::downgrade(&source);
        let prepared = prepare_spatial_v2(&rich_engine(), source, requested_limits()).unwrap();
        let mut rows = rich_tables();
        if late {
            let mut semantic = ShapeItemRow::read_semantic(rows.semantics[1]);
            semantic.clip = Some(u32::MAX);
            rows.semantics[1] = semantic.build_semantic();
        } else {
            let _ = rows.geometry.pop();
        }
        let expected_rows = clone_tables(&rows);

        let expected = if late {
            (
                Kind::InvalidReference,
                output_location(Table::Semantic, 1, Field::Clip),
            )
        } else {
            (
                Kind::RecordCountMismatch,
                crate::error::SpatialErrorLocationV2::Output {
                    table: Table::Geometry,
                },
            )
        };
        expect_output_error(validate(prepared, &rows), expected.0, expected.1);
        assert_eq!(rows.geometry, expected_rows.geometry);
        assert_eq!(rows.clips, expected_rows.clips);
        assert_eq!(rows.paints, expected_rows.paints);
        assert_eq!(rows.hits, expected_rows.hits);
        assert_eq!(rows.semantics, expected_rows.semantics);
        assert!(weak.upgrade().is_none());
    }
}

fn clone_tables(rows: &CandidateTables) -> CandidateTables {
    CandidateTables {
        geometry: rows.geometry.clone(),
        clips: rows.clips.clone(),
        paints: rows.paints.clone(),
        hits: rows.hits.clone(),
        semantics: rows.semantics.clone(),
    }
}
