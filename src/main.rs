//! MTG Forge Rust - Main Binary
//!
//! Text-based Magic: The Gathering game engine with TUI support

use clap::{Parser, Subcommand};
use mtg_forge_rs::{
    game::{GameLoop, RandomController, ZeroController},
    loader::{AsyncCardDatabase, CardDatabase, DeckLoader, GameInitializer, load_deck_cards},
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

        /// Eagerly load all cards from cardsfolder (async, parallel)
        #[arg(long)]
        eager_load_cards: bool,
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
            eager_load_cards,
        } => {
            if eager_load_cards {
                run_tui_async(deck1, deck2, p1, p2, seed).await?
            } else {
                run_tui_sync(deck1, deck2, p1, p2, seed)?
            }
        }
    }

    Ok(())
}

/// Run TUI with async card loading (eager mode)
async fn run_tui_async(
    deck1_path: PathBuf,
    deck2_path: PathBuf,
    p1_type: String,
    p2_type: String,
    seed: Option<u64>,
) -> Result<()> {
    println!("=== MTG Forge Rust - Text UI Mode (Async) ===\n");

    // Load decks first to see what we need
    println!("Loading deck files...");
    let deck1 = DeckLoader::load_from_file(&deck1_path)?;
    let deck2 = DeckLoader::load_from_file(&deck2_path)?;
    println!("  Player 1: {} cards", deck1.total_cards());
    println!("  Player 2: {} cards\n", deck2.total_cards());

    // Create async card database
    let cardsfolder = PathBuf::from("cardsfolder");
    let card_db = AsyncCardDatabase::new(cardsfolder);

    // Eagerly load all cards
    println!("Eagerly loading all cards from cardsfolder...");
    let (count, duration) = card_db.eager_load().await?;
    println!("Loaded card database with {} cards in {:.2}ms\n", count, duration.as_secs_f64() * 1000.0);

    // Convert AsyncCardDatabase to sync CardDatabase for game initialization
    // This is a bit of a workaround - we load cards async, then build a sync DB from them
    let mut sync_db = CardDatabase::new();

    // Load the specific cards we need for the decks
    let (loaded1, dur1) = load_deck_cards(&card_db, &deck1).await?;
    let (loaded2, dur2) = load_deck_cards(&card_db, &deck2).await?;
    let deck_duration = dur1.max(dur2);

    println!("Loaded deck of {} distinct cards in {:.2}ms", loaded1 + loaded2, deck_duration.as_secs_f64() * 1000.0);

    // For now, we still need to use the sync DB for game init
    // Extract cards from async DB and add to sync DB
    // This is temporary until we refactor GameInitializer
    for entry in &deck1.main_deck {
        if let Ok(Some(card_def)) = card_db.get_card(&entry.card_name).await {
            sync_db.add_card(card_def);
        }
    }
    for entry in &deck2.main_deck {
        if let Ok(Some(card_def)) = card_db.get_card(&entry.card_name).await {
            sync_db.add_card(card_def);
        }
    }

    run_game_with_db(sync_db, deck1, deck2, p1_type, p2_type, seed)
}

/// Run TUI with synchronous card loading (default mode)
fn run_tui_sync(
    deck1_path: PathBuf,
    deck2_path: PathBuf,
    p1_type: String,
    p2_type: String,
    seed: Option<u64>,
) -> Result<()> {
    println!("=== MTG Forge Rust - Text UI Mode ===\n");

    // Load card database (sync, all cards)
    println!("Loading card database (sync)...");
    let cardsfolder = PathBuf::from("cardsfolder");
    let card_db = CardDatabase::load_from_cardsfolder(&cardsfolder)?;
    println!("  Loaded {} cards\n", card_db.len());

    // Load decks
    println!("Loading decks...");
    let deck1 = DeckLoader::load_from_file(&deck1_path)?;
    let deck2 = DeckLoader::load_from_file(&deck2_path)?;

    println!("  Player 1: {} cards", deck1.total_cards());
    println!("  Player 2: {} cards\n", deck2.total_cards());

    run_game_with_db(card_db, deck1, deck2, p1_type, p2_type, seed)
}

/// Common game logic (used by both sync and async paths)
fn run_game_with_db(
    card_db: CardDatabase,
    deck1: mtg_forge_rs::loader::DeckList,
    deck2: mtg_forge_rs::loader::DeckList,
    p1_type: String,
    p2_type: String,
    seed: Option<u64>,
) -> Result<()> {

    // Initialize game
    let game_init = GameInitializer::new(&card_db);
    let mut game = game_init.init_game(
        "Player 1".to_string(),
        &deck1,
        "Player 2".to_string(),
        &deck2,
        20, // starting life
    )?;

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
