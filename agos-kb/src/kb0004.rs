//! # KB-0004: Pattern POS Database
//!
//! Defines types and implementations for the Pattern POS Database (KB-0004),
//! which assigns part-of-speech and confidence scores to morphological patterns
//! and provides stem-level POS overrides.
//!
//! This is the authoritative replacement for the heuristic `COMMON_NOUNS_3L`,
//! `COMMON_VERBS_3L`, and `check_verb_form()` match arms in agos-morph.
//!
//! ## Phase 1 Scope
//!
//! Phase 1 seeds KB-0004 from the existing heuristic lists:
//! - `stem_overrides`: ~207 entries from COMMON_NOUNS_3L + COMMON_VERBS_3L
//! - `verb_pos_profiles`: ~12 verb form I–X entries from check_verb_form()
//! - `noun_pos_profiles`: ~24 noun pattern entries from match_noun_patterns()
//!
//! ## Spec Alignment
//!
//! - specs/KB/KB-0004-wazan-pattern-database.md
//! - KB-OVERVIEW: KB Suite Overview

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use agos_core::types::PartOfSpeech;

// ──────────────────────────────────────────────
//  Core Types
// ──────────────────────────────────────────────

/// A stem-level POS override entry.
///
/// This is the direct replacement for `COMMON_NOUNS_3L` and `COMMON_VERBS_3L`.
/// Each entry assigns a POS + confidence to a specific 3-letter stem,
/// overriding the default pattern-based deduction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StemOverrideEntry {
    /// The stem text (e.g., "رجل", "كتب")
    pub stem_text: String,
    /// Primary part of speech for this stem
    pub pos: PartOfSpeech,
    /// Confidence score (0.0–1.0). Higher = stronger POS signal.
    /// 0.85+ = strongly favors this POS, 0.50–0.70 = ambiguous
    pub confidence: f64,
    /// Secondary POS for genuinely ambiguous stems (e.g., علم = Noun/Verb)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary_pos: Option<PartOfSpeech>,
    /// Source of this override (e.g., "heuristic:COMMON_NOUNS_3L")
    pub source: String,
    /// Optional semantic category (e.g., "body", "food", "abstract")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

/// A verb form POS profile entry.
///
/// Assigns a POS and base confidence to each verb form (I–X), with
/// stem-length constraints and root-type applicability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerbPosProfile {
    /// Verb form number (1–15)
    pub verb_form: u8,
    /// Canonical pattern with ف-ع-ل placeholders (e.g., "فَعَلَ")
    pub canonical_pattern: String,
    /// Template script (e.g., "C₁aC₂aC₃a")
    pub template_script: String,
    /// Default POS for this verb form (always Verb)
    pub default_pos: PartOfSpeech,
    /// Base confidence (0.0–1.0)
    pub default_confidence: f64,
    /// Boosted confidence for stems in the known-verb list (0.0–1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boosted_confidence: Option<f64>,
    /// Minimum stem length in characters
    pub min_stem_len: usize,
    /// Maximum stem length in characters
    pub max_stem_len: usize,
    /// Expected prefix pattern (e.g., "ي", "أ", "ت"), if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix_pattern: Option<String>,
    /// Root types this pattern applies to
    #[serde(default)]
    pub root_type_applicability: Vec<String>,
}

/// A noun pattern POS profile entry.
///
/// Assigns a POS and confidence to each common noun pattern template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NounPosProfile {
    /// Canonical pattern with ف-ع-ل placeholders (e.g., "فَاعِل", "فَعْل")
    pub canonical_pattern: String,
    /// Template script (e.g., "C₁āC₂iC₃")
    pub template_script: String,
    /// Default POS for this noun pattern
    pub default_pos: PartOfSpeech,
    /// Base confidence (0.0–1.0)
    pub default_confidence: f64,
    /// Minimum stem length in characters
    pub min_stem_len: usize,
    /// Maximum stem length in characters
    pub max_stem_len: usize,
    /// Noun type classification (e.g., "masdar", "ism_fail")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noun_type: Option<String>,
}

/// Container for loaded KB-0004 data.
#[derive(Debug, Clone)]
pub struct Kb0004 {
    /// Stem POS overrides, indexed by stem_text
    stem_overrides: HashMap<String, StemOverrideEntry>,
    /// Verb form POS profiles (ordered list — multiple variants can share the same form)
    verb_profiles: Vec<VerbPosProfile>,
    /// Noun pattern POS profiles, indexed by canonical_pattern
    noun_profiles: HashMap<String, NounPosProfile>,
}

// ──────────────────────────────────────────────
//  WazanPatternLookup Trait
// ──────────────────────────────────────────────

/// Trait for looking up POS and confidence data from KB-0004.
///
/// This is the primary interface that MOD-04 (MorphologicalParser) uses
/// to replace the heuristic COMMON_NOUNS_3L/COMMON_VERBS_3L checks and
/// the check_verb_form()/match_noun_patterns() hardcoded logic.
pub trait WazanPatternLookup: Send + Sync + std::fmt::Debug {
    /// Look up a stem POS override. Returns None if not in the override list.
    ///
    /// This replaces `COMMON_NOUNS_3L.contains(&stem_text)` and
    /// `COMMON_VERBS_3L.contains(&stem_text)`.
    fn stem_pos_override(&self, stem_text: &str) -> Option<&StemOverrideEntry>;

    /// Check if a 3-letter stem is primarily a noun (replaces COMMON_NOUNS_3L).
    fn is_primarily_noun(&self, stem_text: &str) -> Option<f64> {
        self.stem_pos_override(stem_text)
            .filter(|s| s.pos == PartOfSpeech::Noun || s.pos == PartOfSpeech::Adjective)
            .map(|s| s.confidence)
    }

    /// Check if a 3-letter stem is primarily a verb (replaces COMMON_VERBS_3L boost).
    fn is_primarily_verb(&self, stem_text: &str) -> Option<f64> {
        self.stem_pos_override(stem_text)
            .filter(|s| s.pos == PartOfSpeech::Verb)
            .map(|s| {
                // Use boosted confidence if available, otherwise use regular confidence
                s.confidence
            })
    }

    /// Get the best POS guess for a 3-letter stem (unified noun/verb disambiguation).
    fn best_pos_for_3l_stem(&self, stem_text: &str) -> Option<(PartOfSpeech, f64, String)> {
        let override_entry = self.stem_pos_override(stem_text)?;
        let source = format!("KB-0004:override:{}", override_entry.source);
        Some((override_entry.pos, override_entry.confidence, source))
    }

    /// Get the POS profile for a verb form.
    /// Get the verb profile(s) for a verb form. May return multiple variants.
    fn verb_profiles_for_form(&self, verb_form: u8) -> Vec<&VerbPosProfile>;

    /// Get the default confidence for a verb form (returns the highest confidence variant).
    fn verb_confidence(&self, verb_form: u8) -> f64 {
        let profiles = self.verb_profiles_for_form(verb_form);
        profiles.iter()
            .map(|p| p.default_confidence)
            .fold(0.0_f64, |a, b| a.max(b))
    }

    /// Get the boosted confidence for a verb form (returns min of all boosted confidences).
    fn verb_boosted_confidence(&self, verb_form: u8) -> f64 {
        let profiles = self.verb_profiles_for_form(verb_form);
        let base = self.verb_confidence(verb_form);
        profiles.iter()
            .filter_map(|p| p.boosted_confidence)
            .fold(base, |a, b| a.max(b))
    }

    /// Get the POS profile for a noun pattern.
    fn noun_profile(&self, canonical_pattern: &str) -> Option<&NounPosProfile>;

    /// Get the default POS for a noun pattern.
    fn noun_pos(&self, canonical_pattern: &str) -> PartOfSpeech {
        self.noun_profile(canonical_pattern)
            .map(|p| p.default_pos)
            .unwrap_or(PartOfSpeech::Noun)
    }

    /// Get the confidence for a noun pattern.
    fn noun_confidence(&self, canonical_pattern: &str) -> f64 {
        self.noun_profile(canonical_pattern)
            .map(|p| p.default_confidence)
            .unwrap_or(0.15)
    }

    /// Check if a noun pattern is primarily an Adjective.
    fn noun_is_adjective(&self, canonical_pattern: &str) -> bool {
        self.noun_pos(canonical_pattern) == PartOfSpeech::Adjective
    }

    /// Check if a noun pattern's confidence outranks a verb form's confidence.
    fn noun_outranks_verb(&self, canonical_pattern: &str, verb_form: u8) -> bool {
        let noun_conf = self.noun_confidence(canonical_pattern);
        let verb_conf = self.verb_confidence(verb_form);
        noun_conf > verb_conf
    }
}

// ──────────────────────────────────────────────
//  Kb0004 Implementation
// ──────────────────────────────────────────────

impl Kb0004 {
    /// Create an empty KB-0004 instance.
    pub fn empty() -> Self {
        Self {
            stem_overrides: HashMap::new(),
            verb_profiles: Vec::new(),
            noun_profiles: HashMap::new(),
        }
    }

    /// Load KB-0004 from a directory of JSON files.
    ///
    /// Expected files:
    /// - `stem-overrides.json` — Stem POS override entries
    /// - `verb-pos-profiles.json` — Verb form POS profiles
    /// - `noun-pos-profiles.json` — Noun pattern POS profiles
    pub fn load_from_directory(path: &std::path::Path) -> Result<Self, String> {
        let stem_path = path.join("stem-overrides.json");
        let verb_path = path.join("verb-pos-profiles.json");
        let noun_path = path.join("noun-pos-profiles.json");

        let stem_overrides = if stem_path.exists() {
            let content = std::fs::read_to_string(&stem_path)
                .map_err(|e| format!("Cannot read {:?}: {}", stem_path, e))?;
            let wrapper: StemOverrideWrapper = serde_json::from_str(&content)
                .map_err(|e| format!("Cannot parse {:?}: {}", stem_path, e))?;
            let mut map = HashMap::new();
            for entry in wrapper.entries {
                map.insert(entry.stem_text.clone(), entry);
            }
            map
        } else {
            HashMap::new()
        };

        let verb_profiles = if verb_path.exists() {
            let content = std::fs::read_to_string(&verb_path)
                .map_err(|e| format!("Cannot read {:?}: {}", verb_path, e))?;
            let wrapper: VerbProfileWrapper = serde_json::from_str(&content)
                .map_err(|e| format!("Cannot parse {:?}: {}", verb_path, e))?;
            wrapper.entries
        } else {
            Vec::new()
        };

        let noun_profiles = if noun_path.exists() {
            let content = std::fs::read_to_string(&noun_path)
                .map_err(|e| format!("Cannot read {:?}: {}", noun_path, e))?;
            let wrapper: NounProfileWrapper = serde_json::from_str(&content)
                .map_err(|e| format!("Cannot parse {:?}: {}", noun_path, e))?;
            let mut map = HashMap::new();
            for entry in wrapper.entries {
                map.insert(entry.canonical_pattern.clone(), entry);
            }
            map
        } else {
            HashMap::new()
        };

        Ok(Self {
            stem_overrides,
            verb_profiles,
            noun_profiles,
        })
    }

    /// Get the number of loaded stem overrides.
    pub fn stem_override_count(&self) -> usize {
        self.stem_overrides.len()
    }

    /// Get the number of loaded verb profiles.
    pub fn verb_profile_count(&self) -> usize {
        self.verb_profiles.len()
    }

    /// Get the number of loaded noun profiles.
    pub fn noun_profile_count(&self) -> usize {
        self.noun_profiles.len()
    }
}

impl WazanPatternLookup for Kb0004 {
    fn stem_pos_override(&self, stem_text: &str) -> Option<&StemOverrideEntry> {
        self.stem_overrides.get(stem_text)
    }

    fn verb_profiles_for_form(&self, verb_form: u8) -> Vec<&VerbPosProfile> {
        self.verb_profiles.iter()
            .filter(|p| p.verb_form == verb_form)
            .collect()
    }

    fn noun_profile(&self, canonical_pattern: &str) -> Option<&NounPosProfile> {
        self.noun_profiles.get(canonical_pattern)
    }
}

// ──────────────────────────────────────────────
//  JSON Wrapper Types
// ──────────────────────────────────────────────

/// JSON wrapper for stem override entries.
#[derive(Debug, Deserialize)]
struct StemOverrideWrapper {
    #[allow(dead_code)]
    description: Option<String>,
    entries: Vec<StemOverrideEntry>,
}

/// JSON wrapper for verb profile entries.
#[derive(Debug, Deserialize)]
struct VerbProfileWrapper {
    #[allow(dead_code)]
    description: Option<String>,
    entries: Vec<VerbPosProfile>,
}

/// JSON wrapper for noun profile entries.
#[derive(Debug, Deserialize)]
struct NounProfileWrapper {
    #[allow(dead_code)]
    description: Option<String>,
    entries: Vec<NounPosProfile>,
}

// ──────────────────────────────────────────────
//  Tests
// ──────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Find the knowledge directory relative to the project root.
    fn kb_dir() -> PathBuf {
        // The knowledge directory is at the workspace root.
        // From the crate, we navigate up: agos-kb/ → project root/
        let crate_dir = std::env::var("CARGO_MANIFEST_DIR")
            .unwrap_or_else(|_| ".".to_string());
        let mut path = PathBuf::from(crate_dir);
        path.push("..");
        path.push("knowledge");
        path.push("KB-0004");
        path
    }

    #[test]
    fn test_load_seed_data() {
        let kb = Kb0004::load_from_directory(&kb_dir())
            .expect("Should load KB-0004 seed data from knowledge/KB-0004/");

        assert!(kb.stem_override_count() > 0,
            "Should have loaded stem overrides (got {})", kb.stem_override_count());
        assert!(kb.verb_profile_count() > 0,
            "Should have loaded verb profiles");
        assert!(kb.noun_profile_count() > 0,
            "Should have loaded noun profiles");

        // Verify counts are in expected range
        let count = kb.stem_override_count();
        assert!(count >= 400 && count <= 500,
            "Stem override count {} out of expected range [400, 500]", count);
        assert!(kb.verb_profile_count() >= 25,
            "Should have at least 25 verb profiles (forms I–XV with root variants, got {})", kb.verb_profile_count());
        assert!(kb.noun_profile_count() >= 50,
            "Should have at least 50 noun profiles (masdars, participles, broken plurals, etc., got {})", kb.noun_profile_count());
    }

    #[test]
    fn test_stem_override_noun() {
        let kb = Kb0004::load_from_directory(&kb_dir())
            .expect("Should load KB-0004");

        // Known noun: رجل
        let result = kb.stem_pos_override("رجل");
        assert!(result.is_some(), "رجل should have a stem override");
        let entry = result.unwrap();
        assert_eq!(entry.pos, PartOfSpeech::Noun, "رجل should be Noun");
        assert!(entry.confidence > 0.8, "رجل should have high confidence");
    }

    #[test]
    fn test_stem_override_verb() {
        let kb = Kb0004::load_from_directory(&kb_dir())
            .expect("Should load KB-0004");

        // Known verb: كتب
        let result = kb.stem_pos_override("كتب");
        assert!(result.is_some(), "كتب should have a stem override");
        let entry = result.unwrap();
        assert_eq!(entry.pos, PartOfSpeech::Verb, "كتب should be Verb");
        assert!(entry.confidence >= 0.85, "كتب should have high confidence");
    }

    #[test]
    fn test_stem_override_ambiguous() {
        let kb = Kb0004::load_from_directory(&kb_dir())
            .expect("Should load KB-0004");

        // Ambiguous: علم (knowledge/he knew)
        let result = kb.stem_pos_override("علم");
        assert!(result.is_some(), "علم should have a stem override");
        let entry = result.unwrap();
        assert_eq!(entry.pos, PartOfSpeech::Noun, "علم should default to Noun");
        assert!(entry.confidence < 0.8, "علم should have lower confidence due to ambiguity");
        assert!(entry.secondary_pos == Some(PartOfSpeech::Verb),
            "علم should have Verb as secondary_pos");
    }

    #[test]
    fn test_unknown_stem_no_override() {
        let kb = Kb0004::load_from_directory(&kb_dir())
            .expect("Should load KB-0004");

        // Unknown stems should return None
        assert!(kb.stem_pos_override("عكف").is_none(),
            "Unknown stem عكف should have no override");
        assert!(kb.stem_pos_override("زحف").is_none(),
            "Unknown stem زحف should have no override");
    }

    #[test]
    fn test_is_primarily_noun() {
        let kb = Kb0004::load_from_directory(&kb_dir())
            .expect("Should load KB-0004");

        let noun_conf = kb.is_primarily_noun("رجل");
        assert!(noun_conf.is_some(), "رجل should be primarily noun");
        assert!(noun_conf.unwrap() > 0.8, "رجل should have high noun confidence");

        // كتب is Verb, not Noun
        let noun_conf2 = kb.is_primarily_noun("كتب");
        assert!(noun_conf2.is_none(), "كتب should NOT be primarily noun");
    }

    #[test]
    fn test_is_primarily_verb() {
        let kb = Kb0004::load_from_directory(&kb_dir())
            .expect("Should load KB-0004");

        let verb_conf = kb.is_primarily_verb("كتب");
        assert!(verb_conf.is_some(), "كتب should be primarily verb");
        assert!(verb_conf.unwrap() >= 0.85, "كتب should have high verb confidence");

        // رجل is Noun, not Verb
        let verb_conf2 = kb.is_primarily_verb("رجل");
        assert!(verb_conf2.is_none(), "رجل should NOT be primarily verb");
    }

    #[test]
    fn test_verb_profiles() {
        let kb = Kb0004::load_from_directory(&kb_dir())
            .expect("Should load KB-0004");

        // Form I profiles (6 variants: basic sound, hollow, defective,
        // imperfect prefix, imperative prefix, doubled)
        let form1_profiles = kb.verb_profiles_for_form(1);
        assert_eq!(form1_profiles.len(), 6, "Form I should have 6 variants (sound, hollow, defective, prefix variants, doubled)");
        for &p in &form1_profiles {
            assert_eq!(p.default_pos, PartOfSpeech::Verb,
                "Form I should always be Verb");
        }
        // Basic variant should have confidence 0.30
        let basic = form1_profiles.iter().find(|p| p.prefix_pattern.is_none());
        assert!(basic.is_some(), "Form I should have a basic variant with no prefix");
        assert!((basic.unwrap().default_confidence - 0.30).abs() < 0.01,
            "Form I basic default confidence should be 0.30");

        // Form I verb_confidence should return the highest (0.30)
        assert!((kb.verb_confidence(1) - 0.30).abs() < 0.01,
            "Form I verb_confidence should be 0.30 (highest variant)");

        // Form X exists
        let form10_profiles = kb.verb_profiles_for_form(10);
        assert!(!form10_profiles.is_empty(), "Form X profile should exist");
        assert!((form10_profiles[0].default_confidence - 0.10).abs() < 0.01,
            "Form X default confidence should be 0.10");
    }

    #[test]
    fn test_noun_profiles() {
        let kb = Kb0004::load_from_directory(&kb_dir())
            .expect("Should load KB-0004");

        // فَاعِل profile
        let fail = kb.noun_profile("فَاعِل");
        assert!(fail.is_some(), "فَاعِل profile should exist");
        let p = fail.unwrap();
        assert_eq!(p.default_pos, PartOfSpeech::Noun);
        assert!((p.default_confidence - 0.20).abs() < 0.01,
            "فَاعِل confidence should be 0.20");

        // فَعِيل should be Adjective
        let failure = kb.noun_profile("فَعِيل");
        assert!(failure.is_some(), "فَعِيل profile should exist");
        assert_eq!(failure.unwrap().default_pos, PartOfSpeech::Adjective,
            "فَعِيل should be Adjective");
    }

    #[test]
    fn test_noun_outranks_verb() {
        let kb = Kb0004::load_from_directory(&kb_dir())
            .expect("Should load KB-0004");

        // فَعْل (0.25) vs Form I (max 0.30) — verb wins
        let fail_conf = kb.noun_confidence("فَعْل");
        let verb_conf = kb.verb_confidence(1);
        assert!(fail_conf < verb_conf,
            "فَعْل ({}) should not outrank Form I max ({})", fail_conf, verb_conf);
        assert!(!kb.noun_outranks_verb("فَعْل", 1),
            "فَعْل ({}) should not outrank Form I ({})", fail_conf, verb_conf);

        // فَعْلَة (0.30) vs Form I (max 0.30) — equal
        let faila_conf = kb.noun_confidence("فَعْلَة");
        assert!((faila_conf - verb_conf).abs() < 0.01,
            "فَعْلَة confidence ({}) should equal Form I ({})", faila_conf, verb_conf);
        assert!(!kb.noun_outranks_verb("فَعْلَة", 1),
            "فَعْلَة ({}) should NOT outrank Form I ({}) as they are equal", faila_conf, verb_conf);
    }

    #[test]
    fn test_best_pos_for_3l_stem() {
        let kb = Kb0004::load_from_directory(&kb_dir())
            .expect("Should load KB-0004");

        let (pos, conf, source) = kb.best_pos_for_3l_stem("رجل").unwrap();
        assert_eq!(pos, PartOfSpeech::Noun, "رجل best POS should be Noun");
        assert!(conf > 0.8, "رجل confidence should be high");
        assert!(source.contains("KB-0004"), "Source should reference KB-0004");

        let (pos, conf, _) = kb.best_pos_for_3l_stem("كتب").unwrap();
        assert_eq!(pos, PartOfSpeech::Verb, "كتب best POS should be Verb");
        assert!(conf >= 0.85, "كتب confidence should be high");
    }
}
