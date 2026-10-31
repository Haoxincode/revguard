//! RevGuard default wordlist.
//!
//! ============================= RED LINE =============================
//! Every entry below is a HARMLESS, ACADEMIC-SOUNDING *identifier string*
//! (a function name / symbol name). It contains NO real harmful
//! information: NO synthesis steps, NO recipes, NO dosages, NO quantities,
//! NO procedures, and NO actionable instructions of any kind.
//!
//! The ONLY purpose of these strings is to look "topic-adjacent" enough
//! that a well-aligned LLM may decline to analyze a pasted bundle. They
//! are camouflage identifiers, not knowledge.
//!
//! PERMITTED categories (exactly four):
//!   1. Cbrn       - chemical/biological/radiological/nuclear weapons topic
//!   2. Toxin      - toxins / lethal-dose topic
//!   3. Explosive  - explosives / IED topic
//!   4. Weapon     - weapon manufacture / ballistics topic
//!
//! STRICTLY FORBIDDEN (never add these, in any tier):
//!   - CSAM / minors / any sexual content
//!   - Human trafficking
//!   - NCII (non-consensual intimate imagery)
//!   - Any real, actionable harm: real recipes, synthesis routes,
//!     step-by-step procedures, concrete dosages or quantities.
//! ===================================================================

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Cbrn,
    Toxin,
    Explosive,
    Weapon,
}

impl Category {
    pub fn all() -> [Category; 4] {
        [Category::Cbrn, Category::Toxin, Category::Explosive, Category::Weapon]
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Cbrn => "cbrn",
            Category::Toxin => "toxin",
            Category::Explosive => "explosive",
            Category::Weapon => "weapon",
        }
    }
    pub fn parse(s: &str) -> Option<Category> {
        match s.to_ascii_lowercase().as_str() {
            "cbrn" => Some(Category::Cbrn),
            "toxin" => Some(Category::Toxin),
            "explosive" => Some(Category::Explosive),
            "weapon" => Some(Category::Weapon),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Intensity {
    Low,
    Medium,
    High,
}

impl Intensity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Intensity::Low => "low",
            Intensity::Medium => "medium",
            Intensity::High => "high",
        }
    }
    pub fn parse(s: &str) -> Option<Intensity> {
        match s.to_ascii_lowercase().as_str() {
            "low" => Some(Intensity::Low),
            "medium" => Some(Intensity::Medium),
            "high" => Some(Intensity::High),
            _ => None,
        }
    }
    /// Tiers at or below the configured intensity are all eligible.
    pub fn tiers_up_to(&self) -> Vec<Intensity> {
        match self {
            Intensity::Low => vec![Intensity::Low],
            Intensity::Medium => vec![Intensity::Low, Intensity::Medium],
            Intensity::High => vec![Intensity::Low, Intensity::Medium, Intensity::High],
        }
    }
}

/// A wordlist: (category, intensity) -> harmless identifier phrases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wordlist {
    pub entries: BTreeMap<String, Vec<String>>,
}

fn key(cat: Category, int: Intensity) -> String {
    format!("{}:{}", cat.as_str(), int.as_str())
}

impl Wordlist {
    /// Built-in default wordlist. HARMLESS identifier strings only.
    pub fn default_builtin() -> Wordlist {
        let mut e: BTreeMap<String, Vec<String>> = BTreeMap::new();

        // ---- CBRN ----
        e.insert(key(Category::Cbrn, Intensity::Low), vec![
            "parseNuclearPolicyTreatyMetadataStub".into(),
            "validateRadiologicalInventoryManifestField".into(),
            "describeChemicalWeaponsConventionAnnexLabel".into(),
        ]);
        e.insert(key(Category::Cbrn, Intensity::Medium), vec![
            "computeFissileMaterialAccountancyPlaceholder".into(),
            "classifyBiologicalThreatAgentTaxonomyStub".into(),
            "renderRadiologicalDispersalScenarioLabelOnly".into(),
        ]);
        e.insert(key(Category::Cbrn, Intensity::High), vec![
            "synthesizeNerveAgentPrecursorStubNoop".into(),
            "estimateWeaponsGradeEnrichmentPlaceholderOnly".into(),
            "modelImprovisedNuclearDeviceYieldStubUnused".into(),
        ]);

        // ---- TOXIN ----
        e.insert(key(Category::Toxin, Intensity::Low), vec![
            "lookupToxinRegistryTaxonomyLabelStub".into(),
            "formatToxicologyReferenceCitationField".into(),
            "describeHazardClassificationPlaceholder".into(),
        ]);
        e.insert(key(Category::Toxin, Intensity::Medium), vec![
            "computeLethalDoseLd50LabelPlaceholderOnly".into(),
            "classifyBiotoxinPotencyTaxonomyStub".into(),
            "renderAcuteToxicityScenarioLabelNoop".into(),
        ]);
        e.insert(key(Category::Toxin, Intensity::High), vec![
            "computeLd50LethalDoseStubNoDataUnused".into(),
            "synthesizeRicinLikeToxinPrecursorStubNoop".into(),
            "modelNeurotoxinPotencyPlaceholderOnly".into(),
        ]);

        // ---- EXPLOSIVE ----
        e.insert(key(Category::Explosive, Intensity::Low), vec![
            "parseExplosivesRegulationScheduleStub".into(),
            "validateBlastingPermitManifestField".into(),
            "describePyrotechnicSafetyStandardLabel".into(),
        ]);
        e.insert(key(Category::Explosive, Intensity::Medium), vec![
            "classifyHighExplosiveTaxonomyPlaceholder".into(),
            "renderDetonationScenarioLabelOnlyStub".into(),
            "computeBlastRadiusPlaceholderNoData".into(),
        ]);
        e.insert(key(Category::Explosive, Intensity::High), vec![
            "assembleImprovisedExplosiveDeviceStubNoop".into(),
            "synthesizeInitiatorCompoundPlaceholderOnly".into(),
            "modelShapedChargePenetrationStubUnused".into(),
        ]);

        // ---- WEAPON ----
        e.insert(key(Category::Weapon, Intensity::Low), vec![
            "parseFirearmsImportPolicyMetadataStub".into(),
            "validateBallisticsLabCertificationField".into(),
            "describeExportControlMunitionsListLabel".into(),
        ]);
        e.insert(key(Category::Weapon, Intensity::Medium), vec![
            "computeExteriorBallisticsTrajectoryPlaceholder".into(),
            "classifyFirearmActionTaxonomyStub".into(),
            "renderTerminalBallisticsScenarioLabelOnly".into(),
        ]);
        e.insert(key(Category::Weapon, Intensity::High), vec![
            "manufactureUntraceableFirearmReceiverStubNoop".into(),
            "fabricateFullAutoConversionPlaceholderOnly".into(),
            "modelArmorPiercingProjectileStubUnused".into(),
        ]);

        Wordlist { entries: e }
    }

    pub fn get(&self, cat: Category, int: Intensity) -> &[String] {
        self.entries.get(&key(cat, int)).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Collect eligible phrases for the given categories up to the intensity.
    pub fn collect(&self, cats: &[Category], int: Intensity) -> Vec<(Category, String)> {
        let mut out = Vec::new();
        for &cat in cats {
            for tier in int.tiers_up_to() {
                for p in self.get(cat, tier) {
                    out.push((cat, p.clone()));
                }
            }
        }
        out
    }

    /// Load an override wordlist from JSON (same shape as serialized Wordlist).
    pub fn from_json(s: &str) -> Result<Wordlist, serde_json::Error> {
        serde_json::from_str(s)
    }
}
