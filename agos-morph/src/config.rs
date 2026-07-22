//! # Morphology Engine Configuration
//!
//! Defines configuration structs for MOD-01 through MOD-05.
//! Each config extends the shared `StageConfig` with stage-specific fields.
//!
//! ## Spec Alignment
//!
//! - SPEC-0001-C3 §2.2.1: MOD-01 input config schema
//! - SPEC-0101 §3.2: Morph-04/05 configuration models

use agos_core::types::LogLevel;
use agos_core::version::KnowledgeVersionMap;

/// Base configuration applicable to all morphology pipeline stages.
#[derive(Debug, Clone)]
pub struct MorphConfig {
    /// Knowledge base versions available
    pub knowledge_versions: KnowledgeVersionMap,
    /// Logging level
    pub log_level: LogLevel,
}

impl Default for MorphConfig {
    fn default() -> Self {
        Self {
            knowledge_versions: KnowledgeVersionMap::new(),
            log_level: LogLevel::Info,
        }
    }
}

/// Configuration for MOD-01: UnicodeValidator (SPEC-0001-C3 §2.2.1).
///
/// Controls validation behavior: which characters are allowed, how
/// normalization is performed, and size limits.
#[derive(Debug, Clone)]
pub struct UnicodeValidatorConfig {
    /// Whether to strip or canonicalize tashkeel (Arabic diacritics).
    /// If true, diacritical marks are removed from the text.
    pub normalize_tashkeel: bool,

    /// Whether to remove tatweel/kashida (U+0640) characters.
    pub strip_tatweel: bool,

    /// If true, reject any characters outside the Arabic Unicode blocks.
    /// If false, non-Arabic characters are preserved (downstream stages
    /// will handle them as unknown tokens).
    pub strict_arabic_only: bool,

    /// Allowed Unicode ranges as inclusive hex pairs.
    /// Default: ["0600-06FF", "0750-077F", "08A0-08FF"]
    pub allowed_unicode_ranges: Vec<(u32, u32)>,

    /// Maximum input size in bytes (default: 1,048,576 = 1 MiB).
    pub max_input_size: usize,

    /// Additional Unicode ranges to allow beyond the defaults.
    pub extra_allowed_ranges: Vec<(u32, u32)>,
}

impl Default for UnicodeValidatorConfig {
    fn default() -> Self {
        Self {
            normalize_tashkeel: false,
            strip_tatweel: false,
            strict_arabic_only: false,
            allowed_unicode_ranges: vec![
                (0x0600, 0x06FF), // Arabic
                (0x0750, 0x077F), // Arabic Supplement
                (0x08A0, 0x08FF), // Arabic Extended-A
            ],
            max_input_size: 1_048_576, // 1 MiB
            extra_allowed_ranges: vec![],
        }
    }
}

impl UnicodeValidatorConfig {
    /// Create a strict configuration for Arabic-only text with normalization.
    pub fn strict() -> Self {
        Self {
            normalize_tashkeel: true,
            strip_tatweel: true,
            strict_arabic_only: true,
            ..Default::default()
        }
    }

    /// Create a permissive configuration that accepts diverse input.
    pub fn permissive() -> Self {
        Self {
            normalize_tashkeel: false,
            strip_tatweel: false,
            strict_arabic_only: false,
            ..Default::default()
        }
    }

    /// Get the complete set of allowed Unicode ranges, merging defaults
    /// with any extra ranges.
    pub fn allowed_ranges(&self) -> Vec<(u32, u32)> {
        let mut ranges = self.allowed_unicode_ranges.clone();
        ranges.extend_from_slice(&self.extra_allowed_ranges);
        ranges
    }
}
