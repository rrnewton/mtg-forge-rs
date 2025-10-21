//! End-to-end determinism tests
//!
//! Verifies that games with the same seed produce identical output across multiple runs.
//! This test runs the actual binary and compares stdout to ensure deterministic behavior.

use std::path::PathBuf;
use std::process::Command;

/// Helper to run the mtg binary and capture stdout
fn run_game_with_seed(deck_path: &str, seed: u64, verbosity: &str) -> String {
    let output = Command::new("cargo")
        .args(&[
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

/// Test that running the same game with the same seed produces identical output
#[test]
fn test_deterministic_game_output() {
    let deck_path = "test_decks/simple_bolt.dck";
    if !PathBuf::from(deck_path).exists() {
        // Skip test if deck doesn't exist
        return;
    }

    let seed = 42;
    let verbosity = "verbose";

    // Run the game 3 times with the same seed
    let run1 = run_game_with_seed(deck_path, seed, verbosity);
    let run2 = run_game_with_seed(deck_path, seed, verbosity);
    let run3 = run_game_with_seed(deck_path, seed, verbosity);

    // All runs should produce identical output
    assert_eq!(
        run1, run2,
        "Run 1 and Run 2 produced different output with same seed"
    );
    assert_eq!(
        run1, run3,
        "Run 1 and Run 3 produced different output with same seed"
    );

    // Verify the output is not empty
    assert!(!run1.is_empty(), "Game output should not be empty");
}

/// Test determinism with different seeds produces different but consistent results
#[test]
fn test_different_seeds_produce_different_outputs() {
    let deck_path = "test_decks/simple_bolt.dck";
    if !PathBuf::from(deck_path).exists() {
        return;
    }

    let verbosity = "verbose";

    // Run with different seeds
    let output_seed_42 = run_game_with_seed(deck_path, 42, verbosity);
    let output_seed_100 = run_game_with_seed(deck_path, 100, verbosity);

    // Different seeds should produce different game outcomes
    // (This might not always be true, but for randomized games it's very likely)
    // We mainly want to ensure each seed is consistent with itself

    // Verify each seed produces consistent output
    let output_seed_42_repeat = run_game_with_seed(deck_path, 42, verbosity);
    let output_seed_100_repeat = run_game_with_seed(deck_path, 100, verbosity);

    assert_eq!(
        output_seed_42, output_seed_42_repeat,
        "Seed 42 produced different output on repeat run"
    );
    assert_eq!(
        output_seed_100, output_seed_100_repeat,
        "Seed 100 produced different output on repeat run"
    );

    assert!(
        output_seed_42 != output_seed_100_repeat,
        "Seeds 42 and 100 produced exact same verbose output"
    );
}

/// Test determinism for all available test decks
#[test]
fn test_determinism_all_decks() {
    let test_decks_dir = PathBuf::from("test_decks");
    if !test_decks_dir.exists() {
        return;
    }

    let deck_files: Vec<_> = std::fs::read_dir(&test_decks_dir)
        .expect("Failed to read test_decks directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()?.to_str()? == "dck" {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    if deck_files.is_empty() {
        panic!("No .dck files found in test_decks directory");
    }

    for deck_path in deck_files {
        let deck_str = deck_path.to_str().expect("Invalid deck path");
        println!("Testing determinism for deck: {}", deck_str);

        let seed = 42;
        let verbosity = "normal";

        // Run twice and compare
        let run1 = run_game_with_seed(deck_str, seed, verbosity);
        let run2 = run_game_with_seed(deck_str, seed, verbosity);

        assert_eq!(
            run1, run2,
            "Deck {} produced different output with same seed",
            deck_str
        );
    }
}
