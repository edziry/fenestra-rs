use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::panic::{RefUnwindSafe, UnwindSafe};

use super::*;
use crate::model::{SpatialNodeKeyV2, SpatialPointV2};

fn assert_value<T>()
where
    T: Clone
        + Copy
        + Debug
        + Eq
        + PartialEq
        + Send
        + Sync
        + Unpin
        + UnwindSafe
        + RefUnwindSafe
        + 'static,
{
}

macro_rules! negative_trait {
    ($name:ident, $bound:path) => {
        trait $name<A> {
            fn marker() {}
        }
        impl<T: ?Sized> $name<()> for T {}
        impl<T: ?Sized + $bound> $name<u8> for T {}
    };
}

trait AmbiguousIfDefault<A> {
    fn marker() {}
}
impl<T: ?Sized> AmbiguousIfDefault<()> for T {}
impl<T: Default> AmbiguousIfDefault<u8> for T {}
negative_trait!(AmbiguousIfDisplay, Display);
negative_trait!(AmbiguousIfHash, Hash);
negative_trait!(AmbiguousIfOrd, Ord);
negative_trait!(AmbiguousIfPartialOrd, PartialOrd);

macro_rules! assert_not_default {
    ($type:ty) => {
        let _ = <$type as AmbiguousIfDefault<_>>::marker;
    };
}
macro_rules! assert_not_display {
    ($type:ty) => {
        let _ = <$type as AmbiguousIfDisplay<_>>::marker;
    };
}
macro_rules! assert_not_hash {
    ($type:ty) => {
        let _ = <$type as AmbiguousIfHash<_>>::marker;
    };
}
macro_rules! assert_not_ord {
    ($type:ty) => {
        let _ = <$type as AmbiguousIfOrd<_>>::marker;
    };
}
macro_rules! assert_not_partial_ord {
    ($type:ty) => {
        let _ = <$type as AmbiguousIfPartialOrd<_>>::marker;
    };
}

#[test]
fn hit_result_has_the_exact_value_surface_and_runtime_traits() {
    let _: fn(SpatialHitResultV2) -> u32 = SpatialHitResultV2::key;
    let _: fn(SpatialHitResultV2) -> SpatialNodeKeyV2 = SpatialHitResultV2::owner;
    let _: fn(SpatialHitResultV2) -> u32 = SpatialHitResultV2::item_ordinal;
    let _: fn(SpatialHitResultV2) -> SpatialPointV2 = SpatialHitResultV2::local_point;
    assert_value::<SpatialHitResultV2>();
    assert_not_default!(SpatialHitResultV2);
    assert_not_display!(SpatialHitResultV2);
    assert_not_hash!(SpatialHitResultV2);
    assert_not_ord!(SpatialHitResultV2);
    assert_not_partial_ord!(SpatialHitResultV2);
}

#[test]
fn dedicated_hit_query_implementation_is_allocation_free() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/input_validation/prepared/snapshot");
    let mut source =
        std::fs::read_to_string(root.join("hit.rs")).expect("dedicated hit query module");
    let directory = root.join("hit");
    if directory.is_dir() {
        let mut paths = std::fs::read_dir(directory)
            .expect("hit query submodules")
            .map(|entry| entry.expect("hit query entry").path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
            .collect::<Vec<_>>();
        paths.sort();
        for path in paths {
            source.push_str(&std::fs::read_to_string(path).expect("hit query source"));
        }
    }
    let implementation = implementation_body(&source, "SpatialResolvedSnapshotV2");
    assert!(implementation.contains("pub fn hit_test("));
    for forbidden in [
        "Vec::new(",
        "Vec::with_capacity(",
        "vec![",
        "Box::new(",
        "Box::pin(",
        "String::new(",
        "String::with_capacity(",
        "String::from(",
        "format!(",
        ".collect(",
        "collect::<",
        ".to_vec(",
        ".to_owned(",
        "Rc::new(",
        "Arc::new(",
        "HashMap",
        "BTreeMap",
        "VecDeque",
        "BinaryHeap",
        "Cell<",
        "RefCell",
        "Mutex",
        "RwLock",
        "OnceLock",
        "LazyLock",
        "validate_rect_k1",
        "validate_circle_k1",
        "validate_polygon_k1",
        "validate_path_k1",
        "validate_stroke_k1",
        "prepare_spatial_v2",
        "resolve_spatial_v2",
        "materialize_reference_spatial_v2",
        "validate_spatial_output_v2",
    ] {
        assert!(
            !source.contains(forbidden),
            "hit query allocates via {forbidden}"
        );
    }
}

fn implementation_body<'a>(source: &'a str, owner: &str) -> &'a str {
    let marker = format!("impl {owner}");
    let start = source.find(&marker).expect("hit implementation owner");
    let declaration = &source[start..];
    let open = declaration.find('{').expect("hit implementation body");
    let mut depth = 0_usize;
    for (offset, byte) in declaration
        .as_bytes()
        .iter()
        .copied()
        .enumerate()
        .skip(open)
    {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return &declaration[open + 1..offset];
                }
            }
            _ => {}
        }
    }
    panic!("unterminated hit implementation")
}
