use fenestra_text_screen_common::contract;
use fenestra_text_screen_parley::render;

#[test]
fn shared_corpus_has_owned_geometry_and_raster() {
    contract::shared_corpus_has_owned_geometry_and_raster(render);
}

#[test]
fn measurements_are_proportional_wrapped_and_repeatable() {
    contract::measurements_are_proportional_wrapped_and_repeatable(render);
}
