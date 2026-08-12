use super::source::all_source;
use super::surface_support::{
    assert_const_and_must_use, assert_struct_fields_private, implementation_blocks, names,
    public_constants, public_methods,
};

#[test]
fn owned_input_has_one_exact_private_nonconst_surface() {
    let source = all_source();
    assert_struct_fields_private(&source, "SpatialOwnedInputV2");
    assert_eq!(
        public_methods(&source, "SpatialOwnedInputV2"),
        names(&["new", "as_input"])
    );
    assert!(public_constants(&source, "SpatialOwnedInputV2").is_empty());
    assert_const_and_must_use(&source, "SpatialOwnedInputV2", &[], &["new", "as_input"]);

    for implementation in implementation_blocks(&source, "SpatialOwnedInputV2") {
        assert!(!implementation.contains("pub const fn new"));
        assert!(!implementation.contains("pub const fn as_input"));
    }
}
