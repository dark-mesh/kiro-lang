use kiro_lang::grammar;
use kiro_lang::interpreter;
use kiro_lang::compiler;
use std::fs;
use std::process::Command;
#[tokio::main]
async fn main() {
    // Test Case:
    // 1. Whitespace handling (spaces between tokens)
    // 2. Variable reading (using 'x' in the second line)
    // 3. Math precedence (10 * 5 should happen before x + ...)
    let source = "
        var x = 100
        y = x + 10 * 5
    ";

    match grammar::parse(source) {
        Ok(program) => {
            println!("✅ Parsed successfully!");
            println!("Statements found: {}", program.statements.len());
            // If this prints 2, you have a working language grammar!
            // Create the engine and run the code
            println!("\n--- Transpiling to Rust ---");
            let mut c = compiler::Compiler::new();
            let rust_code = c.compile(program);
            println!("{}", rust_code);
            // save_and_run(rust_code);
        }
        Err(errs) => {
            eprintln!("❌ Parse Failed:");
            for e in errs {
                eprintln!("{:?}", e);
            }
        }
    }
}
use std::fs;
use std::path::Path;
use std::process::Command;

fn save_and_run(rust_code: String) {
    // 1. Define a hidden build directory
    let build_dir = "kiro_build_cache";
    let src_dir = format!("{}/src", build_dir);
    
    // 2. Create the directory structure if it doesn't exist
    // kiro_build_cache/
    // └── src/
    if !Path::new(&src_dir).exists() {
        fs::create_dir_all(&src_dir).expect("Failed to create build directory");
        
        // 3. Create a 'Cargo.toml' that pulls in Tokio
        // We configure it to use the exact same dependencies your language needs
        let cargo_toml = r#"
[package]
name = "kiro_user_script"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
"#;
        fs::write(format!("{}/Cargo.toml", build_dir), cargo_toml)
            .expect("Failed to write Cargo.toml");
    }

    // 4. Save the user's transpiled code to 'src/main.rs'
    fs::write(format!("{}/main.rs", src_dir), rust_code)
        .expect("Failed to write main.rs");

    println!("🔨 Building and Running Kiro Script...");
    println!("---------------------------------------");

    // 5. Run 'cargo run' inside that directory
    // We use --quiet so cargo doesn't spam the console with build logs
    let run_status = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .current_dir(build_dir) // IMPORTANT: Run command inside the folder
        .status();

    match run_status {
        Ok(status) if status.success() => println!("\n✅ Execution Finished."),
        _ => eprintln!("❌ Runtime Error: The generated code failed to run."),
    }
}