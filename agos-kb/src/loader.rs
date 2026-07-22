//! # Knowledge Base Loader
//!
//! Default implementation of `KbLoader` that loads compiled `.agos-kb` files
//! from a directory. In production, this SHOULD use memory-mapped I/O for
//! the sub-microsecond lookup targets.
//!
//! ## Loading Sequence
//!
//! Per KB-OVERVIEW §5.1, KBs should be loaded in this order:
//! 1. KB-0005 (Particles) — fast-path, no dependencies
//! 2. KB-0006 (Pronouns) — fast-path, no dependencies
//! 3. KB-0007 (Features) — cross-cutting taxonomy
//! 4. KB-0001 (Roots) — foundation
//! 5. KB-0002 (Wazan) — patterns
//! 6. KB-0003 (Verb Forms) — verb paradigms
//! 7. KB-0004 (Noun Patterns) — noun patterns
//!
//! ## Spec Alignment
//!
//! - KB-OVERVIEW §5.1: KB Loading Sequence
//! - SPEC-0001-C4 §10.5: KB loading with O(1) and mmap
//! - SPEC-0103: Performance Optimization Guide (mmap, fast loading)

use std::path::{Path, PathBuf};

use crate::error::{KbError, KbResult};
use crate::traits::{KbLoader, KbSuite};
use crate::types::{
    FeatureDatabase, KbId, KbMetadata, KbStore, KnowledgeBase, NounPatternEntry, ParticleEntry,
    PronounEntry, RootEntry, VerbParadigm, WazanEntry,
};

/// Default loader for compiled AGOS knowledge base files.
///
/// Loads `.agos-kb` files from a specified directory. Each file is a
/// compiled binary containing serialized KB entries and a metadata header.
pub struct DefaultKbLoader {
    /// Base directory for KB files
    kb_directory: PathBuf,
}

impl DefaultKbLoader {
    /// Create a new loader that looks for KB files in the given directory.
    pub fn new(kb_directory: impl Into<PathBuf>) -> Self {
        Self {
            kb_directory: kb_directory.into(),
        }
    }

    /// Get the expected file path for a compiled KB.
    fn kb_file_path(&self, kb_id: KbId) -> PathBuf {
        let filename = format!("{}.{}", kb_id.file_name(), kb_id.extension());
        self.kb_directory.join(filename)
    }

    /// The recommended loading order per KB-OVERVIEW §5.1.
    fn loading_order() -> Vec<KbId> {
        vec![
            KbId::Kb0005, // Particles (smallest, fastest, no deps)
            KbId::Kb0006, // Pronouns (small, fast, no deps)
            KbId::Kb0007, // Features (cross-cutting)
            KbId::Kb0001, // Roots (foundation)
            KbId::Kb0002, // Wazan (patterns)
            KbId::Kb0003, // Verb Forms (verb paradigms)
            KbId::Kb0004, // Noun Patterns (noun paradigms)
            KbId::Kb0008, // Particles Developer Reference
        ]
    }
}

impl KbLoader for DefaultKbLoader {
    fn load_suite(&self, kb_directory: &Path) -> KbResult<KbSuite> {
        let mut suite = KbSuite::empty();

        for kb_id in Self::loading_order() {
            let path = if kb_directory == self.kb_directory {
                self.kb_file_path(kb_id)
            } else {
                let filename = format!("{}.{}", kb_id.file_name(), kb_id.extension());
                kb_directory.join(filename)
            };

            if path.exists() {
                let kb = self.load_kb(kb_id, &path)?;
                suite.register(kb);
            }
            // Non-fatal: missing KBs are handled gracefully at lookup time
        }

        Ok(suite)
    }

    fn load_kb(&self, kb_id: KbId, path: &Path) -> KbResult<KnowledgeBase> {
        if !path.exists() {
            return Err(KbError::NotFound(format!(
                "{}: {}",
                kb_id.as_str(),
                path.display()
            )));
        }

        // Read the file bytes
        let bytes = std::fs::read(path)
            .map_err(|e| KbError::IoError(format!("Cannot read {}: {}", path.display(), e)))?;

        // In production, this would:
        // 1. Parse the binary header/metadata
        // 2. Memory-map the data section
        // 3. Zero-copy deserialize entries
        //
        // For now, we deserialize from JSON (the development format)
        let kb = deserialize_kb_from_json(kb_id, &bytes)?;

        Ok(kb)
    }

    fn verify_kb_file(&self, path: &Path) -> KbResult<KbMetadata> {
        if !path.exists() {
            return Err(KbError::NotFound(path.display().to_string()));
        }

        let bytes = std::fs::read(path)
            .map_err(|e| KbError::IoError(format!("Cannot read {}: {}", path.display(), e)))?;

        // For now, try to parse metadata from JSON
        serde_json::from_slice::<KbMetadata>(&bytes)
            .map_err(|e| KbError::FormatError(format!("Invalid metadata: {e}")))
    }
}

/// Deserialize a knowledge base from JSON bytes.
///
/// This is the development format. Production will use a compact binary
/// format with mmap-based zero-copy loading (KB-OVERVIEW §4.1).
fn deserialize_kb_from_json(kb_id: KbId, bytes: &[u8]) -> KbResult<KnowledgeBase> {
    match kb_id {
        KbId::Kb0001 => {
            let store: KbStore<RootEntry> = serde_json::from_slice(bytes)
                .map_err(|e| KbError::DeserializationError(format!("Roots: {e}")))?;
            Ok(KnowledgeBase::Roots(store))
        }
        KbId::Kb0002 => {
            let store: KbStore<WazanEntry> = serde_json::from_slice(bytes)
                .map_err(|e| KbError::DeserializationError(format!("Wazan: {e}")))?;
            Ok(KnowledgeBase::Wazan(store))
        }
        KbId::Kb0003 => {
            let store: KbStore<VerbParadigm> = serde_json::from_slice(bytes)
                .map_err(|e| KbError::DeserializationError(format!("VerbForms: {e}")))?;
            Ok(KnowledgeBase::VerbForms(store))
        }
        KbId::Kb0004 => {
            let store: KbStore<NounPatternEntry> = serde_json::from_slice(bytes)
                .map_err(|e| KbError::DeserializationError(format!("NounPatterns: {e}")))?;
            Ok(KnowledgeBase::NounPatterns(store))
        }
        KbId::Kb0005 => {
            let store: KbStore<ParticleEntry> = serde_json::from_slice(bytes)
                .map_err(|e| KbError::DeserializationError(format!("Particles: {e}")))?;
            Ok(KnowledgeBase::Particles(store))
        }
        KbId::Kb0006 => {
            let store: KbStore<PronounEntry> = serde_json::from_slice(bytes)
                .map_err(|e| KbError::DeserializationError(format!("Pronouns: {e}")))?;
            Ok(KnowledgeBase::Pronouns(store))
        }
        KbId::Kb0007 => {
            let db: FeatureDatabase = serde_json::from_slice(bytes)
                .map_err(|e| KbError::DeserializationError(format!("Features: {e}")))?;
            Ok(KnowledgeBase::Features(db))
        }
        KbId::Kb0008 => {
            // KB-0008 is a compiled particle module derived from KB-0005
            let store: KbStore<ParticleEntry> = serde_json::from_slice(bytes)
                .map_err(|e| KbError::DeserializationError(format!("ParticlesDev: {e}")))?;
            Ok(KnowledgeBase::Particles(store))
        }
    }
}
