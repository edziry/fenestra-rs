use fenestra_ui::{TextAffinity, TextError, TextGeometryRequest, TextPosition};
use parley::{Affinity, Cursor, Layout};

use crate::source::Source;

pub(super) struct Cursors<'a, 'b> {
    pub layout: &'a Layout<()>,
    source: &'a Source<'b>,
    boundaries: Vec<usize>,
}

impl<'a, 'b> Cursors<'a, 'b> {
    pub fn new(
        layout: &'a Layout<()>,
        source: &'a Source<'b>,
        request: TextGeometryRequest<'_>,
    ) -> Result<Self, TextError> {
        let mut boundaries = Vec::new();
        for boundary in request.grapheme_boundaries() {
            boundaries
                .try_reserve(1)
                .map_err(|_| TextError::Engine("text boundary allocation failed".into()))?;
            boundaries.push(boundary);
        }
        Ok(Self {
            layout,
            source,
            boundaries,
        })
    }

    pub fn cursor(&self, position: TextPosition) -> Cursor {
        let affinity = match position.affinity() {
            TextAffinity::Upstream => Affinity::Upstream,
            TextAffinity::Downstream => Affinity::Downstream,
        };
        self.at(position.byte(), affinity)
    }

    pub fn to_position(&self, cursor: Cursor) -> TextPosition {
        let affinity = match cursor.affinity() {
            Affinity::Upstream => TextAffinity::Upstream,
            Affinity::Downstream => TextAffinity::Downstream,
        };
        TextPosition::new(self.source.to_source(cursor.index()), affinity)
    }

    pub fn hit(&self, x: f64, y: f64) -> Cursor {
        let cursor = Cursor::from_point(self.layout, x as f32, y as f32);
        self.valid_near(cursor, x, y)
    }

    pub fn valid_near(&self, cursor: Cursor, x: f64, y: f64) -> Cursor {
        let raw = self.source.to_source(cursor.index());
        let Err(index) = self.boundaries.binary_search(&raw) else {
            return cursor;
        };
        // The candidate may expose a component inside one owned grapheme.
        // Compare only its enclosing valid endpoints using their shaped caret
        // positions, retaining both affinities at bidi and wrap boundaries.
        let first = index.saturating_sub(1);
        let last = index.min(self.boundaries.len() - 1);
        let mut best = self.at(self.boundaries[first], cursor.affinity());
        let mut distance = self.distance(best, x, y);
        for index in [first, last] {
            for affinity in [cursor.affinity(), cursor.affinity().invert()] {
                let candidate = self.at(self.boundaries[index], affinity);
                let candidate_distance = self.distance(candidate, x, y);
                if candidate_distance < distance {
                    best = candidate;
                    distance = candidate_distance;
                }
            }
        }
        best
    }

    pub fn step(&self, cursor: Cursor, right: bool) -> Cursor {
        let next = if right {
            cursor.next_visual(self.layout)
        } else {
            cursor.previous_visual(self.layout)
        };
        let raw = self.source.to_source(next.index());
        let Err(index) = self.boundaries.binary_search(&raw) else {
            return next;
        };
        // Skip an entire grapheme in one bounded operation, rather than
        // repeatedly querying every combining component of a long cluster.
        let rtl = next
            .logical_clusters(self.layout)
            .into_iter()
            .flatten()
            .next()
            .is_some_and(|cluster| cluster.is_rtl());
        let logical_forward = right != rtl;
        let index = if logical_forward {
            index
        } else {
            index.saturating_sub(1)
        };
        self.at(
            self.boundaries[index.min(self.boundaries.len() - 1)],
            if logical_forward {
                Affinity::Upstream
            } else {
                Affinity::Downstream
            },
        )
    }

    fn at(&self, raw: usize, affinity: Affinity) -> Cursor {
        Cursor::from_byte_index(self.layout, self.source.to_layout(raw), affinity)
    }

    fn distance(&self, cursor: Cursor, x: f64, y: f64) -> (f64, f64) {
        let rect = cursor.geometry(self.layout, 0.0);
        let dy = (rect.y0 - y).max(y - rect.y1).max(0.0);
        (dy, (rect.x0 - x).abs())
    }
}
