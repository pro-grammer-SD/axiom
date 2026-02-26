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
#[allow(unused_imports)] use std::env;

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
    /// Determine the default library directory in a fully cross-platform way.
    ///
    /// This deliberately avoids `#[cfg(target_os = "windows")]` because that
    /// cfg is false inside MSYS2 / Cygwin / Git-for-Windows shells even when
    /// the binary is a native Windows PE executable.  Instead we probe
    /// environment variables in priority order, exactly like `AxConf::resolve_home`.
    fn get_default_lib_dir() -> PathBuf {
        // 1. LOCALAPPDATA  (C:\Users\ADMIN\AppData\Local  — preferred on Windows)
        if let Ok(v) = std::env::var("LOCALAPPDATA") {
            if !v.is_empty() {
                return PathBuf::from(v).join("axiom").join("lib");
            }
        }

        // 2. USERPROFILE   (C:\Users\ADMIN — standard Windows)
        if let Ok(v) = std::env::var("USERPROFILE") {
            if !v.is_empty() {
                return PathBuf::from(v).join(".axiom").join("lib");
            }
        }

        // 3. HOMEDRIVE + HOMEPATH  (legacy Windows, e.g. C: + \Users\ADMIN)
        if let (Ok(drive), Ok(path)) = (std::env::var("HOMEDRIVE"), std::env::var("HOMEPATH")) {
            if !drive.is_empty() && !path.is_empty() {
                return PathBuf::from(format!("{}{}", drive, path)).join(".axiom").join("lib");
            }
        }

        // 4. HOME  (Unix, macOS, MSYS2, Cygwin, WSL, Git Bash)
        if let Ok(v) = std::env::var("HOME") {
            if !v.is_empty() {
                return PathBuf::from(v).join(".axiom").join("lib");
            }
        }

        // 5. dirs crate fallback
        if let Some(h) = dirs::home_dir() {
            return h.join(".axiom").join("lib");
        }

        // 6. Absolute last resort — relative to cwd
        PathBuf::from(".axiom").join("lib")
    }

    /// Load a module - DEPRECATED, returns error
    pub fn load(&self, name: &str) -> Result<String, String> {
        Err(format!(
            "Dynamic module loading (.rax) is deprecated. Module '{}' is now a static intrinsic in the axiom binary.",
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

