use fenestra_ui_layout::prototype::{
    LayoutConstraintFieldV1, LayoutErrorLocationV1, LayoutExtentV1, LayoutNodeV1,
    LayoutPaddingSideV1, LayoutViewportV1,
};

use super::error::{
    CandidateProfileErrorFieldV1, CandidateProfileErrorKindV1, CandidateProfileErrorV1,
};

const CANDIDATE_COORDINATE_LIMIT_V1: i32 = 4096;

pub(crate) fn validate_candidate_input_v1(
    viewport: LayoutViewportV1,
    nodes: &[LayoutNodeV1],
) -> Result<(), CandidateProfileErrorV1> {
    check_viewport(viewport.width(), LayoutExtentV1::Width)?;
    check_viewport(viewport.height(), LayoutExtentV1::Height)?;

    for (index, node) in nodes.iter().copied().enumerate() {
        let location = input_node_location(index);
        let style = node.style();
        let width = style.width();
        check_constraint(
            width.minimum(),
            LayoutExtentV1::Width,
            LayoutConstraintFieldV1::Minimum,
            location,
        )?;
        check_constraint(
            width.preferred(),
            LayoutExtentV1::Width,
            LayoutConstraintFieldV1::Preferred,
            location,
        )?;
        check_constraint(
            width.maximum(),
            LayoutExtentV1::Width,
            LayoutConstraintFieldV1::Maximum,
            location,
        )?;

        let height = style.height();
        check_constraint(
            height.minimum(),
            LayoutExtentV1::Height,
            LayoutConstraintFieldV1::Minimum,
            location,
        )?;
        check_constraint(
            height.preferred(),
            LayoutExtentV1::Height,
            LayoutConstraintFieldV1::Preferred,
            location,
        )?;
        check_constraint(
            height.maximum(),
            LayoutExtentV1::Height,
            LayoutConstraintFieldV1::Maximum,
            location,
        )?;

        let padding = style.padding();
        check_padding(padding.left(), LayoutPaddingSideV1::Left, location)?;
        check_padding(padding.right(), LayoutPaddingSideV1::Right, location)?;
        check_padding(padding.top(), LayoutPaddingSideV1::Top, location)?;
        check_padding(padding.bottom(), LayoutPaddingSideV1::Bottom, location)?;
        check_scalar(style.gap(), CandidateProfileErrorFieldV1::Gap, location)?;
    }

    Ok(())
}

fn check_viewport(value: i32, extent: LayoutExtentV1) -> Result<(), CandidateProfileErrorV1> {
    check_scalar(
        value,
        CandidateProfileErrorFieldV1::Viewport(extent),
        LayoutErrorLocationV1::Viewport,
    )
}

fn check_constraint(
    value: i32,
    extent: LayoutExtentV1,
    field: LayoutConstraintFieldV1,
    location: LayoutErrorLocationV1,
) -> Result<(), CandidateProfileErrorV1> {
    check_scalar(
        value,
        CandidateProfileErrorFieldV1::Constraint { extent, field },
        location,
    )
}

fn check_padding(
    value: i32,
    side: LayoutPaddingSideV1,
    location: LayoutErrorLocationV1,
) -> Result<(), CandidateProfileErrorV1> {
    check_scalar(value, CandidateProfileErrorFieldV1::Padding(side), location)
}

fn check_scalar(
    value: i32,
    field: CandidateProfileErrorFieldV1,
    location: LayoutErrorLocationV1,
) -> Result<(), CandidateProfileErrorV1> {
    if value > CANDIDATE_COORDINATE_LIMIT_V1 {
        return Err(CandidateProfileErrorV1::new(
            CandidateProfileErrorKindV1::CoordinateLimit,
            field,
            location,
        ));
    }
    Ok(())
}

fn input_node_location(index: usize) -> LayoutErrorLocationV1 {
    match u32::try_from(index) {
        Ok(index) => LayoutErrorLocationV1::InputNode { index },
        Err(_) => LayoutErrorLocationV1::Input,
    }
}
