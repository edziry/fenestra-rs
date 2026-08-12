#![forbid(unsafe_code)]

//! Unpublished procedural-macro boundary for typed Fenestra authoring.

use proc_macro::TokenStream;

use fenestra_ui_authoring::prototype::{
    REFERENCE_AUTHORING_LIMITS_V1, REFERENCE_AUTHORING_LIMITS_V2, expand_ui,
};

/// Compiles one bounded typed UI document into its versioned raw IR programs.
#[proc_macro]
pub fn ui(input: TokenStream) -> TokenStream {
    expand_ui(
        input.into(),
        REFERENCE_AUTHORING_LIMITS_V1,
        REFERENCE_AUTHORING_LIMITS_V2,
    )
    .into()
}
