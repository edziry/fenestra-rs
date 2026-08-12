#![forbid(unsafe_code)]

//! Private EXP-0008 hybrid spatial evidence probe.

#[cfg(test)]
mod baseline;

#[cfg(all(test, feature = "numeric-spatial"))]
mod lanes;

#[cfg(test)]
mod tests;
