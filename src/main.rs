mod build_manager;
mod compiler;
mod grammar;
mod interpreter;

use crate::build_manager::BuildManager;

use std::fs;

#[tokio::main]
async fn main() {
    println!("🚀 Kiro Build System v0.2");

    // 1. Interpret Main (triggers interpreter recursive imports)
    let args: Vec<String> = std::env::args().collect();
    let filename = if args.len() > 1 {
        &args[1]
    } else {
        "main.kiro"
    };

    // Check existence
    if !std::path::Path::new(filename).exists() {
        eprintln!("❌ Error: '{}' not found.", filename);
        return;
    }

    let source = fs::read_to_string(filename).unwrap();
    let prog = match grammar::parse(&source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parse Error in {}: {:?}", filename, e);
            return;
        }
    };

    println!("🤖 --- INTERPRETER ---");
    let mut i = interpreter::Interpreter::new();
    if let Err(e) = i.run(prog) {
        eprintln!("Interpreter Error: {}", e);
    }

    // 2. Compile Project
    println!("🔨 --- COMPILING ---");
    let pm = BuildManager::new("kiro_build_cache");
    if let Err(e) = pm.init() {
        eprintln!("Init Error: {}", e);
        return;
    }

    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let path = std::path::Path::new(filename);
    let name = path.file_stem().unwrap().to_str().unwrap();
    let dir = path.parent().map(|p| p.to_str().unwrap()).unwrap_or("");
    build_recursive(name, dir, &mut seen, &pm, true);

    if let Err(e) = pm.run() {
        eprintln!("Run Error: {}", e);
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
