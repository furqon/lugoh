//! # AGOS Syntax Engine — Arabic Syntactic (Nahw) Parsing
//!
//! This crate implements MOD-05 of the AGOS compilation pipeline:
//! syntactic parsing with constituency trees, i'rab role assignment,
//! and sentence type classification.
//!
//! ## Pipeline Interface
//!
//! ```text
//! Input:  MorphologicalAnalysis (IR-4)
//! Output: SyntaxTree (IR-5)
//! ```
//!
//! ## Architecture (SPEC-0001-C3 §6)
//!
//! 1. **Sentence Segmentation** — Group tokens into sentence boundaries
//! 2. **Sentence Type Identification** — Verbal (fi'liyyah) vs Nominal (ismiyyah)
//! 3. **Verbal Sentence Parsing** — Fi'l → Fa'il → Maf'ul
//! 4. **Nominal Sentence Parsing** — Mubtada' → Khabar
//! 5. **Construction Detection** — Idafa, Na'at, Harf Jarr
//! 6. **Ambiguity Handling** — Multiple parse trees
//!
//! ## Spec Alignment
//!
//! - SPEC-0001-C3 §6: SyntaxParser — full stage specification
//! - SPEC-0001-C5 §6: IR-5 (SyntaxTree) data schema
//! - SPEC-0001-C4 §7: Module Responsibilities — SyntaxParser

pub mod config;
pub mod error;
pub mod syntax_parser;

pub use config::*;
pub use error::*;
pub use syntax_parser::*;
