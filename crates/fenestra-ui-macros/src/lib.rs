#![forbid(unsafe_code)]

//! Unpublished procedural-macro boundary for typed Fenestra authoring.

use proc_macro::TokenStream;

use fenestra_ui_authoring::prototype::{REFERENCE_AUTHORING_LIMITS_V1, expand_ui_v1};

/// Compiles one bounded typed UI document into the shared raw IR triple.
#[proc_macro]
pub fn ui(input: TokenStream) -> TokenStream {
    expand_ui_v1(input.into(), REFERENCE_AUTHORING_LIMITS_V1).into()
}
