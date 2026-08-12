#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Log {
    pub observations: Vec<Observation>,
    pub final_keys: Vec<u64>,
    pub noop: Noop,
    pub failure: Failure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Noop {
    pub empty_preserved: bool,
    pub same_value_preserved: bool,
    pub round_trip_preserved: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailureKind {
    SingularTransform,
    ScalarOutOfDomain,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailureLocation {
    Input,
    Node { index: u32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Span {
    Synthetic,
    Bytes { source: u32, start: u32, end: u32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Failure {
    pub kind: FailureKind,
    pub location: FailureLocation,
    pub ir_span: Span,
    pub operation_index: Option<usize>,
    pub outer_state_preserved: bool,
    pub spatial_snapshot_preserved: bool,
    pub complete_observation_preserved: bool,
    pub authored_factor_span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Observation {
    pub generation: u64,
    pub viewport: [i32; 2],
    pub receipt: Receipt,
    pub state: State,
    pub projection: Projection,
    pub hit_queries: Vec<HitQuery>,
    pub raster: Raster,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct State {
    pub nodes: Vec<Node>,
    pub fragments: Vec<Fragment>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Node {
    pub path: String,
    pub parent: Option<String>,
    pub template: u32,
    pub component: u32,
    pub properties: Vec<(u32, Value)>,
    pub children: Vec<Child>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
    Bool(bool),
    I32(i32),
    Rgba([u8; 4]),
    Policy(bool),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Child {
    Static(String),
    Region(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fragment {
    pub path: String,
    pub descriptor: u32,
    pub members: Vec<(u64, String)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Receipt {
    pub generation: u64,
    pub invalidation: Vec<u8>,
    pub mutations: Vec<Mutation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Mutation {
    Property {
        node: String,
        property: u32,
        old: Value,
        new: Value,
    },
    Insert {
        fragment: String,
        key: u64,
        root: String,
        final_index: usize,
        created: Vec<ManifestEntry>,
    },
    Move {
        fragment: String,
        key: u64,
        root: String,
        old_index: usize,
        final_index: usize,
    },
    Remove {
        fragment: String,
        key: u64,
        root: String,
        old_index: usize,
        retired: Vec<ManifestEntry>,
    },
    Viewport {
        old: [i32; 2],
        new: [i32; 2],
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ManifestEntry {
    Node(String),
    Fragment(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Projection {
    pub mapping: Vec<(u32, Option<String>)>,
    pub geometry: Vec<Geometry>,
    pub clips: Vec<Clip>,
    pub paints: Vec<Paint>,
    pub hits: Vec<Item>,
    pub semantics: Vec<Item>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Aabb {
    pub empty: bool,
    pub edges: [i64; 4],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Geometry {
    pub key: u32,
    pub path: Option<String>,
    pub base: [i64; 4],
    pub affine: [i64; 6],
    pub determinant: i128,
    pub aabb: Aabb,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Clip {
    pub key: u32,
    pub owner: u32,
    pub path: String,
    pub parent: Option<u32>,
    pub shape: u32,
    pub affine: [i64; 6],
    pub determinant: i128,
    pub primitive: Aabb,
    pub effective: Aabb,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaintReference {
    Coverage { shape: u32, brush: u32 },
    Image { image: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Paint {
    pub key: u32,
    pub owner: u32,
    pub path: String,
    pub affine: [i64; 6],
    pub determinant: i128,
    pub aabb: Aabb,
    pub reference: PaintReference,
    pub clip: Option<u32>,
    pub stack: u32,
    pub item: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Item {
    pub key: u32,
    pub owner: u32,
    pub path: String,
    pub affine: [i64; 6],
    pub determinant: i128,
    pub aabb: Aabb,
    pub shape: u32,
    pub clip: Option<u32>,
    pub stack: u32,
    pub item: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HitQuery {
    pub scene: [i64; 2],
    pub result: Option<Hit>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Hit {
    pub key: u32,
    pub owner: u32,
    pub path: String,
    pub item: u32,
    pub local: [i64; 2],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Raster {
    pub width: u32,
    pub height: u32,
    pub stride: u64,
    pub bytes: Box<[u8]>,
}
