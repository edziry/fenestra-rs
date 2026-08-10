mod build;
mod layout;
mod spec;
mod types;
mod view;

pub use spec::{
    HeadlessProjectionCapacity, HeadlessProjectionErrorKind, HeadlessProjectionLimitKind,
    HeadlessProjectionSpec, HeadlessSurface,
};
pub use types::{HeadlessPoint, HeadlessRect, HeadlessSemanticAction, HeadlessSemanticRole};
pub use view::{
    ComputedStyleView, HeadlessGeometryView, HeadlessHitRegionView, HeadlessProjectionView,
    HeadlessSceneRectangleView, HeadlessSemanticView,
};

pub(crate) use types::{HeadlessProjectionState, HeadlessRuntimeConfig};
