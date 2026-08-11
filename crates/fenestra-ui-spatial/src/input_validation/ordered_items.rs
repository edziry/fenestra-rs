//! Shared owner and owner-local ordering validation for item tables.

use super::make_resolve_error;
use crate::content_diagnostic::{SpatialContentReferenceV2, SpatialOrderedItemTableV2};
use crate::content_error::SpatialContentErrorKindV2;
use crate::error::SpatialErrorLocationV2;
use crate::item_field::{SpatialHitFieldV2, SpatialPaintFieldV2, SpatialSemanticFieldV2};
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

#[derive(Clone, Copy)]
pub(super) struct OrderedItemCandidate {
    owner: u32,
    owner_count: usize,
    item_ordinal: u32,
}

impl OrderedItemCandidate {
    pub(super) const fn owner(self) -> u32 {
        self.owner
    }

    pub(super) const fn owner_count(self) -> usize {
        self.owner_count
    }

    pub(super) const fn item_ordinal(self) -> u32 {
        self.item_ordinal
    }
}

pub(super) struct OrderedItemCursor {
    current_owner: Option<u32>,
    owner_count: usize,
}

impl OrderedItemCursor {
    pub(super) const fn new() -> Self {
        Self {
            current_owner: None,
            owner_count: 0,
        }
    }

    pub(super) fn validate(
        &self,
        table: SpatialOrderedItemTableV2,
        index: u32,
        owner: u32,
        item_ordinal: u32,
        node_count: u128,
    ) -> Result<OrderedItemCandidate, SpatialResolveErrorV2> {
        if owner == 0 || u128::from(owner) >= node_count {
            return Err(content_error(
                SpatialContentErrorKindV2::InvalidReference(SpatialContentReferenceV2::Owner),
                item_location(table, index, OrderedItemHeaderField::Owner),
            ));
        }
        if self.current_owner.is_some_and(|previous| owner < previous) {
            return Err(invalid_order(table, index, OrderedItemHeaderField::Owner));
        }

        let owner_count = if self.current_owner == Some(owner) {
            self.owner_count
                .checked_add(1)
                .expect("an owner-local item count fits its input table")
        } else {
            1
        };
        let expected_ordinal =
            u32::try_from(owner_count - 1).expect("a trusted owner-local item ordinal fits u32");
        if item_ordinal != expected_ordinal {
            return Err(invalid_order(
                table,
                index,
                OrderedItemHeaderField::ItemOrdinal,
            ));
        }

        Ok(OrderedItemCandidate {
            owner,
            owner_count,
            item_ordinal: expected_ordinal,
        })
    }

    pub(super) const fn commit(&mut self, candidate: OrderedItemCandidate) {
        self.current_owner = Some(candidate.owner);
        self.owner_count = candidate.owner_count;
    }
}

#[derive(Clone, Copy)]
enum OrderedItemHeaderField {
    Owner,
    ItemOrdinal,
}

fn invalid_order(
    table: SpatialOrderedItemTableV2,
    index: u32,
    field: OrderedItemHeaderField,
) -> SpatialResolveErrorV2 {
    content_error(
        SpatialContentErrorKindV2::InvalidOrder(table),
        item_location(table, index, field),
    )
}

const fn item_location(
    table: SpatialOrderedItemTableV2,
    index: u32,
    field: OrderedItemHeaderField,
) -> SpatialErrorLocationV2 {
    match (table, field) {
        (SpatialOrderedItemTableV2::Paint, OrderedItemHeaderField::Owner) => {
            SpatialErrorLocationV2::Paint {
                index,
                field: SpatialPaintFieldV2::Owner,
            }
        }
        (SpatialOrderedItemTableV2::Paint, OrderedItemHeaderField::ItemOrdinal) => {
            SpatialErrorLocationV2::Paint {
                index,
                field: SpatialPaintFieldV2::ItemOrdinal,
            }
        }
        (SpatialOrderedItemTableV2::Hit, OrderedItemHeaderField::Owner) => {
            SpatialErrorLocationV2::Hit {
                index,
                field: SpatialHitFieldV2::Owner,
            }
        }
        (SpatialOrderedItemTableV2::Hit, OrderedItemHeaderField::ItemOrdinal) => {
            SpatialErrorLocationV2::Hit {
                index,
                field: SpatialHitFieldV2::ItemOrdinal,
            }
        }
        (SpatialOrderedItemTableV2::Semantic, OrderedItemHeaderField::Owner) => {
            SpatialErrorLocationV2::Semantic {
                index,
                field: SpatialSemanticFieldV2::Owner,
            }
        }
        (SpatialOrderedItemTableV2::Semantic, OrderedItemHeaderField::ItemOrdinal) => {
            SpatialErrorLocationV2::Semantic {
                index,
                field: SpatialSemanticFieldV2::ItemOrdinal,
            }
        }
    }
}

fn content_error(
    kind: SpatialContentErrorKindV2,
    location: SpatialErrorLocationV2,
) -> SpatialResolveErrorV2 {
    make_resolve_error(SpatialResolveErrorKindV2::Content(kind), location)
}
