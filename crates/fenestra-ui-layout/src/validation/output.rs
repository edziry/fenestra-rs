use crate::error::{LayoutErrorLocationV1, LayoutOutputErrorKindV1};
use crate::model::{LayoutNodeV1, LayoutOutputV1};
use crate::vocabulary::{LayoutExtentV1, LayoutOutputFieldV1};

pub(crate) fn validate_output_v1(
    input_nodes: &[LayoutNodeV1],
    output: LayoutOutputV1,
) -> Result<LayoutOutputV1, (LayoutOutputErrorKindV1, LayoutErrorLocationV1)> {
    let records = output.records();
    if records.len() != input_nodes.len() {
        return Err((
            LayoutOutputErrorKindV1::RecordCountMismatch,
            LayoutErrorLocationV1::Output,
        ));
    }

    for (index, (input, record)) in input_nodes.iter().zip(records).enumerate() {
        if record.key() != input.key() {
            return Err((LayoutOutputErrorKindV1::KeyMismatch, record_location(index)));
        }
    }

    for (index, record) in records.iter().enumerate() {
        let bounds = record.bounds();
        for (value, field) in [
            (bounds.x(), LayoutOutputFieldV1::X),
            (bounds.y(), LayoutOutputFieldV1::Y),
            (bounds.width(), LayoutOutputFieldV1::Width),
            (bounds.height(), LayoutOutputFieldV1::Height),
        ] {
            if value < 0 {
                return Err((
                    LayoutOutputErrorKindV1::Negative(field),
                    record_location(index),
                ));
            }
        }
    }

    for (index, record) in records.iter().enumerate() {
        let bounds = record.bounds();
        if bounds.x().checked_add(bounds.width()).is_none() {
            return Err((
                LayoutOutputErrorKindV1::FarEdgeArithmetic(LayoutExtentV1::Width),
                record_location(index),
            ));
        }
        if bounds.y().checked_add(bounds.height()).is_none() {
            return Err((
                LayoutOutputErrorKindV1::FarEdgeArithmetic(LayoutExtentV1::Height),
                record_location(index),
            ));
        }
    }

    Ok(output)
}

fn record_location(index: usize) -> LayoutErrorLocationV1 {
    match u32::try_from(index) {
        Ok(index) => LayoutErrorLocationV1::OutputRecord { index },
        // Dense u32 input keys make this unreachable after input validation.
        // Keep an honest global location if the private seam is called out of order.
        Err(_) => LayoutErrorLocationV1::Output,
    }
}
