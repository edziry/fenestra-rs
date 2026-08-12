/// Closed capacity vocabulary for version-2 spatial resolution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialLimitKindV2 {
    /// Total spatial node records, including the sentinel.
    Nodes,
    /// Total shape records.
    Shapes,
    /// Total brush records.
    Brushes,
    /// Total clip records.
    Clips,
    /// Total paint items.
    PaintItems,
    /// Total hit items.
    HitItems,
    /// Total semantic items.
    SemanticItems,
    /// Total path records.
    Paths,
    /// Total path verb records.
    PathVerbsTotal,
    /// Total polygon point records.
    PolygonPointsTotal,
    /// Total gradient stop records.
    GradientStopsTotal,
    /// Total decoded image resources.
    Images,
    /// Maximum spatial tree depth.
    Depth,
    /// Maximum direct children on one node.
    ChildrenPerNode,
    /// Total nonempty layout islands.
    Islands,
    /// Maximum layout input records in one nonempty island.
    LayoutInputRecordsPerIsland,
    /// Total layout input records across nonempty islands.
    LayoutInputRecordsTotal,
    /// Maximum path verbs in one path.
    PathVerbsPerPath,
    /// Total path subpaths.
    PathSubpathsTotal,
    /// Maximum polygon points in one shape.
    PolygonPointsPerShape,
    /// Maximum gradient stops in one brush.
    GradientStopsPerBrush,
    /// Maximum decoded image edge.
    ImageEdge,
    /// Total decoded image pixels.
    ImagePixelsTotal,
    /// Maximum effective clip-chain depth.
    ClipDepth,
    /// Maximum paint items owned by one node.
    PaintItemsPerNode,
    /// Maximum hit items owned by one node.
    HitItemsPerNode,
    /// Maximum flattened segments in one path.
    FlattenedSegmentsPerPath,
    /// Total flattened path segments.
    FlattenedSegmentsTotal,
    /// Total placement dependency vertices.
    DependencyVertices,
    /// Total placement dependency edges.
    DependencyEdges,
}

impl SpatialLimitKindV2 {
    /// Direct table-count limits in validation order.
    pub const DIRECT_ALL: [Self; 12] = [
        Self::Nodes,
        Self::Shapes,
        Self::Brushes,
        Self::Clips,
        Self::PaintItems,
        Self::HitItems,
        Self::SemanticItems,
        Self::Paths,
        Self::PathVerbsTotal,
        Self::PolygonPointsTotal,
        Self::GradientStopsTotal,
        Self::Images,
    ];

    /// Topology-derived limits in validation order.
    pub const TOPOLOGY_ALL: [Self; 2] = [Self::Depth, Self::ChildrenPerNode];

    /// Layout-island limits in validation order.
    pub const ISLAND_ALL: [Self; 3] = [
        Self::Islands,
        Self::LayoutInputRecordsPerIsland,
        Self::LayoutInputRecordsTotal,
    ];

    /// Content-derived limits in validation order.
    pub const CONTENT_ALL: [Self; 11] = [
        Self::PathVerbsPerPath,
        Self::PathSubpathsTotal,
        Self::PolygonPointsPerShape,
        Self::GradientStopsPerBrush,
        Self::ImageEdge,
        Self::ImagePixelsTotal,
        Self::ClipDepth,
        Self::PaintItemsPerNode,
        Self::HitItemsPerNode,
        Self::FlattenedSegmentsPerPath,
        Self::FlattenedSegmentsTotal,
    ];

    /// Dependency-graph limits in validation order.
    pub const DEPENDENCY_ALL: [Self; 2] = [Self::DependencyVertices, Self::DependencyEdges];

    /// Every spatial limit in complete validation order.
    pub const ALL: [Self; 30] = [
        Self::Nodes,
        Self::Shapes,
        Self::Brushes,
        Self::Clips,
        Self::PaintItems,
        Self::HitItems,
        Self::SemanticItems,
        Self::Paths,
        Self::PathVerbsTotal,
        Self::PolygonPointsTotal,
        Self::GradientStopsTotal,
        Self::Images,
        Self::Depth,
        Self::ChildrenPerNode,
        Self::Islands,
        Self::LayoutInputRecordsPerIsland,
        Self::LayoutInputRecordsTotal,
        Self::PathVerbsPerPath,
        Self::PathSubpathsTotal,
        Self::PolygonPointsPerShape,
        Self::GradientStopsPerBrush,
        Self::ImageEdge,
        Self::ImagePixelsTotal,
        Self::ClipDepth,
        Self::PaintItemsPerNode,
        Self::HitItemsPerNode,
        Self::FlattenedSegmentsPerPath,
        Self::FlattenedSegmentsTotal,
        Self::DependencyVertices,
        Self::DependencyEdges,
    ];

    const fn index(self) -> usize {
        match self {
            Self::Nodes => 0,
            Self::Shapes => 1,
            Self::Brushes => 2,
            Self::Clips => 3,
            Self::PaintItems => 4,
            Self::HitItems => 5,
            Self::SemanticItems => 6,
            Self::Paths => 7,
            Self::PathVerbsTotal => 8,
            Self::PolygonPointsTotal => 9,
            Self::GradientStopsTotal => 10,
            Self::Images => 11,
            Self::Depth => 12,
            Self::ChildrenPerNode => 13,
            Self::Islands => 14,
            Self::LayoutInputRecordsPerIsland => 15,
            Self::LayoutInputRecordsTotal => 16,
            Self::PathVerbsPerPath => 17,
            Self::PathSubpathsTotal => 18,
            Self::PolygonPointsPerShape => 19,
            Self::GradientStopsPerBrush => 20,
            Self::ImageEdge => 21,
            Self::ImagePixelsTotal => 22,
            Self::ClipDepth => 23,
            Self::PaintItemsPerNode => 24,
            Self::HitItemsPerNode => 25,
            Self::FlattenedSegmentsPerPath => 26,
            Self::FlattenedSegmentsTotal => 27,
            Self::DependencyVertices => 28,
            Self::DependencyEdges => 29,
        }
    }
}

/// Caller-supplied inclusive capacities for one spatial computation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialLimitsV2 {
    values: [usize; 30],
}

impl SpatialLimitsV2 {
    /// Creates capacities in `SpatialLimitKindV2::ALL` order.
    #[must_use]
    pub const fn new(values: [usize; 30]) -> Self {
        Self { values }
    }

    /// Returns the inclusive capacity for one limit kind.
    #[must_use]
    pub const fn limit(self, kind: SpatialLimitKindV2) -> usize {
        self.values[kind.index()]
    }
}

/// Registered bounded conformance profile for version-2 spatial evidence.
///
/// This experiment profile is neither a runtime default nor a product capacity.
pub const REGISTERED_SPATIAL_LIMITS_V2: SpatialLimitsV2 = SpatialLimitsV2::new([
    256, 1024, 256, 512, 1024, 512, 256, 256, 4096, 4096, 2048, 64, 32, 64, 64, 128, 192, 256,
    1024, 256, 32, 4096, 4_194_304, 32, 64, 64, 4096, 65_536, 192, 256,
]);
