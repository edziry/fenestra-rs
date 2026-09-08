use std::{borrow::Cow, collections::HashMap, sync::Arc};

use parley::{
    FontContext, FontFamilyName,
    fontique::{Blob, Collection, CollectionOptions, FontInfoOverride},
};
use swash::{CacheKey, FontRef};

use crate::FontError;

const MAX_FONTS: usize = 32;
const MAX_FONT_BYTES: usize = 8 * 1024 * 1024;
const MAX_TOTAL_BYTES: usize = 32 * 1024 * 1024;

pub(crate) struct Fonts {
    pub context: FontContext,
    pub families: Vec<FontFamilyName<'static>>,
    pub raster_keys: HashMap<u64, (u32, CacheKey)>,
}

impl Fonts {
    pub fn new<B: AsRef<[u8]>>(input: impl IntoIterator<Item = B>) -> Result<Self, FontError> {
        let mut sources = Vec::new();
        for source in input {
            check("font count", sources.len() + 1, MAX_FONTS)?;
            sources.push(source);
        }
        if sources.is_empty() {
            return Err(FontError::EmptySet);
        }
        let data: Vec<_> = sources.iter().map(AsRef::as_ref).collect();
        let mut total = 0;
        for bytes in &data {
            check("font bytes", bytes.len(), MAX_FONT_BYTES)?;
            total += bytes.len();
            check("total font bytes", total, MAX_TOTAL_BYTES)?;
        }
        let mut fonts = Self {
            context: FontContext {
                collection: Collection::new(CollectionOptions {
                    system_fonts: false,
                    shared: false,
                }),
                source_cache: Default::default(),
            },
            families: Vec::new(),
            raster_keys: HashMap::new(),
        };
        for (index, bytes) in data.into_iter().enumerate() {
            validate(bytes, index)?;
            let raster = FontRef::from_index(bytes, 0).ok_or(FontError::InvalidFont { index })?;
            let key = (raster.offset, raster.key);
            let data = Blob::new(Arc::new(bytes.to_vec()));
            let data_id = data.id();
            // Isolate equal upstream family names so caller order is preserved.
            let family = format!("fenestra-font-{index}");
            let registered = fonts.context.collection.register_fonts(
                data,
                Some(FontInfoOverride {
                    family_name: Some(&family),
                    ..Default::default()
                }),
            );
            if registered.len() != 1
                || registered[0].1.len() != 1
                || registered[0].1[0].index() != 0
            {
                return Err(FontError::InvalidFont { index });
            }
            fonts
                .families
                .push(FontFamilyName::Named(Cow::Owned(family)));
            fonts.raster_keys.insert(data_id, key);
        }
        Ok(fonts)
    }
}

fn check(resource: &'static str, actual: usize, limit: usize) -> Result<(), FontError> {
    if actual > limit {
        Err(FontError::LimitExceeded {
            resource,
            actual,
            limit,
        })
    } else {
        Ok(())
    }
}

fn validate(bytes: &[u8], index: usize) -> Result<(), FontError> {
    if bytes.starts_with(b"ttcf") {
        return Err(FontError::CollectionUnsupported { index });
    }
    let invalid = FontError::InvalidFont { index };
    if bytes.len() < 12 || !matches!(&bytes[..4], b"\0\x01\0\0" | b"OTTO") {
        return Err(invalid);
    }
    let count = usize::from(u16::from_be_bytes([bytes[4], bytes[5]]));
    let directory_end = 12 + count * 16;
    if count == 0 || directory_end > bytes.len() {
        return Err(invalid);
    }
    let mut tags = Vec::with_capacity(count);
    for record in bytes[12..directory_end].chunks_exact(16) {
        let offset = u32::from_be_bytes(record[8..12].try_into().unwrap()) as usize;
        let length = u32::from_be_bytes(record[12..16].try_into().unwrap()) as usize;
        if offset < directory_end
            || offset
                .checked_add(length)
                .is_none_or(|end| end > bytes.len())
        {
            return Err(invalid);
        }
        tags.push(&record[..4]);
    }
    let has = |tag: &[u8; 4]| tags.contains(&tag.as_slice());
    if [
        b"COLR", b"CPAL", b"CBDT", b"CBLC", b"sbix", b"SVG ", b"EBDT", b"EBLC", b"bdat", b"bloc",
    ]
    .iter()
    .any(|tag| has(tag))
    {
        return Err(FontError::ColorOrBitmapUnsupported { index });
    }
    if ![b"head", b"maxp", b"hhea", b"hmtx", b"cmap", b"name"]
        .iter()
        .all(|tag| has(tag))
    {
        return Err(invalid);
    }
    let head = bytes[12..directory_end]
        .chunks_exact(16)
        .find(|record| &record[..4] == b"head")
        .ok_or(FontError::InvalidFont { index })?;
    let offset = u32::from_be_bytes(head[8..12].try_into().unwrap()) as usize;
    let length = u32::from_be_bytes(head[12..16].try_into().unwrap()) as usize;
    if length < 54
        || !(16..=16_384).contains(&u16::from_be_bytes([
            bytes[offset + 18],
            bytes[offset + 19],
        ]))
    {
        return Err(invalid);
    }
    if !(has(b"glyf") && has(b"loca") || has(b"CFF ") || has(b"CFF2")) {
        return Err(FontError::OutlinesUnavailable { index });
    }
    Ok(())
}
