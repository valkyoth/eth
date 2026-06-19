#![no_std]
#![forbid(unsafe_code)]
//! Test helpers and pinned conformance-corpus metadata for `eth`.

/// A named conformance corpus revision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CorpusRevision<'a> {
    /// Human-readable corpus name.
    pub name: &'a str,
    /// Pinned upstream revision.
    pub revision: &'a str,
}

impl<'a> CorpusRevision<'a> {
    /// Creates a pinned corpus revision.
    #[must_use]
    pub const fn new(name: &'a str, revision: &'a str) -> Self {
        Self { name, revision }
    }
}
