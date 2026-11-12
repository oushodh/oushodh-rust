//! The objects the API answers with, as `openapi.json` describes them.
//!
//! A value no source gives is `None`, never a blank, a zero or an estimate. Every object keeps the
//! fields this version does not know in `extra`, because `/v1` may gain fields.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Fields the API sent that this version of the SDK does not know.
pub type Extra = Map<String, Value>;

/// One medicine: one brand in one strength and dosage form.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Medicine {
    /// The medicine's ID in your account's ID profile, such as `MED-012933`. It never changes and is
    /// never reused.
    pub id: String,
    /// The key that never changes. Store it to match a medicine across syncs.
    pub uuid: String,
    /// The brand, such as `Napa`.
    pub brand: String,
    /// Reserved for a symptom category. `None` for every medicine today.
    pub category: Option<String>,
    /// Brand, strength and dosage form together, such as `Napa 500 mg Tablet`.
    pub product_name: String,
    /// The address of the medicine's picture. `None` when there is none.
    pub product_image: Option<String>,
    /// Every picture of the medicine, `product_image` first. Empty when there is none.
    pub images: Vec<String>,
    /// The pack's unit, such as `Strip` or `Bottle`.
    pub unit: Option<String>,
    /// The pack's size, such as `10 Tablets` or `200 ml`.
    pub size: Option<String>,
    /// MRP in BDT of what `price_per` names. Kept for older clients: read `prices`.
    pub price: Option<f64>,
    /// What `price` buys: `unit`, `strip`, or the pack as its source names it. Kept for older
    /// clients: read `prices`.
    pub price_per: Option<String>,
    /// Where the price comes from.
    pub price_source: Option<String>,
    /// The unit price in each pack size, cheapest first. Kept for older clients: read
    /// `prices.sizes`.
    pub unit_prices: Vec<UnitPrice>,
    /// Every MRP the sources state for the medicine. This is the home of all price data.
    pub prices: Prices,
    /// A description in English.
    pub description_en: Option<String>,
    /// A description in Bangla.
    pub description_bn: Option<String>,
    /// The dosage form, such as `Tablet`.
    pub dosage_form: Option<String>,
    /// The strength, such as `500 mg`.
    pub strength: Option<String>,
    /// The generic name, such as `Paracetamol`.
    pub generic_name: Option<String>,
    /// The manufacturer.
    pub manufacturer: Option<String>,
    /// The page the record was read from.
    pub source_url: Option<String>,
    /// When any field of this record last changed, such as `2026-11-09T15:27:34Z`. Sync uses it.
    pub last_updated: String,
    /// `true` when a DGDA registration (DAR number) links to the medicine.
    pub dgda_listed: bool,
    /// The DGDA register that lists it. `None` when no register lists it.
    pub medicine_type: Option<MedicineType>,
    /// Fields this version does not know.
    #[serde(flatten)]
    pub extra: Extra,
}

/// The DGDA register that lists a medicine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum MedicineType {
    /// Allopathic.
    Allopathic,
    /// Homoeopathic.
    Homoeopathic,
    /// Herbal.
    Herbal,
    /// Ayurvedic.
    Ayurvedic,
    /// Unani.
    Unani,
    /// A register this version does not know.
    #[serde(other)]
    Unknown,
}

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

/// One medicine with its medical text, as Get a medicine answers it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MedicineDetail {
    /// The medicine.
    #[serde(flatten)]
    pub medicine: Medicine,
    /// The medical text. `None` when no source gives the medicine any text.
    pub monograph: Option<Monograph>,
}

impl std::ops::Deref for MedicineDetail {
    type Target = Medicine;

    fn deref(&self) -> &Medicine {
        &self.medicine
    }
}

/// The medical text of a medicine, as its source wrote it, in English. It is information for
/// professionals, not medical advice. Some text holds HTML tags.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Monograph {
    /// The source of the text. Show it with the text.
    pub source: String,
    /// What the medicine is used for.
    pub indications: Option<String>,
    /// What the medicine contains.
    pub composition: Option<String>,
    /// How the medicine works.
    pub pharmacology: Option<String>,
    /// Dosage and how to give it.
    pub dosage: Option<String>,
    /// Interactions with other medicines.
    pub interactions: Option<String>,
    /// When the medicine must not be used.
    pub contraindications: Option<String>,
    /// Side effects.
    pub side_effects: Option<String>,
    /// Use in pregnancy and while breastfeeding.
    pub pregnancy_lactation: Option<String>,
    /// Precautions and warnings.
    pub precautions: Option<String>,
    /// What an overdose does, and its treatment.
    pub overdose: Option<String>,
    /// How to store the medicine.
    pub storage: Option<String>,
    /// Fields this version does not know.
    #[serde(flatten)]
    pub extra: Extra,
}

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
