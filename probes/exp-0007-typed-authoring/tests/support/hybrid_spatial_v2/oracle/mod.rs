mod compare;
mod coverage;
mod hit;
mod model;
mod mutation_failure;
mod mutation_projection;
mod mutations;
mod numeric;
mod paint;
mod projection;
mod scene;
mod scene_static;
pub(crate) mod types;

pub type Log = types::Log;

pub fn assert_log_eq(actual: &Log, expected: &Log) {
    compare::assert_log_eq(actual, expected);
}

pub fn assert_mutation_controls(actual: &Log, expected: &Log) -> usize {
    mutations::assert_mutation_controls(actual, expected)
}

pub fn literal_oracle() -> Log {
    let mut model = model::Model::initial();
    let mut observations = Vec::with_capacity(9);
    for step in 0..=8 {
        let receipt = if step == 0 {
            model::initial_receipt()
        } else {
            model.apply(step)
        };
        let scene = scene::Scene::build(&model);
        let projection = projection::build(&scene);
        let hit_queries = hit::queries(&scene, &projection, model.viewport);
        let raster = paint::raster(&scene, &projection, model.viewport);
        observations.push(types::Observation {
            generation: u64::try_from(step).expect("registered generation should fit"),
            viewport: model.viewport,
            receipt,
            state: model.state(),
            projection,
            hit_queries,
            raster,
        });
    }
    Log {
        observations,
        final_keys: model.keys.iter().map(|(key, _)| *key).collect(),
        noop: types::Noop {
            empty_preserved: true,
            same_value_preserved: true,
            round_trip_preserved: true,
        },
        failure: types::Failure {
            kind: types::FailureKind::SingularTransform,
            location: types::FailureLocation::Node { index: 3 },
            ir_span: types::Span::Bytes {
                source: 0,
                start: 226,
                end: 227,
            },
            operation_index: None,
            outer_state_preserved: true,
            spatial_snapshot_preserved: true,
            complete_observation_preserved: true,
            authored_factor_span: types::Span::Bytes {
                source: 0,
                start: 247,
                end: 248,
            },
        },
    }
}
