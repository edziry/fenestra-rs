use std::fmt;

use super::types::{Log, Observation};

#[derive(Debug, Eq, PartialEq)]
pub(super) struct Difference {
    label: String,
}

impl fmt::Display for Difference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.label)
    }
}

pub fn assert_log_eq(actual: &Log, expected: &Log) {
    if let Err(difference) = compare_log(actual, expected) {
        panic!("literal oracle mismatch: {difference}");
    }
}

pub(super) fn compare_log(actual: &Log, expected: &Log) -> Result<(), Difference> {
    equal(&actual.final_keys, &expected.final_keys, "final keys")?;
    equal(&actual.noop, &expected.noop, "identity no-op checks")?;
    equal(&actual.failure, &expected.failure, "typed rollback failure")?;
    equal(
        &actual.observations.len(),
        &expected.observations.len(),
        "observation count",
    )?;
    for (step, (actual, expected)) in actual
        .observations
        .iter()
        .zip(&expected.observations)
        .enumerate()
    {
        compare_observation(step, actual, expected)?;
    }
    Ok(())
}

fn compare_observation(
    step: usize,
    actual: &Observation,
    expected: &Observation,
) -> Result<(), Difference> {
    equal(
        &actual.generation,
        &expected.generation,
        format!("step {step} generation"),
    )?;
    equal(
        &actual.viewport,
        &expected.viewport,
        format!("step {step} viewport"),
    )?;
    equal(
        &actual.receipt,
        &expected.receipt,
        format!("step {step} receipt"),
    )?;
    equal(&actual.state, &expected.state, format!("step {step} state"))?;
    equal(
        &actual.projection.mapping,
        &expected.projection.mapping,
        format!("step {step} mapping"),
    )?;
    equal(
        &actual.projection.geometry,
        &expected.projection.geometry,
        format!("step {step} geometry"),
    )?;
    equal(
        &actual.projection.clips,
        &expected.projection.clips,
        format!("step {step} clips"),
    )?;
    equal(
        &actual.projection.paints,
        &expected.projection.paints,
        format!("step {step} paints"),
    )?;
    equal(
        &actual.projection.hits,
        &expected.projection.hits,
        format!("step {step} hits"),
    )?;
    equal(
        &actual.projection.semantics,
        &expected.projection.semantics,
        format!("step {step} semantics"),
    )?;
    if actual.hit_queries != expected.hit_queries {
        let index = actual
            .hit_queries
            .iter()
            .zip(&expected.hit_queries)
            .position(|(left, right)| left != right)
            .unwrap_or(actual.hit_queries.len().min(expected.hit_queries.len()));
        return Err(Difference {
            label: format!("step {step} hit query {index}"),
        });
    }
    equal(
        &actual.raster.width,
        &expected.raster.width,
        format!("step {step} raster width"),
    )?;
    equal(
        &actual.raster.height,
        &expected.raster.height,
        format!("step {step} raster height"),
    )?;
    equal(
        &actual.raster.stride,
        &expected.raster.stride,
        format!("step {step} raster stride"),
    )?;
    if actual.raster.bytes != expected.raster.bytes {
        let index = actual
            .raster
            .bytes
            .iter()
            .zip(expected.raster.bytes.iter())
            .position(|(left, right)| left != right)
            .unwrap_or(actual.raster.bytes.len().min(expected.raster.bytes.len()));
        return Err(Difference {
            label: format!("step {step} raster byte {index}"),
        });
    }
    Ok(())
}

fn equal<T: PartialEq>(
    actual: &T,
    expected: &T,
    label: impl Into<String>,
) -> Result<(), Difference> {
    if actual == expected {
        Ok(())
    } else {
        Err(Difference {
            label: label.into(),
        })
    }
}
