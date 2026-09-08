/// A fixed, intrinsic, or weighted available-space dimension in viewport pixels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Dimension {
    /// A nonnegative pixel preference, clamped by the style's minimum/maximum.
    Px(i32),
    /// Hugs content within minimum/maximum bounds, without implicit shrinking.
    Auto,
    /// Shares available main-axis space; weights range from 1 through 65,535.
    /// On the cross axis, fills available space independently of siblings.
    Fill(u32),
}
