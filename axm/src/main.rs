/// Axiom CLI (axm)
/// Orchestrates run, pkg, fmt, and chk commands.

use axiom::{Parser, Runtime, SemanticAnalyzer, format_source};
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
    name = "axm",
    version = "0.1.0",
    about = "The Axiom Language Toolchain",
    long_about = "axm — run, check, format, and manage packages for Axiom (.ax) scripts."
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
}

#[derive(Subcommand)]
enum PkgCommands {
    /// Install a package: axm pkg add <user>/<repo>
    Add { name: String },
    /// Remove a package: axm pkg remove <user>/<repo>
    Remove { name: String },
    /// List installed packages
    List,
    /// Show package info: axm pkg info <user>/<repo>
    Info { name: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let result = std::thread::Builder::new()
        .name("axm-worker".into())
        .stack_size(STACK_SIZE)
        .spawn(move || {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(cli)))
        })
        .expect("failed to spawn axm worker thread")
        .join();

    match result {
        Ok(Ok(inner)) => inner,
        Ok(Err(panic_payload)) | Err(panic_payload) => {
            let msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                format!("internal error (panic): {}", s)
            } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                format!("internal error (panic): {}", s)
            } else {
                "internal error: unexpected panic in axm runtime".to_string()
            };
            eprintln!("axm crashed: {}", msg);
            Err(miette::miette!("{}", msg))
        }
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        // ----------------------------------------------------------------
        // axm run <file.ax>
        // ----------------------------------------------------------------
        Commands::Run { path } => {
            let source = std::fs::read_to_string(&path)
                .map_err(|e| miette::miette!("Cannot read '{}': {}", path.display(), e))?;

            let mut parser = Parser::new(&source, 0);
            let items = parser.parse()
                .map_err(|e| miette::miette!("Parse error: {}", e))?;

            let mut runtime = Runtime::new();
            runtime.run(items)
                .map_err(|e| {
                    eprintln!("Runtime error: {}", e);
                    miette::miette!("{}", e)
                })?;

            std::io::stdout().flush().into_diagnostic()?;
        }

        // ----------------------------------------------------------------
        // axm chk <file.ax>
        // ----------------------------------------------------------------
        Commands::Chk { path } => {
            let source = std::fs::read_to_string(&path)
                .map_err(|e| miette::miette!("Cannot read '{}': {}", path.display(), e))?;

            let mut parser = Parser::new(&source, 0);
            let items = parser.parse()
                .map_err(|e| miette::miette!("Parse error: {}", e))?;

            let mut chk = SemanticAnalyzer::new();
            let diagnostics = chk.check(&items);

            if diagnostics.is_empty() {
                println!("✓ No issues found in '{}'", path.display());
            } else {
                let mut has_error = false;
                for d in &diagnostics {
                    println!("{}", d);
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
        // axm fmt <file.ax> [--write]
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
        // axm pkg <add|remove|list>
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
                PkgCommands::Info { name } => {
                    pm.show_package_info(&name)
                        .map_err(|e| miette::miette!("Failed to show package info: {}", e))?;
                }
            }
        }
    }

    Ok(())
}
