#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SpatialEvidenceV2 {
    pub(crate) cases: Vec<SpatialEvidenceCaseV2>,
    pub(crate) width_witness: WidthWitnessV2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WidthWitnessV2 {
    pub(crate) scalar: i64,
    pub(crate) determinant: i128,
    pub(crate) stride: u64,
    pub(crate) dimension: u32,
    pub(crate) key: u32,
    pub(crate) color: u8,
}

impl WidthWitnessV2 {
    pub(crate) const REGISTERED: Self = Self {
        scalar: 65_536,
        determinant: 4_294_967_296,
        stride: 768,
        dimension: 192,
        key: 1,
        color: 255,
    };
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SpatialEvidenceCaseV2 {
    pub(crate) ordinal: u8,
    pub(crate) name: &'static str,
    pub(crate) observations: Vec<SpatialEvidenceObservationV2>,
    pub(crate) result: CaseResultV2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CaseResultV2 {
    pub(crate) literal_match: bool,
    pub(crate) reference_match: bool,
    pub(crate) repeat_match: bool,
}

impl CaseResultV2 {
    pub(crate) const MATCH: Self = Self {
        literal_match: true,
        reference_match: true,
        repeat_match: true,
    };
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SpatialEvidenceObservationV2 {
    pub(crate) case: u8,
    pub(crate) step: u8,
    pub(crate) generation: Option<u64>,
    pub(crate) viewport: (u32, u32),
    pub(crate) sections: Vec<NormalizedSectionV2>,
}

impl SpatialEvidenceObservationV2 {
    pub(crate) fn section(&self, name: EvidenceSectionV2) -> &NormalizedSectionV2 {
        self.sections
            .iter()
            .find(|section| section.name == name)
            .expect("registered evidence section")
    }

    pub(crate) fn section_mut(&mut self, name: EvidenceSectionV2) -> &mut NormalizedSectionV2 {
        self.sections
            .iter_mut()
            .find(|section| section.name == name)
            .expect("registered evidence section")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedSectionV2 {
    pub(crate) name: EvidenceSectionV2,
    pub(crate) records: Vec<EvidenceRecordV2>,
    pub(crate) record_count: u64,
    pub(crate) encoded: Vec<u8>,
    pub(crate) byte_count: u64,
    pub(crate) digest: u64,
}

impl NormalizedSectionV2 {
    pub(crate) fn new(name: EvidenceSectionV2, records: Vec<EvidenceRecordV2>) -> Self {
        let mut section = Self {
            name,
            records,
            record_count: 0,
            encoded: Vec::new(),
            byte_count: 0,
            digest: 0,
        };
        section.recompute();
        section
    }

    pub(crate) fn recompute(&mut self) {
        self.record_count = u64::try_from(self.records.len()).expect("record count should fit");
        self.encoded.clear();
        put_u64(&mut self.encoded, self.record_count);
        for record in &self.records {
            for field in &record.fields {
                self.encoded.extend_from_slice(&field.encoded);
            }
        }
        self.byte_count = u64::try_from(self.encoded.len()).expect("section bytes should fit");
        self.digest = fnv_evidence_v2(self.name.token(), &self.encoded);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EvidenceRecordV2 {
    pub(crate) fields: Vec<EvidenceFieldV2>,
}

impl EvidenceRecordV2 {
    pub(crate) fn new(fields: Vec<EvidenceFieldV2>) -> Self {
        Self { fields }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EvidenceFieldV2 {
    pub(crate) name: &'static str,
    pub(crate) encoded: Vec<u8>,
}

impl EvidenceFieldV2 {
    pub(crate) fn raw(name: &'static str, encoded: Vec<u8>) -> Self {
        Self { name, encoded }
    }

    pub(crate) fn tag(name: &'static str, value: u8) -> Self {
        Self::raw(name, vec![value])
    }

    pub(crate) fn bool(name: &'static str, value: bool) -> Self {
        Self::tag(name, u8::from(value))
    }

    pub(crate) fn u32(name: &'static str, value: u32) -> Self {
        Self::raw(name, value.to_le_bytes().to_vec())
    }

    pub(crate) fn i32(name: &'static str, value: i32) -> Self {
        Self::raw(name, value.to_le_bytes().to_vec())
    }

    pub(crate) fn u64(name: &'static str, value: u64) -> Self {
        Self::raw(name, value.to_le_bytes().to_vec())
    }

    pub(crate) fn i64(name: &'static str, value: i64) -> Self {
        Self::raw(name, value.to_le_bytes().to_vec())
    }

    pub(crate) fn i128(name: &'static str, value: i128) -> Self {
        Self::raw(name, value.to_le_bytes().to_vec())
    }

    pub(crate) fn optional_u64(name: &'static str, value: Option<u64>) -> Self {
        let mut encoded = vec![u8::from(value.is_some())];
        if let Some(value) = value {
            put_u64(&mut encoded, value);
        }
        Self::raw(name, encoded)
    }

    pub(crate) fn optional_u32(name: &'static str, value: Option<u32>) -> Self {
        let mut encoded = vec![u8::from(value.is_some())];
        if let Some(value) = value {
            put_u32(&mut encoded, value);
        }
        Self::raw(name, encoded)
    }

    pub(crate) fn bytes(name: &'static str, value: &[u8]) -> Self {
        let mut encoded = Vec::new();
        put_u64(
            &mut encoded,
            u64::try_from(value.len()).expect("byte slice should fit"),
        );
        encoded.extend_from_slice(value);
        Self::raw(name, encoded)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvidenceSectionV2 {
    Receipt,
    Mapping,
    Source,
    Geometry,
    Clips,
    Paints,
    Hits,
    Semantics,
    Queries,
    Raster,
}

impl EvidenceSectionV2 {
    pub(crate) const ALL: [Self; 10] = [
        Self::Receipt,
        Self::Mapping,
        Self::Source,
        Self::Geometry,
        Self::Clips,
        Self::Paints,
        Self::Hits,
        Self::Semantics,
        Self::Queries,
        Self::Raster,
    ];

    pub(crate) const fn tag(self) -> u8 {
        self as u8
    }

    pub(crate) const fn token(self) -> &'static str {
        match self {
            Self::Receipt => "receipt",
            Self::Mapping => "mapping",
            Self::Source => "source",
            Self::Geometry => "geometry",
            Self::Clips => "clips",
            Self::Paints => "paints",
            Self::Hits => "hits",
            Self::Semantics => "semantics",
            Self::Queries => "queries",
            Self::Raster => "raster",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EvidenceBuildErrorV2 {
    pub(crate) location: &'static str,
}

pub(crate) fn observation_from_records_v2(
    case: u8,
    step: u8,
    generation: Option<u64>,
    viewport: (u32, u32),
    records: [Vec<EvidenceRecordV2>; 10],
) -> SpatialEvidenceObservationV2 {
    let sections = EvidenceSectionV2::ALL
        .into_iter()
        .zip(records)
        .map(|(name, records)| NormalizedSectionV2::new(name, records))
        .collect();
    SpatialEvidenceObservationV2 {
        case,
        step,
        generation,
        viewport,
        sections,
    }
}

pub(crate) fn fnv_evidence_v2(section: &str, encoded: &[u8]) -> u64 {
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

pub(crate) fn put_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}

pub(crate) fn put_u64(output: &mut Vec<u8>, value: u64) {
    output.extend_from_slice(&value.to_le_bytes());
}
