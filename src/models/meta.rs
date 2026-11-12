use serde::{Deserialize, Serialize};

use super::Extra;

/// The account a key belongs to, and the data it reads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Meta {
    /// The account the key belongs to.
    pub client: String,
    /// The ID profile your medicine IDs come from.
    pub id_profile: String,
    /// How many medicines your key reads.
    pub medicines: u64,
    /// When the data was last imported, such as `2026-11-09T15:27:40Z`.
    pub last_import: Option<String>,
    /// Fields this version does not know.
    #[serde(flatten)]
    pub extra: Extra,
}
