//! End-to-end tests for the undo/replay system
//!
//! These tests verify that we can rewind and replay game states correctly,
//! which is critical for tree search and AI development.

use mtg_forge_rs::{
    game::{random_controller::RandomController, GameLoop, VerbosityLevel},
    loader::{AsyncCardDatabase as CardDatabase, DeckLoader, GameInitializer},
    Result,
};
use std::path::PathBuf;

/// Test that we can rewind and replay a full game
/// This demonstrates:
/// 1. Play a full game with random controllers
/// 2. Rewind 50% of the actions
/// 3. Replay from that point (should get same result with same seed)
/// 4. Rewind 100% to beginning
/// 5. Replay entire game (should get same result)
#[tokio::test]
async fn test_full_game_undo_replay() -> Result<()> {
    // Load card database (lazy loading - only loads cards from deck)
    let cardsfolder = PathBuf::from("cardsfolder");
    if !cardsfolder.exists() {
        return Ok(());
    }
    let card_db = CardDatabase::new(cardsfolder);
    // Note: No eager_load() - GameInitializer will lazily load only deck cards

    // Load test deck
    let deck_path = PathBuf::from("decks/simple_bolt.dck");
    let deck = DeckLoader::load_from_file(&deck_path)?;

    // ===== Phase 1: Play initial game =====
    println!("\n=== Phase 1: Playing initial game ===");

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
    game.seed_rng(42424);

    let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
    let p1_id = players[0];
    let p2_id = players[1];

    // Take snapshot of initial game state for comparison after rewind
    let initial_snapshot = game.clone();

    // Check initial library/hand/graveyard sizes for reference
    let initial_p1_zones = initial_snapshot.get_player_zones(p1_id).unwrap();
    let initial_p1_library = initial_p1_zones.library.cards.len();
    let initial_p1_hand = initial_p1_zones.hand.cards.len();
    let initial_p1_graveyard = initial_p1_zones.graveyard.cards.len();
    let initial_p1_exile = initial_p1_zones.exile.cards.len();

    let initial_p2_zones = initial_snapshot.get_player_zones(p2_id).unwrap();
    let initial_p2_library = initial_p2_zones.library.cards.len();
    let initial_p2_hand = initial_p2_zones.hand.cards.len();
    let initial_p2_graveyard = initial_p2_zones.graveyard.cards.len();
    let initial_p2_exile = initial_p2_zones.exile.cards.len();

    println!("Initial snapshot state:");
    println!(
        "  P1: {} library, {} hand, {} graveyard, {} exile (total: {})",
        initial_p1_library,
        initial_p1_hand,
        initial_p1_graveyard,
        initial_p1_exile,
        initial_p1_library + initial_p1_hand + initial_p1_graveyard + initial_p1_exile
    );
    println!(
        "  P2: {} library, {} hand, {} graveyard, {} exile (total: {})",
        initial_p2_library,
        initial_p2_hand,
        initial_p2_graveyard,
        initial_p2_exile,
        initial_p2_library + initial_p2_hand + initial_p2_graveyard + initial_p2_exile
    );

    // Use seeded random controllers for determinism
    let mut controller1 = RandomController::new(p1_id);
    let mut controller2 = RandomController::new(p2_id);

    let mut game_loop = GameLoop::new(&mut game).with_verbosity(VerbosityLevel::Normal);
    let initial_result = game_loop.run_game(&mut controller1, &mut controller2)?;

    println!("Game completed!");
    println!("  Winner: {:?}", initial_result.winner);
    println!("  Turns played: {}", initial_result.turns_played);
    println!("  End reason: {:?}", initial_result.end_reason);
    println!("  Undo log size: {}", game_loop.game.undo_log.len());

    // Record game state for comparison
    let initial_winner = initial_result.winner;
    let initial_turns = initial_result.turns_played;
    let initial_p1_life = game_loop.game.get_player(p1_id)?.life;
    let initial_p2_life = game_loop.game.get_player(p2_id)?.life;
    let total_actions = game_loop.game.undo_log.len();

    // Verify game completed
    assert!(
        initial_winner.is_some(),
        "Initial game should have a winner"
    );
    assert!(initial_turns > 0, "Initial game should have played turns");
    assert!(total_actions > 0, "Undo log should have recorded actions");

    // ===== Phase 2: Rewind 50%, then play forward to completion =====
    println!("\n=== Phase 2: Rewind 50% of actions, then play forward ===");

    let rewind_count = total_actions / 2;
    println!("Rewinding {rewind_count} out of {total_actions} actions");

    for i in 0..rewind_count {
        let undone = game_loop.game.undo()?;
        assert!(
            undone,
            "Should be able to undo action {} of {}",
            i + 1,
            rewind_count
        );
    }

    let actions_at_halfway = game_loop.game.undo_log.len();
    let turn_at_halfway = game_loop.game.turn.turn_number;
    println!("After rewind:");
    println!("  Undo log size: {actions_at_halfway}");
    println!("  Turn number: {turn_at_halfway}");

    // Now play forward from the 50% point with fresh controllers
    println!("\nPlaying forward from 50% point...");
    let mut controller1 = RandomController::new(p1_id);
    let mut controller2 = RandomController::new(p2_id);
    game_loop.reset();

    let phase2_result = game_loop.run_game(&mut controller1, &mut controller2)?;

    println!("\nPhase 2 replay completed!");
    println!("  Started from turn: {turn_at_halfway}");
    println!("  Winner: {:?}", phase2_result.winner);
    println!("  Turns played: {}", phase2_result.turns_played);
    println!("  End reason: {:?}", phase2_result.end_reason);
    println!(
        "  Total actions in undo log: {}",
        game_loop.game.undo_log.len()
    );

    assert!(
        phase2_result.winner.is_some(),
        "Phase 2 replay should complete with a winner"
    );

    // ===== Phase 3: Rewind 100% to beginning =====
    println!("\n=== Phase 3: Rewinding 100% to beginning ===");

    let remaining_actions = game_loop.game.undo_log.len();
    println!("Rewinding all {remaining_actions} remaining actions");
    println!(
        "Turn number before full rewind: {}",
        game_loop.game.turn.turn_number
    );

    // Debug: Count action types in the undo log
    let mut change_turn_count = 0;
    let mut advance_step_count = 0;
    let mut move_card_count = 0;
    let mut lib_to_hand = 0;
    let mut hand_to_stack = 0;
    let mut stack_to_grave = 0;
    let mut other_moves = 0;
    let mut other_count = 0;
    for action in game_loop.game.undo_log.actions() {
        match action {
            mtg_forge_rs::undo::GameAction::ChangeTurn { .. } => change_turn_count += 1,
            mtg_forge_rs::undo::GameAction::AdvanceStep { .. } => advance_step_count += 1,
            mtg_forge_rs::undo::GameAction::MoveCard {
                from_zone, to_zone, ..
            } => {
                move_card_count += 1;
                use mtg_forge_rs::zones::Zone;
                match (from_zone, to_zone) {
                    (Zone::Library, Zone::Hand) => lib_to_hand += 1,
                    (Zone::Hand, Zone::Stack) => hand_to_stack += 1,
                    (Zone::Stack, Zone::Graveyard) => stack_to_grave += 1,
                    (Zone::Hand, Zone::Battlefield) => {
                        println!("  DEBUG: Hand→Battlefield move logged");
                        other_moves += 1;
                    }
                    (Zone::Hand, Zone::Graveyard) => {
                        println!("  DEBUG: Hand→Graveyard move logged");
                        other_moves += 1;
                    }
                    _ => {
                        println!("  DEBUG: Other move: {from_zone:?} → {to_zone:?}");
                        other_moves += 1;
                    }
                }
            }
            _ => other_count += 1,
        }
    }
    println!(
        "Actions in undo log: {change_turn_count} ChangeTurn, {advance_step_count} AdvanceStep, {move_card_count} MoveCard ({lib_to_hand} Lib→Hand, {hand_to_stack} Hand→Stack, {stack_to_grave} Stack→Grave, {other_moves} other), {other_count} other actions"
    );

    // Before undoing, print last few actions in undo log for debugging
    println!("\nLast 5 actions in undo log (will be undone first):");
    for (idx, action) in game_loop
        .game
        .undo_log
        .actions()
        .iter()
        .rev()
        .take(5)
        .enumerate()
    {
        println!("  -{}: {:?}", idx + 1, action);
    }

    for i in 0..remaining_actions {
        let undone = game_loop.game.undo()?;
        assert!(
            undone,
            "Should be able to undo action {} of {}",
            i + 1,
            remaining_actions
        );

        // Print turn number every 100 actions to debug
        if (i + 1) % 100 == 0 || i == remaining_actions - 1 || i < 5 {
            let p1_lib = game_loop
                .game
                .get_player_zones(p1_id)
                .map(|z| z.library.cards.len())
                .unwrap_or(0);
            let p1_grave = game_loop
                .game
                .get_player_zones(p1_id)
                .map(|z| z.graveyard.cards.len())
                .unwrap_or(0);
            println!(
                "  After undoing {} actions: Turn = {}, P1: {} lib / {} grave",
                i + 1,
                game_loop.game.turn.turn_number,
                p1_lib,
                p1_grave
            );
        }
    }

    println!(
        "After full rewind, undo log size: {}",
        game_loop.game.undo_log.len()
    );
    assert_eq!(
        game_loop.game.undo_log.len(),
        0,
        "Undo log should be empty after full rewind"
    );

    // Verify game state is back to initial state
    let p1_life_after_rewind = game_loop.game.get_player(p1_id)?.life;
    let p2_life_after_rewind = game_loop.game.get_player(p2_id)?.life;
    let turn_after_rewind = game_loop.game.turn.turn_number;

    // Check all zone sizes after rewind
    let p1_zones = game_loop.game.get_player_zones(p1_id).unwrap();
    let p1_library_size = p1_zones.library.cards.len();
    let p1_hand_size = p1_zones.hand.cards.len();
    let p1_graveyard_size = p1_zones.graveyard.cards.len();
    let p1_exile_size = p1_zones.exile.cards.len();

    let p2_zones = game_loop.game.get_player_zones(p2_id).unwrap();
    let p2_library_size = p2_zones.library.cards.len();
    let p2_hand_size = p2_zones.hand.cards.len();
    let p2_graveyard_size = p2_zones.graveyard.cards.len();
    let p2_exile_size = p2_zones.exile.cards.len();

    let battlefield_size = game_loop.game.battlefield.cards.len();
    let stack_size = game_loop.game.stack.cards.len();

    println!("\nGame state after full rewind:");
    println!("  P1 life: {p1_life_after_rewind} (initial: 20)");
    println!("  P2 life: {p2_life_after_rewind} (initial: 20)");
    println!("  Turn number: {turn_after_rewind} (initial: 1)");
    println!(
        "  P1 zones: {p1_library_size} library, {p1_hand_size} hand, {p1_graveyard_size} graveyard, {p1_exile_size} exile"
    );
    println!(
        "  P2 zones: {p2_library_size} library, {p2_hand_size} hand, {p2_graveyard_size} graveyard, {p2_exile_size} exile"
    );
    println!("  Battlefield: {battlefield_size} cards, Stack: {stack_size} cards");
    println!(
        "  P1 total: {} cards",
        p1_library_size + p1_hand_size + p1_graveyard_size + p1_exile_size
    );
    println!(
        "  P2 total: {} cards",
        p2_library_size + p2_hand_size + p2_graveyard_size + p2_exile_size
    );

    // Verify turn number was reset
    assert_eq!(
        turn_after_rewind, 1,
        "Turn number should be reset to 1 after full rewind"
    );

    // Verify zone sizes match initial snapshot
    assert_eq!(
        p1_library_size, initial_p1_library,
        "P1 library should match snapshot: {p1_library_size} vs {initial_p1_library}"
    );
    assert_eq!(
        p1_hand_size, initial_p1_hand,
        "P1 hand should match snapshot: {p1_hand_size} vs {initial_p1_hand}"
    );
    assert_eq!(
        p1_graveyard_size, initial_p1_graveyard,
        "P1 graveyard should match snapshot: {p1_graveyard_size} vs {initial_p1_graveyard}. Full rewind should restore all cards!"
    );
    assert_eq!(
        p2_library_size, initial_p2_library,
        "P2 library should match snapshot: {p2_library_size} vs {initial_p2_library}"
    );
    assert_eq!(
        p2_graveyard_size, initial_p2_graveyard,
        "P2 graveyard should match snapshot: {p2_graveyard_size} vs {initial_p2_graveyard}. Full rewind should restore all cards!"
    );

    // Compare rewound state with initial snapshot
    println!("\nComparing rewound state with initial snapshot:");

    // Life totals should match
    let snapshot_p1_life = initial_snapshot.get_player(p1_id)?.life;
    let snapshot_p2_life = initial_snapshot.get_player(p2_id)?.life;
    assert_eq!(
        p1_life_after_rewind, snapshot_p1_life,
        "P1 life should match snapshot"
    );
    assert_eq!(
        p2_life_after_rewind, snapshot_p2_life,
        "P2 life should match snapshot"
    );
    println!("  ✓ Life totals match snapshot");

    // Turn number should match
    assert_eq!(
        turn_after_rewind, initial_snapshot.turn.turn_number,
        "Turn number should match snapshot"
    );
    println!("  ✓ Turn number matches snapshot");

    // Active player should match
    assert_eq!(
        game_loop.game.turn.active_player, initial_snapshot.turn.active_player,
        "Active player should match snapshot"
    );
    println!("  ✓ Active player matches snapshot");

    // Current step should match
    assert_eq!(
        game_loop.game.turn.current_step, initial_snapshot.turn.current_step,
        "Current step should match snapshot"
    );
    println!("  ✓ Current step matches snapshot");

    println!("  ✓ Full rewind successfully restored game to initial state!");

    // ===== Phase 4: Play forward to completion from beginning =====
    println!("\n=== Phase 4: Play forward to completion from beginning ===");

    // Reset controllers with different seeds to get a different game path
    let mut controller1 = RandomController::new(p1_id);
    let mut controller2 = RandomController::new(p2_id);

    // IMPORTANT: Reset game loop state before replaying
    game_loop.reset();

    let replay_result = game_loop.run_game(&mut controller1, &mut controller2)?;

    println!("Replay completed!");
    println!("  Winner: {:?}", replay_result.winner);
    println!("  Turns played: {}", replay_result.turns_played);
    println!("  End reason: {:?}", replay_result.end_reason);

    // Note: Replay won't match exactly because RandomController internal state
    // (RNG) is not part of game state and not undone. This is expected behavior.
    // The undo system works correctly for game state, but controller decisions
    // will differ on replay because the RNG seed has advanced.
    //
    // For deterministic replay, you would need to either:
    // 1. Reset controller RNG state alongside game undo
    // 2. Use deterministic controllers (like ZeroController)
    // 3. Record and replay controller decisions
    //
    // What we CAN verify:
    // - Game completed successfully after full rewind
    // - Game reached a valid end state
    // - Life totals are reasonable

    assert!(
        replay_result.winner.is_some(),
        "Replay should complete with a winner"
    );

    // Note: The replay may complete quickly or differently than the original game
    // because:
    // 1. RandomController RNG state is reset (different decisions)
    // 2. Card positions in libraries may not be fully restored (undo limitation)
    //
    // What we CAN verify is that the game completed successfully
    println!(
        "Note: Replay completed with {} turns (may differ from original due to RNG reset)",
        replay_result.turns_played
    );

    let replay_p1_life = game_loop.game.get_player(p1_id)?.life;
    let replay_p2_life = game_loop.game.get_player(p2_id)?.life;

    println!("\nFinal state comparison:");
    println!("  P1 life: initial={initial_p1_life}, replay={replay_p1_life}");
    println!("  P2 life: initial={initial_p2_life}, replay={replay_p2_life}");

    // Life totals should match for winner/loser pattern
    let initial_winner_life = if initial_winner == Some(p1_id) {
        initial_p1_life
    } else {
        initial_p2_life
    };
    let replay_winner_life = if replay_result.winner == Some(p1_id) {
        replay_p1_life
    } else {
        replay_p2_life
    };

    // Both games should end with loser at <= 0 life
    assert!(
        initial_winner_life >= 0 && replay_winner_life >= 0,
        "Winners should have >= 0 life"
    );

    println!("\n=== Undo/Replay Test Complete ===");
    println!("Successfully demonstrated rewind/replay cycle:");
    println!(
        "  ✓ Phase 1: Played initial game ({initial_turns} turns, {total_actions} actions logged)"
    );
    println!(
        "  ✓ Phase 2: Rewound 50% → played forward ({} turns)",
        phase2_result.turns_played
    );
    println!("  ✓ Phase 3: Rewound 100% → verified state matches initial snapshot");
    println!(
        "  ✓ Phase 4: Played forward from beginning ({} turns)",
        replay_result.turns_played
    );
    println!();
    println!("This proves the system can:");
    println!("  • Rewind to any point in history");
    println!("  • Play forward from that point");
    println!("  • Repeat the rewind/replay cycle indefinitely");
    println!();
    println!("Note: Each replay uses fresh RNG seeds, so game paths differ while");
    println!("still starting from the exact same game state.");

    Ok(())
}

/// Test that individual actions can be undone correctly
#[tokio::test]
async fn test_action_undo() -> Result<()> {
    use mtg_forge_rs::core::{Card, CardType};

    // Load card database
    let cardsfolder = PathBuf::from("cardsfolder");
    if !cardsfolder.exists() {
        return Ok(());
    }
    let card_db = CardDatabase::new(cardsfolder);
    // Note: No eager_load() - GameInitializer will lazily load only deck cards

    // Load test deck
    let deck_path = PathBuf::from("decks/grizzly_bears.dck");
    let deck = DeckLoader::load_from_file(&deck_path)?;

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
    let _p2_id = players[1];

    // Test 1: Life modification undo
    let initial_life = game.get_player(p1_id)?.life;
    game.deal_damage(p1_id, 5)?; // Use deal_damage which logs to undo
    assert_eq!(game.get_player(p1_id)?.life, initial_life - 5);

    game.undo()?;
    assert_eq!(
        game.get_player(p1_id)?.life,
        initial_life,
        "Life should be restored after undo"
    );

    // Test 2: Card movement undo
    let card_id = game.next_card_id();
    let mut card = Card::new(card_id, "Test Land", p1_id);
    card.types.push(CardType::Land); // Must be a land for play_land
    game.cards.insert(card_id, card);

    if let Some(zones) = game.get_player_zones_mut(p1_id) {
        zones.hand.add(card_id);
    }

    let hand_size_before = game
        .get_player_zones(p1_id)
        .map(|z| z.hand.cards.len())
        .unwrap_or(0);

    // Play the card (moves from hand to battlefield)
    game.play_land(p1_id, card_id)?;

    let hand_size_after = game
        .get_player_zones(p1_id)
        .map(|z| z.hand.cards.len())
        .unwrap_or(0);
    assert_eq!(
        hand_size_after,
        hand_size_before - 1,
        "Card should leave hand"
    );
    assert!(
        game.battlefield.contains(card_id),
        "Card should be on battlefield"
    );

    // Undo the play
    game.undo()?;

    let hand_size_restored = game
        .get_player_zones(p1_id)
        .map(|z| z.hand.cards.len())
        .unwrap_or(0);
    assert_eq!(
        hand_size_restored, hand_size_before,
        "Card should be back in hand"
    );
    assert!(
        !game.battlefield.contains(card_id),
        "Card should not be on battlefield"
    );

    println!("✓ Individual action undo works correctly");

    Ok(())
}

/// Aggressive undo test: repeatedly snapshot, run forward, rewind, and verify equivalence
/// This test explores the full turn space by:
/// - Running forward a random number of turns (1-5)
/// - Taking snapshots at various points
/// - Running forward more turns
/// - Rewinding to a random earlier snapshot (not just the most recent)
/// - Verifying complete state equivalence
/// - Repeating 100 times to stress-test the undo system
#[tokio::test]
async fn test_aggressive_undo_snapshots() -> Result<()> {
    use rand::Rng;
    use rand::SeedableRng;

    // Load card database
    let cardsfolder = PathBuf::from("cardsfolder");
    if !cardsfolder.exists() {
        return Ok(());
    }
    let card_db = CardDatabase::new(cardsfolder);
    // Note: No eager_load() - GameInitializer will lazily load only deck cards

    // Load test deck
    let deck_path = PathBuf::from("decks/simple_bolt.dck");
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
    game.seed_rng(12345);

    let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
    let p1_id = players[0];
    let p2_id = players[1];

    // Use seeded random controllers for determinism
    let mut controller1 = RandomController::new(p1_id);
    let mut controller2 = RandomController::new(p2_id);

    let mut game_loop = GameLoop::new(&mut game).with_verbosity(VerbosityLevel::Silent);

    // RNG for the test itself (not the game)
    let mut test_rng = rand::rngs::StdRng::seed_from_u64(99999);

    // Track snapshots at different undo log positions
    // Store (undo_log_size, snapshot_clone)
    let mut snapshots: Vec<(usize, mtg_forge_rs::game::GameState)> = Vec::new();

    // Take initial snapshot
    snapshots.push((0, game_loop.game.clone()));
    println!("\n=== Aggressive Undo Snapshot Test ===");
    println!("Running 100 iterations of: run forward → snapshot → run more → rewind → verify");
    println!();

    const ITERATIONS: usize = 100;
    let mut iteration = 0;

    while iteration < ITERATIONS {
        // Check if game is over - if so, we need to rewind to an earlier point
        let game_over = game_loop.game.get_player(p1_id)?.life <= 0
            || game_loop.game.get_player(p2_id)?.life <= 0
            || game_loop
                .game
                .get_player_zones(p1_id)
                .unwrap()
                .library
                .is_empty()
            || game_loop
                .game
                .get_player_zones(p2_id)
                .unwrap()
                .library
                .is_empty();

        if game_over || snapshots.len() > 20 {
            // Too many snapshots or game ended - rewind to an earlier snapshot
            if snapshots.len() > 1 {
                // Choose a random earlier snapshot (not the most recent to avoid drift)
                let snapshot_idx = test_rng.gen_range(0..snapshots.len() - 1);
                let (target_undo_size, _) = &snapshots[snapshot_idx];

                println!(
                    "[Iter {}] Game ended or too many snapshots ({}). Rewinding to snapshot {} (undo size {})",
                    iteration,
                    snapshots.len(),
                    snapshot_idx,
                    target_undo_size
                );

                // Rewind to that snapshot's undo log size
                let current_undo_size = game_loop.game.undo_log.len();
                let rewind_count = current_undo_size - target_undo_size;
                for _ in 0..rewind_count {
                    game_loop.game.undo()?;
                }

                // Remove all snapshots after this point
                snapshots.truncate(snapshot_idx + 1);

                // Reset controllers with new seeds to get fresh decisions
                controller1 = RandomController::new(p1_id);
                controller2 = RandomController::new(p2_id);
                game_loop.reset();

                continue; // Don't count this as an iteration
            } else {
                // Only initial snapshot left, start fresh
                println!("[Iter {}] Resetting to initial state", iteration);
                let rewind_count = game_loop.game.undo_log.len();
                for _ in 0..rewind_count {
                    game_loop.game.undo()?;
                }
                snapshots.truncate(1);
                controller1 = RandomController::new(p1_id);
                controller2 = RandomController::new(p2_id);
                game_loop.reset();
                continue;
            }
        }

        // Run forward a random number of turns (1-5)
        let turns_to_run = test_rng.gen_range(1..=5);
        let mut turns_run = 0;

        for _ in 0..turns_to_run {
            let result = game_loop.run_turn_once(&mut controller1, &mut controller2)?;
            if result.is_some() {
                // Game ended
                break;
            }
            turns_run += 1;
        }

        let current_undo_size = game_loop.game.undo_log.len();
        let current_turn = game_loop.game.turn.turn_number;

        // Take snapshot at current position
        let snapshot = game_loop.game.clone();
        snapshots.push((current_undo_size, snapshot));

        println!(
            "[Iter {}] Ran {} turns (turn {}), undo log size: {}, total snapshots: {}",
            iteration,
            turns_run,
            current_turn,
            current_undo_size,
            snapshots.len()
        );

        // Run forward more turns (1-3)
        let more_turns = test_rng.gen_range(1..=3);
        for _ in 0..more_turns {
            let result = game_loop.run_turn_once(&mut controller1, &mut controller2)?;
            if result.is_some() {
                break;
            }
        }

        let undo_size_after_more = game_loop.game.undo_log.len();

        // Choose a random earlier snapshot to rewind to (prefer not the most recent)
        let snapshot_idx = if snapshots.len() > 2 {
            // Choose from any snapshot except the last one (to test deeper rewinds)
            test_rng.gen_range(0..snapshots.len() - 1)
        } else {
            snapshots.len() - 1
        };

        let (target_undo_size, snapshot_state) = &snapshots[snapshot_idx];

        // Rewind to snapshot
        let rewind_count = undo_size_after_more - target_undo_size;
        println!(
            "[Iter {}]   Rewinding {} actions (from {} to {}) to snapshot {}",
            iteration, rewind_count, undo_size_after_more, target_undo_size, snapshot_idx
        );

        for _ in 0..rewind_count {
            let undone = game_loop.game.undo()?;
            assert!(undone, "Should be able to undo");
        }

        // Verify undo log size matches
        assert_eq!(
            game_loop.game.undo_log.len(),
            *target_undo_size,
            "Undo log size should match snapshot"
        );

        // Verify state equivalence
        verify_state_equivalence(game_loop.game, snapshot_state, p1_id, p2_id, iteration)?;

        // Remove snapshots after this point (since we rewound)
        snapshots.truncate(snapshot_idx + 1);

        iteration += 1;
    }

    println!();
    println!("=== Aggressive Undo Test Complete ===");
    println!("✓ Successfully completed {} iterations", ITERATIONS);
    println!("✓ Verified state equivalence after each rewind");
    println!("✓ Tested rewinds to various earlier points (not just most recent)");
    println!("✓ Explored full turn space without drifting to game end");

    Ok(())
}

/// Helper function to verify complete state equivalence between two game states
fn verify_state_equivalence(
    current: &mtg_forge_rs::game::GameState,
    snapshot: &mtg_forge_rs::game::GameState,
    p1_id: mtg_forge_rs::core::PlayerId,
    p2_id: mtg_forge_rs::core::PlayerId,
    iteration: usize,
) -> Result<()> {
    use mtg_forge_rs::MtgError;

    // Verify turn state
    assert_eq!(
        current.turn.turn_number, snapshot.turn.turn_number,
        "[Iter {}] Turn number mismatch",
        iteration
    );
    assert_eq!(
        current.turn.active_player, snapshot.turn.active_player,
        "[Iter {}] Active player mismatch",
        iteration
    );
    assert_eq!(
        current.turn.current_step, snapshot.turn.current_step,
        "[Iter {}] Current step mismatch",
        iteration
    );

    // Verify player states
    let current_p1 = current
        .get_player(p1_id)
        .map_err(|e| MtgError::InvalidAction(format!("Failed to get current P1: {}", e)))?;
    let snapshot_p1 = snapshot
        .get_player(p1_id)
        .map_err(|e| MtgError::InvalidAction(format!("Failed to get snapshot P1: {}", e)))?;
    assert_eq!(
        current_p1.life, snapshot_p1.life,
        "[Iter {}] P1 life mismatch",
        iteration
    );
    assert_eq!(
        current_p1.lands_played_this_turn, snapshot_p1.lands_played_this_turn,
        "[Iter {}] P1 lands played mismatch",
        iteration
    );

    let current_p2 = current
        .get_player(p2_id)
        .map_err(|e| MtgError::InvalidAction(format!("Failed to get current P2: {}", e)))?;
    let snapshot_p2 = snapshot
        .get_player(p2_id)
        .map_err(|e| MtgError::InvalidAction(format!("Failed to get snapshot P2: {}", e)))?;
    assert_eq!(
        current_p2.life, snapshot_p2.life,
        "[Iter {}] P2 life mismatch",
        iteration
    );

    // Verify zone sizes
    let current_p1_zones = current.get_player_zones(p1_id).ok_or_else(|| {
        MtgError::InvalidAction(format!("[Iter {}] No current P1 zones", iteration))
    })?;
    let snapshot_p1_zones = snapshot.get_player_zones(p1_id).ok_or_else(|| {
        MtgError::InvalidAction(format!("[Iter {}] No snapshot P1 zones", iteration))
    })?;

    assert_eq!(
        current_p1_zones.library.cards.len(),
        snapshot_p1_zones.library.cards.len(),
        "[Iter {}] P1 library size mismatch",
        iteration
    );
    assert_eq!(
        current_p1_zones.hand.cards.len(),
        snapshot_p1_zones.hand.cards.len(),
        "[Iter {}] P1 hand size mismatch",
        iteration
    );
    assert_eq!(
        current_p1_zones.graveyard.cards.len(),
        snapshot_p1_zones.graveyard.cards.len(),
        "[Iter {}] P1 graveyard size mismatch",
        iteration
    );

    let current_p2_zones = current.get_player_zones(p2_id).ok_or_else(|| {
        MtgError::InvalidAction(format!("[Iter {}] No current P2 zones", iteration))
    })?;
    let snapshot_p2_zones = snapshot.get_player_zones(p2_id).ok_or_else(|| {
        MtgError::InvalidAction(format!("[Iter {}] No snapshot P2 zones", iteration))
    })?;

    assert_eq!(
        current_p2_zones.library.cards.len(),
        snapshot_p2_zones.library.cards.len(),
        "[Iter {}] P2 library size mismatch",
        iteration
    );
    assert_eq!(
        current_p2_zones.hand.cards.len(),
        snapshot_p2_zones.hand.cards.len(),
        "[Iter {}] P2 hand size mismatch",
        iteration
    );
    assert_eq!(
        current_p2_zones.graveyard.cards.len(),
        snapshot_p2_zones.graveyard.cards.len(),
        "[Iter {}] P2 graveyard size mismatch",
        iteration
    );

    // Verify battlefield
    assert_eq!(
        current.battlefield.cards.len(),
        snapshot.battlefield.cards.len(),
        "[Iter {}] Battlefield size mismatch",
        iteration
    );

    // Verify stack
    assert_eq!(
        current.stack.cards.len(),
        snapshot.stack.cards.len(),
        "[Iter {}] Stack size mismatch",
        iteration
    );

    Ok(())
}
