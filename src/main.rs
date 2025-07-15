use regex::Regex;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn getos() -> String {
    env::consts::OS.to_string()
}

fn getdir() -> String {
    let path: PathBuf = env::current_dir().expect("NO PATH FOUND");
    path.display().to_string()
}

fn find_missing_crates() {
    println!("Analyzing missing crates in main.rs...\n");

    match extract_crates_from_source() {
        Ok(source_crates) => {
            if !source_crates.is_empty() {
                println!("Crates found in use statements:");
                for crate_name in &source_crates {
                    println!("  - {}", crate_name);
                }

                // Automatically install the crates
                println!("\nAttempting to install crates...");
                install_crates(&source_crates);
                println!();
            }
        }
        Err(e) => {
            eprintln!("Error reading source file: {}", e);
        }
    }

    match analyze_missing_crates() {
        Ok(crates) => {
            if !crates.is_empty() {
                println!("Additional missing crates found from compilation errors:");
                for crate_name in &crates {
                    println!("  - {}", crate_name);
                }

                // Automatically install these crates too
                println!("\nAttempting to install additional crates...");
                install_crates(&crates);
            }
        }
        Err(e) => {
            eprintln!("Error analyzing crates: {}", e);

            // Fallback to rustc method
            println!("\nTrying alternative method with rustc...");
            if let Err(e2) = analyze_missing_crates_rustc() {
                eprintln!("Alternative method also failed: {}", e2);
            }
        }
    }
}

fn install_crates(crates: &[String]) {
    for crate_name in crates {
        println!("Installing {}...", crate_name);

        match Command::new("cargo").args(&["add", crate_name]).output() {
            Ok(output) => {
                if output.status.success() {
                    println!("✓ Successfully installed {}", crate_name);
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("✗ Failed to install {}: {}", crate_name, stderr.trim());
                }
            }
            Err(e) => {
                println!("✗ Error running cargo add for {}: {}", crate_name, e);
            }
        }
    }
}

fn extract_crates_from_source() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let source_path = "src/main.rs";
    let content = fs::read_to_string(source_path)?;

    let mut crates = HashSet::new();

    // Regex to match use statements and extract the first word (crate name)
    let use_regex = Regex::new(r"(?m)^use\s+([a-zA-Z_][a-zA-Z0-9_]*)")?;

    for cap in use_regex.captures_iter(&content) {
        if let Some(crate_name) = cap.get(1) {
            let name = crate_name.as_str();
            // Filter out standard library modules and current crate references
            if !is_std_module(name) && name != "self" && name != "super" && name != "crate" {
                crates.insert(name.to_string());
            }
        }
    }

    let mut result: Vec<String> = crates.into_iter().collect();
    result.sort();

    Ok(result)
}

fn analyze_missing_crates() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Run cargo check to get compilation errors
    let output = Command::new("cargo")
        .args(&["check", "--message-format=plain"])
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    let combined_output = format!("{}\n{}", stderr, stdout);

    let missing_crates = extract_missing_crates(&combined_output);

    if missing_crates.is_empty() {
        println!("No missing crates found!");
    } else {
        println!("Missing crates that need to be installed:");
        for crate_name in &missing_crates {
            println!("  - {}", crate_name);
        }

        println!("\nTo install these crates, add them to your Cargo.toml:");
        println!("[dependencies]");
        for crate_name in &missing_crates {
            println!("{} = \"*\"", crate_name);
        }

        println!("\nOr run these commands:");
        for crate_name in &missing_crates {
            println!("cargo add {}", crate_name);
        }
    }

    Ok(missing_crates)
}

fn extract_missing_crates(error_output: &str) -> Vec<String> {
    let mut missing_crates = HashSet::new();

    let patterns = vec![
        Regex::new(r"use of undeclared crate or module `([^`]+)`").unwrap(),
        Regex::new(r"failed to resolve: use of undeclared crate or module `([^`]+)`").unwrap(),
        Regex::new(r"unresolved import `([^`:]+)`").unwrap(),
        Regex::new(r"no external crate `([^`]+)`").unwrap(),
        Regex::new(r"extern crate `([^`]+)` not found").unwrap(),
        Regex::new(r"maybe a missing crate `([^`]+)`\?").unwrap(),
        Regex::new(r"consider adding `extern crate ([^;`]+);`").unwrap(),
    ];

    for pattern in patterns {
        for cap in pattern.captures_iter(error_output) {
            if let Some(crate_name) = cap.get(1) {
                let name = crate_name.as_str();
                if !is_std_module(name) && !name.contains("::") {
                    missing_crates.insert(name.to_string());
                }
            }
        }
    }

    let import_suggestions = Regex::new(r"help: consider importing this.*?`([^`:]+)::").unwrap();
    for cap in import_suggestions.captures_iter(error_output) {
        if let Some(crate_name) = cap.get(1) {
            let name = crate_name.as_str();
            if !is_std_module(name) {
                missing_crates.insert(name.to_string());
            }
        }
    }

    let mut result: Vec<String> = missing_crates.into_iter().collect();
    result.sort();
    result
}

fn is_std_module(name: &str) -> bool {
    let std_modules = vec![
        "std",
        "core",
        "alloc",
        "proc_macro",
        "test",
        "collections",
        "env",
        "fs",
        "io",
        "net",
        "path",
        "process",
        "sync",
        "thread",
        "time",
        "fmt",
        "mem",
        "ptr",
        "slice",
        "str",
        "vec",
        "hash",
        "cmp",
        "ops",
        "iter",
        "option",
        "result",
        "clone",
        "convert",
        "default",
        "drop",
        "marker",
        "ascii",
        "char",
        "f32",
        "f64",
        "i8",
        "i16",
        "i32",
        "i64",
        "i128",
        "isize",
        "u8",
        "u16",
        "u32",
        "u64",
        "u128",
        "usize",
        "bool",
        "never",
        "array",
        "tuple",
        "unit",
        "self",
        "super",
        "crate",
    ];

    std_modules.contains(&name)
}

fn analyze_missing_crates_rustc() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("rustc")
        .args(&["--error-format=human", "--crate-type=bin", "src/main.rs"])
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let missing_crates = extract_missing_crates(&stderr);

    if missing_crates.is_empty() {
        println!("No missing crates found!");
    } else {
        println!("Missing crates that need to be installed:");
        for crate_name in &missing_crates {
            println!("  - {}", crate_name);
        }
    }

    Ok(missing_crates)
}

fn main() {
    if getos() == "windows" {
        println!("PATH for {}: {}\\src\\main.rs", getos(), getdir());
        find_missing_crates();
    } else {
        println!("PATH for {}: {}/src/main.rs", getos(), getdir());
        find_missing_crates();
    }
}
