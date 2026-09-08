pub(super) fn starts(bytes: &[u8], start: usize) -> bool {
    if bytes.get(start) == Some(&b'"') {
        return true;
    }
    if bytes.get(start) != Some(&b'r') {
        return false;
    }
    let mut offset = start + 1;
    while bytes.get(offset) == Some(&b'#') {
        offset += 1;
    }
    bytes.get(offset) == Some(&b'"')
}

pub(super) fn end(text: &str, start: usize) -> Result<usize, &'static str> {
    let bytes = text.as_bytes();
    if bytes[start] == b'r' {
        let mut quote = start + 1;
        while bytes.get(quote) == Some(&b'#') {
            quote += 1;
        }
        let hashes = quote - start - 1;
        if hashes > 255 {
            return Err("raw string literals support at most 255 hash delimiters");
        }
        let mut offset = quote + 1;
        while offset < bytes.len() {
            if bytes[offset] == b'"'
                && bytes
                    .get(offset + 1..offset + 1 + hashes)
                    .is_some_and(|tail| tail.iter().all(|byte| *byte == b'#'))
            {
                return Ok(offset + 1 + hashes);
            }
            offset += 1;
        }
    } else {
        let mut offset = start + 1;
        while offset < bytes.len() {
            match bytes[offset] {
                b'"' => return Ok(offset + 1),
                b'\\' => offset += 2,
                _ => offset += 1,
            }
        }
    }
    Err("unterminated string literal")
}

pub(super) fn decode(literal: &str) -> Result<Box<str>, &'static str> {
    if !starts(literal.as_bytes(), 0) || end(literal, 0)? != literal.len() {
        return Err("expected an unsuffixed Rust string literal");
    }
    let raw = literal.starts_with('r');
    let opening = literal.find('"').expect("string opening was validated");
    let trailing = if raw { opening } else { 1 };
    let body = &literal[opening + 1..literal.len() - trailing];
    let mut input = body.chars().peekable();
    let mut output = String::with_capacity(body.len());
    while let Some(character) = input.next() {
        match character {
            '\r' => {
                if input.next() != Some('\n') {
                    return Err("string literals cannot contain a bare carriage return");
                }
                output.push('\n');
            }
            '\\' if !raw => escape(&mut input, &mut output)?,
            _ => output.push(character),
        }
    }
    Ok(output.into_boxed_str())
}

fn escape(
    input: &mut std::iter::Peekable<std::str::Chars<'_>>,
    output: &mut String,
) -> Result<(), &'static str> {
    let escaped = input.next().ok_or("incomplete string escape")?;
    let character = match escaped {
        'n' => '\n',
        'r' => '\r',
        't' => '\t',
        '0' => '\0',
        '\\' => '\\',
        '\'' => '\'',
        '"' => '"',
        'x' => {
            let high = input.next().and_then(|value| value.to_digit(16));
            let low = input.next().and_then(|value| value.to_digit(16));
            let (Some(high), Some(low)) = (high, low) else {
                return Err("string byte escapes require two hexadecimal digits");
            };
            let value = high * 16 + low;
            if value > 127 {
                return Err("string byte escapes must be ASCII from 00 through 7f");
            }
            char::from_u32(value).expect("ASCII is a Unicode scalar")
        }
        'u' => unicode_escape(input)?,
        '\n' | '\r' => {
            if escaped == '\r' && input.next() != Some('\n') {
                return Err("string line continuations require LF or CRLF");
            }
            while input
                .next_if(|value| matches!(value, ' ' | '\t' | '\r' | '\n'))
                .is_some()
            {}
            return Ok(());
        }
        _ => return Err("unsupported string escape"),
    };
    output.push(character);
    Ok(())
}

fn unicode_escape(
    input: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> Result<char, &'static str> {
    if input.next() != Some('{') {
        return Err("string Unicode escapes require braces");
    }
    let mut digits = 0;
    let mut value = 0;
    loop {
        match input.next() {
            Some('}') if digits > 0 => break,
            Some('_') if digits > 0 => {}
            Some(character) if character.is_ascii_hexdigit() && digits < 6 => {
                value = value * 16 + character.to_digit(16).expect("hexadecimal digit");
                digits += 1;
            }
            _ => return Err("string Unicode escapes require one through six hexadecimal digits"),
        }
    }
    char::from_u32(value).ok_or("string Unicode escape is not a Unicode scalar value")
}
