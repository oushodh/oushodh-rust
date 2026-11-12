use serde::{Deserialize, Serialize};

use super::{Extra, Medicine};

/// One page of List medicines.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Page {
    /// The medicines on this page.
    pub items: Vec<Medicine>,
    /// Pass it back as `cursor` for the next page. `None` on the last page.
    pub next: Option<String>,
    /// How many medicines match, across all pages.
    pub total: u64,
    /// Fields this version does not know.
    #[serde(flatten)]
    pub extra: Extra,
}

/// The answer of Search: the best matches and the count of every match.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SearchResult {
    /// The best matches first, at most `limit`.
    pub items: Vec<Medicine>,
    /// How many medicines match, including those past `limit`.
    pub total: u64,
    /// Fields this version does not know.
    #[serde(flatten)]
    pub extra: Extra,
}
