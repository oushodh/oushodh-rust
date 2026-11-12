//! The objects the API answers with, as `openapi.json` describes them.
//!
//! A value no source gives is `None`, never a blank, a zero or an estimate. Every object keeps the
//! fields this version does not know in `extra`, because `/v1` may gain fields.

mod medicine;
mod meta;
mod monograph;
mod page;
mod prices;
pub(crate) mod time;

use serde_json::{Map, Value};

pub use medicine::{Medicine, MedicineDetail, MedicineType};
pub use meta::Meta;
pub use monograph::Monograph;
pub use page::{Page, SearchResult};
pub use prices::{PackPrice, PriceAmount, Prices, SizePrice, StripPrice, UnitPrice};

/// Fields the API sent that this version of the SDK does not know.
pub type Extra = Map<String, Value>;
