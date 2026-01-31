use std::fs;
use std::path::Path;
use std::process::Command;

pub struct BuildManager {
    build_dir: String,
}
impl BuildManager {
    pub fn new(build_dir: &str) -> Self {
        Self {
            build_dir: build_dir.to_string(),
        }
    }

    /// Sets up the folder structure and generates the Cargo.toml
    pub fn init(&self) -> Result<(), String> {
        let src_dir = format!("{}/src", self.build_dir);

        // 1. Create directories
        if !Path::new(&src_dir).exists() {
            fs::create_dir_all(&src_dir).map_err(|e| e.to_string())?;
            println!("📁 Initialized build directory: {}", self.build_dir);
        }

        // 2. Generate/Update Cargo.toml
        self.create_cargo_toml()?;
        Ok(())
    }

    pub fn save_file(&self, name_without_ext: &str, code: String) -> Result<(), String> {
        let file_path = format!("{}/src/{}.rs", self.build_dir, name_without_ext);
        fs::write(&file_path, code).map_err(|e| e.to_string())?;
        println!("💾 Code saved to {}", file_path);
        Ok(())
    }
    pub fn run(&self) -> Result<(), String> {
        println!("🚀 Compiling and Running...\n");

        let output = Command::new("cargo")
            .arg("run")
            .arg("--quiet") // Less noise
            .current_dir(&self.build_dir)
            .output()
            .map_err(|e| format!("Failed to execute cargo: {}", e))?;

        if !output.stdout.is_empty() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }

        if output.status.success() {
            Ok(())
        } else {
            Err("Runtime execution failed.".to_string())
        }
    }
    fn create_cargo_toml(&self) -> Result<(), String> {
        let my_cargo = fs::read_to_string("Cargo.toml").unwrap_or_else(|_| "".to_string());

        // Simple parser: find the line starting with 'tokio ='
        let tokio_dep = my_cargo.lines()
            .find(|line| line.trim().starts_with("tokio ="))
            .unwrap_or(r#"tokio = { version = "1", features = ["rt-multi-thread", "macros", "sync", "time"] }"#); // Fallback if file missing
        let async_channel_dep = my_cargo
            .lines()
            .find(|line| line.trim().starts_with("async-channel ="))
            .unwrap_or(r#"async-channel = "2.5""#);
        let content = format!(
            r#"
            [package]
            name = "kiro_script"
            version = "0.1.0"
            edition = "2021"

            [dependencies]
            {}
            {} 
            "#,
            tokio_dep, async_channel_dep
        );
        fs::write(format!("{}/Cargo.toml", self.build_dir), content).map_err(|e| e.to_string())
    }
}
