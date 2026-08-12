pub(crate) struct InvalidRecord;

pub(crate) struct Record {
    anchor: u32,
    line: String,
}

impl Record {
    pub(crate) fn new(anchor: u32, kind: &str, fields: String) -> Result<Self, InvalidRecord> {
        let end = anchor.checked_add(1).ok_or(InvalidRecord)?;
        Ok(Self {
            anchor,
            line: format!("record|{anchor}|{kind}|span={anchor}:{end}|{fields}"),
        })
    }

    pub(crate) fn line(&self) -> &str {
        &self.line
    }
}

pub(crate) fn validate_name(name: &str) -> Result<(), InvalidRecord> {
    let mut bytes = name.bytes();
    let valid_start = bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_');
    if !valid_start
        || !bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        || name.contains('|')
    {
        return Err(InvalidRecord);
    }
    Ok(())
}

pub(crate) fn sort_and_validate(
    records: &mut [Record],
    expected_count: usize,
) -> Result<(), InvalidRecord> {
    if records.len() != expected_count {
        return Err(InvalidRecord);
    }
    records.sort_by_key(|record| record.anchor);
    for (ordinal, record) in records.iter().enumerate() {
        let ordinal = u32::try_from(ordinal).map_err(|_| InvalidRecord)?;
        if record.anchor != ordinal {
            return Err(InvalidRecord);
        }
    }
    Ok(())
}
