use super::*;
use support::*;

#[test]
fn remaining_closed_payload_branches_validate() {
    let style = style();
    let path = SpatialShapeDeclarationV2::new(
        field(SpatialShapeSymbolV2::new(0), span(4000)),
        SpatialShapeGeometryV2::Path {
            verbs: vec![
                SpatialPathVerbRecipeV2::MoveTo {
                    to: point(0, 0, 4001),
                    span: span(4003),
                },
                SpatialPathVerbRecipeV2::QuadraticTo {
                    control: point(1, 1, 4004),
                    to: point(2, 2, 4006),
                    span: span(4008),
                },
                SpatialPathVerbRecipeV2::CubicTo {
                    control1: point(3, 3, 4009),
                    control2: point(4, 4, 4011),
                    to: point(5, 5, 4013),
                    span: span(4015),
                },
                SpatialPathVerbRecipeV2::Close { span: span(4016) },
            ],
        },
        span(4017),
    );
    let hit = SpatialHitRecipeV2::new(
        SpatialCoverageRecipeV2::RoundStroke {
            shape: field(SpatialShapeSymbolV2::new(0), span(4020)),
            width: field(SpatialBindingV2::Property(SCALAR), span(4021)),
        },
        None,
        field(SpatialBindingV2::Literal(InputPolicy::Accept), span(4022)),
        span(4023),
    );
    let image = SpatialImageDeclarationV2::new(
        field(SpatialImageSymbolV2::new(0), span(4030)),
        field(1, span(4031)),
        field(1, span(4032)),
        field(4, span(4033)),
        vec![0, 0, 0, 0].into_boxed_slice(),
        span(4034),
    );
    let image_paint = SpatialPaintRecipeV2::ImagePaint {
        image: field(SpatialImageSymbolV2::new(0), span(4040)),
        source_x: field(0, span(4041)),
        source_y: field(0, span(4042)),
        source_width: field(1, span(4043)),
        source_height: field(1, span(4044)),
        destination_origin: SpatialPointRecipeV2::new(
            field(SpatialBindingV2::Property(SCALAR), span(4045)),
            field(SpatialBindingV2::Property(SCALAR), span(4046)),
        ),
        destination_width: field(SpatialBindingV2::Property(SCALAR), span(4047)),
        destination_height: field(SpatialBindingV2::Property(SCALAR), span(4048)),
        opacity: field(255, span(4049)),
        clip: None,
        span: span(4050),
    };
    let root = node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        free_placement(SpatialAnchorTargetRecipeV2::Viewport, 4060),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        4059,
    );
    let child = node_with(
        1,
        STATIC_A,
        parent(0, 4070),
        free_placement(SpatialAnchorTargetRecipeV2::Parent, 4071),
        vec![path],
        Vec::new(),
        Vec::new(),
        vec![image_paint],
        vec![hit],
        Vec::new(),
        4069,
    );
    let input = program_with(
        SUPPORTED_SPATIAL_FORMAT,
        NS,
        REV,
        viewport(4080),
        vec![root, child],
        vec![image],
        span(4086),
    );
    validate(&style, input).expect("every remaining closed branch should validate");
}

#[test]
fn raw_spatial_semantics_remain_deferred() {
    let style = style();
    let dimensions =
        SpatialDimensionRecipeV2::new(lit_i(10, 4100), lit_i(-5, 4101), lit_i(0, 4102));
    let singular = SpatialTransformRecipeV2::new(
        lit_f(0, 4103),
        lit_f(0, 4104),
        lit_f(0, 4105),
        lit_f(0, 4106),
        lit_f(0, 4107),
        lit_f(0, 4108),
        point(0, 0, 4109),
    );
    let malformed_path = SpatialShapeDeclarationV2::new(
        field(SpatialShapeSymbolV2::new(0), span(4111)),
        SpatialShapeGeometryV2::Path {
            verbs: vec![SpatialPathVerbRecipeV2::Close { span: span(4112) }],
        },
        span(4113),
    );
    let gradient = SpatialBrushDeclarationV2::new(
        field(SpatialBrushSymbolV2::new(0), span(4120)),
        SpatialBrushContentV2::LinearGradient {
            start: point(0, 0, 4121),
            end: point(0, 0, 4123),
            stops: vec![SpatialGradientStopV2::new(
                field(u16::MAX, span(4125)),
                field(SpatialBindingV2::Literal([255, 255, 255, 1]), span(4126)),
                span(4127),
            )],
        },
        span(4128),
    );
    let hit = SpatialHitRecipeV2::new(
        SpatialCoverageRecipeV2::RoundStroke {
            shape: field(SpatialShapeSymbolV2::new(0), span(4130)),
            width: lit_f(-1, 4131),
        },
        None,
        field(SpatialBindingV2::Literal(InputPolicy::Ignore), span(4132)),
        span(4133),
    );
    let declaration = node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        SpatialPlacementRecipeV2::Layout(SpatialLayoutPlacementRecipeV2::new(
            dimensions, dimensions, singular,
        )),
        vec![malformed_path],
        vec![gradient],
        Vec::new(),
        Vec::new(),
        vec![hit],
        Vec::new(),
        4099,
    );
    let invalid_image = SpatialImageDeclarationV2::new(
        field(SpatialImageSymbolV2::new(0), span(4140)),
        field(0, span(4141)),
        field(0, span(4142)),
        field(0, span(4143)),
        Vec::new().into_boxed_slice(),
        span(4144),
    );
    let input = program_with(
        SUPPORTED_SPATIAL_FORMAT,
        NS,
        REV,
        viewport(4150),
        vec![declaration],
        vec![invalid_image],
        span(4156),
    );

    validate(&style, input).expect("raw resolver authority must remain deferred");
}
