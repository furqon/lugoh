//! # AGOS Knowledge Base (agos-kb)
//!
//! This crate provides the knowledge base layer for the AGOS platform.
//! It defines the types, traits, and loading infrastructure for all 8
//! linguistic databases (KB-0001 through KB-0008).
//!
//! ## Architecture
//!
//! The KB layer is organized as:
//!
//! - **Types** (`types`): Entry definitions for roots, wazans, verb paradigms,
//!   noun patterns, particles, pronouns, and features
//! - **Traits** (`traits`): `KbLoader` and `KbReader` interfaces
//! - **Loader** (`loader`): Default implementation using file I/O
//! - **Error** (`error`): KB-specific error types
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
//! - KB-OVERVIEW: KB Suite Overview & Architecture
//! - KB-0001 through KB-0008: Individual KB specifications
//! - SPEC-0001-C4: Module Responsibilities & Interfaces
//! - SPEC-0103: Performance Optimization Guide
//! - ADR-0004: Offline-First Architecture

pub mod error;
pub mod kb0004;
pub mod loader;
pub mod traits;
pub mod types;

pub use error::*;
pub use kb0004::*;
pub use loader::*;
pub use traits::*;
pub use types::*;
