use fenestra_ui_ir::prototype::{
    SourceSpan, SpatialBrushContentV2, SpatialShapeGeometryV2, ValidatedSpatialProgramV2,
};

use super::super::error::{RuntimeSpatialIrErrorKindV2, RuntimeSpatialIrErrorV2};
use super::counts::DirectCounts;
use super::model::LiveProgram;

pub(super) fn check_representation(
    program: &ValidatedSpatialProgramV2,
    live: &LiveProgram<'_>,
    counts: &DirectCounts,
) -> Result<(), RuntimeSpatialIrErrorV2> {
    let program_span = program.program().span();
    for (index, count) in counts.observed().into_iter().enumerate() {
        if index <= 7 || index == 11 {
            checked_row_count(count, program_span)?;
        }
        checked_usize(count, program_span)?;
    }

    check_nodes(live)?;
    check_shapes(live)?;
    check_brushes(live)?;
    check_clips(live)?;
    check_items(live, ItemTable::Paint)?;
    check_items(live, ItemTable::Hit)?;
    check_items(live, ItemTable::Semantic)?;
    check_paths(live)?;
    check_path_verbs(live)?;
    check_polygon_points(live)?;
    check_gradient_stops(live)?;
    check_images(program)?;
    Ok(())
}

fn check_nodes(live: &LiveProgram<'_>) -> Result<(), RuntimeSpatialIrErrorV2> {
    for owner in live.expanded() {
        checked_u32(owner.ordinal(), owner.declaration().span())?;
        checked_u32(owner.parent_ordinal(), owner.declaration().span())?;
    }
    Ok(())
}

fn check_shapes(live: &LiveProgram<'_>) -> Result<(), RuntimeSpatialIrErrorV2> {
    let mut cursor = 0u128;
    for owner in live.expanded() {
        for shape in owner.declaration().shapes() {
            checked_u32(cursor, shape.span())?;
            increment(&mut cursor, shape.span())?;
        }
    }
    Ok(())
}

fn check_brushes(live: &LiveProgram<'_>) -> Result<(), RuntimeSpatialIrErrorV2> {
    let mut cursor = 0u128;
    for owner in live.expanded() {
        for brush in owner.declaration().brushes() {
            checked_u32(cursor, brush.span())?;
            increment(&mut cursor, brush.span())?;
        }
    }
    Ok(())
}

fn check_clips(live: &LiveProgram<'_>) -> Result<(), RuntimeSpatialIrErrorV2> {
    let mut cursor = 0u128;
    for owner in live.expanded() {
        for clip in owner.declaration().clips() {
            checked_u32(cursor, clip.span())?;
            increment(&mut cursor, clip.span())?;
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum ItemTable {
    Paint,
    Hit,
    Semantic,
}

fn check_items(live: &LiveProgram<'_>, table: ItemTable) -> Result<(), RuntimeSpatialIrErrorV2> {
    let mut cursor = 0u128;
    for owner in live.expanded() {
        match table {
            ItemTable::Paint => check_item_records(
                &mut cursor,
                owner
                    .declaration()
                    .paint_items()
                    .iter()
                    .map(|record| record.span()),
            )?,
            ItemTable::Hit => check_item_records(
                &mut cursor,
                owner
                    .declaration()
                    .hit_items()
                    .iter()
                    .map(|record| record.span()),
            )?,
            ItemTable::Semantic => check_item_records(
                &mut cursor,
                owner
                    .declaration()
                    .semantic_items()
                    .iter()
                    .map(|record| record.span()),
            )?,
        }
    }
    Ok(())
}

fn check_item_records(
    cursor: &mut u128,
    spans: impl Iterator<Item = SourceSpan>,
) -> Result<(), RuntimeSpatialIrErrorV2> {
    for (ordinal, span) in spans.enumerate() {
        checked_u32(*cursor, span)?;
        increment(cursor, span)?;
        checked_u32(
            u128::try_from(ordinal).expect("usize always fits u128"),
            span,
        )?;
    }
    Ok(())
}

fn check_paths(live: &LiveProgram<'_>) -> Result<(), RuntimeSpatialIrErrorV2> {
    let mut cursor = 0u128;
    for owner in live.expanded() {
        for shape in owner.declaration().shapes() {
            if matches!(shape.geometry(), SpatialShapeGeometryV2::Path { .. }) {
                checked_u32(cursor, shape.span())?;
                increment(&mut cursor, shape.span())?;
            }
        }
    }
    Ok(())
}

fn check_path_verbs(live: &LiveProgram<'_>) -> Result<(), RuntimeSpatialIrErrorV2> {
    let mut cursor = 0u128;
    for owner in live.expanded() {
        for shape in owner.declaration().shapes() {
            if let SpatialShapeGeometryV2::Path { verbs } = shape.geometry() {
                check_range(&mut cursor, verbs.len(), shape.span())?;
            }
        }
    }
    Ok(())
}

fn check_polygon_points(live: &LiveProgram<'_>) -> Result<(), RuntimeSpatialIrErrorV2> {
    let mut cursor = 0u128;
    for owner in live.expanded() {
        for shape in owner.declaration().shapes() {
            if let SpatialShapeGeometryV2::Polygon { points } = shape.geometry() {
                check_range(&mut cursor, points.len(), shape.span())?;
            }
        }
    }
    Ok(())
}

fn check_gradient_stops(live: &LiveProgram<'_>) -> Result<(), RuntimeSpatialIrErrorV2> {
    let mut cursor = 0u128;
    for owner in live.expanded() {
        for brush in owner.declaration().brushes() {
            if let SpatialBrushContentV2::LinearGradient { stops, .. } = brush.content() {
                check_range(&mut cursor, stops.len(), brush.span())?;
            }
        }
    }
    Ok(())
}

fn check_images(program: &ValidatedSpatialProgramV2) -> Result<(), RuntimeSpatialIrErrorV2> {
    let mut key = 0u128;
    let mut byte_total = 0u128;
    for image in program.program().images() {
        checked_u32(key, image.span())?;
        increment(&mut key, image.span())?;
        let length = u128::try_from(image.bytes().len()).expect("usize always fits u128");
        checked_usize(length, image.span())?;
        byte_total = byte_total
            .checked_add(length)
            .ok_or_else(|| arithmetic(image.span()))?;
        checked_usize(byte_total, image.span())?;
    }
    Ok(())
}

fn check_range(
    cursor: &mut u128,
    length: usize,
    span: SourceSpan,
) -> Result<(), RuntimeSpatialIrErrorV2> {
    let length = u128::try_from(length).expect("usize always fits u128");
    checked_u32(*cursor, span)?;
    checked_u32(length, span)?;
    *cursor = cursor.checked_add(length).ok_or_else(|| arithmetic(span))?;
    checked_usize(*cursor, span)?;
    Ok(())
}

fn increment(cursor: &mut u128, span: SourceSpan) -> Result<(), RuntimeSpatialIrErrorV2> {
    *cursor = cursor.checked_add(1).ok_or_else(|| arithmetic(span))?;
    Ok(())
}

fn checked_u32(value: u128, span: SourceSpan) -> Result<(), RuntimeSpatialIrErrorV2> {
    u32::try_from(value)
        .map(|_| ())
        .map_err(|_| arithmetic(span))
}

fn checked_row_count(value: u128, span: SourceSpan) -> Result<(), RuntimeSpatialIrErrorV2> {
    (value <= u32::MAX as u128 + 1)
        .then_some(())
        .ok_or_else(|| arithmetic(span))
}

fn checked_usize(value: u128, span: SourceSpan) -> Result<(), RuntimeSpatialIrErrorV2> {
    usize::try_from(value)
        .map(|_| ())
        .map_err(|_| arithmetic(span))
}

fn arithmetic(span: SourceSpan) -> RuntimeSpatialIrErrorV2 {
    RuntimeSpatialIrErrorV2::new(RuntimeSpatialIrErrorKindV2::ArithmeticExhausted, span)
}
