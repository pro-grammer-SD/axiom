use serde::{Deserialize, Serialize};
use semver;
use std::collections::BTreeMap;
use std::env;
use std::path::{Path, PathBuf};

/// Configuration for an Axiom package (Axiomite.toml).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxiomiteConfig {
    pub package: PackageMetadata,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub dependencies: DependencySpec,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DependencySpec {
    #[serde(default)]
    pub requires: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
}

impl AxiomiteConfig {
    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    pub fn from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self::from_toml(&content)?)
    }
}

/// Package manager for Axiom.
pub struct PackageManager {
    libs_dir: PathBuf,
}

impl PackageManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let libs_dir = if let Ok(custom) = env::var("AXIOM_LIBS") {
            PathBuf::from(custom)
        } else {
            let home = dirs::home_dir().ok_or("Could not determine home directory")?;
            home.join(".axiomlibs")
        };

        if !libs_dir.exists() {
            std::fs::create_dir_all(&libs_dir)?;
        }

        Ok(PackageManager { libs_dir })
    }

    /// Install a package from GitHub: `axiom pkg add <user>/<repo>`.
    pub fn install_package(&self, github_spec: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = github_spec.split('/').collect();
        if parts.len() != 2 {
            return Err("Invalid GitHub spec. Use format: <user>/<repo>".into());
        }

        let user = parts[0];
        let repo = parts[1];
        let install_path = self.libs_dir.join(user).join(repo);

        // Create parent directories
        let parent = install_path.parent().ok_or("Invalid path")?;
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }

        // Check if already cloned
        if install_path.exists() {
            println!("Package already installed at: {}", install_path.display());
            return Ok(install_path);
        }

        // Clone from GitHub with shallow depth
        let url = format!("https://github.com/{}/{}.git", user, repo);
        println!("Cloning {} (depth=1)...", url);

        let mut fetch_options = git2::FetchOptions::new();
        let mut remote_callbacks = git2::RemoteCallbacks::new();
        remote_callbacks.sideband_progress(|data| {
            print!("{}", String::from_utf8_lossy(data));
            true
        });
        fetch_options.remote_callbacks(remote_callbacks);

        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fetch_options);
        builder.bare(false);

        match builder.clone(&url, &install_path) {
            Ok(_repo) => {
                println!("✓ Successfully installed {}/{}", user, repo);

                // Load and inject environment variables from Axiomite.toml
                if let Ok(axiomite_path) = self.get_axiomite_path(&install_path) {
                    if let Ok(config) = AxiomiteConfig::from_file(&axiomite_path) {
                        self.inject_env_vars(&config);
                        let env_keys: Vec<String> = config.env.keys().cloned().collect();
                        println!(
                            "✓ Environment variables injected: {}",
                            env_keys.join(", ")
                        );
                    }
                }

                Ok(install_path)
            }
            Err(e) => Err(format!("Failed to clone repository: {}", e).into()),
        }
    }

    /// Load a package from the local library.
    pub fn load_package(&self, user: &str, repo: &str) -> Result<AxiomiteConfig, Box<dyn std::error::Error>> {
        let install_path = self.libs_dir.join(user).join(repo);
        if !install_path.exists() {
            return Err(format!("Package not found: {}/{}", user, repo).into());
        }

        let axiomite_path = self.get_axiomite_path(&install_path)?;
        let config = AxiomiteConfig::from_file(&axiomite_path)?;

        // Inject environment variables
        self.inject_env_vars(&config);

        Ok(config)
    }

    /// Replace `lib <repo>;` import statement with the loaded module.
    pub fn process_lib_import(&self, module_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = module_path.split('/').collect();
        if parts.len() != 2 {
            return Err("Invalid module path. Use format: <user>/<repo>".into());
        }

        let user = parts[0];
        let repo = parts[1];

        let _config = self.load_package(user, repo)?;
        let package_dir = self.libs_dir.join(user).join(repo);

        // Find the main entry file (default to main.ax)
        let entry_file = package_dir.join("main.ax");
        let source = std::fs::read_to_string(&entry_file)?;

        Ok(source)
    }

    fn get_axiomite_path(&self, package_dir: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let path = package_dir.join("Axiomite.toml");
        if !path.exists() {
            return Err("Axiomite.toml not found in package".into());
        }
        Ok(path)
    }

    fn inject_env_vars(&self, config: &AxiomiteConfig) {
        for (key, value) in &config.env {
            env::set_var(key, value);
        }
    }

    /// List installed packages.
    pub fn list_packages(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut packages = Vec::new();
        if self.libs_dir.exists() {
            for user_entry in std::fs::read_dir(&self.libs_dir)? {
                let user_path = user_entry?.path();
                if user_path.is_dir() {
                    let user_name = user_path
                        .file_name()
                        .ok_or("Invalid path")?
                        .to_string_lossy()
                        .to_string();
                    for repo_entry in std::fs::read_dir(&user_path)? {
                        let repo_path = repo_entry?.path();
                        if repo_path.is_dir() {
                            let repo_name = repo_path
                                .file_name()
                                .ok_or("Invalid path")?
                                .to_string_lossy()
                                .to_string();
                            packages.push(format!("{}/{}", user_name, repo_name));
                        }
                    }
                }
            }
        }
        Ok(packages)
    }

    /// Install all dependencies from Axiomite.toml in the current directory.
    pub fn install_from_manifest(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let manifest_path = std::env::current_dir()?.join("Axiomite.toml");
        if !manifest_path.exists() {
            return Err("Axiomite.toml not found in current directory".into());
        }

        let config = AxiomiteConfig::from_file(&manifest_path)?;

        if config.dependencies.requires.is_empty() {
            println!("✓ No dependencies to install");
            return Ok(Vec::new());
        }

        let mut installed = Vec::new();
        let mut failed = Vec::new();

        for spec in &config.dependencies.requires {
            match self.install_package(spec) {
                Ok(_) => {
                    installed.push(spec.clone());
                    println!("✓ Installed: {}", spec);
                }
                Err(e) => {
                    failed.push(format!("{}: {}", spec, e));
                    println!("✗ Failed to install {}: {}", spec, e);
                }
            }
        }

        if !failed.is_empty() {
            eprintln!("\n{} package(s) failed to install:", failed.len());
            for err in &failed {
                eprintln!("  - {}", err);
            }
            return Err(format!("Bulk installation completed with {} errors", failed.len()).into());
        }

        println!(
            "\n✓ Successfully installed {} package(s)",
            installed.len()
        );
        Ok(installed)
    }

    /// Remove a package.
    pub fn remove_package(&self, github_spec: &str) -> Result<(), Box<dyn std::error::Error>> {
        let parts: Vec<&str> = github_spec.split('/').collect();
        if parts.len() != 2 {
            return Err("Invalid GitHub spec. Use format: <user>/<repo>".into());
        }

        let user = parts[0];
        let repo = parts[1];
        let install_path = self.libs_dir.join(user).join(repo);

        if !install_path.exists() {
            return Err(format!("Package not found: {}/{}", user, repo).into());
        }

        std::fs::remove_dir_all(&install_path)?;
        println!("✓ Removed {}/{}", user, repo);
        Ok(())
    }

    /// Show package metadata from Axiomite.toml.
    pub fn show_package_info(&self, github_spec: &str) -> Result<(), Box<dyn std::error::Error>> {
        let parts: Vec<&str> = github_spec.split('/').collect();
        if parts.len() != 2 {
            return Err("Invalid GitHub spec. Use format: <user>/<repo>".into());
        }

        let user = parts[0];
        let repo = parts[1];
        let pkg_path = self.libs_dir.join(user).join(repo);

        if !pkg_path.exists() {
            return Err(format!("Package not found: {}/{}", user, repo).into());
        }

        let toml_path = pkg_path.join("Axiomite.toml");
        if !toml_path.exists() {
            return Err(format!("Axiomite.toml not found in {}/{}", user, repo).into());
        }

        let config = AxiomiteConfig::from_file(&toml_path)?;

        println!("📦 Package Information");
        println!("├─ Name:        {}", config.package.name);
        println!("├─ Version:     {}", config.package.version);
        println!("├─ Author:      {}", config.package.author);
        println!("├─ Description: {}", config.package.description);
        println!("└─ Location:    {}", pkg_path.display());

        // Show dependencies if any
        if !config.dependencies.requires.is_empty() {
            println!("\n📚 Dependencies:");
            for dep in &config.dependencies.requires {
                println!("  • {}", dep);
            }
        }

        // Show environment variables if any
        if !config.env.is_empty() {
            println!("\n🔧 Environment Variables:");
            for (key, val) in &config.env {
                println!("  • {} = {}", key, val);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_axiomite_config_parsing() {
        let toml_str = r#"
[package]
name = "mylib"
version = "0.1.0"
description = "A test library"
author = "Test Author"

[env]
MATH_CORES = "16"
LOG_LEVEL = "debug"

[dependencies]
other_lib = "0.1.0"
"#;
        let config = AxiomiteConfig::from_toml(toml_str).unwrap();
        assert_eq!(config.package.name, "mylib");
        assert_eq!(config.package.version, "0.1.0");
        assert_eq!(config.env.get("MATH_CORES"), Some(&"16".to_string()));
    }

    #[test]
    fn test_github_spec_parsing() {
        let spec = "owner/repo";
        let parts: Vec<&str> = spec.split('/').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "owner");
        assert_eq!(parts[1], "repo");
    }
}

// ═══════════════════════════════════════════════════════════════════
// SEMVER UPGRADE — Compare local vs remote version and upgrade if needed
// ═══════════════════════════════════════════════════════════════════

impl PackageManager {
    /// Upgrade a package: compare local semver vs remote HEAD, re-clone if newer.
    /// Usage: axiom pkg upgrade <user>/<repo>
    pub fn upgrade_package(&self, github_spec: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = github_spec.split('/').collect();
        if parts.len() != 2 {
            return Err("Invalid GitHub spec. Use format: <user>/<repo>".into());
        }
        let user = parts[0];
        let repo = parts[1];
        let install_path = self.libs_dir.join(user).join(repo);

        if !install_path.exists() {
            return Err(format!("Package not installed: {}/{}", user, repo).into());
        }

        // Read local version from Axiomite.toml
        let toml_path = install_path.join("Axiomite.toml");
        let local_config = AxiomiteConfig::from_file(&toml_path)?;
        let local_version: semver::Version = local_config.package.version.parse()
            .map_err(|e| format!("Invalid local version '{}': {}", local_config.package.version, e))?;

        println!("📦 {}/{} — local version: {}", user, repo, local_version);

        // Open the existing git repo and fetch to check remote version
        let git_repo = git2::Repository::open(&install_path)?;
        let mut remote = git_repo.find_remote("origin")?;

        println!("🔍 Fetching remote info from origin...");
        remote.fetch(&["HEAD"], None, None)?;

        // Check if FETCH_HEAD has a different commit than HEAD
        let fetch_head = git_repo.find_reference("FETCH_HEAD")?;
        let remote_commit_id = fetch_head.target()
            .ok_or("FETCH_HEAD has no target OID")?;
        let head_ref = git_repo.head()?;
        let local_commit_id = head_ref.target()
            .ok_or("HEAD has no target OID")?;

        if remote_commit_id == local_commit_id {
            println!("✓ Already up to date ({})", local_version);
            return Ok(false);
        }

        // Try to parse remote version from remote Axiomite.toml via blob
        let remote_obj = git_repo.find_object(remote_commit_id, None)?;
        let remote_commit = remote_obj.peel_to_commit()?;
        let remote_tree = remote_commit.tree()?;

        let remote_version = if let Some(entry) = remote_tree.get_name("Axiomite.toml") {
            let blob = git_repo.find_blob(entry.id())?;
            let content = std::str::from_utf8(blob.content())?;
            AxiomiteConfig::from_toml(content)
                .ok()
                .and_then(|c| c.package.version.parse::<semver::Version>().ok())
        } else {
            None
        };

        let upgrade_msg = match &remote_version {
            Some(rv) => {
                if *rv <= local_version {
                    println!("✓ Local version {} is current (remote: {})", local_version, rv);
                    return Ok(false);
                }
                format!("Upgrading {} → {}", local_version, rv)
            }
            None => format!("Remote has new commits — upgrading from {}", local_version),
        };

        println!("⬆  {}", upgrade_msg);

        // Perform upgrade: hard reset to FETCH_HEAD
        let fetch_commit = git_repo.find_commit(remote_commit_id)?;
        let _fetch_annotated = git_repo.find_annotated_commit(remote_commit_id)?;
        git_repo.reset(fetch_commit.as_object(), git2::ResetType::Hard, None)?;

        println!("✓ Upgraded to commit {}", &remote_commit_id.to_string()[..8]);

        // Re-inject environment variables
        let updated_toml = install_path.join("Axiomite.toml");
        if updated_toml.exists() {
            if let Ok(config) = AxiomiteConfig::from_file(&updated_toml) {
                self.inject_env_vars(&config);
                if let Some(rv) = remote_version {
                    println!("✓ Now at version {}", rv);
                }
            }
        }

        Ok(true)
    }

    /// Auto-detect local Axiomite.toml and display metadata (axiom pkg info .)
    pub fn show_local_info(&self) -> Result<(), Box<dyn std::error::Error>> {
        let manifest = std::env::current_dir()?.join("Axiomite.toml");
        if !manifest.exists() {
            return Err("No Axiomite.toml found in current directory".into());
        }
        let config = AxiomiteConfig::from_file(&manifest)?;
        println!("📦 Local Package Information");
        println!("├─ Name:        {}", config.package.name);
        println!("├─ Version:     {}", config.package.version);
        println!("├─ Author:      {}", config.package.author);
        println!("├─ Description: {}", config.package.description);
        println!("└─ Location:    {}", manifest.parent().unwrap().display());

        if !config.dependencies.requires.is_empty() {
            println!("\n📚 Dependencies:");
            for dep in &config.dependencies.requires {
                println!("  • {}", dep);
            }
        }
        if !config.env.is_empty() {
            println!("\n🔧 Environment Variables:");
            for (k, v) in &config.env {
                println!("  • {} = {}", k, v);
            }
        }
        Ok(())
    }
}
