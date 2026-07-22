//! # Version Management
//!
//! Types for semantic versioning of the AGOS platform, its modules,
//! intermediate representations, and knowledge bases.
//!
//! ## Spec Alignment
//!
//! - SPEC-0001-C4 §18: Versioning & Compatibility Policy
//! - KB-OVERVIEW §6.2: Cross-KB Version Compatibility Matrix

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Semantic Version (SemVer 2.0.0) for AGOS components.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemVer {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl SemVer {
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self { major, minor, patch }
    }

    /// Parse a version string in the form "MAJOR.MINOR.PATCH".
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return None;
        }
        Some(Self {
            major: parts[0].parse().ok()?,
            minor: parts[1].parse().ok()?,
            patch: parts[2].parse().ok()?,
        })
    }

    /// Check if this version is compatible with a minimum required version.
    pub fn is_compatible_with(&self, minimum: &SemVer) -> bool {
        self.major == minimum.major
            && (self.minor > minimum.minor || (self.minor == minimum.minor && self.patch >= minimum.patch))
    }
}

impl std::fmt::Display for SemVer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// A map of knowledge base IDs to their versions (SPEC-0001-C4 §2.4).
///
/// Used throughout the pipeline to ensure reproducibility by recording which
/// KB versions were used during analysis.
pub type KnowledgeVersionMap = HashMap<String, String>;

/// Version information for a single pipeline module (SPEC-0001-C4 §18.4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleVersion {
    /// Module identifier (e.g., "MOD-04")
    pub module_id: String,

    /// Semantic version of the module implementation
    pub version: SemVer,

    /// API version this module implements
    pub api_version: SemVer,

    /// Versioned dependencies on other modules
    pub dependencies: Vec<ModuleDependency>,
}

/// A versioned dependency on another module (SPEC-0001-C4 §18.4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDependency {
    /// Module ID of the dependency
    pub module_id: String,

    /// Minimum compatible version
    pub min_version: SemVer,
}

/// Version information for the GVM (SPEC-0001-C4 §12.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GVMVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    pub supported_bytecode_versions: Vec<String>,
}
