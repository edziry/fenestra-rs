use std::fmt;

/// Provisional downstream work classifications used by experiment artifacts.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum InvalidationClass {
    /// Logical structure changed.
    Structure,
    /// Style matching inputs changed.
    StyleMatch,
    /// Intrinsic measurement inputs changed.
    Intrinsic,
    /// Layout inputs changed.
    Layout,
    /// Semantic projection inputs changed.
    Semantics,
    /// Hit-testing inputs changed.
    HitTest,
    /// Paint inputs changed.
    Paint,
    /// Composition inputs changed.
    Composition,
    /// Native surface inputs changed.
    Surface,
}

impl InvalidationClass {
    const ALL: [Self; 9] = [
        Self::Structure,
        Self::StyleMatch,
        Self::Intrinsic,
        Self::Layout,
        Self::Semantics,
        Self::HitTest,
        Self::Paint,
        Self::Composition,
        Self::Surface,
    ];

    const fn bit(self) -> u16 {
        1 << self as u16
    }
}

/// Replaceable set representation for provisional invalidation classes.
#[derive(Clone, Copy, Default, Eq, Hash, PartialEq)]
pub struct InvalidationSet(u16);

impl InvalidationSet {
    /// Empty invalidation set.
    pub const NONE: Self = Self(0);

    /// Creates a set containing one class.
    #[must_use]
    pub const fn from_class(class: InvalidationClass) -> Self {
        Self(class.bit())
    }

    /// Returns the union of two sets.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Returns whether the set contains a class.
    #[must_use]
    pub const fn contains(self, class: InvalidationClass) -> bool {
        self.0 & class.bit() != 0
    }

    /// Returns whether the set contains no classes.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Iterates classes in the documented artifact order.
    pub fn iter(self) -> InvalidationIter {
        InvalidationIter { set: self, next: 0 }
    }
}

impl fmt::Debug for InvalidationSet {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_set().entries(self.iter()).finish()
    }
}

/// Iterator over invalidation classes in deterministic artifact order.
pub struct InvalidationIter {
    set: InvalidationSet,
    next: usize,
}

impl Iterator for InvalidationIter {
    type Item = InvalidationClass;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(class) = InvalidationClass::ALL.get(self.next).copied() {
            self.next += 1;
            if self.set.contains(class) {
                return Some(class);
            }
        }
        None
    }
}
