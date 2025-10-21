//! Performance benchmarks for MTG Forge game engine
//!
//! This benchmark measures game execution performance using Criterion.rs.
//! It supports three different iteration modes:
//!
//! 1. **Fresh** - Allocate a new game for each iteration
//! 2. **Rewind** - Use undo log to rewind game to start (NOT YET IMPLEMENTED)
//! 3. **Snapshot** - Save/restore game state each iteration (NOT YET IMPLEMENTED)
//!
//! The benchmark is based on RandomController vs RandomController playing
//! with simple_bolt.dck (Mountains + Lightning Bolts).

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use mtg_forge_rs::{
    game::{GameLoop, RandomController},
    loader::{CardDatabase, DeckList, DeckLoader, GameInitializer},
    Result,
};
use std::path::PathBuf;
use std::time::Duration;

/// Metrics collected during game execution
#[derive(Debug, Clone)]
struct GameMetrics {
    /// Total turns played
    turns: u32,
    /// Total actions (from UndoLog)
    actions: usize,
    /// Game duration
    duration: Duration,
}

impl GameMetrics {
    /// Calculate actions per second
    fn actions_per_sec(&self) -> f64 {
        self.actions as f64 / self.duration.as_secs_f64()
    }

    /// Calculate turns per second
    fn turns_per_sec(&self) -> f64 {
        self.turns as f64 / self.duration.as_secs_f64()
    }

    /// Calculate average actions per turn
    fn actions_per_turn(&self) -> f64 {
        if self.turns == 0 {
            0.0
        } else {
            self.actions as f64 / self.turns as f64
        }
    }
}

/// Setup data needed for benchmarking (loaded once, reused across iterations)
struct BenchmarkSetup {
    card_db: CardDatabase,
    deck: DeckList,
}

impl BenchmarkSetup {
    fn load() -> Result<Self> {
        let cardsfolder = PathBuf::from("cardsfolder");
        let card_db = CardDatabase::load_from_cardsfolder(&cardsfolder)?;

        let deck_path = PathBuf::from("test_decks/simple_bolt.dck");
        let deck = DeckLoader::load_from_file(&deck_path)?;

        Ok(BenchmarkSetup { card_db, deck })
    }
}

/// Run a single game and collect metrics
/// Takes pre-loaded setup data to avoid measuring file I/O
fn run_game_with_metrics(setup: &BenchmarkSetup, seed: u64) -> Result<GameMetrics> {
    let start = std::time::Instant::now();

    // Initialize game
    let game_init = GameInitializer::new(&setup.card_db);
    let mut game = game_init.init_game(
        "Player 1".to_string(),
        &setup.deck,
        "Player 2".to_string(),
        &setup.deck,
        20,
    )?;
    game.rng_seed = seed;

    // Create random controllers
    let players: Vec<_> = game.players.iter().map(|(id, _)| *id).collect();
    let p1_id = players[0];
    let p2_id = players[1];

    let mut controller1 = RandomController::with_seed(p1_id, seed);
    let mut controller2 = RandomController::with_seed(p2_id, seed + 1);

    // Run game (still within timing)
    let mut game_loop = GameLoop::new(&mut game).with_verbose(false); // Quiet mode
    let result = game_loop.run_game(&mut controller1, &mut controller2)?;

    let duration = start.elapsed();

    // Collect metrics
    let actions = game_loop.game.undo_log.len();
    let metrics = GameMetrics {
        turns: result.turns_played,
        actions,
        duration,
    };

    Ok(metrics)
}

/// Benchmark: Fresh mode - allocate new game each iteration
fn bench_game_fresh(c: &mut Criterion) {
    // Check if test resources exist and load once
    let setup = match BenchmarkSetup::load() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping benchmark - failed to load resources: {}", e);
            return;
        }
    };

    let mut group = c.benchmark_group("game_execution");

    // Configure for long-running benchmarks
    group.sample_size(10); // Reduce sample size since games can be long
    group.measurement_time(Duration::from_secs(30)); // 30 seconds per benchmark

    // Benchmark with different seeds to get variance
    for seed in [42u64, 12345u64, 99999u64].iter() {
        // Run a warmup game to print metrics
        println!("\nWarmup game (seed {}):", seed);
        if let Ok(metrics) = run_game_with_metrics(&setup, *seed) {
            println!("  Turns: {}", metrics.turns);
            println!("  Actions: {}", metrics.actions);
            println!("  Duration: {:?}", metrics.duration);
            println!("  Actions/sec: {:.2}", metrics.actions_per_sec());
            println!("  Turns/sec: {:.2}", metrics.turns_per_sec());
            println!("  Actions/turn: {:.2}", metrics.actions_per_turn());
        }

        group.bench_with_input(
            BenchmarkId::new("fresh", seed),
            seed,
            |b, &seed| {
                b.iter(|| {
                    run_game_with_metrics(&setup, black_box(seed))
                        .expect("Game should complete successfully")
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_game_fresh);
criterion_main!(benches);
