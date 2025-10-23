//! MTG Forge Rust - Main Binary
//!
//! Text-based Magic: The Gathering game engine with TUI support

use clap::{Parser, Subcommand, ValueEnum};
use mtg_forge_rs::{
    game::{
        random_controller::RandomController, zero_controller::ZeroController, GameLoop,
        VerbosityLevel,
    },
    loader::{AsyncCardDatabase as CardDatabase, DeckLoader, GameInitializer},
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
    // TODO: Add these when implemented
    // /// Interactive text UI controller for human play
    // Tui,
    // /// AI controller with strategic decision making
    // Ai,
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
                "invalid verbosity level '{}' (expected: silent/0, minimal/1, normal/2, verbose/3)",
                s
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
        /// Deck file (.dck) for player 1
        #[arg(value_name = "PLAYER1_DECK")]
        deck1: PathBuf,

        /// Deck file (.dck) for player 2
        #[arg(value_name = "PLAYER2_DECK")]
        deck2: PathBuf,

        /// Player 1 controller type
        #[arg(long, value_enum, default_value = "random")]
        p1: ControllerType,

        /// Player 2 controller type
        #[arg(long, value_enum, default_value = "random")]
        p2: ControllerType,

        /// Set random seed for deterministic testing
        #[arg(long)]
        seed: Option<u64>,

        /// Load all cards from cardsfolder (default: only load cards in decks)
        #[arg(long)]
        load_all_cards: bool,

        /// Verbosity level for game output (0=silent, 1=minimal, 2=normal, 3=verbose)
        #[arg(long, default_value = "normal", short = 'v')]
        verbosity: VerbosityArg,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Tui {
            deck1,
            deck2,
            p1,
            p2,
            seed,
            load_all_cards,
            verbosity,
        } => run_tui(deck1, deck2, p1, p2, seed, load_all_cards, verbosity).await?,
    }

    Ok(())
}

/// Run TUI with async card loading
async fn run_tui(
    deck1_path: PathBuf,
    deck2_path: PathBuf,
    p1_type: ControllerType,
    p2_type: ControllerType,
    seed: Option<u64>,
    load_all_cards: bool,
    verbosity: VerbosityArg,
) -> Result<()> {
    let verbosity: VerbosityLevel = verbosity.into();
    println!("=== MTG Forge Rust - Text UI Mode ===\n");

    // Load decks
    println!("Loading deck files...");
    let deck1 = DeckLoader::load_from_file(&deck1_path)?;
    let deck2 = DeckLoader::load_from_file(&deck2_path)?;
    println!("  Player 1: {} cards", deck1.total_cards());
    println!("  Player 2: {} cards\n", deck2.total_cards());

    // Create async card database
    let cardsfolder = PathBuf::from("cardsfolder");
    let card_db = CardDatabase::new(cardsfolder);

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
    println!("  Loaded {} cards", count);
    eprintln!("  (Loading time: {:.2}ms)", duration.as_secs_f64() * 1000.0);

    // Initialize game
    println!("Initializing game...");
    let game_init = GameInitializer::new(&card_db);
    let mut game = game_init
        .init_game(
            "Player 1".to_string(),
            &deck1,
            "Player 2".to_string(),
            &deck2,
            20, // starting life
        )
        .await?;

    // Set random seed if provided
    if let Some(seed_value) = seed {
        game.rng_seed = seed_value;
        println!("Using random seed: {seed_value}");
    }

    println!("Game initialized!");
    println!("  Player 1: Player 1 ({:?})", p1_type);
    println!("  Player 2: Player 2 ({:?})\n", p2_type);

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
                Box::new(RandomController::with_seed(p1_id, seed_value).with_verbosity(verbosity))
            } else {
                Box::new(RandomController::new(p1_id).with_verbosity(verbosity))
            }
        }
    };

    let mut controller2: Box<dyn mtg_forge_rs::game::controller::PlayerController> = match p2_type {
        ControllerType::Zero => Box::new(ZeroController::new(p2_id)),
        ControllerType::Random => {
            if let Some(seed_value) = seed {
                // Use seed + 1 for player 2 so they have different random sequences
                Box::new(
                    RandomController::with_seed(p2_id, seed_value + 1).with_verbosity(verbosity),
                )
            } else {
                Box::new(RandomController::new(p2_id).with_verbosity(verbosity))
            }
        }
    };

    if verbosity >= VerbosityLevel::Minimal {
        println!("=== Starting Game ===\n");
    }

    // Run the game loop
    let mut game_loop = GameLoop::new(&mut game).with_verbosity(verbosity);
    let result = game_loop.run_game(&mut *controller1, &mut *controller2)?;

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
