use std::collections::BTreeMap;

pub(crate) const HEADER: &str = "spatial-v2|artifact=2|contract=2|corpus=2|kind=baseline";
pub(crate) const PACKAGES: &str =
    "packages|probe=0.2.0|ir=0.2.0|layout=0.2.0|spatial=0.2.0|runtime=0.2.0";
pub(crate) const PROFILE: &str =
    "profile|spatial=registered-v2|raster=registered-v2|candidate-count=0";
pub(crate) const LIMITS: &str = concat!(
    "limits|spatial=256,1024,256,512,1024,512,256,256,4096,4096,2048,64,32,",
    "64,64,128,192,256,1024,256,32,4096,4194304,32,64,64,4096,65536,192,256|",
    "raster-pixels=4194304|records=4096|line-bytes=1024|artifact-bytes=1048576"
);

#[derive(Debug)]
pub(crate) struct ParsedArtifact<'a> {
    pub(crate) lines: Vec<&'a str>,
    pub(crate) sections: BTreeMap<(u8, u8, &'a str), ParsedSection>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ParsedSection {
    pub(crate) records: u64,
    pub(crate) bytes: u64,
    pub(crate) digest: u64,
}

pub(crate) fn parse(bytes: &[u8]) -> ParsedArtifact<'_> {
    assert!(bytes.is_ascii(), "artifact must be printable ASCII");
    assert!(bytes.ends_with(b"\n"), "artifact must have a final LF");
    assert!(!bytes.ends_with(b"\n\n"), "artifact must have one final LF");
    assert!(!bytes.contains(&b'\r'), "artifact must use LF only");

    let text = std::str::from_utf8(bytes).expect("ASCII is valid UTF-8");
    let lines = text
        .strip_suffix('\n')
        .expect("final LF checked")
        .split('\n')
        .collect::<Vec<_>>();
    let mut sections = BTreeMap::new();
    for line in &lines {
        assert!(line.len() <= 1024, "canonical line exceeds its byte limit");
        assert!(line.bytes().all(|byte| (0x20..=0x7e).contains(&byte)));
        if line.starts_with("section|") {
            let fields = fields(line);
            let case = decimal(fields["case"]) as u8;
            let step = decimal(fields["step"]) as u8;
            let name = fields["name"];
            let value = ParsedSection {
                records: decimal(fields["records"]),
                bytes: decimal(fields["bytes"]),
                digest: hex16(fields["digest"]),
            };
            assert!(sections.insert((case, step, name), value).is_none());
        }
    }
    ParsedArtifact { lines, sections }
}

pub(crate) fn fields(line: &str) -> BTreeMap<&str, &str> {
    let mut values = BTreeMap::new();
    for field in line.split('|').skip(1) {
        let (name, value) = field.split_once('=').expect("named artifact field");
        assert!(
            values.insert(name, value).is_none(),
            "duplicate field {name}"
        );
    }
    values
}

pub(crate) fn decimal(value: &str) -> u64 {
    assert!(!value.is_empty());
    assert!(value.bytes().all(|byte| byte.is_ascii_digit()));
    assert!(value == "0" || !value.starts_with('0'));
    value.parse().expect("bounded unsigned decimal")
}

pub(crate) fn hex16(value: &str) -> u64 {
    assert_eq!(value.len(), 16);
    assert!(
        value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    );
    u64::from_str_radix(value, 16).expect("canonical hex16")
}

pub(crate) fn token(value: &str) -> bool {
    let mut bytes = value.bytes();
    matches!(bytes.next(), Some(byte) if byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && bytes.all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'.' | b'_' | b':' | b'-')
        })
}

pub(crate) fn fnv1a64(section: &str, encoded: &[u8]) -> u64 {
    let mut digest = 14_695_981_039_346_656_037_u64;
    for byte in b"spatial-evidence-v2"
        .iter()
        .copied()
        .chain([0])
        .chain(section.bytes())
        .chain([0])
        .chain(encoded.iter().copied())
    {
        digest ^= u64::from(byte);
        digest = digest.wrapping_mul(1_099_511_628_211);
    }
    digest
}
