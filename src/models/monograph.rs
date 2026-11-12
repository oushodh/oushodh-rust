use serde::{Deserialize, Serialize};

use super::Extra;

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
