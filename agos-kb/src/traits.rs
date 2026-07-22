//! # Knowledge Base Traits
//!
//! Defines the core traits for loading, compiling, and reading knowledge bases.
//! These traits are the interface between the KB layer and the pipeline modules
//! that consume KB data.
//!
//! ## Spec Alignment
//!
//! - KB-OVERVIEW §5: Pipeline Integration (KB loading sequence)
//! - SPEC-0001-C4 §10: MOD-08 KnowledgeGraphResolver Interface
//! - SPEC-0601: Plugin System (custom KB plugins)
//! - ADR-0004: Offline-First Architecture (mmap-based loading)

use std::path::Path;

use agos_core::version::SemVer;

use crate::error::KbResult;
use crate::types::{
    FeatureDatabase, KbId, KbMetadata, KbStore, KnowledgeBase, NounPatternEntry, ParticleEntry,
    PronounEntry, RootEntry, VerbParadigm, WazanEntry,
};

/// Common trait for loading a KB from its compiled binary format.
///
/// Implementations SHOULD use memory-mapped I/O for production use
/// to achieve the sub-microsecond lookup targets specified in the KB
/// performance budgets.
pub trait KbLoader: Send + Sync {
    /// Load a complete KB suite from a directory of compiled `.agos-kb` files.
    fn load_suite(&self, kb_directory: &Path) -> KbResult<KbSuite>;

    /// Load a single KB by its ID from a compiled file.
    fn load_kb(&self, kb_id: KbId, path: &Path) -> KbResult<KnowledgeBase>;

    /// Verify that a compiled KB file has the correct format and checksum.
    fn verify_kb_file(&self, path: &Path) -> KbResult<KbMetadata>;
}

/// Trait for reading from loaded knowledge bases at runtime.
///
/// All methods have O(1) or O(log n) lookup targets as specified in
/// the KB performance budgets (KB-OVERVIEW §4.2).
pub trait KbReader: Send + Sync {
    /// Look up a root by its consonantal skeleton (KB-0001).
    /// Target: < 1 μs.
    fn lookup_root(&self, root_text: &str) -> Option<&RootEntry>;

    /// Look up a wazan pattern by its text or signature (KB-0002).
    /// Target: < 500 ns.
    fn lookup_wazan(&self, pattern: &str) -> Option<&WazanEntry>;

    /// Look up a verb paradigm by form + class (KB-0003).
    /// Target: < 1 μs.
    fn lookup_verb_paradigm(&self, form: u8, class: &str) -> Option<&VerbParadigm>;

    /// Look up a noun pattern by its type (KB-0004).
    /// Target: < 2 μs.
    fn lookup_noun_pattern(&self, noun_type: &str) -> Option<&NounPatternEntry>;

    /// Look up a particle by its text (KB-0005 — fast path).
    /// Target: < 500 ns.
    fn lookup_particle(&self, particle_text: &str) -> Option<&ParticleEntry>;

    /// Look up a pronoun by its text (KB-0006 — fast path).
    /// Target: < 500 ns.
    fn lookup_pronoun(&self, pronoun_text: &str) -> Option<&PronounEntry>;

    /// Check whether the given KB is loaded and ready.
    fn is_kb_loaded(&self, kb_id: KbId) -> bool;

    /// Get the version of a loaded KB.
    fn kb_version(&self, kb_id: KbId) -> Option<SemVer>;

    /// Get the KB IDs of all loaded knowledge bases.
    fn loaded_kbs(&self) -> Vec<KbId>;
}

/// A fully loaded suite of knowledge bases.
///
/// This is the primary entry point for pipeline modules that need
/// access to linguistic data. The suite is loaded once at startup
/// and shared across all analysis requests.
#[derive(Debug, Clone)]
pub struct KbSuite {
    /// Loaded KB entries
    roots: Option<KbStore<RootEntry>>,
    wazan: Option<KbStore<WazanEntry>>,
    verb_paradigms: Option<KbStore<VerbParadigm>>,
    noun_patterns: Option<KbStore<NounPatternEntry>>,
    particles: Option<KbStore<ParticleEntry>>,
    pronouns: Option<KbStore<PronounEntry>>,

    /// KB-0007: Morphological Features database
    features: Option<FeatureDatabase>,

    /// Version map of all loaded KBs
    versions: std::collections::HashMap<KbId, SemVer>,
}

impl KbSuite {
    /// Create an empty KB suite.
    pub fn empty() -> Self {
        Self {
            roots: None,
            wazan: None,
            verb_paradigms: None,
            noun_patterns: None,
            particles: None,
            pronouns: None,
            features: None,
            versions: std::collections::HashMap::new(),
        }
    }

    /// Register a loaded KB into the suite.
    pub fn register(&mut self, kb: KnowledgeBase) {
        match kb {
            KnowledgeBase::Roots(store) => {
                self.versions.insert(KbId::Kb0001, store.metadata.version.clone());
                self.roots = Some(store);
            }
            KnowledgeBase::Wazan(store) => {
                self.versions.insert(KbId::Kb0002, store.metadata.version.clone());
                self.wazan = Some(store);
            }
            KnowledgeBase::VerbForms(store) => {
                self.versions.insert(KbId::Kb0003, store.metadata.version.clone());
                self.verb_paradigms = Some(store);
            }
            KnowledgeBase::NounPatterns(store) => {
                self.versions.insert(KbId::Kb0004, store.metadata.version.clone());
                self.noun_patterns = Some(store);
            }
            KnowledgeBase::Particles(store) => {
                self.versions.insert(KbId::Kb0005, store.metadata.version.clone());
                self.particles = Some(store);
            }
            KnowledgeBase::Pronouns(store) => {
                self.versions.insert(KbId::Kb0006, store.metadata.version.clone());
                self.pronouns = Some(store);
            }
            KnowledgeBase::Features(db) => {
                self.versions.insert(KbId::Kb0007, SemVer::new(1, 0, 0));
                self.features = Some(db);
            }
        }
    }

    /// Get the roots KB store.
    pub fn roots(&self) -> Option<&KbStore<RootEntry>> {
        self.roots.as_ref()
    }

    /// Get the wazan KB store.
    pub fn wazan(&self) -> Option<&KbStore<WazanEntry>> {
        self.wazan.as_ref()
    }

    /// Get the verb paradigms KB store.
    pub fn verb_paradigms(&self) -> Option<&KbStore<VerbParadigm>> {
        self.verb_paradigms.as_ref()
    }

    /// Get the noun patterns KB store.
    pub fn noun_patterns(&self) -> Option<&KbStore<NounPatternEntry>> {
        self.noun_patterns.as_ref()
    }

    /// Get the particles KB store.
    pub fn particles(&self) -> Option<&KbStore<ParticleEntry>> {
        self.particles.as_ref()
    }

    /// Get the pronouns KB store.
    pub fn pronouns(&self) -> Option<&KbStore<PronounEntry>> {
        self.pronouns.as_ref()
    }

    /// Get the feature database (KB-0007).
    pub fn features(&self) -> Option<&FeatureDatabase> {
        self.features.as_ref()
    }

    /// Get the versions of all loaded KBs.
    pub fn versions(&self) -> &std::collections::HashMap<KbId, SemVer> {
        &self.versions
    }

    /// Check if all required core KBs (KB-0001 through KB-0007) are loaded.
    pub fn is_complete(&self) -> bool {
        self.roots.is_some()
            && self.wazan.is_some()
            && self.verb_paradigms.is_some()
            && self.noun_patterns.is_some()
            && self.particles.is_some()
            && self.pronouns.is_some()
            && self.features.is_some()
    }

    /// Get a knowledge version map suitable for use in pipeline IRs.
    pub fn knowledge_version_map(&self) -> agos_core::version::KnowledgeVersionMap {
        let mut map = std::collections::HashMap::new();
        for (kb_id, version) in &self.versions {
            map.insert(kb_id.as_str().to_string(), version.to_string());
        }
        map
    }
}

// Implement KbReader for KbSuite

impl KbReader for KbSuite {
    fn lookup_root(&self, root_text: &str) -> Option<&RootEntry> {
        let store = self.roots.as_ref()?;
        let indices = store.lookup(root_text)?;
        let idx = indices.first()?;
        store.get(*idx)
    }

    fn lookup_wazan(&self, pattern: &str) -> Option<&WazanEntry> {
        let store = self.wazan.as_ref()?;
        let indices = store.lookup(pattern)?;
        let idx = indices.first()?;
        store.get(*idx)
    }

    fn lookup_verb_paradigm(&self, _form: u8, _class: &str) -> Option<&VerbParadigm> {
        let store = self.verb_paradigms.as_ref()?;
        // Compound key lookup: form + class
        let key = format!("{}-{}", _form, _class);
        let indices = store.lookup(&key)?;
        let idx = indices.first()?;
        store.get(*idx)
    }

    fn lookup_noun_pattern(&self, noun_type: &str) -> Option<&NounPatternEntry> {
        let store = self.noun_patterns.as_ref()?;
        let indices = store.lookup(noun_type)?;
        let idx = indices.first()?;
        store.get(*idx)
    }

    fn lookup_particle(&self, particle_text: &str) -> Option<&ParticleEntry> {
        let store = self.particles.as_ref()?;
        let indices = store.lookup(particle_text)?;
        let idx = indices.first()?;
        store.get(*idx)
    }

    fn lookup_pronoun(&self, pronoun_text: &str) -> Option<&PronounEntry> {
        let store = self.pronouns.as_ref()?;
        let indices = store.lookup(pronoun_text)?;
        let idx = indices.first()?;
        store.get(*idx)
    }

    fn is_kb_loaded(&self, kb_id: KbId) -> bool {
        self.versions.contains_key(&kb_id)
    }

    fn kb_version(&self, kb_id: KbId) -> Option<SemVer> {
        self.versions.get(&kb_id).cloned()
    }

    fn loaded_kbs(&self) -> Vec<KbId> {
        self.versions.keys().copied().collect()
    }
}
