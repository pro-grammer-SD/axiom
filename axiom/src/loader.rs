/// Axiom Module Loader
///
/// Handles loading of local `.ax` modules.
/// Standard library dynamic loading has been removed.

use std::path::PathBuf;

/// Resolve the path to a local module file.
///
/// # Arguments
/// * `name` - Module name (e.g., "mymod" → resolves to "./mymod.ax")
pub fn resolve_module_path(name: &str) -> PathBuf {
    PathBuf::from(format!("{}.ax", name))
}

/// Load a local module by name.
///
/// # Arguments
/// * `name` - Module name (file in current directory, no extension)
///
/// # Returns
/// * `Result<String, String>` - Module source code or error message
pub fn load_local_module(name: &str) -> Result<String, String> {
    let path = resolve_module_path(name);

    if path.exists() {
        std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read module '{}': {}", name, e))
    } else {
        Err(format!(
            "Module '{}' not found at '{}'",
            name,
            path.display()
        ))
    }
}
