#[cfg(all(
    feature = "cpu-reference",
    feature = "image-resource",
    feature = "native-renderer",
    feature = "numeric-spatial",
    feature = "path-hit"
))]
pub(crate) mod artifact;
#[cfg(feature = "cpu-reference")]
pub(crate) mod cpu_reference;
#[cfg(feature = "image-resource")]
pub(crate) mod image_resource;
#[cfg(feature = "native-renderer")]
pub(crate) mod native_renderer;
#[cfg(feature = "numeric-spatial")]
pub(crate) mod numeric_spatial;
#[cfg(feature = "path-hit")]
pub(crate) mod path_hit;
