//! Profiling binary for game execution
//!
//! This binary runs games in a tight loop for profiling with cargo-flamegraph.
//! Unlike the Criterion benchmarks, this has minimal overhead and produces
//! cleaner flamegraphs.
//!
//! Usage:
//!   cargo flamegraph --bin profile
//!   # or via makefile:
//!   make profile

use mtg_forge_rs::{
    game::{GameLoop, RandomController},
    loader::{CardDatabase, DeckLoader, GameInitializer},
};
use std::path::PathBuf;

fn main() {
    // Load card database and deck once
    let cardsfolder = PathBuf::from("cardsfolder");
    let card_db = CardDatabase::load_from_cardsfolder(&cardsfolder)
        .expect("Failed to load card database");

    let deck_path = PathBuf::from("test_decks/simple_bolt.dck");
    let deck = DeckLoader::load_from_file(&deck_path).expect("Failed to load deck");

    // Allow overriding iterations via environment variable
    let iterations = std::env::var("PROFILE_ITERATIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000);

    println!("Profiling game execution...");
    println!("Running {} games with seed 42", iterations);
    println!("Output will be saved to flamegraph.svg");
    println!();

    let seed = 42u64;

    // Run games in a tight loop for profiling
    for i in 0..iterations {
        // Initialize game
        let game_init = GameInitializer::new(&card_db);
        let mut game = game_init
            .init_game(
                "Player 1".to_string(),
                &deck,
                "Player 2".to_string(),
                &deck,
                20,
            )
            .expect("Failed to initialize game");
        game.rng_seed = seed;

        // Create random controllers
        let players: Vec<_> = game.players.iter().map(|(id, _)| *id).collect();
        let p1_id = players[0];
        let p2_id = players[1];

        let mut controller1 = RandomController::with_seed(p1_id, seed);
        let mut controller2 = RandomController::with_seed(p2_id, seed + 1);

        // Run game
        let mut game_loop = GameLoop::new(&mut game).with_verbose(false);
        game_loop
            .run_game(&mut controller1, &mut controller2)
            .expect("Game execution failed");

        // Print progress every 100 games
        if (i + 1) % 100 == 0 {
            println!("Completed {} games", i + 1);
        }
    }

    println!();
    println!("Profiling complete! {} games executed.", iterations);
    println!("Flamegraph saved to: flamegraph.svg");
}
