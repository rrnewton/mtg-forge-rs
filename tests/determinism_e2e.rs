//! End-to-end determinism tests
//!
//! Verifies that games with the same seed produce identical output across multiple runs.
//! This test runs the actual binary and compares stdout to ensure deterministic behavior.
//!
//! ## Adding tests for new decks
//!
//! When adding a new deck file to `test_decks/`, create a corresponding test function:
//! ```ignore
//! #[test]
//! fn test_determinism_your_deck_name() {
//!     test_deck_determinism("test_decks/your_deck.dck", 42, "verbose");
//! }
//! ```

use std::path::PathBuf;
use std::process::Command;

/// Helper to run the mtg binary and capture stdout
fn run_game_with_seed(deck_path: &str, seed: u64, verbosity: &str) -> String {
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--bin",
            "mtg",
            "--",
            "tui",
            deck_path,
            deck_path,
            "--seed",
            &seed.to_string(),
            "--p1=random",
            "--p2=random",
            &format!("--verbosity={}", verbosity),
        ])
        .output()
        .expect("Failed to run mtg binary");

    String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout")
}

/// Helper function to test determinism for a specific deck
/// Runs the game twice with the same seed and verifies identical output
fn test_deck_determinism(deck_path: &str, seed: u64, verbosity: &str) {
    if !PathBuf::from(deck_path).exists() {
        // Skip test if deck doesn't exist
        eprintln!("Skipping test: deck {} does not exist", deck_path);
        return;
    }

    // Run the game twice with the same seed
    let run1 = run_game_with_seed(deck_path, seed, verbosity);
    let run2 = run_game_with_seed(deck_path, seed, verbosity);

    // Verify output is not empty
    assert!(
        !run1.is_empty(),
        "Deck {} produced empty output",
        deck_path
    );

    // Verify both runs produce identical output
    assert_eq!(
        run1, run2,
        "Deck {} produced different output with same seed (seed={})",
        deck_path, seed
    );
}

// ============================================================================
// Individual deck determinism tests
// ============================================================================
// Each deck gets its own test function for clear test output
// When adding a new deck, add a new test function following this pattern

/// Test determinism for simple_bolt.dck with verbose output
#[test]
fn test_determinism_simple_bolt() {
    test_deck_determinism("test_decks/simple_bolt.dck", 42, "verbose");
}

// Add more deck-specific tests here as new decks are added to test_decks/
// Example:
// #[test]
// fn test_determinism_creature_combat() {
//     test_deck_determinism("test_decks/creature_combat.dck", 42, "verbose");
// }

// ============================================================================
// Multi-seed and cross-validation tests
// ============================================================================

/// Test that different seeds produce consistent but different results
#[test]
fn test_different_seeds_consistency() {
    let deck_path = "test_decks/simple_bolt.dck";
    if !PathBuf::from(deck_path).exists() {
        return;
    }

    let verbosity = "verbose";

    // Verify seed 42 is consistent
    let seed42_run1 = run_game_with_seed(deck_path, 42, verbosity);
    let seed42_run2 = run_game_with_seed(deck_path, 42, verbosity);
    assert_eq!(
        seed42_run1, seed42_run2,
        "Seed 42 produced inconsistent output"
    );

    // Verify seed 100 is consistent
    let seed100_run1 = run_game_with_seed(deck_path, 100, verbosity);
    let seed100_run2 = run_game_with_seed(deck_path, 100, verbosity);
    assert_eq!(
        seed100_run1, seed100_run2,
        "Seed 100 produced inconsistent output"
    );

    // Verify different seeds produce different output
    assert_ne!(
        seed42_run1, seed100_run1,
        "Different seeds produced identical output (highly unlikely)"
    );
}

/// Discovery test: warns if there are deck files without corresponding tests
#[test]
fn test_all_decks_have_dedicated_tests() {
    let test_decks_dir = PathBuf::from("test_decks");
    if !test_decks_dir.exists() {
        return;
    }

    let known_tested_decks = vec!["simple_bolt.dck"];

    let deck_files: Vec<_> = std::fs::read_dir(&test_decks_dir)
        .expect("Failed to read test_decks directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()?.to_str()? == "dck" {
                path.file_name()?.to_str().map(String::from)
            } else {
                None
            }
        })
        .collect();

    let mut missing_tests = Vec::new();
    for deck_file in &deck_files {
        if !known_tested_decks.contains(&deck_file.as_str()) {
            missing_tests.push(deck_file.clone());
        }
    }

    if !missing_tests.is_empty() {
        panic!(
            "Found deck files without dedicated determinism tests: {:?}\n\
             Please add a test function for each deck in tests/determinism_e2e.rs",
            missing_tests
        );
    }
}
