/// Axiom Build System
///
/// Handles project-level build operations.
/// Standard library bootstrapping has been removed — axiom is self-contained.

use std::path::PathBuf;

/// Return the axiomlibs package directory (~/.axiomlibs).
/// Used by the package manager for third-party packages.
pub fn get_axiomlibs_dir() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".axiomlibs");
    path
}
