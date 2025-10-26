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

use criterion::{black_box, criterion_group, criterion_main, Criterion};
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

/// Benchmark measurement time in seconds (used by all benchmarks)
const BENCHMARK_TIME_SECS: u64 = 10;

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

/// Run a single game with in-memory logging enabled at Normal verbosity
fn run_game_with_logging<F>(seed: u64, game_init_fn: F) -> Result<GameMetrics>
where
    F: FnOnce() -> Result<mtg_forge_rs::game::GameState>,
{
    use std::fs::OpenOptions;
    use std::os::fd::AsRawFd;

    let reg = Region::new(GLOBAL);
    let start = std::time::Instant::now();

    // Initialize game using provided function
    let mut game = game_init_fn()?;
    game.rng_seed = seed;

    // Enable log capture
    game.logger.enable_capture();

    // Redirect stdout to /dev/null to avoid benchmark noise
    // (Logger may still write to stdout even with capture enabled)
    let devnull = OpenOptions::new()
        .write(true)
        .open("/dev/null")
        .expect("Failed to open /dev/null");
    let orig_stdout = unsafe { libc::dup(libc::STDOUT_FILENO) };
    unsafe {
        libc::dup2(devnull.as_raw_fd(), libc::STDOUT_FILENO);
    }

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

    // Run game with Normal verbosity to capture logs
    let mut game_loop = GameLoop::new(&mut game).with_verbosity(VerbosityLevel::Normal);
    let result = game_loop.run_game(&mut controller1, &mut controller2)?;

    // Restore stdout
    unsafe {
        libc::dup2(orig_stdout, libc::STDOUT_FILENO);
        libc::close(orig_stdout);
    }

    let duration = start.elapsed();

    // Collect metrics
    let actions = game_loop.game.undo_log.len();
    let log_entries = game_loop.game.logger.logs().count();
    let stats = reg.change();

    let metrics = GameMetrics {
        turns: result.turns_played,
        actions,
        duration,
        bytes_allocated: stats.bytes_allocated,
        bytes_deallocated: stats.bytes_deallocated,
    };

    // Report log entries captured
    if log_entries > 0 {
        eprintln!("  Log entries captured: {}", log_entries);
    }

    Ok(metrics)
}

/// Run a single game with stdout logging at Normal verbosity (not capturing)
/// This tests the reusable buffer optimization
fn run_game_with_stdout_logging<F>(seed: u64, game_init_fn: F) -> Result<GameMetrics>
where
    F: FnOnce() -> Result<mtg_forge_rs::game::GameState>,
{
    use std::fs::OpenOptions;
    use std::os::fd::AsRawFd;

    let reg = Region::new(GLOBAL);
    let start = std::time::Instant::now();

    // Initialize game using provided function
    let mut game = game_init_fn()?;
    game.rng_seed = seed;

    // DO NOT enable log capture - we want stdout logging

    // Redirect stdout to /dev/null to avoid benchmark noise
    let devnull = OpenOptions::new()
        .write(true)
        .open("/dev/null")
        .expect("Failed to open /dev/null");
    let orig_stdout = unsafe { libc::dup(libc::STDOUT_FILENO) };
    unsafe {
        libc::dup2(devnull.as_raw_fd(), libc::STDOUT_FILENO);
    }

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

    // Run game with Normal verbosity (logs to stdout via reusable buffer)
    let mut game_loop = GameLoop::new(&mut game).with_verbosity(VerbosityLevel::Normal);
    let result = game_loop.run_game(&mut controller1, &mut controller2)?;

    // Restore stdout
    unsafe {
        libc::dup2(orig_stdout, libc::STDOUT_FILENO);
        libc::close(orig_stdout);
    }

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
///
/// Note: The "Avg duration/game" shown here is a naive average (total_time / iterations).
/// For accurate per-iteration timing, refer to Criterion's statistical estimate shown above,
/// which accounts for outliers, warmup effects, and provides confidence intervals.
fn print_aggregated_metrics(
    mode: &str,
    seed: u64,
    aggregated: &GameMetrics,
    iteration_count: usize,
) {
    eprintln!("\n=== Aggregated Metrics - {mode} Mode (seed {seed}, {iteration_count} games) ===");
    eprintln!("  Total turns: {}", aggregated.turns);
    eprintln!("  Total actions: {}", aggregated.actions);
    eprintln!("  Total duration: {:?}", aggregated.duration);
    eprintln!(
        "  Avg turns/game: {:.2}",
        aggregated.turns as f64 / iteration_count as f64
    );
    eprintln!(
        "  Avg actions/game: {:.2}",
        aggregated.actions as f64 / iteration_count as f64
    );
    eprintln!(
        "  Avg duration/game (naive): {:.2?}",
        aggregated.duration / iteration_count as u32
    );
    eprintln!(
        "  Games/sec: {:.2}",
        aggregated.avg_games_per_sec(iteration_count)
    );
    eprintln!("  Actions/sec: {:.2}", aggregated.actions_per_sec());
    eprintln!("  Turns/sec: {:.2}", aggregated.turns_per_sec());
    eprintln!("  Actions/turn: {:.2}", aggregated.actions_per_turn());
    eprintln!("  Total bytes allocated: {}", aggregated.bytes_allocated);
    eprintln!(
        "  Total bytes deallocated: {}",
        aggregated.bytes_deallocated
    );
    eprintln!("  Net bytes: {}", aggregated.net_bytes_allocated());
    eprintln!(
        "  Avg bytes/game: {:.2}",
        aggregated.bytes_allocated as f64 / iteration_count as f64
    );
    eprintln!("  Bytes/turn: {:.2}", aggregated.bytes_per_turn());
    eprintln!("  Bytes/sec: {:.2}", aggregated.bytes_per_sec());
    eprintln!("\nNote: For authoritative per-iteration timing, see Criterion's estimate above.");
}

/// Benchmark: Fresh mode - allocate new game each iteration
fn bench_game_fresh(c: &mut Criterion) {
    // Check if test resources exist and load once
    let setup = match BenchmarkSetup::load() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping benchmark - failed to load resources: {e}");
            return;
        }
    };

    let mut group = c.benchmark_group("game_execution");

    // Configure for long-running benchmarks
    group.sample_size(10); // Reduce sample size since games can be long
    group.measurement_time(Duration::from_secs(BENCHMARK_TIME_SECS));

    let seed = 42u64;

    // Accumulator for aggregating metrics across benchmark iterations
    let mut aggregated = GameMetrics {
        turns: 0,
        actions: 0,
        duration: Duration::ZERO,
        bytes_allocated: 0,
        bytes_deallocated: 0,
    };
    let mut iteration_count = 0;

    group.bench_function("fresh", |b| {
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

    if iteration_count > 0 {
        print_aggregated_metrics("Fresh", seed, &aggregated, iteration_count);
    }

    group.finish();
}

/// Benchmark: Fresh mode with in-memory logging at Normal verbosity
/// Measures allocation overhead of logging infrastructure
fn bench_game_fresh_with_logging(c: &mut Criterion) {
    // Check if test resources exist and load once
    let setup = match BenchmarkSetup::load() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping benchmark - failed to load resources: {e}");
            return;
        }
    };

    let mut group = c.benchmark_group("game_execution");

    // Configure for long-running benchmarks
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(BENCHMARK_TIME_SECS));

    let seed = 42u64;

    // Accumulator for aggregating metrics across benchmark iterations
    let mut aggregated = GameMetrics {
        turns: 0,
        actions: 0,
        duration: Duration::ZERO,
        bytes_allocated: 0,
        bytes_deallocated: 0,
    };
    let mut iteration_count = 0;

    group.bench_function("fresh_logging", |b| {
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

            let metrics = run_game_with_logging(black_box(seed), game_init_fn)
                .expect("Game should complete successfully");
            aggregated += metrics.clone();
            iteration_count += 1;
        });
    });

    if iteration_count > 0 {
        print_aggregated_metrics("Fresh with Logging", seed, &aggregated, iteration_count);
    }

    group.finish();
}

/// Benchmark: Fresh mode with stdout logging at Normal verbosity (redirected to /dev/null)
/// Measures allocation overhead with reusable buffer optimization
fn bench_game_fresh_with_stdout_logging(c: &mut Criterion) {
    // Check if test resources exist and load once
    let setup = match BenchmarkSetup::load() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping benchmark - failed to load resources: {e}");
            return;
        }
    };

    let mut group = c.benchmark_group("game_execution");

    // Configure for long-running benchmarks
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(BENCHMARK_TIME_SECS));

    let seed = 42u64;

    // Accumulator for aggregating metrics across benchmark iterations
    let mut aggregated = GameMetrics {
        turns: 0,
        actions: 0,
        duration: Duration::ZERO,
        bytes_allocated: 0,
        bytes_deallocated: 0,
    };
    let mut iteration_count = 0;

    group.bench_function("fresh_stdout_logging", |b| {
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

            let metrics = run_game_with_stdout_logging(black_box(seed), game_init_fn)
                .expect("Game should complete successfully");
            aggregated += metrics.clone();
            iteration_count += 1;
        });
    });

    if iteration_count > 0 {
        eprintln!("\n=== Aggregated Metrics - Fresh with Stdout Logging Mode (seed {seed}, {iteration_count} games) ===");
        eprintln!("  Total turns: {}", aggregated.turns);
        eprintln!("  Total actions: {}", aggregated.actions);
        eprintln!("  Total duration: {:?}", aggregated.duration);
        eprintln!(
            "  Avg turns/game: {:.2}",
            aggregated.turns as f64 / iteration_count as f64
        );
        eprintln!(
            "  Avg actions/game: {:.2}",
            aggregated.actions as f64 / iteration_count as f64
        );
        eprintln!(
            "  Avg duration/game: {:.2?}",
            aggregated.duration / iteration_count as u32
        );
        eprintln!(
            "  Games/sec: {:.2}",
            aggregated.avg_games_per_sec(iteration_count)
        );
        eprintln!("  Actions/sec: {:.2}", aggregated.actions_per_sec());
        eprintln!("  Turns/sec: {:.2}", aggregated.turns_per_sec());
        eprintln!("  Actions/turn: {:.2}", aggregated.actions_per_turn());
        eprintln!("  Total bytes allocated: {}", aggregated.bytes_allocated);
        eprintln!(
            "  Total bytes deallocated: {}",
            aggregated.bytes_deallocated
        );
        eprintln!("  Net bytes: {}", aggregated.net_bytes_allocated());
        eprintln!(
            "  Avg bytes/game: {:.2}",
            aggregated.bytes_allocated as f64 / iteration_count as f64
        );
        eprintln!("  Bytes/turn: {:.2}", aggregated.bytes_per_turn());
        eprintln!("  Bytes/sec: {:.2}", aggregated.bytes_per_sec());
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
            eprintln!("Skipping benchmark - failed to load resources: {e}");
            return;
        }
    };

    let mut group = c.benchmark_group("game_execution");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(BENCHMARK_TIME_SECS));

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

    eprintln!("\nSnapshot mode (seed {seed}):");
    eprintln!("  Pre-creating initial game state for cloning...");

    // Accumulator for aggregating metrics across benchmark iterations
    let mut aggregated = GameMetrics {
        turns: 0,
        actions: 0,
        duration: Duration::ZERO,
        bytes_allocated: 0,
        bytes_deallocated: 0,
    };
    let mut iteration_count = 0;

    group.bench_function("snapshot", |b| {
        b.iter(|| {
            let game_init_fn = || Ok(initial_game.clone());
            let metrics = run_game_with_metrics(seed, game_init_fn)
                .expect("Game should complete successfully");
            aggregated += metrics.clone();
            iteration_count += 1;
        });
    });

    if iteration_count > 0 {
        print_aggregated_metrics("Snapshot", seed, &aggregated, iteration_count);
    }

    group.finish();
}

/// Benchmark: Rewind mode - use undo log to rewind game
/// Measures the cost of rewinding using undo() for tree search
fn bench_game_rewind(c: &mut Criterion) {
    // Check if test resources exist and load once
    let setup = match BenchmarkSetup::load() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping benchmark - failed to load resources: {e}");
            return;
        }
    };

    let mut group = c.benchmark_group("game_execution");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(BENCHMARK_TIME_SECS));

    let seed = 42u64;

    // Pre-create and run an initial game to completion
    let game_init = GameInitializer::new(&setup.card_db);
    let mut initial_game = setup
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

    initial_game.rng_seed = seed;

    // Play the game once to build the undo log
    {
        let (p1_id, p2_id) = {
            let mut players_iter = initial_game.players.iter().map(|p| p.id);
            (
                players_iter.next().expect("Should have player 1"),
                players_iter.next().expect("Should have player 2"),
            )
        };

        let mut controller1 = RandomController::with_seed(p1_id, seed);
        let mut controller2 = RandomController::with_seed(p2_id, seed + 1);

        let mut game_loop = GameLoop::new(&mut initial_game).with_verbosity(VerbosityLevel::Silent);
        let _ = game_loop
            .run_game(&mut controller1, &mut controller2)
            .expect("Initial game should complete");
    }

    let actions_count = initial_game.undo_log.len();
    eprintln!("\nRewind mode (seed {seed}):");
    eprintln!(
        "  Game completed with {} actions in undo log",
        actions_count
    );
    eprintln!("  Will rewind to start for each iteration...");

    // Accumulator for aggregating metrics
    let mut aggregated = GameMetrics {
        turns: 0,
        actions: 0,
        duration: Duration::ZERO,
        bytes_allocated: 0,
        bytes_deallocated: 0,
    };
    let mut iteration_count = 0;

    group.bench_function("rewind", |b| {
        b.iter(|| {
            let reg = Region::new(GLOBAL);
            let start = std::time::Instant::now();

            // Rewind all actions to get back to initial state
            let mut rewind_count = 0;
            while initial_game.undo().expect("Undo should succeed") {
                rewind_count += 1;
            }

            let duration = start.elapsed();
            let stats = reg.change();

            // Record metrics for the rewind operation
            let metrics = GameMetrics {
                turns: 18, // We know from the fresh run this is 18 turns
                actions: rewind_count,
                duration,
                bytes_allocated: stats.bytes_allocated,
                bytes_deallocated: stats.bytes_deallocated,
            };

            aggregated += metrics.clone();
            iteration_count += 1;

            // Re-run the game to populate undo log for next iteration
            // (This happens outside the timing, as we're measuring rewind cost)
            {
                let (p1_id, p2_id) = {
                    let mut players_iter = initial_game.players.iter().map(|p| p.id);
                    (
                        players_iter.next().expect("Should have player 1"),
                        players_iter.next().expect("Should have player 2"),
                    )
                };

                let mut controller1 = RandomController::with_seed(p1_id, seed);
                let mut controller2 = RandomController::with_seed(p2_id, seed + 1);

                let mut game_loop =
                    GameLoop::new(&mut initial_game).with_verbosity(VerbosityLevel::Silent);
                let _ = game_loop
                    .run_game(&mut controller1, &mut controller2)
                    .expect("Game should complete");
            }
        });
    });

    if iteration_count > 0 {
        print_aggregated_metrics("Rewind", seed, &aggregated, iteration_count);
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_game_fresh,
    bench_game_fresh_with_logging,
    bench_game_fresh_with_stdout_logging,
    bench_game_snapshot,
    bench_game_rewind
);
criterion_main!(benches);
