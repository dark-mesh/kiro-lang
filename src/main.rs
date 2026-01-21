use kiro_lang::build_manager::BuildManager;
use kiro_lang::grammar;
use kiro_lang::interpreter;
use kiro_lang::compiler;
use kiro_lang::build_manager;
use std::fs;
use std::process::Command;
#[tokio::main]
async fn main() {
    let filename = "main.kiro";
    println!("📖 Reading {}...", filename);
    
    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("❌ Error: Could not find '{}'.", filename);
            eprintln!("   Please create this file in the project root.");
            return;
        }
    };
    match grammar::parse(&source) {
        Ok(program) => {
            // 1. Compile (Transpile)
            let mut c = compiler::Compiler::new();
            let rust_code = c.compile(program);

            // 2. Manage Project Execution
            let pm = BuildManager::new("kiro_build_cache");
            
            if let Err(e) = pm.init() {
                eprintln!("❌ Init Error: {}", e);
                return;
            }

            if let Err(e) = pm.save_code(rust_code) {
                eprintln!("❌ Save Error: {}", e);
                return;
            }

            if let Err(e) = pm.run() {
                eprintln!("❌ Run Error: {}", e);
            }
        }
        Err(e) => eprintln!("Parse Error: {:?}", e),
    }
}