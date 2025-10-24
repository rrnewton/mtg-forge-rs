//! End-to-end tests for the TUI
//!
//! These tests verify that the TUI can successfully run complete games
//! from start to finish with various deck configurations.

use mtg_forge_rs::{
    game::{zero_controller::ZeroController, GameLoop, VerbosityLevel},
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
    let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
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

    // With seed 42, Player 2 should win (behavior changed with unified spell ability interface)
    assert_eq!(
        result.winner.unwrap(),
        p2_id,
        "With seed 42, Player 2 should win"
    );

    // Verify the game ended due to player death (Lightning Bolts dealing damage)
    // With the correct spell casting implementation, players cast bolts and deal damage
    assert!(
        matches!(
            result.end_reason,
            mtg_forge_rs::game::GameEndReason::PlayerDeath(_)
        ),
        "Game should end by player death with Lightning Bolts being cast: {:?}",
        result.end_reason
    );

    // Verify damage was dealt (losing player should have 0 or less life)
    let p1_life = game.get_player(p1_id)?.life;
    let p2_life = game.get_player(p2_id)?.life;

    // The losing player should have 0 or less life
    assert!(
        p1_life <= 0 || p2_life <= 0,
        "At least one player should have 0 or less life. P1: {p1_life}, P2: {p2_life}"
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

        let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
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

    let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
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
    assert!(game.get_player(p1_id).is_ok());
    assert!(game.get_player(p2_id).is_ok());

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

    let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
    let p1_id = players[0];
    let p2_id = players[1];

    let mut controller1 = mtg_forge_rs::game::random_controller::RandomController::new(p1_id);
    let mut controller2 = mtg_forge_rs::game::random_controller::RandomController::new(p2_id);

    let mut game_loop = GameLoop::new(&mut game).with_verbosity(VerbosityLevel::Verbose);
    let result = game_loop.run_game(&mut controller1, &mut controller2)?;

    // Verify game completed
    assert!(result.winner.is_some(), "Game should have a winner");

    // Verify that damage was dealt (at least one player should have life != 20)
    let p1_life = game.get_player(p1_id)?.life;
    let p2_life = game.get_player(p2_id)?.life;

    assert!(
        p1_life != 20 || p2_life != 20,
        "At least one player should have taken damage. P1: {p1_life}, P2: {p2_life}"
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
    let loser_life = game.get_player(loser)?.life;
    assert!(
        loser_life <= 0,
        "Loser should have <= 0 life, got {loser_life}"
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

    let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
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
    let mut controller1 = mtg_forge_rs::game::zero_controller::ZeroController::new(p1_id);
    let mut controller2 = mtg_forge_rs::game::zero_controller::ZeroController::new(p2_id);

    // Set Player 1 as active player
    game.turn.active_player = p1_id;

    // Run cleanup step through game loop
    let mut game_loop = GameLoop::new(&mut game).with_verbosity(VerbosityLevel::Verbose);
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

/// Test that games with creature-heavy decks play out correctly
/// This tests vigilance, combat, and creature interactions in a full game
#[tokio::test]
async fn test_creature_combat_game() -> Result<()> {
    // Load card database
    let cardsfolder = PathBuf::from("cardsfolder");
    if !cardsfolder.exists() {
        return Ok(());
    }
    let card_db = CardDatabase::new(cardsfolder);
    card_db.eager_load().await?;

    // Load creature decks
    let vigilance_deck_path = PathBuf::from("test_decks/vigilance_deck.dck");
    let vigilance_deck = DeckLoader::load_from_file(&vigilance_deck_path)?;

    let bears_deck_path = PathBuf::from("test_decks/grizzly_bears.dck");
    let bears_deck = DeckLoader::load_from_file(&bears_deck_path)?;

    // Run a game with random controllers
    let game_init = GameInitializer::new(&card_db);
    let mut game = game_init
        .init_game(
            "Player 1".to_string(),
            &vigilance_deck,
            "Player 2".to_string(),
            &bears_deck,
            20,
        )
        .await?;
    game.rng_seed = 77777;

    let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
    let p1_id = players[0];
    let p2_id = players[1];

    let mut controller1 =
        mtg_forge_rs::game::random_controller::RandomController::with_seed(p1_id, 77777);
    let mut controller2 =
        mtg_forge_rs::game::random_controller::RandomController::with_seed(p2_id, 77778);

    let mut game_loop = GameLoop::new(&mut game).with_verbosity(VerbosityLevel::Silent);
    let result = game_loop.run_game(&mut controller1, &mut controller2)?;

    // Verify game completed
    assert!(result.winner.is_some(), "Game should have a winner");
    assert!(
        result.turns_played > 0,
        "Game should have played some turns"
    );

    // Verify that combat happened (at least one player should have taken damage)
    let p1_life = game.get_player(p1_id)?.life;
    let p2_life = game.get_player(p2_id)?.life;

    assert!(
        p1_life != 20 || p2_life != 20,
        "At least one player should have taken damage from combat. P1: {p1_life}, P2: {p2_life}"
    );

    // Game should end by player death (not deck out) with creature combat
    assert!(
        matches!(
            result.end_reason,
            mtg_forge_rs::game::GameEndReason::PlayerDeath(_)
        ),
        "Game should end by player death with creatures attacking: {:?}",
        result.end_reason
    );

    // Verify the losing player has 0 or less life
    let winner = result.winner.unwrap();
    let loser = if winner == p1_id { p2_id } else { p1_id };
    let loser_life = game.get_player(loser)?.life;
    assert!(
        loser_life <= 0,
        "Loser should have <= 0 life, got {loser_life}"
    );

    // Verify that creatures were summoned (check graveyard has creatures)
    // At minimum, the winner should have creatures in graveyard (died during combat)
    let winner_zones = game.get_player_zones(winner).ok_or_else(|| {
        mtg_forge_rs::MtgError::InvalidAction("Winner zones not found".to_string())
    })?;

    // Count creatures on battlefield (owned by winner)
    let battlefield_creatures = game
        .battlefield
        .cards
        .iter()
        .filter(|&&card_id| {
            if let Ok(card) = game.cards.get(card_id) {
                card.owner == winner && card.is_creature()
            } else {
                false
            }
        })
        .count();

    let total_creatures = battlefield_creatures + winner_zones.graveyard.cards.len();

    assert!(
        total_creatures > 0,
        "Winner should have played at least one creature (battlefield + graveyard)"
    );

    Ok(())
}

/// Test that different deck matchups work correctly
/// Tests a game between two different decks with different strategies
#[tokio::test]
async fn test_different_deck_matchup() -> Result<()> {
    use mtg_forge_rs::core::CardType;

    // Load card database
    let cardsfolder = PathBuf::from("cardsfolder");
    if !cardsfolder.exists() {
        return Ok(());
    }
    let card_db = CardDatabase::new(cardsfolder);
    card_db.eager_load().await?;

    // Load two different decks
    let bolt_deck_path = PathBuf::from("test_decks/simple_bolt.dck");
    let bolt_deck = DeckLoader::load_from_file(&bolt_deck_path)?;

    let bears_deck_path = PathBuf::from("test_decks/grizzly_bears.dck");
    let bears_deck = DeckLoader::load_from_file(&bears_deck_path)?;

    // Run multiple games to test consistency
    for seed in [11111, 22222, 33333] {
        let game_init = GameInitializer::new(&card_db);
        let mut game = game_init
            .init_game(
                "Bolt Player".to_string(),
                &bolt_deck,
                "Bears Player".to_string(),
                &bears_deck,
                20,
            )
            .await?;
        game.rng_seed = seed;

        let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
        let p1_id = players[0];
        let p2_id = players[1];

        let mut controller1 =
            mtg_forge_rs::game::random_controller::RandomController::with_seed(p1_id, seed);
        let mut controller2 =
            mtg_forge_rs::game::random_controller::RandomController::with_seed(p2_id, seed + 1);

        let mut game_loop = GameLoop::new(&mut game).with_verbosity(VerbosityLevel::Silent);
        let result = game_loop.run_game(&mut controller1, &mut controller2)?;

        // Verify game completed successfully
        assert!(
            result.winner.is_some(),
            "Game with seed {seed} should have a winner"
        );
        assert!(
            result.turns_played > 0 && result.turns_played <= 200,
            "Game with seed {} should play reasonable number of turns, got {}",
            seed,
            result.turns_played
        );

        // Verify end reason is valid
        assert!(
            matches!(
                result.end_reason,
                mtg_forge_rs::game::GameEndReason::PlayerDeath(_)
                    | mtg_forge_rs::game::GameEndReason::Decking(_)
            ),
            "Game should end by player death or decking: {:?}",
            result.end_reason
        );

        // Verify both players still exist and have valid state
        let p1 = game.get_player(p1_id)?;
        let p2 = game.get_player(p2_id)?;

        // Winner should have positive life (unless both died simultaneously)
        let winner = result.winner.unwrap();
        let winner_life = if winner == p1_id { p1.life } else { p2.life };

        // Note: Winner might have negative life if both dealt lethal simultaneously
        // but should be better off than the loser
        let loser = if winner == p1_id { p2_id } else { p1_id };
        let loser_life = if loser == p1_id { p1.life } else { p2.life };

        assert!(
            winner_life >= loser_life,
            "Winner should have >= life than loser. Winner: {winner_life}, Loser: {loser_life}"
        );

        // Verify cards are in valid zones (not lost or duplicated)
        let p1_zones = game.get_player_zones(p1_id).ok_or_else(|| {
            mtg_forge_rs::MtgError::InvalidAction("P1 zones not found".to_string())
        })?;
        let p2_zones = game.get_player_zones(p2_id).ok_or_else(|| {
            mtg_forge_rs::MtgError::InvalidAction("P2 zones not found".to_string())
        })?;

        let p1_total = p1_zones.hand.cards.len()
            + p1_zones.library.cards.len()
            + p1_zones.graveyard.cards.len();

        let p2_total = p2_zones.hand.cards.len()
            + p2_zones.library.cards.len()
            + p2_zones.graveyard.cards.len();

        // Also count battlefield creatures (not lands which are shared)
        let p1_battlefield = game
            .battlefield
            .cards
            .iter()
            .filter(|&&card_id| {
                if let Ok(card) = game.cards.get(card_id) {
                    card.owner == p1_id && !card.types.contains(&CardType::Land)
                } else {
                    false
                }
            })
            .count();

        let p2_battlefield = game
            .battlefield
            .cards
            .iter()
            .filter(|&&card_id| {
                if let Ok(card) = game.cards.get(card_id) {
                    card.owner == p2_id && !card.types.contains(&CardType::Land)
                } else {
                    false
                }
            })
            .count();

        // Each player should have <= 60 total cards (some may have been exiled or lost in edge cases)
        assert!(
            p1_total + p1_battlefield <= 60,
            "Player 1 should have at most 60 cards across all zones, got {}",
            p1_total + p1_battlefield
        );

        assert!(
            p2_total + p2_battlefield <= 60,
            "Player 2 should have at most 60 cards across all zones, got {}",
            p2_total + p2_battlefield
        );
    }

    Ok(())
}
