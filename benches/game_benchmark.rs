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
    game::{random_controller::RandomController, GameLoop, VerbosityLevel},
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

    /// Calculate average games per second (for aggregated metrics)
    fn avg_games_per_sec(&self, num_games: usize) -> f64 {
        num_games as f64 / self.duration.as_secs_f64()
    }
}

/// Implement addition for GameMetrics to support aggregation
impl std::ops::Add for GameMetrics {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        GameMetrics {
            turns: self.turns + other.turns,
            actions: self.actions + other.actions,
            duration: self.duration + other.duration,
            bytes_allocated: self.bytes_allocated + other.bytes_allocated,
            bytes_deallocated: self.bytes_deallocated + other.bytes_deallocated,
        }
    }
}

impl std::ops::AddAssign for GameMetrics {
    fn add_assign(&mut self, other: Self) {
        self.turns += other.turns;
        self.actions += other.actions;
        self.duration += other.duration;
        self.bytes_allocated += other.bytes_allocated;
        self.bytes_deallocated += other.bytes_deallocated;
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
/// Takes a game initializer function to support different initialization strategies
fn run_game_with_metrics<F>(seed: u64, game_init_fn: F) -> Result<GameMetrics>
where
    F: FnOnce() -> Result<mtg_forge_rs::game::GameState>,
{
    let reg = Region::new(GLOBAL);
    let start = std::time::Instant::now();

    // Initialize game using provided function
    let mut game = game_init_fn()?;
    game.rng_seed = seed;

    // Create random controllers
    let (p1_id, p2_id) = {
        let mut players_iter = game.players.iter().map(|p| p.id);
        (
            players_iter.next().expect("Should have player 1"),
            players_iter.next().expect("Should have player 2"),
        )
    };

    let mut controller1 = RandomController::with_seed(p1_id, seed);
    let mut controller2 = RandomController::with_seed(p2_id, seed + 1);

    // Run game (still within timing)
    let mut game_loop = GameLoop::new(&mut game).with_verbosity(VerbosityLevel::Silent);
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

/// Helper function to print aggregated metrics
fn print_aggregated_metrics(
    mode: &str,
    seed: u64,
    aggregated: &GameMetrics,
    iteration_count: usize,
) {
    println!(
        "\n=== Aggregated Metrics - {} Mode (seed {}, {} games) ===",
        mode, seed, iteration_count
    );
    println!("  Total turns: {}", aggregated.turns);
    println!("  Total actions: {}", aggregated.actions);
    println!("  Total duration: {:?}", aggregated.duration);
    println!(
        "  Avg turns/game: {:.2}",
        aggregated.turns as f64 / iteration_count as f64
    );
    println!(
        "  Avg actions/game: {:.2}",
        aggregated.actions as f64 / iteration_count as f64
    );
    println!(
        "  Avg duration/game: {:.2?}",
        aggregated.duration / iteration_count as u32
    );
    println!(
        "  Games/sec: {:.2}",
        aggregated.avg_games_per_sec(iteration_count)
    );
    println!("  Actions/sec: {:.2}", aggregated.actions_per_sec());
    println!("  Turns/sec: {:.2}", aggregated.turns_per_sec());
    println!("  Actions/turn: {:.2}", aggregated.actions_per_turn());
    println!("  Total bytes allocated: {}", aggregated.bytes_allocated);
    println!(
        "  Total bytes deallocated: {}",
        aggregated.bytes_deallocated
    );
    println!("  Net bytes: {}", aggregated.net_bytes_allocated());
    println!(
        "  Avg bytes/game: {:.2}",
        aggregated.bytes_allocated as f64 / iteration_count as f64
    );
    println!("  Bytes/turn: {:.2}", aggregated.bytes_per_turn());
    println!("  Bytes/sec: {:.2}", aggregated.bytes_per_sec());
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
    group.measurement_time(Duration::from_secs(10)); // 30 seconds per benchmark

    let seed = 42u64;

    // Run a warmup game to print metrics
    println!("\nWarmup game - Fresh mode (seed {}):", seed);
    let game_init_fn = || {
        let game_init = GameInitializer::new(&setup.card_db);
        setup.runtime.block_on(async {
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
    };

    if let Ok(metrics) = run_game_with_metrics(seed, game_init_fn) {
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

    // Accumulator for aggregating metrics across benchmark iterations
    let mut aggregated = GameMetrics {
        turns: 0,
        actions: 0,
        duration: Duration::ZERO,
        bytes_allocated: 0,
        bytes_deallocated: 0,
    };
    let mut iteration_count = 0;

    group.bench_with_input(BenchmarkId::new("fresh", seed), &seed, |b, &seed| {
        b.iter(|| {
            let game_init_fn = || {
                let game_init = GameInitializer::new(&setup.card_db);
                setup.runtime.block_on(async {
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
            };

            let metrics = run_game_with_metrics(black_box(seed), game_init_fn)
                .expect("Game should complete successfully");
            aggregated += metrics.clone();
            iteration_count += 1;
        });
    });

    print_aggregated_metrics("Fresh", seed, &aggregated, iteration_count);

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

    // Accumulator for aggregating metrics across benchmark iterations
    let mut aggregated = GameMetrics {
        turns: 0,
        actions: 0,
        duration: Duration::ZERO,
        bytes_allocated: 0,
        bytes_deallocated: 0,
    };
    let mut iteration_count = 0;

    group.bench_function(BenchmarkId::new("snapshot", seed), |b| {
        b.iter(|| {
            let game_init_fn = || Ok(initial_game.clone());
            let metrics = run_game_with_metrics(seed, game_init_fn)
                .expect("Game should complete successfully");
            aggregated += metrics.clone();
            iteration_count += 1;
        });
    });

    print_aggregated_metrics("Snapshot", seed, &aggregated, iteration_count);

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
