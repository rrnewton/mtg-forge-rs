//! MTG Forge Rust - Main Binary
//!
//! Text-based Magic: The Gathering game engine with TUI support

use clap::{Parser, Subcommand, ValueEnum};
use mtg_forge_rs::{
    game::{
        random_controller::RandomController, zero_controller::ZeroController,
        FixedScriptController, GameLoop, GameSnapshot, HeuristicController, InteractiveController,
        StopCondition, VerbosityLevel,
    },
    loader::{AsyncCardDatabase as CardDatabase, DeckLoader, GameInitializer},
    puzzle::{loader::load_puzzle_into_game, PuzzleFile},
    Result,
};
use std::path::PathBuf;

/// Controller type for AI agents
#[derive(Debug, Clone, Copy, ValueEnum)]
enum ControllerType {
    /// Always chooses first meaningful action (for testing)
    Zero,
    /// Makes random choices
    Random,
    /// Text UI controller for human play via stdin
    Tui,
    /// Heuristic AI controller with strategic decision making
    Heuristic,
    /// Fixed script controller with predetermined choices (requires --fixed-inputs)
    Fixed,
}

/// Verbosity level for game output (custom parser supporting both names and numbers)
#[derive(Debug, Clone, Copy)]
struct VerbosityArg(VerbosityLevel);

impl std::str::FromStr for VerbosityArg {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "silent" | "0" => Ok(VerbosityArg(VerbosityLevel::Silent)),
            "minimal" | "1" => Ok(VerbosityArg(VerbosityLevel::Minimal)),
            "normal" | "2" => Ok(VerbosityArg(VerbosityLevel::Normal)),
            "verbose" | "3" => Ok(VerbosityArg(VerbosityLevel::Verbose)),
            _ => Err(format!(
                "invalid verbosity level '{s}' (expected: silent/0, minimal/1, normal/2, verbose/3)"
            )),
        }
    }
}

impl From<VerbosityArg> for VerbosityLevel {
    fn from(arg: VerbosityArg) -> Self {
        arg.0
    }
}

#[derive(Parser)]
#[command(name = "mtg")]
#[command(about = "MTG Forge Rust - Magic: The Gathering Game Engine", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
enum Commands {
    /// Text UI Mode - Interactive Forge Gameplay
    Tui {
        /// Deck file (.dck) for player 1 (required unless --start-state or --start-from is provided)
        #[arg(value_name = "PLAYER1_DECK", required_unless_present_any = ["start_state", "start_from"])]
        deck1: Option<PathBuf>,

        /// Deck file (.dck) for player 2 (required unless --start-state or --start-from is provided)
        #[arg(value_name = "PLAYER2_DECK", required_unless_present_any = ["start_state", "start_from"])]
        deck2: Option<PathBuf>,

        /// Load game state from puzzle file (.pzl)
        #[arg(long, value_name = "PUZZLE_FILE")]
        start_state: Option<PathBuf>,

        /// Player 1 controller type
        #[arg(long, value_enum, default_value = "random")]
        p1: ControllerType,

        /// Player 2 controller type
        #[arg(long, value_enum, default_value = "random")]
        p2: ControllerType,

        /// Player 1 name (default: Alice)
        #[arg(long, default_value = "Alice")]
        p1_name: String,

        /// Player 2 name (default: Bob)
        #[arg(long, default_value = "Bob")]
        p2_name: String,

        /// Fixed script input for player 1 (space or comma separated indices, e.g., "1 1 2" or "1,1,2")
        #[arg(long, value_name = "CHOICES")]
        p1_fixed_inputs: Option<String>,

        /// Fixed script input for player 2 (space or comma separated indices, e.g., "1 1 2" or "1,1,2")
        #[arg(long, value_name = "CHOICES")]
        p2_fixed_inputs: Option<String>,

        /// Set random seed for deterministic testing
        #[arg(long)]
        seed: Option<u64>,

        /// Load all cards from cardsfolder (default: only load cards in decks)
        #[arg(long)]
        load_all_cards: bool,

        /// Verbosity level for game output (0=silent, 1=minimal, 2=normal, 3=verbose)
        #[arg(long, default_value = "normal", short = 'v')]
        verbosity: VerbosityArg,

        /// Use numeric-only choice format (for comparison with Java Forge)
        #[arg(long)]
        numeric_choices: bool,

        /// Stop after N choices by specified player(s) and save snapshot
        /// Format: [p1|p2|both]:choice:<NUM>
        /// Examples: p1:choice:5, both:choice:10
        #[arg(long, value_name = "CONDITION")]
        stop_every: Option<String>,

        /// Output file for game snapshot (default: game.snapshot)
        #[arg(long, value_name = "FILE", default_value = "game.snapshot")]
        snapshot_output: PathBuf,

        /// Load and resume game from snapshot file
        #[arg(long, value_name = "FILE")]
        start_from: Option<PathBuf>,

        /// Save final game state when game ends (for determinism testing)
        #[arg(long, value_name = "FILE")]
        save_final_gamestate: Option<PathBuf>,
    },

    /// Run games for profiling (use with cargo-heaptrack or cargo-flamegraph)
    Profile {
        /// Number of games to run
        #[arg(long, short = 'g', default_value_t = 1000)]
        games: usize,

        /// Random seed for deterministic profiling
        #[arg(long, default_value_t = 42)]
        seed: u64,

        /// Deck file to use (uses same deck for both players)
        #[arg(long, short = 'd', default_value = "test_decks/simple_bolt.dck")]
        deck: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Tui {
            deck1,
            deck2,
            start_state,
            p1,
            p2,
            p1_name,
            p2_name,
            p1_fixed_inputs,
            p2_fixed_inputs,
            seed,
            load_all_cards,
            verbosity,
            numeric_choices,
            stop_every,
            snapshot_output,
            start_from,
            save_final_gamestate,
        } => {
            run_tui(
                deck1,
                deck2,
                start_state,
                p1,
                p2,
                p1_name,
                p2_name,
                p1_fixed_inputs,
                p2_fixed_inputs,
                seed,
                load_all_cards,
                verbosity,
                numeric_choices,
                stop_every,
                snapshot_output,
                start_from,
                save_final_gamestate,
            )
            .await?
        }
        Commands::Profile { games, seed, deck } => run_profile(games, seed, deck).await?,
    }

    Ok(())
}

/// Parse fixed input string into a vector of choice indices
fn parse_fixed_inputs(input: &str) -> std::result::Result<Vec<usize>, String> {
    input
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.parse::<usize>()
                .map_err(|_| format!("invalid choice index: '{}'", s))
        })
        .collect()
}

// StopCondition is now imported from mtg_forge_rs::game module

/// Run TUI with async card loading
#[allow(clippy::too_many_arguments)] // CLI parameters naturally map to function args
async fn run_tui(
    deck1_path: Option<PathBuf>,
    deck2_path: Option<PathBuf>,
    puzzle_path: Option<PathBuf>,
    p1_type: ControllerType,
    p2_type: ControllerType,
    p1_name: String,
    p2_name: String,
    p1_fixed_inputs: Option<String>,
    p2_fixed_inputs: Option<String>,
    seed: Option<u64>,
    load_all_cards: bool,
    verbosity: VerbosityArg,
    numeric_choices: bool,
    stop_every: Option<String>,
    snapshot_output: PathBuf,
    start_from: Option<PathBuf>,
    save_final_gamestate: Option<PathBuf>,
) -> Result<()> {
    let verbosity: VerbosityLevel = verbosity.into();
    println!("=== MTG Forge Rust - Text UI Mode ===\n");

    // Parse stop condition if provided
    let stop_condition = if let Some(ref stop_str) = stop_every {
        let condition = StopCondition::parse(stop_str).map_err(|e| {
            mtg_forge_rs::MtgError::InvalidAction(format!("Error parsing --stop-every: {}", e))
        })?;
        println!("Stop condition: {:?}", condition);
        println!("Snapshot output: {}\n", snapshot_output.display());
        Some(condition)
    } else {
        None
    };

    // Check for conflicting options
    if start_from.is_some()
        && (deck1_path.is_some() || deck2_path.is_some() || puzzle_path.is_some())
    {
        return Err(mtg_forge_rs::MtgError::InvalidAction(
            "Cannot specify both --start-from and deck/puzzle files".to_string(),
        ));
    }

    // Create async card database
    let cardsfolder = PathBuf::from("cardsfolder");
    let card_db = CardDatabase::new(cardsfolder);

    // Load snapshot early if resuming, so we can extract both game state and player-specific choices
    let loaded_snapshot: Option<GameSnapshot> = if let Some(ref snapshot_file) = start_from {
        let snapshot = GameSnapshot::load_from_file(snapshot_file).map_err(|e| {
            mtg_forge_rs::MtgError::InvalidAction(format!("Failed to load snapshot: {}", e))
        })?;
        Some(snapshot)
    } else {
        None
    };

    let snapshot_turn_number: Option<u32> = loaded_snapshot.as_ref().map(|s| s.turn_number);

    let mut game = if let Some(ref snapshot) = loaded_snapshot {
        // Load game from snapshot
        if verbosity >= VerbosityLevel::Minimal {
            println!(
                "Loading snapshot from: {}",
                start_from.as_ref().unwrap().display()
            );
            println!("  Turn number: {}", snapshot.turn_number);
            println!(
                "  Intra-turn choices to replay: {}",
                snapshot.choice_count()
            );
            println!("Game loaded from snapshot!\n");
        }

        // Note: We don't need to load cards for snapshots since the GameState
        // already contains all the card data
        snapshot.game_state.clone()
    } else if let Some(puzzle_file) = puzzle_path {
        // Load game from puzzle file
        println!("Loading puzzle file: {}", puzzle_file.display());
        let puzzle_contents = std::fs::read_to_string(&puzzle_file)?;
        let puzzle = PuzzleFile::parse(&puzzle_contents)?;
        println!("  Puzzle: {}", puzzle.metadata.name);
        println!("  Goal: {:?}", puzzle.metadata.goal);
        println!("  Difficulty: {:?}\n", puzzle.metadata.difficulty);

        // Load cards needed for puzzle
        println!("Loading card database...");
        let (count, duration) = if load_all_cards {
            card_db.eager_load().await?
        } else {
            // Extract card names from puzzle state
            let mut card_names = std::collections::HashSet::new();
            for player in &puzzle.state.players {
                for card_def in player
                    .hand
                    .iter()
                    .chain(player.battlefield.iter())
                    .chain(player.graveyard.iter())
                    .chain(player.library.iter())
                    .chain(player.exile.iter())
                {
                    card_names.insert(card_def.name.clone());
                }
            }
            card_db
                .load_cards(&card_names.into_iter().collect::<Vec<_>>())
                .await?
        };
        println!("  Loaded {count} cards");
        eprintln!("  (Loading time: {:.2}ms)", duration.as_secs_f64() * 1000.0);

        println!("Initializing game from puzzle...");
        load_puzzle_into_game(&puzzle, &card_db).await?
    } else {
        // Load game from deck files
        let deck1_path = deck1_path.expect("deck1 required when not loading from puzzle");
        let deck2_path = deck2_path.expect("deck2 required when not loading from puzzle");

        println!("Loading deck files...");
        let deck1 = DeckLoader::load_from_file(&deck1_path)?;
        let deck2 = DeckLoader::load_from_file(&deck2_path)?;
        println!("  Player 1: {} cards", deck1.total_cards());
        println!("  Player 2: {} cards\n", deck2.total_cards());

        // Load cards based on mode
        println!("Loading card database...");
        let (count, duration) = if load_all_cards {
            // Load all cards from cardsfolder
            card_db.eager_load().await?
        } else {
            // Load only cards needed for the two decks
            let mut unique_names = deck1.unique_card_names();
            unique_names.extend(deck2.unique_card_names());
            card_db.load_cards(&unique_names).await?
        };
        println!("  Loaded {count} cards");
        eprintln!("  (Loading time: {:.2}ms)", duration.as_secs_f64() * 1000.0);

        // Initialize game
        println!("Initializing game...");
        let game_init = GameInitializer::new(&card_db);
        game_init
            .init_game(
                p1_name.clone(),
                &deck1,
                p2_name.clone(),
                &deck2,
                20, // starting life
            )
            .await?
    };

    // Set random seed if provided
    if let Some(seed_value) = seed {
        game.seed_rng(seed_value);
        println!("Using random seed: {seed_value}");
    }

    // Enable numeric choices mode if requested
    if numeric_choices {
        game.logger.set_numeric_choices(true);
        println!("Numeric choices mode: enabled");
    }

    println!("Game initialized!");
    println!("  Player 1: {} ({p1_type:?})", p1_name);
    println!("  Player 2: {} ({p2_type:?})\n", p2_name);

    // Create controllers based on agent types
    let (p1_id, p2_id) = {
        let p1 = game.get_player_by_idx(0).expect("Should have player 1");
        let p2 = game.get_player_by_idx(1).expect("Should have player 2");
        (p1.id, p2.id)
    };

    // Create base controllers
    let base_controller1: Box<dyn mtg_forge_rs::game::controller::PlayerController> = match p1_type
    {
        ControllerType::Zero => Box::new(ZeroController::new(p1_id)),
        ControllerType::Random => Box::new(RandomController::new(p1_id)),
        ControllerType::Tui => Box::new(InteractiveController::with_numeric_choices(
            p1_id,
            numeric_choices,
        )),
        ControllerType::Heuristic => Box::new(HeuristicController::new(p1_id)),
        ControllerType::Fixed => {
            // Priority: CLI --p1-fixed-inputs > snapshot state > error
            let controller = if let Some(input) = &p1_fixed_inputs {
                // CLI override - use provided script
                let script = parse_fixed_inputs(input).map_err(|e| {
                    mtg_forge_rs::MtgError::InvalidAction(format!(
                        "Error parsing --p1-fixed-inputs: {}",
                        e
                    ))
                })?;
                FixedScriptController::new(p1_id, script)
            } else if let Some(ref snapshot) = loaded_snapshot {
                // Restore from snapshot if available
                if let Some(controller_state) = &snapshot.p1_controller_state {
                    if verbosity >= VerbosityLevel::Verbose {
                        println!(
                            "Player 1 Fixed controller restored from snapshot (at index {})",
                            controller_state.current_index
                        );
                    }
                    controller_state.clone()
                } else {
                    return Err(mtg_forge_rs::MtgError::InvalidAction(
                        "--p1-fixed-inputs is required when --p1=fixed (no snapshot state available)".to_string(),
                    ));
                }
            } else {
                return Err(mtg_forge_rs::MtgError::InvalidAction(
                    "--p1-fixed-inputs is required when --p1=fixed".to_string(),
                ));
            };

            Box::new(controller)
        }
    };

    let base_controller2: Box<dyn mtg_forge_rs::game::controller::PlayerController> = match p2_type
    {
        ControllerType::Zero => Box::new(ZeroController::new(p2_id)),
        ControllerType::Random => Box::new(RandomController::new(p2_id)),
        ControllerType::Tui => Box::new(InteractiveController::with_numeric_choices(
            p2_id,
            numeric_choices,
        )),
        ControllerType::Heuristic => Box::new(HeuristicController::new(p2_id)),
        ControllerType::Fixed => {
            // Priority: CLI --p2-fixed-inputs > snapshot state > error
            let controller = if let Some(input) = &p2_fixed_inputs {
                // CLI override - use provided script
                let script = parse_fixed_inputs(input).map_err(|e| {
                    mtg_forge_rs::MtgError::InvalidAction(format!(
                        "Error parsing --p2-fixed-inputs: {}",
                        e
                    ))
                })?;
                FixedScriptController::new(p2_id, script)
            } else if let Some(ref snapshot) = loaded_snapshot {
                // Restore from snapshot if available
                if let Some(controller_state) = &snapshot.p2_controller_state {
                    if verbosity >= VerbosityLevel::Verbose {
                        println!(
                            "Player 2 Fixed controller restored from snapshot (at index {})",
                            controller_state.current_index
                        );
                    }
                    controller_state.clone()
                } else {
                    return Err(mtg_forge_rs::MtgError::InvalidAction(
                        "--p2-fixed-inputs is required when --p2=fixed (no snapshot state available)".to_string(),
                    ));
                }
            } else {
                return Err(mtg_forge_rs::MtgError::InvalidAction(
                    "--p2-fixed-inputs is required when --p2=fixed".to_string(),
                ));
            };

            Box::new(controller)
        }
    };

    // Wrap with ReplayController if resuming from snapshot
    // CRITICAL: Each controller must only replay its OWN choices, not the other player's!
    //
    // EXCEPTION: Don't wrap FixedScriptController with ReplayController.
    // Fixed controller already has the full game script and wrapping it would cause
    // double-replay (ReplayController replays intra-turn, then Fixed restarts from index 0).
    let mut controller1: Box<dyn mtg_forge_rs::game::controller::PlayerController> =
        if let Some(ref snapshot) = loaded_snapshot {
            // Check if base controller is Fixed - don't wrap if it is
            let is_fixed = matches!(p1_type, ControllerType::Fixed);
            if is_fixed {
                if verbosity >= VerbosityLevel::Verbose {
                    println!("Player 1 using Fixed controller (skipping Replay wrapper)");
                }
                base_controller1
            } else {
                let p1_replay_choices = snapshot.extract_replay_choices_for_player(p1_id);
                if verbosity >= VerbosityLevel::Verbose {
                    println!(
                        "Player 1 will replay {} intra-turn choices",
                        p1_replay_choices.len()
                    );
                }
                Box::new(mtg_forge_rs::game::ReplayController::new(
                    p1_id,
                    base_controller1,
                    p1_replay_choices,
                ))
            }
        } else {
            base_controller1
        };

    let mut controller2: Box<dyn mtg_forge_rs::game::controller::PlayerController> =
        if let Some(ref snapshot) = loaded_snapshot {
            // Check if base controller is Fixed - don't wrap if it is
            let is_fixed = matches!(p2_type, ControllerType::Fixed);
            if is_fixed {
                if verbosity >= VerbosityLevel::Verbose {
                    println!("Player 2 using Fixed controller (skipping Replay wrapper)");
                }
                base_controller2
            } else {
                let p2_replay_choices = snapshot.extract_replay_choices_for_player(p2_id);
                if verbosity >= VerbosityLevel::Verbose {
                    println!(
                        "Player 2 will replay {} intra-turn choices",
                        p2_replay_choices.len()
                    );
                }
                Box::new(mtg_forge_rs::game::ReplayController::new(
                    p2_id,
                    base_controller2,
                    p2_replay_choices,
                ))
            }
        } else {
            base_controller2
        };

    if verbosity >= VerbosityLevel::Minimal {
        if snapshot_turn_number.is_some() {
            println!("=== Continuing Game ===\n");
        } else {
            println!("=== Starting Game ===\n");
        }
    }

    // Run the game loop (with or without snapshots)
    let mut game_loop = GameLoop::new(&mut game).with_verbosity(verbosity);

    // If loading from snapshot, restore the turn counter
    // Note: snapshot.turn_number represents the turn we're STARTING,
    // but turns_elapsed tracks COMPLETED turns, so we need turn_number - 1
    if let Some(turn_num) = snapshot_turn_number {
        // Turn numbers are 1-based (turn 1, 2, 3...), never 0
        // If we see turn 0, that's a critical bug in snapshot serialization
        if turn_num == 0 {
            return Err(mtg_forge_rs::MtgError::InvalidAction(
                "Invalid snapshot: turn_number is 0 (turns are 1-based, not 0-based)".to_string(),
            ));
        }
        let turns_elapsed = turn_num - 1;
        game_loop = game_loop.with_turn_counter(turns_elapsed);
    }

    let result = if let Some(ref stop_cond) = stop_condition {
        // Run with snapshot functionality
        game_loop.run_game_with_snapshots(
            &mut *controller1,
            &mut *controller2,
            p1_id, // Player 1 ID for filtering player choices
            stop_cond,
            &snapshot_output,
        )?
    } else {
        // Normal game loop
        game_loop.run_game(&mut *controller1, &mut *controller2)?
    };

    // If game ended with a snapshot, reload and add controller state
    use mtg_forge_rs::game::GameEndReason;
    if result.end_reason == GameEndReason::Snapshot && snapshot_output.exists() {
        // Extract controller states by calling get_snapshot_state()
        let p1_state_json = controller1.get_snapshot_state();
        let p2_state_json = controller2.get_snapshot_state();

        // If either controller has state to preserve, update the snapshot
        if p1_state_json.is_some() || p2_state_json.is_some() {
            if let Ok(mut snapshot) = GameSnapshot::load_from_file(&snapshot_output) {
                // Deserialize JSON back to FixedScriptController if present
                snapshot.p1_controller_state =
                    p1_state_json.and_then(|json| serde_json::from_value(json).ok());
                snapshot.p2_controller_state =
                    p2_state_json.and_then(|json| serde_json::from_value(json).ok());

                if let Err(e) = snapshot.save_to_file(&snapshot_output) {
                    eprintln!(
                        "Warning: Failed to update snapshot with controller state: {}",
                        e
                    );
                } else if verbosity >= VerbosityLevel::Verbose {
                    println!("Snapshot updated with controller state");
                }
            }
        }
    }

    // Display results (suppress for snapshot exits)
    if verbosity >= VerbosityLevel::Minimal && result.end_reason != GameEndReason::Snapshot {
        println!("\n=== Game Over ===");
        match result.winner {
            Some(winner_id) => {
                let winner = game.get_player(winner_id)?;
                println!("Winner: {}", winner.name);
            }
            None => {
                println!("Game ended in a draw");
            }
        }
        println!("Turns played: {}", result.turns_played);
        println!("Reason: {:?}", result.end_reason);

        // Final state
        println!("\n=== Final State ===");
        for player in game.players.iter() {
            println!("  {}: {} life", player.name, player.life);
        }
    }

    // Save final gamestate if requested (for determinism testing)
    if let Some(final_state_path) = save_final_gamestate {
        if result.end_reason != GameEndReason::Snapshot {
            // Create a snapshot with the final game state
            let final_snapshot = GameSnapshot::new(
                game.clone(),
                result.turns_played,
                Vec::new(), // No intra-turn choices for final state
            );

            final_snapshot
                .save_to_file(&final_state_path)
                .map_err(|e| {
                    mtg_forge_rs::MtgError::InvalidAction(format!(
                        "Failed to save final gamestate: {}",
                        e
                    ))
                })?;

            if verbosity >= VerbosityLevel::Verbose {
                println!(
                    "\nFinal game state saved to: {}",
                    final_state_path.display()
                );
            }
        }
    }

    Ok(())
}

/// Run profiling games
async fn run_profile(iterations: usize, seed: u64, deck_path: PathBuf) -> Result<()> {
    println!("=== MTG Forge Rust - Profiling Mode ===\n");

    // Load deck
    println!("Loading deck...");
    let deck = DeckLoader::load_from_file(&deck_path)?;
    println!("  Deck: {} cards", deck.total_cards());

    // Create card database (lazy loading - only loads cards on-demand)
    let cardsfolder = PathBuf::from("cardsfolder");
    let card_db = CardDatabase::new(cardsfolder);

    // Prefetch deck cards (not all 31k cards, just what we need)
    let start = std::time::Instant::now();
    let unique_names = deck.unique_card_names();
    let (count, _) = card_db.load_cards(&unique_names).await?;
    let duration = start.elapsed();
    println!(
        "  Loaded {count} cards in {:.2}ms\n",
        duration.as_secs_f64() * 1000.0
    );

    println!("Profiling game execution...");
    println!("Running {iterations} games with seed {seed}");
    println!();

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
            .await?;
        game.seed_rng(seed);

        // Create random controllers
        let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
        let p1_id = players[0];
        let p2_id = players[1];

        let mut controller1 = RandomController::new(p1_id);
        let mut controller2 = RandomController::new(p2_id);

        // Run game
        let mut game_loop = GameLoop::new(&mut game).with_verbosity(VerbosityLevel::Silent);
        game_loop.run_game(&mut controller1, &mut controller2)?;

        // Print progress every 100 games
        if (i + 1) % 100 == 0 {
            println!("Completed {} games", i + 1);
        }
    }

    println!();
    println!("Profiling complete! {iterations} games executed.");
    println!();
    println!("For heap profiling:");
    println!("  cargo heaptrack --bin mtg -- profile --games {iterations} --seed {seed}");
    println!("  Or: make heapprofile");
    println!();
    println!("For CPU profiling:");
    println!("  cargo flamegraph --bin mtg -- profile --games {iterations} --seed {seed}");
    println!("  Or: make profile");

    Ok(())
}
