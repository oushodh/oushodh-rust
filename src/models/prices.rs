use serde::{Deserialize, Serialize};

use super::Extra;

/// The unit price in one pack size, from `unit_prices`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct UnitPrice {
    /// The pack size as the source names it. `None` when the source names none.
    pub size: Option<String>,
    /// MRP in BDT of one unit in that size.
    pub price: f64,
    /// Fields this version does not know.
    #[serde(flatten)]
    pub extra: Extra,
}

/// Every MRP the sources state for a medicine: per unit, per strip and per pack. No amount is
/// computed from another one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Prices {
    /// Always `BDT`, Bangladeshi taka.
    pub currency: String,
    /// The MRP of one unit. `None` when no source states it, or when it differs by pack size.
    pub unit: Option<PriceAmount>,
    /// The unit price in each pack size, cheapest first. Empty otherwise.
    pub sizes: Vec<SizePrice>,
    /// The MRP of one strip. `None` when no source states it.
    pub strip: Option<StripPrice>,
    /// The MRP of each pack or box, cheapest first. Empty when no source lists any.
    pub packs: Vec<PackPrice>,
    /// Fields this version does not know.
    #[serde(flatten)]
    pub extra: Extra,
}

/// An MRP and the source that states it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PriceAmount {
    /// MRP in BDT.
    pub amount: f64,
    /// The source that states this price.
    pub source: String,
    /// Fields this version does not know.
    #[serde(flatten)]
    pub extra: Extra,
}

/// The unit price in one pack size, from `prices.sizes`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SizePrice {
    /// The pack size as the source names it. `None` when it names none.
    pub size: Option<String>,
    /// MRP in BDT.
    pub amount: f64,
    /// The source that states this price.
    pub source: String,
    /// Fields this version does not know.
    #[serde(flatten)]
    pub extra: Extra,
}

/// The MRP of one strip.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StripPrice {
    /// MRP in BDT.
    pub amount: f64,
    /// How many units the strip holds. `None` when the source does not say.
    pub units: Option<f64>,
    /// The source that states this price.
    pub source: String,
    /// Fields this version does not know.
    #[serde(flatten)]
    pub extra: Extra,
}

/// The MRP of one pack or box.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PackPrice {
    /// MRP in BDT.
    pub amount: f64,
    /// How many units the pack holds. `None` when the source does not say.
    pub units: Option<f64>,
    /// The pack as the source names it, such as `30's pack`.
    pub label: String,
    /// The source that states this price.
    pub source: String,
    /// Fields this version does not know.
    #[serde(flatten)]
    pub extra: Extra,
}
