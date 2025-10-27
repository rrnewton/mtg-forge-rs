//! Integration tests that wrap shell and Python scripts
//!
//! This module discovers and runs all .sh and .py scripts in the tests directory,
//! making them available through `cargo test`.

use std::path::PathBuf;
use std::process::Command;

/// Helper function to run a shell script and check its exit status
fn run_shell_script(script_name: &str) {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let script_path = workspace_root.join("tests").join(script_name);

    assert!(
        script_path.exists(),
        "Shell script not found: {}",
        script_path.display()
    );

    let output = Command::new("bash")
        .arg(&script_path)
        .current_dir(&workspace_root) // Run from workspace root
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute {}: {}", script_name, e));

    if !output.status.success() {
        eprintln!("--- STDOUT ---");
        eprintln!("{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("--- STDERR ---");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        panic!(
            "Shell script {} failed with exit code: {}",
            script_name,
            output.status.code().unwrap_or(-1)
        );
    }
}

/// Helper function to run a Python script and check its exit status
fn run_python_script(script_name: &str) {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let script_path = workspace_root.join("tests").join(script_name);

    assert!(
        script_path.exists(),
        "Python script not found: {}",
        script_path.display()
    );

    let output = Command::new("python3")
        .arg(&script_path)
        .current_dir(&workspace_root) // Run from workspace root
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute {}: {}", script_name, e));

    if !output.status.success() {
        eprintln!("--- STDOUT ---");
        eprintln!("{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("--- STDERR ---");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        panic!(
            "Python script {} failed with exit code: {}",
            script_name,
            output.status.code().unwrap_or(-1)
        );
    }
}

#[test]
fn test_heuristic_grizzly_bears_attack() {
    run_shell_script("heuristic_grizzly_bears_attack_e2e.sh");
}

#[test]
fn test_heuristic_royal_assassin() {
    run_shell_script("heuristic_royal_assassin_e2e.sh");
}

#[test]
fn test_interactive_tui() {
    run_shell_script("interactive_tui_e2e.sh");
}

#[test]
fn test_puzzle_load() {
    run_shell_script("puzzle_load_e2e.sh");
}

#[test]
fn test_snapshot_stress() {
    run_python_script("snapshot_stress_test.py");
}
