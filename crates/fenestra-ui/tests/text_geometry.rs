use fenestra_ui::{
    TextAffinity, TextError, TextGeometryQuery, TextGeometryRequest, TextLimits,
    TextMeasureRequest, TextPosition, TextSelection, TextStyle,
};

mod text_geometry {
    mod compatibility;
    mod output;
    mod requests;
}

fn position(byte: usize) -> TextPosition {
    TextPosition::new(byte, TextAffinity::Downstream)
}

fn selection(anchor: usize, focus: usize) -> TextSelection {
    TextSelection::new(position(anchor), position(focus))
}

fn request(
    text: &str,
    selection: TextSelection,
    query: TextGeometryQuery,
) -> Result<TextGeometryRequest<'_>, TextError> {
    let measure = TextMeasureRequest::new(
        text,
        TextStyle::new(),
        None,
        TextLimits::new(text.len(), 0, text.len()),
    )?;
    TextGeometryRequest::new(measure, selection, query)
}
