use fenestra_ui_testkit::prototype::{FragmentPathV1, NodePathV1, PathSegmentV1};

pub(super) fn node_path(path: &NodePathV1) -> String {
    let mut output = String::from("root");
    for segment in path.segments() {
        match *segment {
            PathSegmentV1::Static { authored_slot } => {
                output.push_str(&format!("/s:{authored_slot}"));
            }
            PathSegmentV1::Member { region_slot, key } => {
                output.push_str(&format!("/m:{region_slot}:{key}"));
            }
        }
    }
    output
}

pub(super) fn fragment_path(path: &FragmentPathV1) -> String {
    format!("{}/r:{}", node_path(path.owner()), path.region_slot())
}
