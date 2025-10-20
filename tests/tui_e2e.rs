//! End-to-end tests for the TUI
//!
//! These tests verify that the TUI can successfully run complete games
//! from start to finish with various deck configurations.

use mtg_forge_rs::{
    game::{GameLoop, ZeroController},
    loader::{CardDatabase, DeckLoader, GameInitializer},
    Result,
};
use std::path::PathBuf;

/// Test that two zero controllers can complete a game with simple decks
#[test]
fn test_tui_zero_vs_zero_simple_bolt() -> Result<()> {
    // Load card database
    let cardsfolder = PathBuf::from("cardsfolder");
    if !cardsfolder.exists() {
        // Skip test if cardsfolder doesn't exist
        return Ok(());
    }
    let card_db = CardDatabase::load_from_cardsfolder(&cardsfolder)?;

    // Load test decks
    let deck_path = PathBuf::from("test_decks/simple_bolt.dck");
    let deck = DeckLoader::load_from_file(&deck_path)?;

    // Initialize game
    let game_init = GameInitializer::new(&card_db);
    let mut game = game_init.init_game(
        "Player 1".to_string(),
        &deck,
        "Player 2".to_string(),
        &deck,
        20, // starting life
    )?;

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
#[test]
fn test_tui_deterministic_with_seed() -> Result<()> {
    // Load card database
    let cardsfolder = PathBuf::from("cardsfolder");
    if !cardsfolder.exists() {
        return Ok(());
    }
    let card_db = CardDatabase::load_from_cardsfolder(&cardsfolder)?;

    // Load test deck
    let deck_path = PathBuf::from("test_decks/simple_bolt.dck");
    let deck = DeckLoader::load_from_file(&deck_path)?;

    // Run the same game twice with the same seed
    let mut results = Vec::new();
    for _ in 0..2 {
        let game_init = GameInitializer::new(&card_db);
        let mut game = game_init.init_game(
            "Player 1".to_string(),
            &deck,
            "Player 2".to_string(),
            &deck,
            20,
        )?;
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
#[test]
fn test_tui_runs_to_completion() -> Result<()> {
    // Load card database
    let cardsfolder = PathBuf::from("cardsfolder");
    if !cardsfolder.exists() {
        return Ok(());
    }
    let card_db = CardDatabase::load_from_cardsfolder(&cardsfolder)?;

    // Load test deck
    let deck_path = PathBuf::from("test_decks/simple_bolt.dck");
    let deck = DeckLoader::load_from_file(&deck_path)?;

    // Run a game and verify it completes
    let game_init = GameInitializer::new(&card_db);
    let mut game = game_init.init_game(
        "Player 1".to_string(),
        &deck,
        "Player 2".to_string(),
        &deck,
        20,
    )?;
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
