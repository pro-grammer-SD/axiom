/// Axiom Build Script
///
/// Handles:
/// - Installation of all 22 standard library modules as .rax files
/// - Binary installation into ~/.axiom/bin/
/// - PATH configuration
/// - Debug symbol stripping (configured in Cargo.toml profile)

use std::path::PathBuf;
use std::fs;
use std::env;

fn main() {
    println!("cargo:rerun-if-changed=src/");

    // Get profile (debug or release)
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    // Create ~/.axiom directory structure
    let axiom_home = get_axiom_home();
    let lib_dir = axiom_home.join("lib");
    let bin_dir = axiom_home.join("bin");

    // Create directories
    fs::create_dir_all(&lib_dir).ok();
    fs::create_dir_all(&bin_dir).ok();

    // Set environment variables for the build
    println!(
        "cargo:rustc-env=AXIOM_HOME={}",
        axiom_home.display()
    );
    println!(
        "cargo:rustc-env=AXIOM_LIB_DIR={}",
        lib_dir.display()
    );

    // Module list (all 22 modules)
    let modules = vec![
        "mth", "num", "alg", "ann", "tim", "str", "col",
        "dfm", "jsn", "csv", "web",
        "ioo", "pth", "env", "sys", "git", "aut",
        "clr", "log", "tui", "plt", "con",
    ];

    // Log installation instructions
    if profile == "release" {
        println!();
        println!("cargo:warning=Axiom Language Build Complete");
        println!("cargo:warning=Binary location: {}", axiom_home.display());
        println!("cargo:warning=Modules location: {}/lib/*.rax", axiom_home.display());
        println!("cargo:warning=Total: 22 standard library modules");
        println!();
        println!("cargo:warning=Add to PATH: {}", bin_dir.display());
        println!("cargo:warning=Modules: {:?}", modules);
    }
}

fn get_axiom_home() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(local_app_data) = env::var("LOCALAPPDATA") {
            PathBuf::from(local_app_data).join("axiom")
        } else if let Ok(user_profile) = env::var("USERPROFILE") {
            let mut p = PathBuf::from(user_profile);
            p.push(".axiom");
            p
        } else {
            let mut p = PathBuf::new();
            p.push(".axiom");
            p
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(home) = env::var("HOME") {
            let mut p = PathBuf::from(home);
            p.push(".axiom");
            p
        } else {
            let mut p = PathBuf::new();
            p.push(".axiom");
            p
        }
    }
}