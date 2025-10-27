//! MTG Forge Rust - Main Binary
//!
//! Text-based Magic: The Gathering game engine with TUI support

use clap::{Parser, Subcommand, ValueEnum};
use mtg_forge_rs::{
    game::{
        random_controller::RandomController, zero_controller::ZeroController,
        FixedScriptController, GameLoop, HeuristicController, InteractiveController,
        VerbosityLevel,
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
enum Commands {
    /// Text UI Mode - Interactive Forge Gameplay
    Tui {
        /// Deck file (.dck) for player 1 (required unless --start-state is provided)
        #[arg(value_name = "PLAYER1_DECK", required_unless_present = "start_state")]
        deck1: Option<PathBuf>,

        /// Deck file (.dck) for player 2 (required unless --start-state is provided)
        #[arg(value_name = "PLAYER2_DECK", required_unless_present = "start_state")]
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

        /// Player 1 name
        #[arg(long, default_value = "Player 1")]
        p1_name: String,

        /// Player 2 name
        #[arg(long, default_value = "Player 2")]
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

        /// Stop every N choices for specified player(s) and save snapshot
        /// Format: [p1|p2|both]:choice:<NUM>
        /// Example: --stop-every=p1:choice:1 stops after player 1 makes 1 choice
        #[arg(long, value_name = "CONDITION")]
        stop_every: Option<String>,

        /// Output file for game snapshot (default: game.snapshot)
        #[arg(long, default_value = "game.snapshot")]
        snapshot_output: PathBuf,

        /// Load and resume game from snapshot file
        #[arg(long, value_name = "SNAPSHOT_FILE")]
        start_from: Option<PathBuf>,
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
            )
            .await?
        }
        Commands::Profile { games, seed, deck } => run_profile(games, seed, deck).await?,
    }

    Ok(())
}

/// Stop condition for game snapshots
#[derive(Debug, Clone)]
struct StopCondition {
    /// Which player(s) to track
    player: StopPlayer,
    /// Number of choices to allow before stopping
    choice_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StopPlayer {
    P1,
    P2,
    Both,
}

impl StopCondition {
    /// Parse stop condition from string like "p1:choice:1" or "both:choice:5"
    fn parse(s: &str) -> std::result::Result<Self, String> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 3 {
            return Err(format!(
                "invalid stop condition format '{}' (expected: [p1|p2|both]:choice:<NUM>)",
                s
            ));
        }

        let player = match parts[0].to_lowercase().as_str() {
            "p1" => StopPlayer::P1,
            "p2" => StopPlayer::P2,
            "both" => StopPlayer::Both,
            _ => {
                return Err(format!(
                    "invalid player '{}' (expected: p1, p2, or both)",
                    parts[0]
                ))
            }
        };

        if parts[1] != "choice" {
            return Err(format!(
                "invalid condition type '{}' (expected: choice)",
                parts[1]
            ));
        }

        let choice_count = parts[2].parse::<usize>().map_err(|_| {
            format!(
                "invalid choice count '{}' (expected positive integer)",
                parts[2]
            )
        })?;

        if choice_count == 0 {
            return Err("choice count must be greater than 0".to_string());
        }

        Ok(StopCondition {
            player,
            choice_count,
        })
    }

    /// Check if this condition applies to a specific player
    fn applies_to(&self, p1_id: mtg_forge_rs::core::PlayerId, player_id: mtg_forge_rs::core::PlayerId) -> bool {
        match self.player {
            StopPlayer::P1 => player_id == p1_id,
            StopPlayer::P2 => player_id != p1_id,
            StopPlayer::Both => true,
        }
    }
}

/// Choice tracking wrapper for controllers
struct ChoiceTrackingWrapper {
    inner: Box<dyn mtg_forge_rs::game::controller::PlayerController>,
    choice_count: std::sync::Arc<std::sync::Mutex<usize>>,
    player_id: mtg_forge_rs::core::PlayerId,
}

impl ChoiceTrackingWrapper {
    fn new(
        inner: Box<dyn mtg_forge_rs::game::controller::PlayerController>,
        choice_count: std::sync::Arc<std::sync::Mutex<usize>>,
    ) -> Self {
        let player_id = inner.player_id();
        ChoiceTrackingWrapper {
            inner,
            choice_count,
            player_id,
        }
    }

    fn increment_choice(&self) {
        let mut count = self.choice_count.lock().unwrap();
        *count += 1;
    }
}

impl mtg_forge_rs::game::controller::PlayerController for ChoiceTrackingWrapper {
    fn player_id(&self) -> mtg_forge_rs::core::PlayerId {
        self.player_id
    }

    fn choose_spell_ability_to_play(
        &mut self,
        view: &mtg_forge_rs::game::controller::GameStateView,
        available: &[mtg_forge_rs::core::SpellAbility],
    ) -> Option<mtg_forge_rs::core::SpellAbility> {
        if !available.is_empty() {
            self.increment_choice();
        }
        self.inner.choose_spell_ability_to_play(view, available)
    }

    fn choose_targets(
        &mut self,
        view: &mtg_forge_rs::game::controller::GameStateView,
        spell: mtg_forge_rs::core::CardId,
        valid_targets: &[mtg_forge_rs::core::CardId],
    ) -> smallvec::SmallVec<[mtg_forge_rs::core::CardId; 4]> {
        if !valid_targets.is_empty() {
            self.increment_choice();
        }
        self.inner.choose_targets(view, spell, valid_targets)
    }

    fn choose_mana_sources_to_pay(
        &mut self,
        view: &mtg_forge_rs::game::controller::GameStateView,
        cost: &mtg_forge_rs::core::ManaCost,
        available_sources: &[mtg_forge_rs::core::CardId],
    ) -> smallvec::SmallVec<[mtg_forge_rs::core::CardId; 8]> {
        if !available_sources.is_empty() && cost.cmc() > 0 {
            self.increment_choice();
        }
        self.inner
            .choose_mana_sources_to_pay(view, cost, available_sources)
    }

    fn choose_attackers(
        &mut self,
        view: &mtg_forge_rs::game::controller::GameStateView,
        available_creatures: &[mtg_forge_rs::core::CardId],
    ) -> smallvec::SmallVec<[mtg_forge_rs::core::CardId; 8]> {
        if !available_creatures.is_empty() {
            self.increment_choice();
        }
        self.inner.choose_attackers(view, available_creatures)
    }

    fn choose_blockers(
        &mut self,
        view: &mtg_forge_rs::game::controller::GameStateView,
        available_blockers: &[mtg_forge_rs::core::CardId],
        attackers: &[mtg_forge_rs::core::CardId],
    ) -> smallvec::SmallVec<[(mtg_forge_rs::core::CardId, mtg_forge_rs::core::CardId); 8]> {
        if !available_blockers.is_empty() && !attackers.is_empty() {
            self.increment_choice();
        }
        self.inner
            .choose_blockers(view, available_blockers, attackers)
    }

    fn choose_damage_assignment_order(
        &mut self,
        view: &mtg_forge_rs::game::controller::GameStateView,
        attacker: mtg_forge_rs::core::CardId,
        blockers: &[mtg_forge_rs::core::CardId],
    ) -> smallvec::SmallVec<[mtg_forge_rs::core::CardId; 4]> {
        if blockers.len() > 1 {
            self.increment_choice();
        }
        self.inner
            .choose_damage_assignment_order(view, attacker, blockers)
    }

    fn choose_cards_to_discard(
        &mut self,
        view: &mtg_forge_rs::game::controller::GameStateView,
        hand: &[mtg_forge_rs::core::CardId],
        count: usize,
    ) -> smallvec::SmallVec<[mtg_forge_rs::core::CardId; 7]> {
        if count > 0 && !hand.is_empty() {
            self.increment_choice();
        }
        self.inner.choose_cards_to_discard(view, hand, count)
    }

    fn on_priority_passed(&mut self, view: &mtg_forge_rs::game::controller::GameStateView) {
        self.inner.on_priority_passed(view)
    }

    fn on_game_end(&mut self, view: &mtg_forge_rs::game::controller::GameStateView, won: bool) {
        self.inner.on_game_end(view, won)
    }
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
) -> Result<()> {
    let verbosity: VerbosityLevel = verbosity.into();
    println!("=== MTG Forge Rust - Text UI Mode ===\n");

    // Parse stop condition if provided
    let stop_condition = if let Some(ref condition_str) = stop_every {
        Some(StopCondition::parse(condition_str).map_err(|e| {
            mtg_forge_rs::MtgError::InvalidAction(format!("Error parsing --stop-every: {}", e))
        })?)
    } else {
        None
    };

    // Create async card database
    let cardsfolder = PathBuf::from("cardsfolder");
    let card_db = CardDatabase::new(cardsfolder);

    let mut game = if let Some(snapshot_file) = start_from {
        // Load game from snapshot file
        println!("Loading game from snapshot: {}", snapshot_file.display());
        let snapshot_contents = std::fs::read_to_string(&snapshot_file)?;
        let game: mtg_forge_rs::game::GameState = serde_json::from_str(&snapshot_contents)
            .map_err(|e| {
                mtg_forge_rs::MtgError::InvalidAction(format!(
                    "Error parsing snapshot file: {}",
                    e
                ))
            })?;
        println!("  Snapshot loaded successfully!\n");
        game
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
        game.rng_seed = seed_value;
        println!("Using random seed: {seed_value}");
    }

    // Enable numeric choices mode if requested
    if numeric_choices {
        game.logger.set_numeric_choices(true);
        println!("Numeric choices mode: enabled");
    }

    println!("Game initialized!");
    println!("  {}: ({p1_type:?})", p1_name);
    println!("  {}: ({p2_type:?})\n", p2_name);

    // Create controllers based on agent types
    let (p1_id, p2_id) = {
        let p1 = game.get_player_by_idx(0).expect("Should have player 1");
        let p2 = game.get_player_by_idx(1).expect("Should have player 2");
        (p1.id, p2.id)
    };

    let mut controller1: Box<dyn mtg_forge_rs::game::controller::PlayerController> = match p1_type {
        ControllerType::Zero => Box::new(ZeroController::new(p1_id)),
        ControllerType::Random => {
            if let Some(seed_value) = seed {
                Box::new(RandomController::with_seed(p1_id, seed_value))
            } else {
                Box::new(RandomController::new(p1_id))
            }
        }
        ControllerType::Tui => Box::new(InteractiveController::new(p1_id)),
        ControllerType::Heuristic => Box::new(HeuristicController::new(p1_id)),
        ControllerType::Fixed => {
            let script = match &p1_fixed_inputs {
                Some(input) => parse_fixed_inputs(input).map_err(|e| {
                    mtg_forge_rs::MtgError::InvalidAction(format!(
                        "Error parsing --p1-fixed-inputs: {}",
                        e
                    ))
                })?,
                None => {
                    return Err(mtg_forge_rs::MtgError::InvalidAction(
                        "--p1-fixed-inputs is required when --p1=fixed".to_string(),
                    ));
                }
            };
            Box::new(FixedScriptController::new(p1_id, script))
        }
    };

    let mut controller2: Box<dyn mtg_forge_rs::game::controller::PlayerController> = match p2_type {
        ControllerType::Zero => Box::new(ZeroController::new(p2_id)),
        ControllerType::Random => {
            if let Some(seed_value) = seed {
                // Use seed + 1 for player 2 so they have different random sequences
                Box::new(RandomController::with_seed(p2_id, seed_value + 1))
            } else {
                Box::new(RandomController::new(p2_id))
            }
        }
        ControllerType::Tui => Box::new(InteractiveController::new(p2_id)),
        ControllerType::Heuristic => Box::new(HeuristicController::new(p2_id)),
        ControllerType::Fixed => {
            let script = match &p2_fixed_inputs {
                Some(input) => parse_fixed_inputs(input).map_err(|e| {
                    mtg_forge_rs::MtgError::InvalidAction(format!(
                        "Error parsing --p2-fixed-inputs: {}",
                        e
                    ))
                })?,
                None => {
                    return Err(mtg_forge_rs::MtgError::InvalidAction(
                        "--p2-fixed-inputs is required when --p2=fixed".to_string(),
                    ));
                }
            };
            Box::new(FixedScriptController::new(p2_id, script))
        }
    };

    // Wrap controllers with choice tracking if stop condition is provided
    let (p1_choice_count, p2_choice_count) = if stop_condition.is_some() {
        let p1_count = std::sync::Arc::new(std::sync::Mutex::new(0));
        let p2_count = std::sync::Arc::new(std::sync::Mutex::new(0));

        controller1 = Box::new(ChoiceTrackingWrapper::new(controller1, p1_count.clone()));
        controller2 = Box::new(ChoiceTrackingWrapper::new(controller2, p2_count.clone()));

        (Some(p1_count), Some(p2_count))
    } else {
        (None, None)
    };

    if verbosity >= VerbosityLevel::Minimal {
        println!("=== Starting Game ===\n");
    }

    // Run the game loop
    let mut game_loop = GameLoop::new(&mut game).with_verbosity(verbosity);
    let result = game_loop.run_game(&mut *controller1, &mut *controller2)?;

    // Check if we should save a snapshot
    if let Some(ref condition) = stop_condition {
        let p1_choices = p1_choice_count.as_ref().map(|c| *c.lock().unwrap()).unwrap_or(0);
        let p2_choices = p2_choice_count.as_ref().map(|c| *c.lock().unwrap()).unwrap_or(0);

        let should_save = match condition.player {
            StopPlayer::P1 => p1_choices >= condition.choice_count,
            StopPlayer::P2 => p2_choices >= condition.choice_count,
            StopPlayer::Both => p1_choices >= condition.choice_count || p2_choices >= condition.choice_count,
        };

        if should_save {
            println!("\n=== Saving Game Snapshot ===");
            println!("  Player 1 choices: {}", p1_choices);
            println!("  Player 2 choices: {}", p2_choices);
            println!("  Snapshot file: {}", snapshot_output.display());

            let snapshot_json = serde_json::to_string_pretty(&game)
                .map_err(|e| mtg_forge_rs::MtgError::InvalidAction(format!("Error serializing game state: {}", e)))?;

            std::fs::write(&snapshot_output, snapshot_json)?;
            println!("  Snapshot saved successfully!\n");
        }
    }

    // Display results
    if verbosity >= VerbosityLevel::Minimal {
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
        game.rng_seed = seed;

        // Create random controllers
        let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
        let p1_id = players[0];
        let p2_id = players[1];

        let mut controller1 = RandomController::with_seed(p1_id, seed);
        let mut controller2 = RandomController::with_seed(p2_id, seed + 1);

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
