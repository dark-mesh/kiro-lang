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
    let name = std::path::Path::new(filename)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();
    build_recursive(name, &mut seen, &pm, true);

    if let Err(e) = pm.run() {
        eprintln!("Run Error: {}", e);
    }
}

fn build_recursive(
    name: &str,
    seen: &mut std::collections::HashSet<String>,
    pm: &BuildManager,
    is_root: bool,
) {
    if seen.contains(name) {
        return;
    }
    seen.insert(name.to_string());

    let filename = format!("{}.kiro", name);
    let src = match fs::read_to_string(&filename) {
        Ok(s) => s,
        Err(_) => {
            eprintln!(
                "❌ Compiler Warning: File '{}' not found during build.",
                filename
            );
            return;
        }
    };

    let prog = grammar::parse(&src).expect("Parse error during build");

    // Find imports to recurse
    for s in &prog.statements {
        if let grammar::grammar::Statement::Import { module_name, .. } = s {
            build_recursive(module_name, seen, pm, false);
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
}
