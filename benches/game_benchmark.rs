//! Performance benchmarks for MTG Forge game engine
//!
//! This benchmark measures game execution performance using Criterion.rs.
//! It supports three different iteration modes:
//!
//! 1. **Fresh** - Allocate a new game for each iteration
//! 2. **Rewind** - Use undo log to rewind game to start
//! 3. **Snapshot** - Save/restore game state each iteration using Clone
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

/// Default seed for deterministic benchmarks
const DEFAULT_SEED: u64 = 42;

// ============================================================================
// Game Metrics and Statistics
// ============================================================================

/// Metrics collected during a single game execution
#[derive(Debug, Clone)]
struct GameMetrics {
    /// Turns played in this game
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

/// Accumulator for collecting metrics across multiple games
#[derive(Debug, Default)]
struct MetricsAccumulator {
    /// Per-game samples for statistical analysis
    turns_per_game: Vec<f64>,
    actions_per_game: Vec<f64>,
    bytes_per_turn: Vec<f64>,

    /// Aggregated totals
    total_turns: u64,
    total_actions: u64,
    total_bytes_allocated: u64,
    total_bytes_deallocated: u64,
    total_duration: Duration,
    game_count: usize,
}

impl MetricsAccumulator {
    fn new() -> Self {
        Self::default()
    }

    fn add(&mut self, metrics: &GameMetrics) {
        self.turns_per_game.push(metrics.turns as f64);
        self.actions_per_game.push(metrics.actions as f64);
        if metrics.turns > 0 {
            self.bytes_per_turn
                .push(metrics.bytes_allocated as f64 / metrics.turns as f64);
        }

        self.total_turns += metrics.turns as u64;
        self.total_actions += metrics.actions as u64;
        self.total_bytes_allocated += metrics.bytes_allocated as u64;
        self.total_bytes_deallocated += metrics.bytes_deallocated as u64;
        self.total_duration += metrics.duration;
        self.game_count += 1;
    }

    /// Compute mean of a sample
    fn mean(samples: &[f64]) -> f64 {
        if samples.is_empty() {
            return 0.0;
        }
        samples.iter().sum::<f64>() / samples.len() as f64
    }

    /// Compute standard deviation of a sample
    fn std_dev(samples: &[f64], mean: f64) -> f64 {
        if samples.len() < 2 {
            return 0.0;
        }
        let variance =
            samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (samples.len() - 1) as f64;
        variance.sqrt()
    }

    /// Print comprehensive statistics
    fn print_statistics(&self, mode: &str) {
        if self.game_count == 0 {
            return;
        }

        eprintln!(
            "\n=== Benchmark Results: {} Mode ({} games) ===",
            mode, self.game_count
        );
        eprintln!();

        // Throughput metrics (from Criterion's time measurement + our samples)
        let games_per_sec = self.game_count as f64 / self.total_duration.as_secs_f64();
        let mean_turns_per_game = Self::mean(&self.turns_per_game);
        let std_turns_per_game = Self::std_dev(&self.turns_per_game, mean_turns_per_game);
        let turns_per_sec = games_per_sec * mean_turns_per_game;

        let mean_bytes_per_turn = Self::mean(&self.bytes_per_turn);
        let std_bytes_per_turn = Self::std_dev(&self.bytes_per_turn, mean_bytes_per_turn);
        let bytes_per_sec = mean_bytes_per_turn * turns_per_sec;

        eprintln!("Throughput Metrics:");
        eprintln!("  Games/sec:       {:.2}", games_per_sec);
        eprintln!(
            "  Turns/game:      {:.2} ± {:.2}",
            mean_turns_per_game, std_turns_per_game
        );
        eprintln!("  Turns/sec:       {:.2}", turns_per_sec);
        eprintln!(
            "  Bytes/turn:      {:.2} ± {:.2}",
            mean_bytes_per_turn, std_bytes_per_turn
        );
        eprintln!(
            "  Bytes/sec:       {:.2} ({:.2} MB/sec)",
            bytes_per_sec,
            bytes_per_sec / 1_000_000.0
        );
        eprintln!();

        // Context metrics
        let mean_actions_per_game = Self::mean(&self.actions_per_game);
        let actions_per_turn = self.total_actions as f64 / self.total_turns as f64;
        let net_bytes = self.total_bytes_allocated as i64 - self.total_bytes_deallocated as i64;

        eprintln!("Context Metrics:");
        eprintln!("  Total turns:              {}", self.total_turns);
        eprintln!("  Total actions:            {}", self.total_actions);
        eprintln!("  Actions/game:             {:.2}", mean_actions_per_game);
        eprintln!("  Actions/turn:             {:.2}", actions_per_turn);
        eprintln!(
            "  Total bytes allocated:    {} ({:.2} MB)",
            self.total_bytes_allocated,
            self.total_bytes_allocated as f64 / 1_000_000.0
        );
        eprintln!(
            "  Total bytes deallocated:  {} ({:.2} MB)",
            self.total_bytes_deallocated,
            self.total_bytes_deallocated as f64 / 1_000_000.0
        );
        eprintln!(
            "  Net bytes:                {} ({:.2} MB)",
            net_bytes,
            net_bytes as f64 / 1_000_000.0
        );
        eprintln!("  Total duration:           {:?}", self.total_duration);
        eprintln!();
    }
}

// ============================================================================
// Benchmark Setup and Helpers
// ============================================================================

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

    // Run game
    let mut game_loop = GameLoop::new(&mut game).with_verbosity(VerbosityLevel::Silent);
    let result = game_loop.run_game(&mut controller1, &mut controller2)?;

    let duration = start.elapsed();

    // Collect metrics
    let actions = game_loop.game.undo_log.len();
    let stats = reg.change();

    Ok(GameMetrics {
        turns: result.turns_played,
        actions,
        duration,
        bytes_allocated: stats.bytes_allocated,
        bytes_deallocated: stats.bytes_deallocated,
    })
}

/// Run a single game with in-memory logging enabled at Normal verbosity
fn run_game_with_logging<F>(seed: u64, game_init_fn: F) -> Result<GameMetrics>
where
    F: FnOnce() -> Result<mtg_forge_rs::game::GameState>,
{
    let reg = Region::new(GLOBAL);
    let start = std::time::Instant::now();

    let mut game = game_init_fn()?;
    game.rng_seed = seed;
    game.logger.enable_capture();

    let (p1_id, p2_id) = {
        let mut players_iter = game.players.iter().map(|p| p.id);
        (
            players_iter.next().expect("Should have player 1"),
            players_iter.next().expect("Should have player 2"),
        )
    };

    let mut controller1 = RandomController::with_seed(p1_id, seed);
    let mut controller2 = RandomController::with_seed(p2_id, seed + 1);

    let mut game_loop = GameLoop::new(&mut game).with_verbosity(VerbosityLevel::Normal);
    let result = game_loop.run_game(&mut controller1, &mut controller2)?;

    let duration = start.elapsed();
    let actions = game_loop.game.undo_log.len();
    let stats = reg.change();

    Ok(GameMetrics {
        turns: result.turns_played,
        actions,
        duration,
        bytes_allocated: stats.bytes_allocated,
        bytes_deallocated: stats.bytes_deallocated,
    })
}

/// Run a single game with stdout logging at Normal verbosity (redirected to /dev/null)
fn run_game_with_stdout_logging<F>(seed: u64, game_init_fn: F) -> Result<GameMetrics>
where
    F: FnOnce() -> Result<mtg_forge_rs::game::GameState>,
{
    use std::fs::OpenOptions;
    use std::os::fd::AsRawFd;

    let reg = Region::new(GLOBAL);
    let start = std::time::Instant::now();

    let mut game = game_init_fn()?;
    game.rng_seed = seed;

    // Redirect stdout to /dev/null
    let devnull = OpenOptions::new()
        .write(true)
        .open("/dev/null")
        .expect("Failed to open /dev/null");
    let orig_stdout = unsafe { libc::dup(libc::STDOUT_FILENO) };
    unsafe {
        libc::dup2(devnull.as_raw_fd(), libc::STDOUT_FILENO);
    }

    let (p1_id, p2_id) = {
        let mut players_iter = game.players.iter().map(|p| p.id);
        (
            players_iter.next().expect("Should have player 1"),
            players_iter.next().expect("Should have player 2"),
        )
    };

    let mut controller1 = RandomController::with_seed(p1_id, seed);
    let mut controller2 = RandomController::with_seed(p2_id, seed + 1);

    let mut game_loop = GameLoop::new(&mut game).with_verbosity(VerbosityLevel::Normal);
    let result = game_loop.run_game(&mut controller1, &mut controller2)?;

    // Restore stdout
    unsafe {
        libc::dup2(orig_stdout, libc::STDOUT_FILENO);
        libc::close(orig_stdout);
    }

    let duration = start.elapsed();
    let actions = game_loop.game.undo_log.len();
    let stats = reg.change();

    Ok(GameMetrics {
        turns: result.turns_played,
        actions,
        duration,
        bytes_allocated: stats.bytes_allocated,
        bytes_deallocated: stats.bytes_deallocated,
    })
}

// ============================================================================
// Benchmark Functions
// ============================================================================

/// Benchmark: Fresh mode - allocate new game each iteration
fn bench_game_fresh(c: &mut Criterion) {
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

    let mut metrics_acc = MetricsAccumulator::new();

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

            let metrics = run_game_with_metrics(black_box(DEFAULT_SEED), game_init_fn)
                .expect("Game should complete successfully");
            metrics_acc.add(&metrics);
        });
    });
    group.finish();

    metrics_acc.print_statistics("Fresh");
}

/// Benchmark: Fresh mode with in-memory logging
fn bench_game_fresh_with_logging(c: &mut Criterion) {
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

    let mut metrics_acc = MetricsAccumulator::new();

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

            let metrics = run_game_with_logging(black_box(DEFAULT_SEED), game_init_fn)
                .expect("Game should complete successfully");
            metrics_acc.add(&metrics);
        });
    });
    group.finish();

    metrics_acc.print_statistics("Fresh with Logging");
}

/// Benchmark: Fresh mode with stdout logging (redirected to /dev/null)
fn bench_game_fresh_with_stdout_logging(c: &mut Criterion) {
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

    let mut metrics_acc = MetricsAccumulator::new();

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

            let metrics = run_game_with_stdout_logging(black_box(DEFAULT_SEED), game_init_fn)
                .expect("Game should complete successfully");
            metrics_acc.add(&metrics);
        });
    });
    group.finish();

    metrics_acc.print_statistics("Fresh with Stdout Logging");
}

/// Benchmark: Snapshot mode - save/restore game state each iteration using Clone
fn bench_game_snapshot(c: &mut Criterion) {
    let setup = match BenchmarkSetup::load() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping benchmark - failed to load resources: {e}");
            return;
        }
    };

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

    eprintln!("\nSnapshot mode: Pre-creating initial game state for cloning...");

    let mut group = c.benchmark_group("game_execution");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(BENCHMARK_TIME_SECS));

    let mut metrics_acc = MetricsAccumulator::new();

    group.bench_function("snapshot", |b| {
        b.iter(|| {
            let game_init_fn = || Ok(initial_game.clone());
            let metrics = run_game_with_metrics(DEFAULT_SEED, game_init_fn)
                .expect("Game should complete successfully");
            metrics_acc.add(&metrics);
        });
    });
    group.finish();

    metrics_acc.print_statistics("Snapshot");
}

/// Benchmark: Rewind mode - use undo log to rewind game
fn bench_game_rewind(c: &mut Criterion) {
    let setup = match BenchmarkSetup::load() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping benchmark - failed to load resources: {e}");
            return;
        }
    };

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

    initial_game.rng_seed = DEFAULT_SEED;

    // Play the game once to build the undo log
    {
        let (p1_id, p2_id) = {
            let mut players_iter = initial_game.players.iter().map(|p| p.id);
            (
                players_iter.next().expect("Should have player 1"),
                players_iter.next().expect("Should have player 2"),
            )
        };

        let mut controller1 = RandomController::with_seed(p1_id, DEFAULT_SEED);
        let mut controller2 = RandomController::with_seed(p2_id, DEFAULT_SEED + 1);

        let mut game_loop = GameLoop::new(&mut initial_game).with_verbosity(VerbosityLevel::Silent);
        let _ = game_loop
            .run_game(&mut controller1, &mut controller2)
            .expect("Initial game should complete");
    }

    let actions_count = initial_game.undo_log.len();
    eprintln!(
        "\nRewind mode: Game completed with {} actions in undo log",
        actions_count
    );
    eprintln!("  Will rewind to start for each iteration...");

    let mut group = c.benchmark_group("game_execution");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(BENCHMARK_TIME_SECS));

    let mut metrics_acc = MetricsAccumulator::new();

    group.bench_function("rewind", |b| {
        b.iter(|| {
            let reg = Region::new(GLOBAL);
            let start = std::time::Instant::now();

            // Rewind all actions
            let mut rewind_count = 0;
            while initial_game.undo().expect("Undo should succeed") {
                rewind_count += 1;
            }

            let duration = start.elapsed();
            let stats = reg.change();

            let metrics = GameMetrics {
                turns: 18, // Known from fresh run
                actions: rewind_count,
                duration,
                bytes_allocated: stats.bytes_allocated,
                bytes_deallocated: stats.bytes_deallocated,
            };
            metrics_acc.add(&metrics);

            // Re-run the game to populate undo log for next iteration
            {
                let (p1_id, p2_id) = {
                    let mut players_iter = initial_game.players.iter().map(|p| p.id);
                    (
                        players_iter.next().expect("Should have player 1"),
                        players_iter.next().expect("Should have player 2"),
                    )
                };

                let mut controller1 = RandomController::with_seed(p1_id, DEFAULT_SEED);
                let mut controller2 = RandomController::with_seed(p2_id, DEFAULT_SEED + 1);

                let mut game_loop =
                    GameLoop::new(&mut initial_game).with_verbosity(VerbosityLevel::Silent);
                let _ = game_loop
                    .run_game(&mut controller1, &mut controller2)
                    .expect("Game should complete");
            }
        });
    });
    group.finish();

    metrics_acc.print_statistics("Rewind");
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
