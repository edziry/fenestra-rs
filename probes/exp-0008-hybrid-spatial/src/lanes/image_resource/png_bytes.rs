pub(crate) struct PngMetadataV2<'a> {
    pub(crate) gamma_scaled: Option<u32>,
    pub(crate) icc_profile: Option<&'a [u8]>,
    pub(crate) exif: Option<&'a [u8]>,
}

pub(crate) fn rgba8_png(
    width: u32,
    height: u32,
    rgba8: &[u8],
    metadata: PngMetadataV2<'_>,
) -> Vec<u8> {
    assert_eq!(rgba8.len(), width as usize * height as usize * 4);
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut header = Vec::with_capacity(13);
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[8, 6, 0, 0, 0]);
    push_chunk(&mut png, *b"IHDR", &header);
    if let Some(gamma) = metadata.gamma_scaled {
        push_chunk(&mut png, *b"gAMA", &gamma.to_be_bytes());
    }
    if let Some(profile) = metadata.icc_profile {
        let mut chunk = b"Fenestra\0\0".to_vec();
        chunk.extend_from_slice(&stored_zlib(profile));
        push_chunk(&mut png, *b"iCCP", &chunk);
    }
    if let Some(exif) = metadata.exif {
        push_chunk(&mut png, *b"eXIf", exif);
    }
    let mut scanlines = Vec::with_capacity(rgba8.len() + height as usize);
    let stride = width as usize * 4;
    for row in rgba8.chunks_exact(stride) {
        scanlines.push(0);
        scanlines.extend_from_slice(row);
    }
    push_chunk(&mut png, *b"IDAT", &stored_zlib(&scanlines));
    push_chunk(&mut png, *b"IEND", &[]);
    png
}

fn stored_zlib(bytes: &[u8]) -> Vec<u8> {
    assert!(bytes.len() <= u16::MAX as usize);
    let length = bytes.len() as u16;
    let mut output = vec![0x78, 0x01, 0x01];
    output.extend_from_slice(&length.to_le_bytes());
    output.extend_from_slice(&(!length).to_le_bytes());
    output.extend_from_slice(bytes);
    output.extend_from_slice(&adler32(bytes).to_be_bytes());
    output
}

fn push_chunk(output: &mut Vec<u8>, kind: [u8; 4], data: &[u8]) {
    output.extend_from_slice(&(data.len() as u32).to_be_bytes());
    output.extend_from_slice(&kind);
    output.extend_from_slice(data);
    let mut crc_input = kind.to_vec();
    crc_input.extend_from_slice(data);
    output.extend_from_slice(&crc32(&crc_input).to_be_bytes());
}

fn adler32(bytes: &[u8]) -> u32 {
    let mut first = 1_u32;
    let mut second = 0_u32;
    for byte in bytes {
        first = (first + u32::from(*byte)) % 65_521;
        second = (second + first) % 65_521;
    }
    (second << 16) | first
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}
