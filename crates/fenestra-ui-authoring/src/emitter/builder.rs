use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

pub(super) fn ir_path(segments: &[&str]) -> TokenStream {
    let mut stream = TokenStream::new();
    push_double_colon(&mut stream);
    push_ident(&mut stream, "fenestra_ui_ir");
    push_double_colon(&mut stream);
    push_ident(&mut stream, "prototype");
    for segment in segments {
        push_double_colon(&mut stream);
        push_ident(&mut stream, segment);
    }
    stream
}

pub(super) fn ir_call(
    segments: &[&str],
    arguments: Vec<TokenStream>,
    trailing_comma: bool,
) -> TokenStream {
    call(ir_path(segments), arguments, trailing_comma)
}

pub(super) fn call(
    mut callee: TokenStream,
    arguments: Vec<TokenStream>,
    trailing_comma: bool,
) -> TokenStream {
    callee.extend([TokenTree::Group(Group::new(
        Delimiter::Parenthesis,
        separated(arguments, trailing_comma),
    ))]);
    callee
}

pub(super) fn method_call(
    mut receiver: TokenStream,
    method: &str,
    arguments: Vec<TokenStream>,
    trailing_comma: bool,
) -> TokenStream {
    receiver.extend([TokenTree::Punct(Punct::new('.', Spacing::Alone))]);
    push_ident(&mut receiver, method);
    receiver.extend([TokenTree::Group(Group::new(
        Delimiter::Parenthesis,
        separated(arguments, trailing_comma),
    ))]);
    receiver
}

pub(super) fn array(items: Vec<TokenStream>, trailing_comma: bool) -> TokenStream {
    TokenStream::from(TokenTree::Group(Group::new(
        Delimiter::Bracket,
        separated(items, trailing_comma),
    )))
}

pub(super) fn array_into(items: Vec<TokenStream>, trailing_comma: bool) -> TokenStream {
    method_call(array(items, trailing_comma), "into", Vec::new(), false)
}

pub(super) fn tuple(items: Vec<TokenStream>, trailing_comma: bool) -> TokenStream {
    TokenStream::from(TokenTree::Group(Group::new(
        Delimiter::Parenthesis,
        separated(items, trailing_comma),
    )))
}

pub(super) fn bool_literal(value: bool) -> TokenStream {
    ident(if value { "true" } else { "false" })
}

pub(super) fn i32_literal(value: i32) -> TokenStream {
    let mut stream = TokenStream::new();
    if value.is_negative() {
        stream.extend([TokenTree::Punct(Punct::new('-', Spacing::Alone))]);
    }
    stream.extend([TokenTree::Literal(Literal::u32_unsuffixed(
        value.unsigned_abs(),
    ))]);
    stream
}

pub(super) fn u32_literal(value: u32) -> TokenStream {
    TokenStream::from(TokenTree::Literal(Literal::u32_unsuffixed(value)))
}

pub(super) fn u64_literal(value: u64) -> TokenStream {
    TokenStream::from(TokenTree::Literal(Literal::u64_unsuffixed(value)))
}

pub(super) fn u8_literal(value: u8) -> TokenStream {
    TokenStream::from(TokenTree::Literal(Literal::u8_unsuffixed(value)))
}

fn ident(value: &str) -> TokenStream {
    TokenStream::from(TokenTree::Ident(Ident::new(value, Span::call_site())))
}

fn separated(items: Vec<TokenStream>, trailing_comma: bool) -> TokenStream {
    let item_count = items.len();
    let mut stream = TokenStream::new();
    for (index, item) in items.into_iter().enumerate() {
        stream.extend(item);
        if index + 1 < item_count || trailing_comma {
            stream.extend([TokenTree::Punct(Punct::new(',', Spacing::Alone))]);
        }
    }
    stream
}

fn push_double_colon(stream: &mut TokenStream) {
    stream.extend([
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
    ]);
}

fn push_ident(stream: &mut TokenStream, value: &str) {
    stream.extend([TokenTree::Ident(Ident::new(value, Span::call_site()))]);
}
