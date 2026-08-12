use super::types::{Child, Fragment, ManifestEntry, Mutation, Node, Receipt, State, Value};

const ROOT: &str = "root";
const STACK: &str = "root/s:0";
const FLOATING: &str = "root/s:0/s:0";
const FLOATING_CHILD: &str = "root/s:0/s:0/s:0";
const FRAGMENT: &str = "root/s:0/s:0/r:1";
const GUIDE: &str = "root/s:1";
const VIEWPORT_LAYER: &str = "root/s:1/s:0";

#[derive(Clone)]
pub struct Model {
    pub viewport: [i32; 2],
    pub span_x: i32,
    pub tone: [u8; 4],
    pub policy: bool,
    pub keys: Vec<(u64, i32)>,
}

impl Model {
    pub fn initial() -> Self {
        Self {
            viewport: [192, 128],
            span_x: 180,
            tone: [96, 72, 48, 255],
            policy: false,
            keys: vec![(10, 12), (20, 12)],
        }
    }

    pub fn apply(&mut self, step: usize) -> Receipt {
        let generation = u64::try_from(step).expect("registered generation should fit");
        let (invalidation, mutation) = match step {
            1 => {
                let old = self.viewport;
                self.viewport = [224, 160];
                (
                    layout_all(),
                    Mutation::Viewport {
                        old,
                        new: self.viewport,
                    },
                )
            }
            2 => {
                let old = self.span_x;
                self.span_x = 176;
                (
                    layout_all(),
                    Mutation::Property {
                        node: ROOT.to_owned(),
                        property: 0,
                        old: Value::I32(old),
                        new: Value::I32(self.span_x),
                    },
                )
            }
            3 => {
                let old = self.tone;
                self.tone = [80, 40, 24, 255];
                (
                    vec![6],
                    Mutation::Property {
                        node: ROOT.to_owned(),
                        property: 4,
                        old: Value::Rgba(old),
                        new: Value::Rgba(self.tone),
                    },
                )
            }
            4 => {
                let old = self.policy;
                self.policy = true;
                (
                    vec![5],
                    Mutation::Property {
                        node: ROOT.to_owned(),
                        property: 7,
                        old: Value::Policy(old),
                        new: Value::Policy(self.policy),
                    },
                )
            }
            5 => {
                self.keys.insert(1, (30, 12));
                let root = member_path(30);
                (
                    structure_all(),
                    Mutation::Insert {
                        fragment: FRAGMENT.to_owned(),
                        key: 30,
                        root: root.clone(),
                        final_index: 1,
                        created: vec![ManifestEntry::Node(root)],
                    },
                )
            }
            6 => {
                let member = self.keys.remove(1);
                self.keys.insert(2, member);
                (
                    structure_all(),
                    Mutation::Move {
                        fragment: FRAGMENT.to_owned(),
                        key: 30,
                        root: member_path(30),
                        old_index: 1,
                        final_index: 2,
                    },
                )
            }
            7 => {
                let (_, height) = self
                    .keys
                    .iter_mut()
                    .find(|(key, _)| *key == 30)
                    .expect("key 30 should be live");
                let old = *height;
                *height = 14;
                (
                    layout_all(),
                    Mutation::Property {
                        node: member_path(30),
                        property: 1,
                        old: Value::I32(old),
                        new: Value::I32(*height),
                    },
                )
            }
            8 => {
                let old_index = self
                    .keys
                    .iter()
                    .position(|(key, _)| *key == 20)
                    .expect("key 20 should be live");
                self.keys.remove(old_index);
                let root = member_path(20);
                (
                    structure_all(),
                    Mutation::Remove {
                        fragment: FRAGMENT.to_owned(),
                        key: 20,
                        root: root.clone(),
                        old_index,
                        retired: vec![ManifestEntry::Node(root)],
                    },
                )
            }
            _ => panic!("only successful registered steps are applied"),
        };
        Receipt {
            generation,
            invalidation,
            mutations: vec![mutation],
        }
    }

    pub fn state(&self) -> State {
        let mut nodes = vec![
            node(
                ROOT,
                None,
                0,
                values(self.span_x, 120, 2, 1, self.tone, self.policy),
                vec![
                    Child::Static(STACK.to_owned()),
                    Child::Static(GUIDE.to_owned()),
                ],
            ),
            node(
                STACK,
                Some(ROOT),
                1,
                values(80, 60, 2, 1, [24, 48, 72, 255], false),
                vec![Child::Static(FLOATING.to_owned())],
            ),
            node(
                FLOATING,
                Some(STACK),
                2,
                values(40, 30, 2, 1, [64, 48, 32, 255], false),
                vec![
                    Child::Static(FLOATING_CHILD.to_owned()),
                    Child::Region(FRAGMENT.to_owned()),
                ],
            ),
            node(
                FLOATING_CHILD,
                Some(FLOATING),
                3,
                values(12, 10, 2, 1, [64, 48, 32, 255], false),
                Vec::new(),
            ),
        ];
        nodes.extend(self.keys.iter().map(|(key, height)| {
            node(
                &member_path(*key),
                Some(FLOATING),
                4,
                values(16, *height, 2, 1, [80, 120, 160, 192], true),
                Vec::new(),
            )
        }));
        nodes.extend([
            node(
                GUIDE,
                Some(ROOT),
                5,
                values(50, 40, 2, 1, [64, 48, 32, 255], false),
                vec![Child::Static(VIEWPORT_LAYER.to_owned())],
            ),
            node(
                VIEWPORT_LAYER,
                Some(GUIDE),
                6,
                values(20, 16, 2, 1, [64, 48, 32, 255], true),
                Vec::new(),
            ),
        ]);
        State {
            nodes,
            fragments: vec![Fragment {
                path: FRAGMENT.to_owned(),
                descriptor: 0,
                members: self
                    .keys
                    .iter()
                    .map(|(key, _)| (*key, member_path(*key)))
                    .collect(),
            }],
        }
    }
}

pub fn initial_receipt() -> Receipt {
    Receipt {
        generation: 0,
        invalidation: Vec::new(),
        mutations: Vec::new(),
    }
}

pub fn member_path(key: u64) -> String {
    format!("{FLOATING}/m:1:{key}")
}

fn node(
    path: &str,
    parent: Option<&str>,
    template: u32,
    properties: Vec<(u32, Value)>,
    children: Vec<Child>,
) -> Node {
    Node {
        path: path.to_owned(),
        parent: parent.map(str::to_owned),
        template,
        component: 0,
        properties,
        children,
    }
}

fn values(
    width: i32,
    height: i32,
    pad: i32,
    factor: i32,
    tone: [u8; 4],
    policy: bool,
) -> Vec<(u32, Value)> {
    vec![
        (0, Value::I32(width)),
        (1, Value::I32(height)),
        (2, Value::I32(pad)),
        (3, Value::I32(factor)),
        (4, Value::Rgba(tone)),
        (5, Value::Rgba([16, 64, 96, 192])),
        (6, Value::Bool(true)),
        (7, Value::Policy(policy)),
    ]
}

fn layout_all() -> Vec<u8> {
    vec![3, 4, 5, 6, 7]
}

fn structure_all() -> Vec<u8> {
    vec![0, 3, 4, 5, 6, 7]
}
