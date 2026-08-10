use fenestra_ui_layout::prototype::{
    LayoutErrorLocationV1, LayoutExtentV1, LayoutNodeKeyV1, LayoutNodeV1, LayoutOutputFieldV1,
    LayoutOutputV1, LayoutRecordV1, LayoutRectV1,
};

use super::error::{
    CandidateEdgeV1, CandidateProfileErrorFieldV1, CandidateProfileErrorKindV1,
    CandidateProfileErrorV1,
};

const CANDIDATE_OUTPUT_EDGE_LIMIT_V1: f32 = 524_288.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CandidateRawRecordV1 {
    key: LayoutNodeKeyV1,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl CandidateRawRecordV1 {
    pub(crate) const fn new(key: LayoutNodeKeyV1, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            key,
            x,
            y,
            width,
            height,
        }
    }

    pub(crate) const fn key(self) -> LayoutNodeKeyV1 {
        self.key
    }
}

#[derive(Clone, Copy)]
struct AbsoluteEdgesV1 {
    x_near: f32,
    x_far: f32,
    y_near: f32,
    y_far: f32,
}

pub(crate) fn convert_candidate_output_v1(
    nodes: &[LayoutNodeV1],
    raw_records: &[CandidateRawRecordV1],
) -> Result<LayoutOutputV1, CandidateProfileErrorV1> {
    validate_raw_fields_v1(raw_records, RawFieldPhaseV1::NonFinite)?;
    validate_raw_fields_v1(raw_records, RawFieldPhaseV1::Negative)?;

    let mut absolute_edges: Vec<AbsoluteEdgesV1> = Vec::with_capacity(raw_records.len());
    for (index, raw) in raw_records.iter().copied().enumerate() {
        let (parent_x, parent_y) = match nodes[index].parent() {
            Some(parent) => {
                let parent_edges = absolute_edges[parent.get() as usize];
                (parent_edges.x_near, parent_edges.y_near)
            }
            None => (0.0, 0.0),
        };
        let edges = AbsoluteEdgesV1 {
            x_near: parent_x + raw.x,
            x_far: parent_x + raw.x + raw.width,
            y_near: parent_y + raw.y,
            y_far: parent_y + raw.y + raw.height,
        };
        validate_edge_v1(
            edges.x_near,
            LayoutExtentV1::Width,
            CandidateEdgeV1::Near,
            index,
        )?;
        validate_edge_v1(
            edges.x_far,
            LayoutExtentV1::Width,
            CandidateEdgeV1::Far,
            index,
        )?;
        validate_edge_v1(
            edges.y_near,
            LayoutExtentV1::Height,
            CandidateEdgeV1::Near,
            index,
        )?;
        validate_edge_v1(
            edges.y_far,
            LayoutExtentV1::Height,
            CandidateEdgeV1::Far,
            index,
        )?;
        absolute_edges.push(edges);
    }

    let records = raw_records
        .iter()
        .copied()
        .zip(absolute_edges)
        .map(|(raw, edges)| {
            let x_near = edges.x_near.round() as i32;
            let x_far = edges.x_far.round() as i32;
            let y_near = edges.y_near.round() as i32;
            let y_far = edges.y_far.round() as i32;
            LayoutRecordV1::new(
                raw.key,
                LayoutRectV1::new(x_near, y_near, x_far - x_near, y_far - y_near),
            )
        })
        .collect();
    Ok(LayoutOutputV1::new(records))
}

#[derive(Clone, Copy)]
enum RawFieldPhaseV1 {
    NonFinite,
    Negative,
}

fn validate_raw_fields_v1(
    raw_records: &[CandidateRawRecordV1],
    phase: RawFieldPhaseV1,
) -> Result<(), CandidateProfileErrorV1> {
    for (index, raw) in raw_records.iter().copied().enumerate() {
        validate_raw_field_v1(raw.x, LayoutOutputFieldV1::X, index, phase)?;
        validate_raw_field_v1(raw.y, LayoutOutputFieldV1::Y, index, phase)?;
        validate_raw_field_v1(raw.width, LayoutOutputFieldV1::Width, index, phase)?;
        validate_raw_field_v1(raw.height, LayoutOutputFieldV1::Height, index, phase)?;
    }
    Ok(())
}

fn validate_raw_field_v1(
    value: f32,
    field: LayoutOutputFieldV1,
    index: usize,
    phase: RawFieldPhaseV1,
) -> Result<(), CandidateProfileErrorV1> {
    let rejected = match phase {
        RawFieldPhaseV1::NonFinite => !value.is_finite(),
        RawFieldPhaseV1::Negative => value < 0.0,
    };
    if rejected {
        let kind = match phase {
            RawFieldPhaseV1::NonFinite => CandidateProfileErrorKindV1::NonFiniteOutput,
            RawFieldPhaseV1::Negative => CandidateProfileErrorKindV1::NegativeOutput,
        };
        return Err(CandidateProfileErrorV1::new(
            kind,
            CandidateProfileErrorFieldV1::Output(field),
            output_record_location(index),
        ));
    }
    Ok(())
}

fn validate_edge_v1(
    value: f32,
    extent: LayoutExtentV1,
    edge: CandidateEdgeV1,
    index: usize,
) -> Result<(), CandidateProfileErrorV1> {
    let kind = if !value.is_finite() {
        Some(CandidateProfileErrorKindV1::NonFiniteOutput)
    } else if value < 0.0 {
        Some(CandidateProfileErrorKindV1::NegativeOutput)
    } else if value > CANDIDATE_OUTPUT_EDGE_LIMIT_V1 {
        Some(CandidateProfileErrorKindV1::OutputEdgeLimit)
    } else {
        None
    };
    if let Some(kind) = kind {
        return Err(CandidateProfileErrorV1::new(
            kind,
            CandidateProfileErrorFieldV1::OutputEdge { extent, edge },
            output_record_location(index),
        ));
    }
    Ok(())
}

fn output_record_location(index: usize) -> LayoutErrorLocationV1 {
    match u32::try_from(index) {
        Ok(index) => LayoutErrorLocationV1::OutputRecord { index },
        Err(_) => LayoutErrorLocationV1::Output,
    }
}
