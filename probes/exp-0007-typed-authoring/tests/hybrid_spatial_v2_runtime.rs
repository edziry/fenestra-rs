#[path = "support/hybrid_spatial_v2/oracle/mod.rs"]
mod oracle;
#[path = "support/hybrid_spatial_v2/runtime/mod.rs"]
mod runtime;
#[path = "support/hybrid_spatial_v2/mod.rs"]
mod support;

use fenestra_ui_exp_0007_typed_authoring::{generated_hybrid_spatial_v2, macro_hybrid_spatial_v2};
use fenestra_ui_ir::prototype::{SourceId, SourceSpan};
use fenestra_ui_spatial::prototype::{
    SpatialErrorLocationV2, SpatialResolveErrorKindV2, SpatialTransformErrorKindV2,
    SpatialViewportV2,
};

use runtime::run_authored_spatial_lane;
use support::manual_hybrid_spatial_v2;

#[test]
fn manual_fen_and_ui_lanes_publish_the_same_complete_spatial_log() {
    let manual = run_authored_spatial_lane(manual_hybrid_spatial_v2());
    let fen = run_authored_spatial_lane(generated_hybrid_spatial_v2());
    let ui = run_authored_spatial_lane(macro_hybrid_spatial_v2());
    let expected = oracle::literal_oracle();

    oracle::assert_log_eq(&manual.oracle_log(), &expected);
    oracle::assert_log_eq(&fen.oracle_log(), &expected);
    oracle::assert_log_eq(&ui.oracle_log(), &expected);
    assert_eq!(manual, fen);
    assert_eq!(manual, ui);
    assert_eq!(manual.generations(), (0_u64..=8).collect::<Vec<_>>());
    assert_eq!(
        manual.viewports(),
        std::iter::once(SpatialViewportV2::new(192, 128))
            .chain(std::iter::repeat_n(SpatialViewportV2::new(224, 160), 8))
            .collect::<Vec<_>>()
    );
    assert_eq!(manual.mapping_counts(), [9, 9, 9, 9, 9, 10, 10, 10, 9]);
    assert_eq!(manual.geometry_counts(), manual.mapping_counts());
    assert_eq!(manual.clip_counts(), [4, 4, 4, 4, 4, 5, 5, 5, 4]);
    assert_eq!(manual.paint_counts(), [5, 5, 5, 5, 5, 6, 6, 6, 5]);
    assert_eq!(manual.hit_counts(), [5, 5, 5, 5, 5, 6, 6, 6, 5]);
    assert_eq!(manual.semantic_counts(), [4, 4, 4, 4, 4, 5, 5, 5, 4]);
    assert_eq!(
        manual.hit_query_counts(),
        [
            24_580, 35_844, 35_844, 35_844, 35_844, 35_844, 35_844, 35_844, 35_844
        ]
    );
    assert_eq!(
        manual.raster_byte_counts(),
        [
            98_304, 143_360, 143_360, 143_360, 143_360, 143_360, 143_360, 143_360, 143_360
        ]
    );
    assert_eq!(manual.final_keys(), &[10, 30]);
    assert!(manual.noop_checks().all_preserved());
}

#[test]
fn singular_authored_property_failure_is_exact_and_rolls_back_every_observation() {
    for programs in [
        manual_hybrid_spatial_v2(),
        generated_hybrid_spatial_v2(),
        macro_hybrid_spatial_v2(),
    ] {
        let log = run_authored_spatial_lane(programs);
        let failure = log.failure();
        assert_eq!(
            failure.resolve_kind(),
            SpatialResolveErrorKindV2::Transform(SpatialTransformErrorKindV2::SingularTransform)
        );
        assert_eq!(
            failure.resolve_location(),
            SpatialErrorLocationV2::Node { index: 3 }
        );
        assert_eq!(
            failure.ir_span(),
            SourceSpan::bytes(SourceId::new(0), 226, 227)
        );
        assert_eq!(
            log.authored_factor_span(),
            SourceSpan::bytes(SourceId::new(0), 247, 248)
        );
        assert_eq!(failure.operation_index(), None);
        assert!(failure.outer_state_preserved());
        assert!(failure.spatial_snapshot_preserved());
        assert!(failure.complete_observation_preserved());
    }
}

#[test]
fn literal_oracle_source_boundary_excludes_candidate_implementations() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/support/hybrid_spatial_v2/oracle");
    let mut entries = std::fs::read_dir(&directory)
        .expect("the literal oracle directory should exist")
        .map(|entry| {
            entry
                .expect("oracle directory entries should be readable")
                .path()
        })
        .collect::<Vec<_>>();
    entries.sort();
    assert_eq!(
        entries
            .iter()
            .map(|path| {
                let file_type = std::fs::symlink_metadata(path)
                    .expect("oracle source metadata should be readable")
                    .file_type();
                assert!(
                    file_type.is_file() && !file_type.is_symlink(),
                    "oracle sources must remain flat regular files: {path:?}"
                );
                path.file_name()
                    .and_then(|name| name.to_str())
                    .expect("oracle source names should be UTF-8")
            })
            .collect::<Vec<_>>(),
        [
            "compare.rs",
            "coverage.rs",
            "hit.rs",
            "mod.rs",
            "model.rs",
            "mutation_failure.rs",
            "mutation_projection.rs",
            "mutations.rs",
            "numeric.rs",
            "paint.rs",
            "projection.rs",
            "scene.rs",
            "scene_static.rs",
            "types.rs",
        ]
    );

    let forbidden = [
        "fenestra_ui_",
        "fenestra-ui-",
        "generated_hybrid",
        "manual_hybrid",
        "macro_hybrid",
        "candidate",
        "AuthoredSpatialLaneLog",
        "Normalized",
        "run_authored_spatial_lane",
        "runtime::",
        "support::",
        "crate::",
        "super::super",
        "extern crate",
        "#[path",
        "include!",
        "include_str!",
        "include_bytes!",
        "env!",
        "option_env!",
        "core::",
        "alloc::",
    ];
    for path in entries {
        let source = std::fs::read_to_string(&path).expect("oracle source should be UTF-8");
        assert!(source.is_ascii(), "oracle source must be ASCII: {path:?}");
        for name in forbidden {
            assert!(
                !source.contains(name),
                "oracle source imports forbidden candidate boundary {name}: {path:?}"
            );
        }
        for line in source.lines().map(str::trim) {
            let import = line
                .strip_prefix("use ")
                .or_else(|| line.strip_prefix("pub use "));
            if let Some(import) = import {
                assert!(
                    import == "std::fmt;" || import.starts_with("super::"),
                    "oracle imports are closed to std::fmt and sibling modules: {path:?}: {line}"
                );
            }
            if let Some((_, qualified)) = line.split_once("std::") {
                assert!(
                    qualified.starts_with("fmt") || qualified.starts_with("array::"),
                    "oracle std use is closed to fmt and array: {path:?}: {line}"
                );
            }
        }
    }
}

#[test]
fn literal_oracle_detects_every_normalized_field_and_order_mutation() {
    let candidate = run_authored_spatial_lane(manual_hybrid_spatial_v2()).oracle_log();
    let expected = oracle::literal_oracle();

    assert_eq!(oracle::assert_mutation_controls(&candidate, &expected), 219);
}
