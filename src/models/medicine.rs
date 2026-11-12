use serde::{Deserialize, Serialize};

use super::{Extra, Monograph, Prices, UnitPrice};

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

impl Medicine {
    /// `last_updated` as a time, to pass back as `updated_since`. `None` if the API ever sends it in
    /// another form.
    pub fn last_updated_at(&self) -> Option<std::time::SystemTime> {
        super::time::parse(&self.last_updated)
    }
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
