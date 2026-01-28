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
    let filename = "main.kiro";

    // Check existence
    if !std::path::Path::new(filename).exists() {
        eprintln!("❌ Error: '{}' not found.", filename);
        return;
    }

    let source = fs::read_to_string(filename).unwrap();
    let prog = match grammar::parse(&source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parse Error in main.kiro: {:?}", e);
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

    let mut seen = std::collections::HashSet::new();
    build_recursive("main", &mut seen, &pm);

    if let Err(e) = pm.run() {
        eprintln!("Run Error: {}", e);
    }
}

fn build_recursive(name: &str, seen: &mut std::collections::HashSet<String>, pm: &BuildManager) {
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
            build_recursive(module_name, seen, pm);
        }
    }

    // Compile
    let mut c = compiler::Compiler::new();
    let code = c.compile(prog, name == "main");

    if let Err(e) = pm.save_file(name, code) {
        eprintln!("Failed to save {}: {}", name, e);
    } else {
        println!("  - Compiled {}", name);
    }
}
