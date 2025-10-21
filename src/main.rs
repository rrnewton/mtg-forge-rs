//! MTG Forge Rust - Main Binary
//!
//! Text-based Magic: The Gathering game engine with TUI support

use clap::{Parser, Subcommand};
use mtg_forge_rs::{
    game::{GameLoop, RandomController, ZeroController},
    loader::{AsyncCardDatabase as CardDatabase, DeckLoader, GameInitializer},
    Result,
};
use std::path::PathBuf;

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

        /// Player 1 agent type (tui, ai, random, zero)
        #[arg(long, default_value = "zero")]
        p1: String,

        /// Player 2 agent type (tui, ai, random, zero)
        #[arg(long, default_value = "zero")]
        p2: String,

        /// Set random seed for deterministic testing
        #[arg(long)]
        seed: Option<u64>,

        /// Load all cards from cardsfolder (default: only load cards in decks)
        #[arg(long)]
        load_all_cards: bool,
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
        } => run_tui(deck1, deck2, p1, p2, seed, load_all_cards).await?,
    }

    Ok(())
}

/// Run TUI with async card loading
async fn run_tui(
    deck1_path: PathBuf,
    deck2_path: PathBuf,
    p1_type: String,
    p2_type: String,
    seed: Option<u64>,
    load_all_cards: bool,
) -> Result<()> {
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
    println!(
        "  Loaded {} cards in {:.2}ms\n",
        count,
        duration.as_secs_f64() * 1000.0
    );

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
    println!("  Player 1: Player 1 ({})", p1_type);
    println!("  Player 2: Player 2 ({})\n", p2_type);

    // Create controllers based on agent types
    let players: Vec<_> = game.players.iter().map(|(id, _)| *id).collect();
    let p1_id = players[0];
    let p2_id = players[1];

    let mut controller1: Box<dyn mtg_forge_rs::game::PlayerController> = match p1_type.as_str() {
        "zero" => Box::new(ZeroController::new(p1_id)),
        "random" => Box::new(RandomController::new(p1_id)),
        _ => {
            eprintln!("Unknown controller type: {p1_type}");
            eprintln!("Supported types: zero, random");
            std::process::exit(1);
        }
    };

    let mut controller2: Box<dyn mtg_forge_rs::game::PlayerController> = match p2_type.as_str() {
        "zero" => Box::new(ZeroController::new(p2_id)),
        "random" => Box::new(RandomController::new(p2_id)),
        _ => {
            eprintln!("Unknown controller type: {p2_type}");
            eprintln!("Supported types: zero, random");
            std::process::exit(1);
        }
    };

    println!("=== Starting Game ===\n");

    // Run the game loop
    let mut game_loop = GameLoop::new(&mut game);
    let result = game_loop.run_game(&mut *controller1, &mut *controller2)?;

    // Display results
    println!("\n=== Game Over ===");
    match result.winner {
        Some(winner_id) => {
            let winner = game.players.get(winner_id)?;
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
    for (_player_id, player) in game.players.iter() {
        println!("  {}: {} life", player.name, player.life);
    }

    Ok(())
}
