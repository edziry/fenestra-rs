use std::borrow::Cow;

use super::{Diagnostic, origin};

pub(super) fn normalize(text: &str) -> Result<Cow<'_, str>, Diagnostic> {
    let bytes = text.as_bytes();
    let mut output = None;
    let mut offset = 0;
    while offset + 1 < bytes.len() {
        let Some(end) = comment_end(bytes, offset)? else {
            offset += 1;
            continue;
        };
        let start = offset;
        let output = output.get_or_insert_with(|| bytes.to_vec());
        // Retain original byte offsets and physical lines, including Unicode
        // comment contents and CRLF endings, for downstream token diagnostics.
        for byte in &mut output[start..end] {
            if !matches!(*byte, b'\n' | b'\r') {
                *byte = b' ';
            }
        }
        offset = end;
    }
    Ok(match output {
        Some(bytes) => Cow::Owned(String::from_utf8(bytes).expect("comments become ASCII spaces")),
        None => Cow::Borrowed(text),
    })
}

pub(super) fn skip_trivia(bytes: &[u8], mut offset: usize) -> Result<usize, Diagnostic> {
    loop {
        while bytes
            .get(offset)
            .is_some_and(|byte| matches!(*byte, b' ' | b'\t' | b'\r' | b'\n'))
        {
            offset += 1;
        }
        match comment_end(bytes, offset)? {
            Some(end) => offset = end,
            None => return Ok(offset),
        }
    }
}

fn comment_end(bytes: &[u8], start: usize) -> Result<Option<usize>, Diagnostic> {
    if bytes.get(start) != Some(&b'/') || !matches!(bytes.get(start + 1), Some(b'/' | b'*')) {
        return Ok(None);
    }
    if is_documentation_comment(bytes, start) {
        return Err(Diagnostic::new(
            "documentation comments are not view syntax; use // or /* */ comments",
            origin(start, start + 3),
        ));
    }
    if bytes[start + 1] == b'*' {
        return block_end(bytes, start).map(Some);
    }
    let mut offset = start + 2;
    while offset < bytes.len() && bytes[offset] != b'\n' {
        offset += 1;
    }
    Ok(Some(offset))
}

fn block_end(bytes: &[u8], start: usize) -> Result<usize, Diagnostic> {
    let mut offset = start + 2;
    let mut depth = 1;
    while offset + 1 < bytes.len() {
        match (bytes[offset], bytes[offset + 1]) {
            (b'/', b'*') => {
                depth += 1;
                offset += 2;
            }
            (b'*', b'/') => {
                depth -= 1;
                offset += 2;
                if depth == 0 {
                    return Ok(offset);
                }
            }
            _ => offset += 1,
        }
    }
    Err(Diagnostic::new(
        "unterminated block comment",
        origin(start, start + 2),
    ))
}

fn is_documentation_comment(bytes: &[u8], start: usize) -> bool {
    let third = bytes.get(start + 2).copied();
    let fourth = bytes.get(start + 3).copied();
    third == Some(b'!')
        || (bytes[start + 1] == b'/' && third == Some(b'/') && fourth != Some(b'/'))
        || (bytes[start + 1] == b'*' && third == Some(b'*') && !matches!(fourth, Some(b'*' | b'/')))
}
