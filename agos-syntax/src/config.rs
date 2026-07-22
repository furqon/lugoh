//! # MOD-05 Configuration
//!
//! Configuration for the SyntaxParser stage.
//!
//! ## Spec Alignment
//!
//! - SPEC-0001-C3 §6.2.1: Input Schema (config fields)

use agos_core::types::GrammarSchool;

/// Configuration for MOD-05: SyntaxParser (SPEC-0001-C3 §6.2.1).
#[derive(Debug, Clone)]
pub struct SyntaxParserConfig {
    /// Grammar school for syntactic analysis
    pub school: GrammarSchool,
    /// Maximum parse trees in the ambiguity forest (default: 8)
    pub max_parse_trees: usize,
    /// Maximum tokens for full sentence parsing (default: 200)
    pub max_sentence_length: usize,
    /// Enable partial parse fallback when full parse fails (default: true)
    pub enable_partial_parse: bool,
}

impl Default for SyntaxParserConfig {
    fn default() -> Self {
        Self {
            school: GrammarSchool::Basra,
            max_parse_trees: 8,
            max_sentence_length: 200,
            enable_partial_parse: true,
        }
    }
}
