use super::super::*;

#[test]
fn limit_phase_vocabularies_and_registered_values_are_exact() {
    use SpatialLimitKindV2::{
        Brushes, ChildrenPerNode, ClipDepth, Clips, DependencyEdges, DependencyVertices, Depth,
        FlattenedSegmentsPerPath, FlattenedSegmentsTotal, GradientStopsPerBrush,
        GradientStopsTotal, HitItems, HitItemsPerNode, ImageEdge, ImagePixelsTotal, Images,
        Islands, LayoutInputRecordsPerIsland, LayoutInputRecordsTotal, Nodes, PaintItems,
        PaintItemsPerNode, PathSubpathsTotal, PathVerbsPerPath, PathVerbsTotal, Paths,
        PolygonPointsPerShape, PolygonPointsTotal, SemanticItems, Shapes,
    };

    assert_eq!(
        SpatialLimitKindV2::DIRECT_ALL,
        [
            Nodes,
            Shapes,
            Brushes,
            Clips,
            PaintItems,
            HitItems,
            SemanticItems,
            Paths,
            PathVerbsTotal,
            PolygonPointsTotal,
            GradientStopsTotal,
            Images,
        ]
    );
    assert_eq!(SpatialLimitKindV2::TOPOLOGY_ALL, [Depth, ChildrenPerNode]);
    assert_eq!(
        SpatialLimitKindV2::ISLAND_ALL,
        [
            Islands,
            LayoutInputRecordsPerIsland,
            LayoutInputRecordsTotal,
        ]
    );
    assert_eq!(
        SpatialLimitKindV2::CONTENT_ALL,
        [
            PathVerbsPerPath,
            PathSubpathsTotal,
            PolygonPointsPerShape,
            GradientStopsPerBrush,
            ImageEdge,
            ImagePixelsTotal,
            ClipDepth,
            PaintItemsPerNode,
            HitItemsPerNode,
            FlattenedSegmentsPerPath,
            FlattenedSegmentsTotal,
        ]
    );
    assert_eq!(
        SpatialLimitKindV2::DEPENDENCY_ALL,
        [DependencyVertices, DependencyEdges]
    );

    let expected_kinds = [
        Nodes,
        Shapes,
        Brushes,
        Clips,
        PaintItems,
        HitItems,
        SemanticItems,
        Paths,
        PathVerbsTotal,
        PolygonPointsTotal,
        GradientStopsTotal,
        Images,
        Depth,
        ChildrenPerNode,
        Islands,
        LayoutInputRecordsPerIsland,
        LayoutInputRecordsTotal,
        PathVerbsPerPath,
        PathSubpathsTotal,
        PolygonPointsPerShape,
        GradientStopsPerBrush,
        ImageEdge,
        ImagePixelsTotal,
        ClipDepth,
        PaintItemsPerNode,
        HitItemsPerNode,
        FlattenedSegmentsPerPath,
        FlattenedSegmentsTotal,
        DependencyVertices,
        DependencyEdges,
    ];
    assert_eq!(SpatialLimitKindV2::ALL, expected_kinds);

    let expected_values = [
        256, 1024, 256, 512, 1024, 512, 256, 256, 4096, 4096, 2048, 64, 32, 64, 64, 128, 192, 256,
        1024, 256, 32, 4096, 4_194_304, 32, 64, 64, 4096, 65_536, 192, 256,
    ];
    let explicit = SpatialLimitsV2::new(expected_values);
    assert_eq!(REGISTERED_SPATIAL_LIMITS_V2, explicit);
    for (kind, value) in SpatialLimitKindV2::ALL.into_iter().zip(expected_values) {
        assert_eq!(REGISTERED_SPATIAL_LIMITS_V2.limit(kind), value);
    }

    let distinct_values = core::array::from_fn(|index| index);
    let distinct = SpatialLimitsV2::new(distinct_values);
    for (index, kind) in SpatialLimitKindV2::ALL.into_iter().enumerate() {
        assert_eq!(distinct.limit(kind), index);
    }
}
