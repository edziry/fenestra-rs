use super::*;

#[test]
fn every_direct_count_accepts_equality_and_rejects_one_over() {
    const MAXIMUM: usize = 3;
    let limits = limits_with_direct([MAXIMUM; DIRECT_COUNT]);

    for kind in SpatialLimitKindV2::DIRECT_ALL {
        expect_valid(validate_direct_count(kind, MAXIMUM, limits));

        expect_limit(
            validate_direct_count(kind, MAXIMUM + 1, limits),
            kind,
            MAXIMUM as u128 + 1,
            MAXIMUM as u128,
        );
    }
}

#[cfg(target_pointer_width = "64")]
#[test]
fn globally_indexed_tables_use_the_u32_row_capacity_ceiling() {
    let limits = limits_with_direct([usize::MAX; DIRECT_COUNT]);
    let capacity = usize::try_from(U32_ROW_CAPACITY).expect("u32 row capacity fits u64 usize");

    for index in GLOBALLY_INDEXED_DIRECT_INDICES {
        let kind = SpatialLimitKindV2::DIRECT_ALL[index];
        expect_valid(validate_direct_count(kind, capacity, limits));

        expect_limit(
            validate_direct_count(kind, capacity + 1, limits),
            kind,
            U32_ROW_CAPACITY + 1,
            U32_ROW_CAPACITY,
        );
    }
}

#[cfg(target_pointer_width = "64")]
#[test]
fn payload_tables_use_the_caller_maximum_above_u32_capacity() {
    let caller_maximum = usize::try_from(U32_ROW_CAPACITY + 17)
        .expect("payload maximum above u32 capacity fits u64 usize");
    let mut maxima = [0; DIRECT_COUNT];

    for index in PAYLOAD_DIRECT_INDICES {
        maxima[index] = caller_maximum;
    }
    let limits = limits_with_direct(maxima);

    for index in PAYLOAD_DIRECT_INDICES {
        let kind = SpatialLimitKindV2::DIRECT_ALL[index];
        expect_valid(validate_direct_count(kind, caller_maximum, limits));

        expect_limit(
            validate_direct_count(kind, caller_maximum + 1, limits),
            kind,
            caller_maximum as u128 + 1,
            caller_maximum as u128,
        );
    }
}
