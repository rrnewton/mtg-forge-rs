//! End-to-end tests for the TUI
//!
//! These tests verify that the TUI can successfully run complete games
//! from start to finish with various deck configurations.

use mtg_forge_rs::{
    game::{GameLoop, ZeroController},
    loader::{AsyncCardDatabase as CardDatabase, DeckLoader, GameInitializer},
    Result,
};
use std::path::PathBuf;

/// Test that two zero controllers can complete a game with simple decks
#[tokio::test]
async fn test_tui_zero_vs_zero_simple_bolt() -> Result<()> {
    // Load card database
    let cardsfolder = PathBuf::from("cardsfolder");
    if !cardsfolder.exists() {
        // Skip test if cardsfolder doesn't exist
        return Ok(());
    }
    let card_db = CardDatabase::new(cardsfolder);
    card_db.eager_load().await?;

    // Load test decks
    let deck_path = PathBuf::from("test_decks/simple_bolt.dck");
    let deck = DeckLoader::load_from_file(&deck_path)?;

    // Initialize game
    let game_init = GameInitializer::new(&card_db);
    let mut game = game_init
        .init_game(
            "Player 1".to_string(),
            &deck,
            "Player 2".to_string(),
            &deck,
            20, // starting life
        )
        .await?;

    // Set deterministic seed for reproducible results
    game.rng_seed = 42;

    // Create zero controllers
    let players: Vec<_> = game.players.iter().map(|(id, _)| *id).collect();
    let p1_id = players[0];
    let p2_id = players[1];

    let mut controller1 = ZeroController::new(p1_id);
    let mut controller2 = ZeroController::new(p2_id);

    // Run the game loop
    let mut game_loop = GameLoop::new(&mut game);
    let result = game_loop.run_game(&mut controller1, &mut controller2)?;

    // Verify game completed successfully
    assert!(result.winner.is_some(), "Game should have a winner");
    assert!(
        result.turns_played > 0,
        "Game should have played some turns"
    );

    // With seed 42, Player 1 should win by decking
    assert_eq!(
        result.winner.unwrap(),
        p1_id,
        "With seed 42, Player 1 should win"
    );

    // Verify the game ended due to decking (empty library)
    assert!(
        matches!(
            result.end_reason,
            mtg_forge_rs::game::GameEndReason::Decking(_)
        ),
        "Game should end by decking with these decks: {:?}",
        result.end_reason
    );

    // Verify both players still have 20 life (no damage dealt in this setup)
    assert_eq!(
        game.players.get(p1_id)?.life,
        20,
        "Player 1 should have 20 life"
    );
    assert_eq!(
        game.players.get(p2_id)?.life,
        20,
        "Player 2 should have 20 life"
    );

    Ok(())
}

/// Test that the game respects the deterministic seed
#[tokio::test]
async fn test_tui_deterministic_with_seed() -> Result<()> {
    // Load card database
    let cardsfolder = PathBuf::from("cardsfolder");
    if !cardsfolder.exists() {
        return Ok(());
    }
    let card_db = CardDatabase::new(cardsfolder);
    card_db.eager_load().await?;

    // Load test deck
    let deck_path = PathBuf::from("test_decks/simple_bolt.dck");
    let deck = DeckLoader::load_from_file(&deck_path)?;

    // Run the same game twice with the same seed
    let mut results = Vec::new();
    for _ in 0..2 {
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
        game.rng_seed = 12345;

        let players: Vec<_> = game.players.iter().map(|(id, _)| *id).collect();
        let p1_id = players[0];
        let p2_id = players[1];

        let mut controller1 = ZeroController::new(p1_id);
        let mut controller2 = ZeroController::new(p2_id);

        let mut game_loop = GameLoop::new(&mut game);
        let result = game_loop.run_game(&mut controller1, &mut controller2)?;

        results.push((result.winner, result.turns_played, result.end_reason));
    }

    // Both runs should produce identical results
    assert_eq!(
        results[0], results[1],
        "Games with same seed should produce identical results"
    );

    Ok(())
}

/// Test that the game runs to completion without errors (sanity check)
#[tokio::test]
async fn test_tui_runs_to_completion() -> Result<()> {
    // Load card database
    let cardsfolder = PathBuf::from("cardsfolder");
    if !cardsfolder.exists() {
        return Ok(());
    }
    let card_db = CardDatabase::new(cardsfolder);
    card_db.eager_load().await?;

    // Load test deck
    let deck_path = PathBuf::from("test_decks/simple_bolt.dck");
    let deck = DeckLoader::load_from_file(&deck_path)?;

    // Run a game and verify it completes
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
    game.rng_seed = 54321;

    let players: Vec<_> = game.players.iter().map(|(id, _)| *id).collect();
    let p1_id = players[0];
    let p2_id = players[1];

    let mut controller1 = ZeroController::new(p1_id);
    let mut controller2 = ZeroController::new(p2_id);

    let mut game_loop = GameLoop::new(&mut game);
    let result = game_loop.run_game(&mut controller1, &mut controller2)?;

    // Verify basic properties
    assert!(result.winner.is_some(), "Game should have a winner");
    assert!(
        result.turns_played > 0 && result.turns_played <= 150,
        "Game should play between 1-150 turns, got {}",
        result.turns_played
    );

    // Both players should still exist
    assert!(game.players.contains(p1_id));
    assert!(game.players.contains(p2_id));

    Ok(())
}

/// Test that random controllers successfully play lands and cast spells
#[tokio::test]
async fn test_tui_random_vs_random_deals_damage() -> Result<()> {
    // Load card database
    let cardsfolder = PathBuf::from("cardsfolder");
    if !cardsfolder.exists() {
        return Ok(());
    }
    let card_db = CardDatabase::new(cardsfolder);
    card_db.eager_load().await?;

    // Load test deck with Mountains and Lightning Bolts
    let deck_path = PathBuf::from("test_decks/simple_bolt.dck");
    let deck = DeckLoader::load_from_file(&deck_path)?;

    // Run a game with random controllers
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
    game.rng_seed = 42;

    let players: Vec<_> = game.players.iter().map(|(id, _)| *id).collect();
    let p1_id = players[0];
    let p2_id = players[1];

    let mut controller1 = mtg_forge_rs::game::RandomController::new(p1_id);
    let mut controller2 = mtg_forge_rs::game::RandomController::new(p2_id);

    let mut game_loop = GameLoop::new(&mut game).with_verbose(true);
    let result = game_loop.run_game(&mut controller1, &mut controller2)?;

    // Verify game completed
    assert!(result.winner.is_some(), "Game should have a winner");

    // Verify that damage was dealt (at least one player should have life != 20)
    let p1_life = game.players.get(p1_id)?.life;
    let p2_life = game.players.get(p2_id)?.life;

    assert!(
        p1_life != 20 || p2_life != 20,
        "At least one player should have taken damage. P1: {}, P2: {}",
        p1_life,
        p2_life
    );

    // With the simple bolt deck and random controllers, someone should die
    // (not just deck out)
    assert!(
        matches!(
            result.end_reason,
            mtg_forge_rs::game::GameEndReason::PlayerDeath(_)
        ),
        "Game should end by player death with random controllers casting bolts: {:?}",
        result.end_reason
    );

    // Verify at least one player has <= 0 life
    let winner = result.winner.unwrap();
    let loser = if winner == p1_id { p2_id } else { p1_id };
    let loser_life = game.players.get(loser)?.life;
    assert!(
        loser_life <= 0,
        "Loser should have <= 0 life, got {}",
        loser_life
    );

    // Note: Winner may also have negative life if both players dealt lethal
    // damage in the same priority round. The game picks the first dead player
    // as the loser.

    Ok(())
}

/// Test that players discard down to 7 cards during cleanup step
#[tokio::test]
async fn test_discard_to_hand_size() -> Result<()> {
    use mtg_forge_rs::core::{Card, CardType, EntityId};

    // Load card database
    let cardsfolder = PathBuf::from("cardsfolder");
    if !cardsfolder.exists() {
        return Ok(());
    }
    let card_db = CardDatabase::new(cardsfolder);
    card_db.eager_load().await?;

    // Load test deck
    let deck_path = PathBuf::from("test_decks/simple_bolt.dck");
    let deck = DeckLoader::load_from_file(&deck_path)?;

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

    let players: Vec<_> = game.players.iter().map(|(id, _)| *id).collect();
    let p1_id = players[0];
    let p2_id = players[1];

    // Give Player 1 10 cards in hand (exceeding max hand size of 7)
    for i in 0..10 {
        let card_id = EntityId::<Card>::new(1000 + i);
        let mut card = Card::new(card_id, "Mountain", p1_id);
        card.types.push(CardType::Land);
        game.cards.insert(card_id, card);

        if let Some(zones) = game.get_player_zones_mut(p1_id) {
            zones.hand.add(card_id);
        }
    }

    // Verify Player 1 has 10 cards in hand
    let hand_size_before = game
        .get_player_zones(p1_id)
        .map(|z| z.hand.cards.len())
        .unwrap_or(0);
    assert_eq!(
        hand_size_before, 10,
        "Player 1 should start with 10 cards in hand"
    );

    // Create controllers and run cleanup step
    let mut controller1 = mtg_forge_rs::game::ZeroController::new(p1_id);
    let mut controller2 = mtg_forge_rs::game::ZeroController::new(p2_id);

    // Set Player 1 as active player
    game.turn.active_player = p1_id;

    // Run cleanup step through game loop
    let mut game_loop = GameLoop::new(&mut game).with_verbose(true);
    game_loop.game.turn.current_step = mtg_forge_rs::game::Step::Cleanup;

    // Execute cleanup step manually
    game_loop.execute_step(&mut controller1, &mut controller2)?;

    // Verify Player 1 now has exactly 7 cards in hand
    let hand_size_after = game_loop
        .game
        .get_player_zones(p1_id)
        .map(|z| z.hand.cards.len())
        .unwrap_or(0);
    assert_eq!(
        hand_size_after, 7,
        "Player 1 should have discarded down to 7 cards"
    );

    // Verify 3 cards were discarded to graveyard
    let graveyard_size = game_loop
        .game
        .get_player_zones(p1_id)
        .map(|z| z.graveyard.cards.len())
        .unwrap_or(0);
    assert_eq!(
        graveyard_size, 3,
        "Player 1 should have 3 cards in graveyard after discarding"
    );

    Ok(())
}
