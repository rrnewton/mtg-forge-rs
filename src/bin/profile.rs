//! Profiling binary for game execution
//!
//! This binary runs games in a tight loop for profiling with cargo-flamegraph
//! or cargo-heaptrack. Unlike the Criterion benchmarks, this has minimal overhead
//! and produces cleaner profiles.
//!
//! Usage:
//!   cargo flamegraph --bin profile -- [iterations]
//!   cargo heaptrack --bin profile -- [iterations]
//!   # or via makefile:
//!   make profile
//!   make heapprofile

use clap::Parser;
use mtg_forge_rs::{
    game::{random_controller::RandomController, GameLoop, VerbosityLevel},
    loader::{prefetch_deck_cards, AsyncCardDatabase as CardDatabase, DeckLoader, GameInitializer},
};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "profile")]
#[command(about = "Run games for profiling", long_about = None)]
struct Args {
    /// Number of games to run (default: 1000 for time profiling, 100 for heap profiling)
    #[arg(default_value_t = 1000)]
    iterations: usize,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Load deck
    let deck_path = PathBuf::from("test_decks/simple_bolt.dck");
    let deck = DeckLoader::load_from_file(&deck_path).expect("Failed to load deck");

    // Create card database (lazy loading - only loads cards on-demand)
    let cardsfolder = PathBuf::from("cardsfolder");
    let card_db = CardDatabase::new(cardsfolder);

    // Prefetch deck cards (not all 31k cards, just what we need)
    let start = std::time::Instant::now();
    let (count, _) = prefetch_deck_cards(&card_db, &deck)
        .await
        .expect("Failed to prefetch deck cards");
    let duration = start.elapsed();
    println!("Prefetched {} deck cards in {:.2?}", count, duration);

    let iterations = args.iterations;

    println!("Profiling game execution...");
    println!("Running {} games with seed 42", iterations);
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
            .await
            .expect("Failed to initialize game");
        game.rng_seed = seed;

        // Create random controllers
        let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
        let p1_id = players[0];
        let p2_id = players[1];

        let mut controller1 = RandomController::with_seed(p1_id, seed);
        let mut controller2 = RandomController::with_seed(p2_id, seed + 1);

        // Run game
        let mut game_loop = GameLoop::new(&mut game).with_verbosity(VerbosityLevel::Silent);
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
}
