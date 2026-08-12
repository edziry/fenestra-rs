mod construction;
mod graph;
mod schema;
mod spatial;
mod style;

pub use construction::validate_construction;
pub use schema::validate_schema;
pub use spatial::validate_spatial;
pub use style::validate_style;

use crate::error::{IrValidationError, IrValidationErrorKind, ValidationLimitKind};
use crate::source::SourceSpan;

fn failure(kind: IrValidationErrorKind, span: SourceSpan) -> IrValidationError {
    IrValidationError::new(kind, span)
}

fn limit_failure(kind: ValidationLimitKind, span: SourceSpan) -> IrValidationError {
    failure(IrValidationErrorKind::LimitExceeded(kind), span)
}

fn add_count(
    count: &mut usize,
    limit: usize,
    kind: ValidationLimitKind,
    span: SourceSpan,
) -> Result<(), IrValidationError> {
    *count = count
        .checked_add(1)
        .ok_or_else(|| limit_failure(kind, span))?;
    if *count > limit {
        return Err(limit_failure(kind, span));
    }
    Ok(())
}
