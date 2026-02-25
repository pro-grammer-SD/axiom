/// Axiom Module Loader (.rax System)
///
/// DEPRECATED: This system has been replaced by static IntrinsicRegistry.
/// Modules are now compiled directly into the binary as Rust intrinsics.
///
/// This file is retained for reference but is no longer used.

use std::path::PathBuf;
use std::collections::HashMap;
use std::fs;
use parking_lot::RwLock;

/// Module loader - now deprecated, use IntrinsicRegistry instead
pub struct ModuleLoader {
    /// Cache of loaded modules - empty, unused
    loaded: RwLock<HashMap<String, String>>,

    /// Path to the module directory - kept for compatibility
    lib_dir: PathBuf,
}

impl ModuleLoader {
    /// Create a new module loader - DEPRECATED
    pub fn new(lib_dir: PathBuf) -> Self {
        ModuleLoader {
            loaded: RwLock::new(HashMap::new()),
            lib_dir,
        }
    }

    /// Create with default directory (~/.axiom/lib) - DEPRECATED
    pub fn with_default_path() -> Self {
        let lib_dir = Self::get_default_lib_dir();
        fs::create_dir_all(&lib_dir).ok();
        ModuleLoader {
            loaded: RwLock::new(HashMap::new()),
            lib_dir,
        }
    }

    /// Get the default library directory
    fn get_default_lib_dir() -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                PathBuf::from(local_app_data)
                    .join("axiom")
                    .join("lib")
            } else if let Ok(user_profile) = std::env::var("USERPROFILE") {
                PathBuf::from(user_profile)
                    .join(".axiom")
                    .join("lib")
            } else {
                PathBuf::from(".axiom").join("lib")
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            if let Ok(home) = std::env::var("HOME") {
                PathBuf::from(home).join(".axiom").join("lib")
            } else {
                PathBuf::from(".axiom").join("lib")
            }
        }
    }

    /// Load a module - DEPRECATED, returns error
    pub fn load(&self, name: &str) -> Result<String, String> {
        Err(format!(
            "Dynamic module loading (.rax) is deprecated. Module '{}' is now a static intrinsic in the axm binary.",
            name
        ))
    }

    /// Load all standard library modules - DEPRECATED
    pub fn load_all_stdlib(&self) -> Result<HashMap<String, String>, Vec<String>> {
        Err(vec![
            "Dynamic module loading (.rax) is deprecated. All 22 standard library modules are now static intrinsics.".to_string()
        ])
    }
}

