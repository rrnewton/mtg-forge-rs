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
    loader::{
        prefetch_deck_cards, AsyncCardDatabase as CardDatabase, DeckList, DeckLoader,
        GameInitializer,
    },
    Result,
};
use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};
use std::alloc::System;
use std::path::PathBuf;
use std::time::Duration;
use tokio::runtime::Runtime;

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

/// Metrics collected during game execution
#[derive(Debug, Clone)]
struct GameMetrics {
    /// Total turns played
    turns: u32,
    /// Total actions (from UndoLog)
    actions: usize,
    /// Game duration
    duration: Duration,
    /// Bytes allocated during game execution
    bytes_allocated: usize,
    /// Bytes deallocated during game execution
    bytes_deallocated: usize,
}

impl GameMetrics {
    /// Calculate games per second
    fn games_per_sec(&self) -> f64 {
        1.0 / self.duration.as_secs_f64()
    }

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

    /// Calculate net bytes allocated (allocated - deallocated)
    fn net_bytes_allocated(&self) -> i64 {
        self.bytes_allocated as i64 - self.bytes_deallocated as i64
    }

    /// Calculate bytes allocated per turn
    fn bytes_per_turn(&self) -> f64 {
        if self.turns == 0 {
            0.0
        } else {
            self.bytes_allocated as f64 / self.turns as f64
        }
    }

    /// Calculate bytes allocated per second
    fn bytes_per_sec(&self) -> f64 {
        self.bytes_allocated as f64 / self.duration.as_secs_f64()
    }
}

/// Setup data needed for benchmarking (loaded once, reused across iterations)
struct BenchmarkSetup {
    card_db: CardDatabase,
    deck: DeckList,
    runtime: Runtime,
}

impl BenchmarkSetup {
    fn load() -> Result<Self> {
        let runtime = Runtime::new().expect("Failed to create tokio runtime");

        let cardsfolder = PathBuf::from("cardsfolder");
        let card_db = CardDatabase::new(cardsfolder);

        let deck_path = PathBuf::from("test_decks/simple_bolt.dck");
        let deck = DeckLoader::load_from_file(&deck_path)?;

        // Prefetch deck cards
        runtime.block_on(async { prefetch_deck_cards(&card_db, &deck).await })?;

        Ok(BenchmarkSetup {
            card_db,
            deck,
            runtime,
        })
    }
}

/// Run a single game and collect metrics
/// Takes pre-loaded setup data to avoid measuring file I/O
fn run_game_with_metrics(setup: &BenchmarkSetup, seed: u64) -> Result<GameMetrics> {
    let reg = Region::new(GLOBAL);
    let start = std::time::Instant::now();

    // Initialize game
    let game_init = GameInitializer::new(&setup.card_db);
    let mut game = setup.runtime.block_on(async {
        game_init
            .init_game(
                "Player 1".to_string(),
                &setup.deck,
                "Player 2".to_string(),
                &setup.deck,
                20,
            )
            .await
    })?;
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
    let stats = reg.change();

    let metrics = GameMetrics {
        turns: result.turns_played,
        actions,
        duration,
        bytes_allocated: stats.bytes_allocated,
        bytes_deallocated: stats.bytes_deallocated,
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

    // Previously also used seeds 12345u64, 99999u64, but behavior is similar.
    let seed = 42u64;
    {
        // Run a warmup game to print metrics
        println!("\nWarmup game (seed {}):", seed);
        if let Ok(metrics) = run_game_with_metrics(&setup, seed) {
            println!("  Turns: {}", metrics.turns);
            println!("  Actions: {}", metrics.actions);
            println!("  Duration: {:?}", metrics.duration);
            println!("  Games/sec: {:.2}", metrics.games_per_sec());
            println!("  Actions/sec: {:.2}", metrics.actions_per_sec());
            println!("  Turns/sec: {:.2}", metrics.turns_per_sec());
            println!("  Actions/turn: {:.2}", metrics.actions_per_turn());
            println!("  Bytes allocated: {}", metrics.bytes_allocated);
            println!("  Bytes deallocated: {}", metrics.bytes_deallocated);
            println!("  Net bytes: {}", metrics.net_bytes_allocated());
            println!("  Bytes/turn: {:.2}", metrics.bytes_per_turn());
            println!("  Bytes/sec: {:.2}", metrics.bytes_per_sec());
        }

        group.bench_with_input(BenchmarkId::new("fresh", seed), &seed, |b, &seed| {
            b.iter(|| {
                run_game_with_metrics(&setup, black_box(seed))
                    .expect("Game should complete successfully")
            });
        });
    }

    group.finish();
}

/// Benchmark: Snapshot mode - save/restore game state each iteration
/// Uses Clone to create a fresh copy of the initial game state
fn bench_game_snapshot(c: &mut Criterion) {
    // Check if test resources exist and load once
    let setup = match BenchmarkSetup::load() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping benchmark - failed to load resources: {}", e);
            return;
        }
    };

    let mut group = c.benchmark_group("game_execution");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));

    // Use single seed for snapshot mode (comparing with fresh mode)
    let seed = 42u64;

    // Pre-create the initial game state (the "snapshot")
    let game_init = GameInitializer::new(&setup.card_db);
    let initial_game = setup
        .runtime
        .block_on(async {
            game_init
                .init_game(
                    "Player 1".to_string(),
                    &setup.deck,
                    "Player 2".to_string(),
                    &setup.deck,
                    20,
                )
                .await
        })
        .expect("Failed to initialize game");

    println!("\nSnapshot mode (seed {}):", seed);
    println!("  Pre-creating initial game state for cloning...");

    group.bench_function(BenchmarkId::new("snapshot", seed), |b| {
        b.iter(|| {
            // Clone the initial game state (this is the "restore" part)
            let mut game = initial_game.clone();
            game.rng_seed = seed;

            let players: Vec<_> = game.players.iter().map(|(id, _)| *id).collect();
            let p1_id = players[0];
            let p2_id = players[1];

            let mut controller1 = RandomController::with_seed(p1_id, seed);
            let mut controller2 = RandomController::with_seed(p2_id, seed + 1);

            let mut game_loop = GameLoop::new(&mut game).with_verbose(false);
            game_loop
                .run_game(&mut controller1, &mut controller2)
                .expect("Game should complete successfully")
        });
    });

    group.finish();
}

/// Benchmark: Rewind mode - use undo log to rewind game (NOT YET IMPLEMENTED)
/// This requires implementing GameState::undo() functionality
fn bench_game_rewind(_c: &mut Criterion) {
    eprintln!("\n=== Rewind mode benchmark NOT YET IMPLEMENTED ===");
    eprintln!("Requires implementing GameState::undo() to replay actions in reverse");
    eprintln!("See src/undo.rs for the UndoLog infrastructure");
    eprintln!("TODO: Implement undo() method that processes GameAction in reverse");
}

criterion_group!(
    benches,
    bench_game_fresh,
    bench_game_snapshot,
    bench_game_rewind
);
criterion_main!(benches);
