mod build_manager;
mod compiler;
mod grammar;
mod interpreter;

use crate::build_manager::BuildManager;

use std::fs;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Default file argument (if no subcommand is used)
    file: Option<String>,

    /// Skip interpreter step
    #[arg(long)]
    no_interpret: bool,

    /// Skip execution after build
    #[arg(long)]
    no_run: bool,

    /// Output generated Rust code to stdout
    #[arg(long)]
    emit_rust: bool,

    /// Show compiler output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Parse, Compile, and Execute (Default)
    Run {
        file: String,
        #[arg(long)]
        no_interpret: bool,
        #[arg(long)]
        no_run: bool,
        #[arg(long)]
        emit_rust: bool,
        #[arg(short, long)]
        verbose: bool,
    },
    /// Interpret ONLY (No Compilation, No Host Modules)
    Check { file: String },
    /// Transpile and Build ONLY (No Execution)
    Build {
        file: String,
        #[arg(long)]
        emit_rust: bool,
        #[arg(short, long)]
        verbose: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Run {
            file,
            no_interpret,
            no_run,
            emit_rust,
            verbose,
        }) => {
            if !execute_pipeline(&file, !*no_interpret, !*no_run, *emit_rust, *verbose) {
                std::process::exit(1);
            }
        }
        Some(Commands::Check { file }) => {
            if !run_interpreter(&file) {
                std::process::exit(1);
            }
        }
        Some(Commands::Build {
            file,
            emit_rust,
            verbose,
        }) => {
            if run_compiler(&file, *emit_rust, *verbose).is_err() {
                std::process::exit(1);
            }
        }
        None => {
            if let Some(file) = &cli.file {
                // Default behavior: Interpret -> Compile -> Run
                if !execute_pipeline(
                    file,
                    !cli.no_interpret,
                    !cli.no_run,
                    cli.emit_rust,
                    cli.verbose,
                ) {
                    std::process::exit(1);
                }
            } else {
                <Cli as clap::CommandFactory>::command()
                    .print_help()
                    .unwrap();
            }
        }
    }
}

// Returns true if success
fn execute_pipeline(
    file: &str,
    do_interpret: bool,
    do_run: bool,
    emit_rust: bool,
    verbose: bool,
) -> bool {
    println!("🚀 Kiro Build System v0.2");

    if do_interpret {
        println!("🤖 --- INTERPRETER ---");
        if !run_interpreter(file) {
            return false;
        }
    }

    if verbose {
        println!("🔨 --- COMPILING ---");
    } else {
        println!("🔨 --- COMPILING --- (Output hidden, use --verbose to show)");
    }

    match run_compiler(file, emit_rust, verbose) {
        Ok(exe_path) => {
            if do_run {
                println!("🚀 --- RUNNING ---");
                if let Err(e) = execute_binary(exe_path) {
                    eprintln!("Execution Error: {}", e);
                    return false;
                }
            }
        }
        Err(e) => {
            eprintln!("Compiler Error: {}", e);
            return false;
        }
    }

    true
}

fn run_interpreter(filename: &str) -> bool {
    if !std::path::Path::new(filename).exists() {
        eprintln!("❌ Error: '{}' not found.", filename);
        return false;
    }

    let source = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Read Error: {}", e);
            return false;
        }
    };

    let prog = match grammar::parse(&source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parse Error in {}: {:?}", filename, e);
            return false;
        }
    };

    let mut i = interpreter::Interpreter::new();
    if let Err(e) = i.run(prog) {
        eprintln!("Interpreter Error: {}", e);
        return false;
    }
    true
}

fn run_compiler(filename: &str, _emit_rust: bool, verbose: bool) -> Result<PathBuf, String> {
    if !std::path::Path::new(filename).exists() {
        return Err(format!("'{}' not found.", filename));
    }

    let pm = BuildManager::new("kiro_build_cache");
    if let Err(e) = pm.init() {
        return Err(format!("Init Error: {}", e));
    }

    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let path = std::path::Path::new(filename);
    let name = path.file_stem().unwrap().to_str().unwrap();
    let dir = path.parent().map(|p| p.to_str().unwrap()).unwrap_or("");
    build_recursive(name, dir, &mut seen, &pm, true);

    match pm.build(verbose) {
        Ok(output_path) => Ok(output_path),
        Err(e) => Err(format!("Build Error: {}", e)),
    }
}

fn execute_binary(path: PathBuf) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("Binary not found at {:?}", path));
    }

    let status = Command::new(&path)
        .status()
        .map_err(|e| format!("Failed to execute binary: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("Process exited with status: {}", status))
    }
}

#[derive(rust_embed::RustEmbed)]
#[folder = "src/kiro_std/"]
pub struct StdAssets;

fn build_recursive(
    name: &str,
    base_dir: &str,
    seen: &mut std::collections::HashSet<String>,
    pm: &BuildManager,
    is_root: bool,
) {
    if seen.contains(name) {
        return;
    }
    seen.insert(name.to_string());

    // Try to resolve module path:
    // 1. If starts with "std_", look in embedded assets
    // 2. Otherwise, look in base_dir or current directory as {name}.kiro
    let src = if name.starts_with("std_") {
        let module_name = &name[4..]; // Remove "std_" prefix
        // Map std_fs -> fs/std_fs.kiro
        let asset_path = format!("{}/{}.kiro", module_name, name);
        StdAssets::get(&asset_path)
            .map(|f| std::str::from_utf8(f.data.as_ref()).unwrap().to_string())
            .expect(&format!(
                "Standard library module '{}' not found in embedded assets",
                name
            ))
    } else {
        let filename = if !base_dir.is_empty() {
            format!("{}/{}.kiro", base_dir, name)
        } else {
            format!("{}.kiro", name)
        };

        match fs::read_to_string(&filename) {
            Ok(s) => s,
            Err(_) => {
                eprintln!(
                    "❌ Compiler Warning: File '{}' not found during build.",
                    filename
                );
                return;
            }
        }
    };

    let prog = grammar::parse(&src).expect("Parse error during build");

    // Find imports to recurse
    for s in &prog.statements {
        if let grammar::grammar::Statement::Import { module_name, .. } = s {
            // For imports, use base_dir for relative imports or "" for std imports
            let import_dir = if module_name.starts_with("std_") {
                ""
            } else {
                base_dir
            };
            build_recursive(module_name, import_dir, seen, pm, false);
        }
    }

    // Compile
    let mut c = compiler::Compiler::new();
    let code = c.compile(prog, is_root);

    let save_name = if is_root { "main" } else { name };
    if let Err(e) = pm.save_file(save_name, code) {
        eprintln!("Failed to save {}: {}", save_name, e);
    } else {
        println!("  - Compiled {}", name);
    }

    // If this is a std module, also copy its header.rs content
    if name.starts_with("std_") {
        let module_suffix = &name[4..];
        let header_path = format!("{}/header.rs", module_suffix);
        if let Some(file) = StdAssets::get(&header_path) {
            let header_content = std::str::from_utf8(file.data.as_ref()).unwrap();
            // Strip the initial use statement since we already have it in the main header
            let content = header_content
                .lines()
                .filter(|l| {
                    !l.trim().starts_with("use crate::")
                        && !l.trim().starts_with("use kiro_runtime")
                })
                .collect::<Vec<_>>()
                .join("\n");

            if let Err(e) = pm.append_header(&content) {
                eprintln!("Failed to append header for {}: {}", name, e);
            }
        }
    }
}
