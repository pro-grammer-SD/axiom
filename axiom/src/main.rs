/// Axiom CLI (axiom)
/// Orchestrates run, pkg, fmt, chk, and conf commands.

use axiom::{Parser, Runtime, SemanticAnalyzer, format_source};
use axiom::conf::{cmd_conf_set, cmd_conf_get, cmd_conf_list, cmd_conf_reset, cmd_conf_describe};
use axiom::pkg::PackageManager;
use axiom::errors::DiagnosticLevel;
use clap::{Parser as ClapParser, Subcommand};
use miette::{Result, IntoDiagnostic};
use std::io::Write;
use std::path::PathBuf;

// 64 MB stack — handles deeply-recursive Axiom programs without overflow.
const STACK_SIZE: usize = 64 * 1024 * 1024;

#[derive(ClapParser)]
#[command(
    name = "axiom",
    version = "0.1.0",
    about = "The Axiom Language Toolchain",
    long_about = "axiom — run, check, format, and manage packages for Axiom (.ax) scripts."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute an Axiom script (.ax)
    Run {
        path: PathBuf,
    },
    /// Perform semantic analysis and type checking (does NOT execute)
    Chk {
        path: PathBuf,
    },
    /// Format an Axiom script to standard style
    Fmt {
        path: PathBuf,
        /// Write formatted output back to the file (default: print to stdout)
        #[arg(short, long)]
        write: bool,
    },
    /// Axiomide package manager
    Pkg {
        #[command(subcommand)]
        cmd: PkgCommands,
    },
    /// Manage Axiom runtime configuration (~/.axiom/conf.txt)
    Conf {
        #[command(subcommand)]
        cmd: ConfCommands,
    },
}

#[derive(Subcommand)]
enum ConfCommands {
    /// Set a property: axiom conf set property=value
    Set { spec: String },
    /// Get a property: axiom conf get property
    Get { key: String },
    /// List all properties with current values
    List,
    /// Reset all properties to their defaults
    Reset,
    /// Show detailed documentation for a property
    Describe { key: String },
}

#[derive(Subcommand)]
enum PkgCommands {
    /// Install a package: axiom pkg add <user>/<repo>
    Add { name: String },
    /// Remove a package: axiom pkg remove <user>/<repo>
    Remove { name: String },
    /// Upgrade a package to latest: axiom pkg upgrade <user>/<repo>
    Upgrade { name: String },
    /// List installed packages
    List,
    /// Show package info: axiom pkg info <user>/<repo>  OR  axiom pkg info .
    Info { name: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let result = std::thread::Builder::new()
        .name("axiom-worker".into())
        .stack_size(STACK_SIZE)
        .spawn(move || {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(cli)))
        })
        .expect("failed to spawn axiom worker thread")
        .join();

    match result {
        Ok(Ok(inner)) => inner,
        Ok(Err(panic_payload)) | Err(panic_payload) => {
            let msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                format!("internal error (panic): {}", s)
            } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                format!("internal error (panic): {}", s)
            } else {
                "internal error: unexpected panic in axiom runtime".to_string()
            };
            eprintln!("axiom crashed: {}", msg);
            Err(miette::miette!("{}", msg))
        }
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        // ----------------------------------------------------------------
        // axiom run <file.ax>
        // ----------------------------------------------------------------
        Commands::Run { path } => {
            let source = std::fs::read_to_string(&path)
                .map_err(|e| miette::miette!("Cannot read '{}': {}", path.display(), e))?;

            let mut parser = Parser::new(&source, 0);
            let items = parser.parse()
                .map_err(|e| {
                    use axiom::diagnostics::DiagnosticEngine;
                    let engine = DiagnosticEngine::new(path.display().to_string(), &source);
                    miette::Report::new(engine.from_parser(&e)) // Returns a pretty report
                })?;

            let mut runtime = Runtime::new();
            runtime.run(items)
                .map_err(|e| {
                    use axiom::diagnostics::DiagnosticEngine;
                    let engine = DiagnosticEngine::new(path.display().to_string(), &source);
                    let diag = engine.from_runtime(&e);
                    engine.emit(&diag);
                    miette::miette!("{}", e)
                })?;

            std::io::stdout().flush().into_diagnostic()?;
        }

        // ----------------------------------------------------------------
        // axiom chk <file.ax>
        // ----------------------------------------------------------------
        Commands::Chk { path } => {
            use axiom::diagnostics::{DiagnosticEngine, ErrorCode, AxiomDiagnostic};
            use axiom::errors::RuntimeError;

            // 1. Read source - Fixed the no_source call
            let source = std::fs::read_to_string(&path).map_err(|e| {
                let msg = format!("Cannot read '{}': {}", path.display(), e);
                // no_source belongs to AxiomDiagnostic, not DiagnosticEngine
                miette::Report::new(AxiomDiagnostic::no_source(ErrorCode::IoError, msg))
            })?;

            let engine = DiagnosticEngine::new(path.display().to_string(), &source);

            // 2. Parse - Fixed to use miette::Report
            let mut parser = Parser::new(&source, 0);
            let items = parser.parse().map_err(|e| {
                miette::Report::new(engine.from_parser(&e))
            })?;

            // 3. Semantic Analysis
            let mut chk = SemanticAnalyzer::new();
            let diagnostics = chk.check(&items);

            if diagnostics.is_empty() {
                println!("✓ No issues found in '{}'", path.display());
            } else {
                let mut has_error = false;
                for d in &diagnostics {
                    // FIX: Convert the generic Diagnostic into a RuntimeError 
                    // so from_runtime can handle it, or use a custom converter.
                    let runtime_err = RuntimeError::GenericError { 
                        message: d.message.clone(), 
                        span: d.span 
                    };
                    
                    let axiom_diag = engine.from_runtime(&runtime_err);
                    engine.emit(&axiom_diag);

                    if matches!(d.level, DiagnosticLevel::Error) {
                        has_error = true;
                    }
                }
                if has_error {
                    return Err(miette::miette!("Semantic analysis reported errors"));
                }
            }
        }
        
        // ----------------------------------------------------------------
        // axiom fmt <file.ax> [--write]
        // ----------------------------------------------------------------
        Commands::Fmt { path, write } => {
            let source = std::fs::read_to_string(&path)
                .map_err(|e| miette::miette!("Cannot read '{}': {}", path.display(), e))?;

            let formatted = format_source(&source);

            if write {
                std::fs::write(&path, &formatted)
                    .map_err(|e| miette::miette!("Cannot write '{}': {}", path.display(), e))?;
                println!("✓ Formatted '{}'", path.display());
            } else {
                print!("{}", formatted);
                std::io::stdout().flush().into_diagnostic()?;
            }
        }

        // ----------------------------------------------------------------
        // axiom pkg <add|remove|list>
        // ----------------------------------------------------------------
        Commands::Pkg { cmd } => {
            let pm = PackageManager::new()
                .map_err(|e| miette::miette!("Package manager init failed: {}", e))?;

            match cmd {
                PkgCommands::Add { name } => {
                    pm.install_package(&name)
                        .map_err(|e| miette::miette!("Failed to install '{}': {}", name, e))?;
                }
                PkgCommands::Remove { name } => {
                    pm.remove_package(&name)
                        .map_err(|e| miette::miette!("Failed to remove '{}': {}", name, e))?;
                }
                PkgCommands::List => {
                    let pkgs = pm.list_packages()
                        .map_err(|e| miette::miette!("Failed to list packages: {}", e))?;
                    if pkgs.is_empty() {
                        println!("No packages installed.");
                    } else {
                        println!("Installed packages:");
                        for p in pkgs {
                            println!("  {}", p);
                        }
                    }
                }
                PkgCommands::Upgrade { name } => {
                    pm.upgrade_package(&name)
                        .map_err(|e| miette::miette!("Failed to upgrade '{}': {}", name, e))?;
                }
                PkgCommands::Info { name } => {
                    if name == "." {
                        // Auto-detect local Axiomite.toml
                        pm.show_local_info()
                            .map_err(|e| miette::miette!("Failed to show local info: {}", e))?;
                    } else {
                        pm.show_package_info(&name)
                            .map_err(|e| miette::miette!("Failed to show package info: {}", e))?;
                    }
                }
            }
        }
        // ----------------------------------------------------------------
        // axiom conf <set|get|list|reset|describe>
        // ----------------------------------------------------------------
        Commands::Conf { cmd } => {
            match cmd {
                ConfCommands::Set { spec } => {
                    cmd_conf_set(&spec).map_err(|e| miette::miette!("{}", e))?;
                }
                ConfCommands::Get { key } => {
                    cmd_conf_get(&key).map_err(|e| miette::miette!("{}", e))?;
                }
                ConfCommands::List => {
                    cmd_conf_list();
                }
                ConfCommands::Reset => {
                    cmd_conf_reset().map_err(|e| miette::miette!("{}", e))?;
                }
                ConfCommands::Describe { key } => {
                    cmd_conf_describe(&key);
                }
            }
        }
    }

    Ok(())
}
