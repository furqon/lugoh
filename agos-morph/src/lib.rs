//! # AGOS Morphology Engine — Arabic Morphological & Syntactic Analysis
//!
//! This crate implements the front-end stages of the AGOS compilation pipeline:
//!
//! | Module | Stage | Description |
//! |--------|-------|-------------|
//! | MOD-01 | UnicodeValidator | Arabic text validation and normalization |
//! | MOD-02 | Lexer | Raw tokenization of normalized text |
//! | MOD-03 | Tokenizer | Morpheme segmentation (clitic identification) |
//! | MOD-04 | MorphologicalParser | Root extraction, wazan matching, feature extraction |
//! | MOD-05 | SyntaxParser | Syntactic parsing (nahw), constituency trees |
//!
//! ## Spec Alignment
//!
//! - SPEC-0001-C3: Compilation Pipeline (stage-by-stage descriptions)
//! - SPEC-0101: Morphology Engine — detailed implementation spec
//! - KB-0001 through KB-0007: Knowledge bases consumed by morphology stages

pub mod config;
pub mod error;
pub mod lexer;
pub mod morphological_parser;
pub mod tokenizer;
pub mod unicode_validator;

pub use config::*;
pub use error::*;
pub use lexer::*;
pub use morphological_parser::*;
pub use tokenizer::*;
pub use unicode_validator::*;
