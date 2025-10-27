//! Game loop implementation
//!
//! Manages the main game loop, turn progression, and priority system

/// Macro for conditional logging that avoids allocation when feature is disabled
///
/// When verbose-logging feature is disabled, this becomes a no-op at compile time,
/// eliminating all format! allocations that are a major performance bottleneck.
macro_rules! log_if_verbose {
    ($self:expr, $($arg:tt)*) => {
        #[cfg(feature = "verbose-logging")]
        {
            $self.log_normal(&format!($($arg)*));
        }
        #[cfg(not(feature = "verbose-logging"))]
        {
            let _ = &$self; // Suppress unused variable warning
        }
    };
}

use crate::core::{CardId, PlayerId};
use crate::game::controller::GameStateView;
use crate::game::controller::PlayerController;
use crate::game::phase::Step;
use crate::game::GameState;
use crate::{MtgError, Result};

// Legacy v1 action type (kept for compatibility with dead code)
#[allow(dead_code)]
#[derive(Debug, Clone)]
enum PlayerAction {
    PlayLand(CardId),
    CastSpell {
        card_id: CardId,
        targets: Vec<CardId>,
    },
    TapForMana(CardId),
    DeclareAttacker(CardId),
    DeclareBlocker {
        blocker: CardId,
        attackers: Vec<CardId>,
    },
    FinishDeclareAttackers,
    FinishDeclareBlockers,
    PassPriority,
}
use smallvec::SmallVec;

/// Verbosity level for game output
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum VerbosityLevel {
    /// Silent - no output during game
    Silent = 0,
    /// Minimal - only game outcome
    Minimal = 1,
    /// Normal - turns, steps, and key actions (default)
    #[default]
    Normal = 2,
    /// Verbose - all actions and state changes
    Verbose = 3,
}

/// Result of running a game to completion
#[derive(Debug, Clone)]
pub struct GameResult {
    /// Winner of the game (None if draw or game didn't complete)
    pub winner: Option<PlayerId>,
    /// Total number of turns played
    pub turns_played: u32,
    /// Reason the game ended
    pub end_reason: GameEndReason,
}

/// Reason the game ended
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameEndReason {
    /// A player won by reducing opponent's life to 0 or less
    PlayerDeath(PlayerId),
    /// A player won by decking their opponent
    Decking(PlayerId),
    /// Game reached maximum turn limit
    TurnLimit,
    /// Game ended in a draw
    Draw,
    /// Game was manually ended
    Manual,
    /// Game was stopped to save a snapshot
    Snapshot,
}

/// Game loop manager
///
/// Handles turn progression, priority, and win condition checking
pub struct GameLoop<'a> {
    /// The game state
    pub game: &'a mut GameState,
    /// Maximum turns before forcing a draw
    max_turns: u32,
    /// Turn counter for the loop
    turns_elapsed: u32,
    /// Verbosity level for output (cached from game.logger)
    pub verbosity: VerbosityLevel,
    /// Track if current step header has been printed (for lazy printing)
    step_header_printed: bool,
    /// Track targets for spells on the stack (spell_id -> chosen_targets)
    /// This is needed because targets are chosen at cast time but used at resolution time
    spell_targets: Vec<(CardId, Vec<CardId>)>,
    /// Global choice counter for tracking all player choices
    /// Increments each time a controller makes any decision
    choice_counter: u32,
}

impl<'a> GameLoop<'a> {
    /// Create a new game loop for the given game state
    pub fn new(game: &'a mut GameState) -> Self {
        let verbosity = game.logger.verbosity();
        GameLoop {
            game,
            max_turns: 1000, // Default maximum turns
            turns_elapsed: 0,
            verbosity,
            step_header_printed: false,
            spell_targets: Vec::new(),
            choice_counter: 0,
        }
    }

    /// Set maximum turns before forcing a draw
    pub fn with_max_turns(mut self, max_turns: u32) -> Self {
        self.max_turns = max_turns;
        self
    }

    /// Set verbosity level for output
    ///
    /// This sets the verbosity on both the game loop and the game's centralized logger,
    /// which is accessed by controllers via GameStateView.
    pub fn with_verbosity(mut self, verbosity: VerbosityLevel) -> Self {
        self.verbosity = verbosity;
        self.game.logger.set_verbosity(verbosity);
        self
    }

    /// Set initial turn counter (for resuming from snapshots)
    ///
    /// This should be called when loading a game from a snapshot to ensure
    /// turn numbering continues correctly.
    pub fn with_turn_counter(mut self, turns_elapsed: u32) -> Self {
        self.turns_elapsed = turns_elapsed;
        self
    }

    /// Enable verbose output (deprecated, use with_verbosity)
    #[deprecated(note = "Use with_verbosity instead")]
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        let verbosity = if verbose {
            VerbosityLevel::Verbose
        } else {
            VerbosityLevel::Silent
        };
        self.verbosity = verbosity;
        self.game.logger.set_verbosity(verbosity);
        self
    }

    /// Reset the game loop state (turn counter, step header flag)
    ///
    /// Call this after rewinding game state to prepare for replay.
    /// Note: This does NOT reset the underlying GameState - use game.undo() for that.
    pub fn reset(&mut self) {
        self.turns_elapsed = 0;
        self.step_header_printed = false;
        self.spell_targets.clear();
        self.choice_counter = 0;
        self.game.logger.reset_step_header();
    }

    /// Log a choice point to the undo log and increment choice counter
    ///
    /// Call this every time a controller makes a decision.
    fn log_choice_point(&mut self, player_id: PlayerId) {
        self.choice_counter += 1;
        self.game
            .undo_log
            .log(crate::undo::GameAction::ChoicePoint {
                player_id,
                choice_id: self.choice_counter,
            });
    }

    /// Run the game loop with the given player controllers
    ///
    /// Returns when the game reaches a win condition or turn limit
    pub fn run_game(
        &mut self,
        controller1: &mut dyn PlayerController,
        controller2: &mut dyn PlayerController,
    ) -> Result<GameResult> {
        // Setup: verify controllers and shuffle libraries
        let (player1_id, player2_id) = self.setup_game(controller1, controller2)?;

        // Main game loop - repeatedly run turns until game ends
        loop {
            // Run one turn and check if game should end
            if let Some(result) = self.run_turn_once(controller1, controller2)? {
                // Notify controllers of game end
                self.notify_game_end(
                    controller1,
                    controller2,
                    player1_id,
                    player2_id,
                    result.winner,
                );
                return Ok(result);
            }
        }
    }

    /// Run a bounded number of turns
    ///
    /// This is a convenience method for testing that runs up to `turns_to_run` turns,
    /// stopping early if the game ends.
    ///
    /// Returns:
    /// - `Ok(GameResult)` with the game outcome if the game ended
    /// - `Ok(GameResult)` with `GameEndReason::Manual` if all turns completed without ending
    pub fn run_turns(
        &mut self,
        controller1: &mut dyn PlayerController,
        controller2: &mut dyn PlayerController,
        turns_to_run: u32,
    ) -> Result<GameResult> {
        for _ in 0..turns_to_run {
            if let Some(result) = self.run_turn_once(controller1, controller2)? {
                // Game ended, return the result
                return Ok(result);
            }
        }

        // Completed all turns without game ending
        Ok(GameResult {
            winner: None,
            turns_played: self.turns_elapsed,
            end_reason: GameEndReason::Manual,
        })
    }

    /// Run the game with stop-and-save snapshot functionality
    ///
    /// This method runs the game but stops after a certain number of player choices
    /// (filtered by the stop condition) and saves a snapshot to disk. The snapshot includes:
    /// - GameState at the most recent turn boundary
    /// - All intra-turn choices made since that boundary
    ///
    /// ## Parameters
    /// - `controller1`, `controller2`: Player controllers
    /// - `p1_id`: Player 1's ID (used for filtering player choices)
    /// - `stop_condition`: Specifies which player's choices to count and how many
    /// - `snapshot_path`: Where to save the snapshot file
    ///
    /// ## Returns
    /// - `Ok(GameResult)` with `GameEndReason::Snapshot` if snapshot was saved
    /// - `Ok(GameResult)` with normal end reason if game finished before limit
    pub fn run_game_with_snapshots<P: AsRef<std::path::Path>>(
        &mut self,
        controller1: &mut dyn PlayerController,
        controller2: &mut dyn PlayerController,
        p1_id: PlayerId,
        stop_condition: &crate::game::StopCondition,
        snapshot_path: P,
    ) -> Result<GameResult> {
        // Setup: verify controllers and shuffle libraries
        let (player1_id, player2_id) = self.setup_game(controller1, controller2)?;

        // Track per-player choices that match the stop condition
        let mut filtered_choice_count: usize = 0;

        // Main game loop - run turns until game ends or choice limit reached
        loop {
            // Check if we've reached the filtered choice limit
            if filtered_choice_count >= stop_condition.choice_count {
                // Save snapshot and return early
                return self.save_snapshot_and_exit(stop_condition.choice_count, snapshot_path);
            }

            // Run one turn and check if game should end
            if let Some(result) = self.run_turn_once(controller1, controller2)? {
                // Game ended normally, notify controllers
                self.notify_game_end(
                    controller1,
                    controller2,
                    player1_id,
                    player2_id,
                    result.winner,
                );
                return Ok(result);
            }

            // Update filtered choice count
            filtered_choice_count = self.count_filtered_choices(p1_id, stop_condition);

            // Check again after the turn completes (in case we hit the limit mid-turn)
            if filtered_choice_count >= stop_condition.choice_count {
                // Save snapshot and return early
                return self.save_snapshot_and_exit(stop_condition.choice_count, snapshot_path);
            }
        }
    }

    /// Count how many choices in the undo log match the stop condition filter
    fn count_filtered_choices(
        &self,
        p1_id: PlayerId,
        stop_condition: &crate::game::StopCondition,
    ) -> usize {
        self.game
            .undo_log
            .actions()
            .iter()
            .filter_map(|action| {
                if let crate::undo::GameAction::ChoicePoint { player_id, .. } = action {
                    if stop_condition.applies_to(p1_id, *player_id) {
                        Some(())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .count()
    }

    /// Set up a game for two-player gameplay
    ///
    /// This verifies that:
    /// - Exactly 2 players exist in the game
    /// - Controllers match the player IDs
    /// - Libraries are shuffled using the game's RNG seed (unless resuming from snapshot)
    ///
    /// Returns the player IDs for both players.
    fn setup_game(
        &mut self,
        controller1: &mut dyn PlayerController,
        controller2: &mut dyn PlayerController,
    ) -> Result<(PlayerId, PlayerId)> {
        // Verify controllers match players (extract exactly 2 player IDs without allocating)
        let (player1_id, player2_id) = {
            let mut players_iter = self.game.players.iter().map(|p| p.id);
            let player1_id = players_iter.next().ok_or_else(|| {
                MtgError::InvalidAction("Game loop requires exactly 2 players".to_string())
            })?;
            let player2_id = players_iter.next().ok_or_else(|| {
                MtgError::InvalidAction("Game loop requires exactly 2 players".to_string())
            })?;
            if players_iter.next().is_some() {
                return Err(MtgError::InvalidAction(
                    "Game loop requires exactly 2 players".to_string(),
                ));
            }
            (player1_id, player2_id)
        };

        if controller1.player_id() != player1_id || controller2.player_id() != player2_id {
            return Err(MtgError::InvalidAction(
                "Controller player IDs don't match game players".to_string(),
            ));
        }

        // Only shuffle libraries if this is a fresh game (not resuming from snapshot)
        // We detect snapshot resume by checking if undo log has actions
        let is_resuming_from_snapshot = !self.game.undo_log.actions().is_empty();

        if !is_resuming_from_snapshot {
            // Shuffle each player's library at game start (MTG Rules 103.2a)
            // This uses the game's RNG which can be seeded for deterministic testing
            use rand::SeedableRng;
            let seed = self.game.rng_seed;
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

            // Extract player IDs to avoid borrow checker issues
            let player_ids: [PlayerId; 2] = [player1_id, player2_id];
            for &player_id in &player_ids {
                if let Some(zones) = self.game.get_player_zones_mut(player_id) {
                    zones.library.shuffle(&mut rng);
                }
            }
        }

        Ok((player1_id, player2_id))
    }

    /// Save a snapshot when choice limit is reached and exit
    ///
    /// This rewinds the undo log to the most recent turn boundary, extracts
    /// intra-turn choices, and saves a GameSnapshot to disk.
    ///
    /// Returns a GameResult with `GameEndReason::Snapshot`.
    fn save_snapshot_and_exit<P: AsRef<std::path::Path>>(
        &mut self,
        choice_limit: usize,
        snapshot_path: P,
    ) -> Result<GameResult> {
        // Rewind to the most recent turn boundary and extract intra-turn choices
        if let Some((turn_number, intra_turn_choices, actions_rewound)) =
            self.game.undo_log.rewind_to_turn_start()
        {
            // Clone the game state at the turn boundary
            let game_state_snapshot = self.game.clone();

            // Create snapshot with state + choices
            let snapshot = crate::game::GameSnapshot::new(
                game_state_snapshot,
                turn_number,
                intra_turn_choices,
            );

            // Save to file
            snapshot
                .save_to_file(&snapshot_path)
                .map_err(|e| MtgError::InvalidAction(format!("Failed to save snapshot: {}", e)))?;

            // Log snapshot info
            if self.verbosity >= VerbosityLevel::Minimal {
                println!("\n=== Snapshot Saved ===");
                println!("  Choice limit reached: {} choices", choice_limit);
                println!("  Snapshot saved to: {}", snapshot_path.as_ref().display());
                println!("  Turn number: {}", turn_number);
                println!("  Intra-turn choices: {}", snapshot.choice_count());
                println!("  Actions rewound: {}", actions_rewound);
            }

            // Return early with Snapshot end reason
            Ok(GameResult {
                winner: None,
                turns_played: self.turns_elapsed,
                end_reason: GameEndReason::Snapshot,
            })
        } else {
            // Failed to rewind to turn start (shouldn't happen)
            Err(MtgError::InvalidAction(
                "Failed to rewind to turn start for snapshot".to_string(),
            ))
        }
    }

    /// Notify both controllers that the game has ended
    ///
    /// Calls the `on_game_end` callback for each controller with their view
    /// of the game state and whether they won.
    fn notify_game_end(
        &self,
        controller1: &mut dyn PlayerController,
        controller2: &mut dyn PlayerController,
        player1_id: PlayerId,
        player2_id: PlayerId,
        winner_id: Option<PlayerId>,
    ) {
        controller1.on_game_end(
            &GameStateView::new(self.game, player1_id),
            winner_id == Some(player1_id),
        );
        controller2.on_game_end(
            &GameStateView::new(self.game, player2_id),
            winner_id == Some(player2_id),
        );
    }

    /// Run a single turn and check for game-ending conditions
    ///
    /// This method runs exactly one turn of the game, including all phases and steps.
    /// After the turn completes, it checks for win conditions and turn limits.
    ///
    /// Returns:
    /// - `Ok(Some(GameResult))` if the game should end (win condition or turn limit reached)
    /// - `Ok(None)` if the game should continue with another turn
    /// - `Err(_)` if an error occurred during turn execution
    pub fn run_turn_once(
        &mut self,
        controller1: &mut dyn PlayerController,
        controller2: &mut dyn PlayerController,
    ) -> Result<Option<GameResult>> {
        // Check win conditions before running the turn
        if let Some(result) = self.check_win_condition() {
            return Ok(Some(result));
        }

        // Check turn limit
        if self.turns_elapsed >= self.max_turns {
            return Ok(Some(GameResult {
                winner: None,
                turns_played: self.turns_elapsed,
                end_reason: GameEndReason::TurnLimit,
            }));
        }

        // Run the turn
        self.run_turn(controller1, controller2)?;
        self.turns_elapsed += 1;

        // Check win conditions after running the turn
        if let Some(result) = self.check_win_condition() {
            return Ok(Some(result));
        }

        // Game continues
        Ok(None)
    }

    /// Run a single turn through all its phases and steps
    ///
    /// This is an internal method that executes one complete turn from untap through cleanup.
    /// For running one turn and checking end conditions, use `run_turn_once` instead.
    fn run_turn(
        &mut self,
        controller1: &mut dyn PlayerController,
        controller2: &mut dyn PlayerController,
    ) -> Result<()> {
        let active_player = self.game.turn.active_player;

        if self.verbosity >= VerbosityLevel::Normal {
            let player_name = self.get_player_name(active_player);
            println!("\n========================================");
            println!("Turn {} - {}'s turn", self.turns_elapsed + 1, player_name);
            println!("========================================");

            // Print detailed battlefield state for both players
            self.print_battlefield_state();
        }

        // Reset turn-based state
        self.reset_turn_state(active_player)?;

        // Run through all steps of the turn
        loop {
            // Execute the step
            self.execute_step(controller1, controller2)?;

            // Try to advance to next step
            // IMPORTANT: Call game.advance_step() not turn.advance_step()
            // to ensure step changes are logged to undo log
            self.game.advance_step()?;

            // Check if we reached end of turn
            if self.game.turn.current_step == crate::game::Step::Untap {
                // We wrapped back to Untap, which means a new turn started
                // The turn change was already logged by advance_step()
                break;
            }
        }

        Ok(())
    }

    /// Get player name for display
    fn get_player_name(&self, player_id: PlayerId) -> String {
        self.game
            .get_player(player_id)
            .map(|p| p.name.to_string())
            .unwrap_or_else(|_| format!("Player {player_id:?}"))
    }

    /// Get step name for display
    fn step_name(&self, step: Step) -> &'static str {
        match step {
            Step::Untap => "Untap Step",
            Step::Upkeep => "Upkeep Step",
            Step::Draw => "Draw Step",
            Step::Main1 => "Main Phase 1",
            Step::BeginCombat => "Beginning of Combat",
            Step::DeclareAttackers => "Declare Attackers Step",
            Step::DeclareBlockers => "Declare Blockers Step",
            Step::CombatDamage => "Combat Damage Step",
            Step::EndCombat => "End of Combat Step",
            Step::Main2 => "Main Phase 2",
            Step::End => "End Step",
            Step::Cleanup => "Cleanup Step",
        }
    }

    /// Print detailed battlefield state for both players
    fn print_battlefield_state(&self) {
        // Print state for each player
        for (idx, player) in self.game.players.iter().enumerate() {
            let is_active = player.id == self.game.turn.active_player;
            let marker = if is_active { " (active)" } else { "" };

            println!("\n{}{}: ", player.name, marker);
            println!("  Life: {}", player.life);

            // Zone sizes
            if let Some(zones) = self.game.get_player_zones(player.id) {
                println!(
                    "  Hand: {} | Library: {} | Graveyard: {} | Exile: {}",
                    zones.hand.len(),
                    zones.library.len(),
                    zones.graveyard.len(),
                    zones.exile.len()
                );
            }

            // Battlefield permanents controlled by this player
            let mut player_permanents: Vec<_> = self
                .game
                .battlefield
                .cards
                .iter()
                .filter_map(|&card_id| {
                    self.game.cards.get(card_id).ok().and_then(|card| {
                        if card.controller == player.id {
                            Some((card_id, card))
                        } else {
                            None
                        }
                    })
                })
                .collect();

            // Sort by card type for better readability: lands first, then creatures, then others
            player_permanents.sort_by_key(|(_, card)| {
                if card.is_land() {
                    0
                } else if card.is_creature() {
                    1
                } else {
                    2
                }
            });

            if player_permanents.is_empty() {
                println!("  Battlefield: (empty)");
            } else {
                println!("  Battlefield:");
                for (card_id, card) in player_permanents {
                    let tap_status = if card.tapped { " (tapped)" } else { "" };

                    // Check for summoning sickness (creatures that entered this turn and don't have haste)
                    let has_summoning_sickness = if card.is_creature() {
                        if let Some(entered_turn) = card.turn_entered_battlefield {
                            entered_turn == self.game.turn.turn_number
                                && !card.has_keyword(&crate::core::Keyword::Haste)
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    let sickness_status = if has_summoning_sickness {
                        " (summoning sickness)"
                    } else {
                        ""
                    };

                    // Format card display based on type
                    if card.is_creature() {
                        let power = card.power.unwrap_or(0) + card.power_bonus as i8;
                        let toughness = card.toughness.unwrap_or(0) + card.toughness_bonus as i8;
                        println!(
                            "    {} ({}) - {}/{}{}{}",
                            card.name, card_id, power, toughness, tap_status, sickness_status
                        );
                    } else {
                        println!("    {} ({}){}", card.name, card_id, tap_status);
                    }
                }
            }

            // Add spacing between players (but not after the last one)
            if idx < self.game.players.len() - 1 {
                println!();
            }
        }
        println!();
    }

    /// Print step header lazily (only when first action happens in this step)
    /// Used for Normal verbosity level
    fn print_step_header_if_needed(&mut self) {
        if self.verbosity == VerbosityLevel::Normal && !self.step_header_printed {
            let step = self.game.turn.current_step;
            println!("--- {} ---", self.step_name(step));
            self.step_header_printed = true;
        }
    }

    // === Logging Helpers ===
    // These methods encapsulate lazy header printing + message output

    /// Log a message at Normal verbosity level (with lazy step header)
    /// Most game events use this level
    fn log_normal(&mut self, message: &str) {
        if self.verbosity >= VerbosityLevel::Normal {
            self.print_step_header_if_needed();
            println!("  {message}");
        }
    }

    /// Log a message at Verbose verbosity level (with lazy step header)
    /// Used for detailed action-by-action logging
    #[allow(dead_code)] // Legacy v1 interface, will be removed
    fn log_verbose(&mut self, message: &str) {
        if self.verbosity >= VerbosityLevel::Verbose {
            self.print_step_header_if_needed();
            println!("  {message}");
        }
    }

    /// Log a message at Minimal verbosity level (no step header needed)
    /// Used for major game events like outcomes
    #[allow(dead_code)]
    fn log_minimal(&mut self, message: &str) {
        if self.verbosity >= VerbosityLevel::Minimal {
            println!("{message}");
        }
    }

    /// Log the execution of a spell effect (damage, draw, etc.)
    fn log_effect_execution(
        &self,
        source_name: &str,
        source_id: CardId,
        effect: &crate::core::Effect,
        _source_owner: PlayerId,
    ) {
        use crate::core::{Effect, TargetRef};

        match effect {
            Effect::DealDamage { target, amount } => match target {
                TargetRef::Player(target_player_id) => {
                    let target_name = self.get_player_name(*target_player_id);
                    println!(
                        "  {source_name} ({source_id}) deals {amount} damage to {target_name}"
                    );
                }
                TargetRef::Permanent(target_card_id) => {
                    let target_name = self
                        .game
                        .cards
                        .get(*target_card_id)
                        .map(|c| c.name.as_str())
                        .unwrap_or("Unknown");
                    println!(
                        "  {source_name} ({source_id}) deals {amount} damage to {target_name} ({target_card_id})"
                    );
                }
                TargetRef::None => {
                    // Target will be filled in by resolve_spell - log against opponent
                    if let Some(opponent_id) = self
                        .game
                        .players
                        .iter()
                        .map(|p| p.id)
                        .find(|id| *id != _source_owner)
                    {
                        let target_name = self.get_player_name(opponent_id);
                        println!(
                            "  {source_name} ({source_id}) deals {amount} damage to {target_name}"
                        );
                    }
                }
            },
            Effect::DrawCards { player, count } => {
                let player_name = self.get_player_name(*player);
                println!(
                    "  {source_name} ({source_id}) causes {player_name} to draw {count} card(s)"
                );
            }
            Effect::GainLife { player, amount } => {
                let player_name = self.get_player_name(*player);
                println!(
                    "  {source_name} ({source_id}) causes {player_name} to gain {amount} life"
                );
            }
            Effect::DestroyPermanent { target } => {
                let target_name = self
                    .game
                    .cards
                    .get(*target)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Unknown");
                println!("  {source_name} ({source_id}) destroys {target_name} ({target})");
            }
            Effect::TapPermanent { target } => {
                let target_name = self
                    .game
                    .cards
                    .get(*target)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Unknown");
                println!("  {source_name} ({source_id}) taps {target_name} ({target})");
            }
            Effect::UntapPermanent { target } => {
                let target_name = self
                    .game
                    .cards
                    .get(*target)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Unknown");
                println!("  {source_name} ({source_id}) untaps {target_name} ({target})");
            }
            Effect::PumpCreature {
                target,
                power_bonus,
                toughness_bonus,
            } => {
                let target_name = self
                    .game
                    .cards
                    .get(*target)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Unknown");
                println!(
                    "  {source_name} ({source_id}) gives {target_name} ({target}) {power_bonus:+}/{toughness_bonus:+} until end of turn"
                );
            }
            Effect::Mill { player, count } => {
                let player_name = self.get_player_name(*player);
                println!(
                    "  {source_name} ({source_id}) causes {player_name} to mill {count} card(s)"
                );
            }
            Effect::CounterSpell { target } => {
                let target_name = self
                    .game
                    .cards
                    .get(*target)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Unknown");
                println!("  {source_name} ({source_id}) counters {target_name} ({target})");
            }
            Effect::AddMana { player, mana } => {
                let player_name = self.get_player_name(*player);
                println!("  {source_name} ({source_id}) adds {mana} to {player_name}'s mana pool");
            }
            Effect::PutCounter {
                target,
                counter_type,
                amount,
            } => {
                let target_name = self
                    .game
                    .cards
                    .get(*target)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Unknown");
                println!(
                    "  {source_name} ({source_id}) puts {amount} {counter_type:?} counter(s) on {target_name} ({target})"
                );
            }
            Effect::RemoveCounter {
                target,
                counter_type,
                amount,
            } => {
                let target_name = self
                    .game
                    .cards
                    .get(*target)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Unknown");
                println!(
                    "  {source_name} ({source_id}) removes {amount} {counter_type:?} counter(s) from {target_name} ({target})"
                );
            }
        }
    }

    /// Reset turn-based state for the active player
    fn reset_turn_state(&mut self, active_player: PlayerId) -> Result<()> {
        // Reset lands played this turn
        if let Ok(player) = self.game.get_player_mut(active_player) {
            player.reset_lands_played();
        }

        // Empty mana pools at start of turn
        // Use fixed-size array instead of Vec allocation (MTG always has 2 players)
        let player_ids: [PlayerId; 2] = [self.game.players[0].id, self.game.players[1].id];
        for player_id in player_ids {
            if let Ok(player) = self.game.get_player_mut(player_id) {
                player.mana_pool.clear();
            }
        }

        Ok(())
    }

    /// Execute a single step
    pub fn execute_step(
        &mut self,
        controller1: &mut dyn PlayerController,
        controller2: &mut dyn PlayerController,
    ) -> Result<()> {
        let step = self.game.turn.current_step;

        // Reset step header tracking for each new step
        self.step_header_printed = false;

        // In verbose mode, always print step header immediately
        if self.verbosity >= VerbosityLevel::Verbose {
            println!("--- {} ---", self.step_name(step));
        }

        match step {
            Step::Untap => self.untap_step(),
            Step::Upkeep => self.upkeep_step(controller1, controller2),
            Step::Draw => self.draw_step(controller1, controller2),
            Step::Main1 | Step::Main2 => self.main_phase(controller1, controller2),
            Step::BeginCombat => self.begin_combat_step(controller1, controller2),
            Step::DeclareAttackers => self.declare_attackers_step(controller1, controller2),
            Step::DeclareBlockers => self.declare_blockers_step(controller1, controller2),
            Step::CombatDamage => self.combat_damage_step(controller1, controller2),
            Step::EndCombat => self.end_combat_step(controller1, controller2),
            Step::End => self.end_step(controller1, controller2),
            Step::Cleanup => self.cleanup_step(controller1, controller2),
        }
    }

    /// Untap step - untap all permanents controlled by active player
    fn untap_step(&mut self) -> Result<()> {
        let active_player = self.game.turn.active_player;

        // Untap all permanents controlled by active player
        // Use SmallVec to avoid heap allocation for typical small counts of tapped cards
        let cards_to_untap: SmallVec<[CardId; 8]> = self
            .game
            .battlefield
            .cards
            .iter()
            .copied()
            .filter(|&card_id| {
                self.game
                    .cards
                    .get(card_id)
                    .map(|c| c.owner == active_player && c.tapped)
                    .unwrap_or(false)
            })
            .collect();

        for card_id in cards_to_untap {
            if let Ok(card) = self.game.cards.get_mut(card_id) {
                card.untap();
            }
        }

        Ok(())
    }

    /// Upkeep step - priority round for triggers and actions
    fn upkeep_step(
        &mut self,
        controller1: &mut dyn PlayerController,
        controller2: &mut dyn PlayerController,
    ) -> Result<()> {
        // TODO: Handle triggered abilities
        // For now, just pass priority
        self.priority_round(controller1, controller2)?;
        Ok(())
    }

    /// Draw step - active player draws a card
    fn draw_step(
        &mut self,
        controller1: &mut dyn PlayerController,
        controller2: &mut dyn PlayerController,
    ) -> Result<()> {
        let active_player = self.game.turn.active_player;

        // Skip draw on first turn (player going first doesn't draw)
        if self.game.turn.turn_number == 1 {
            self.log_normal("(First turn - no draw)");
            return Ok(());
        }

        // Draw a card
        self.game.draw_card(active_player)?;

        #[cfg(feature = "verbose-logging")]
        {
            let player_name = self.get_player_name(active_player);
            if let Some(zones) = self.game.get_player_zones(active_player) {
                if let Some(&card_id) = zones.hand.cards.last() {
                    if let Ok(card) = self.game.cards.get(card_id) {
                        log_if_verbose!(self, "{} draws {} ({})", player_name, card.name, card_id);
                    } else {
                        log_if_verbose!(self, "{} draws a card", player_name);
                    }
                } else {
                    log_if_verbose!(self, "{} draws a card", player_name);
                }
            } else {
                log_if_verbose!(self, "{} draws a card", player_name);
            }
        }

        // MTG Rules 504.2: After draw, players receive priority
        self.priority_round(controller1, controller2)?;

        Ok(())
    }

    /// Main phase - players can play spells and lands
    fn main_phase(
        &mut self,
        controller1: &mut dyn PlayerController,
        controller2: &mut dyn PlayerController,
    ) -> Result<()> {
        // Priority round where players can take actions
        self.priority_round(controller1, controller2)?;
        Ok(())
    }

    /// Combat phases (simplified for now)
    fn begin_combat_step(
        &mut self,
        controller1: &mut dyn PlayerController,
        controller2: &mut dyn PlayerController,
    ) -> Result<()> {
        self.priority_round(controller1, controller2)?;
        Ok(())
    }

    fn declare_attackers_step(
        &mut self,
        controller1: &mut dyn PlayerController,
        controller2: &mut dyn PlayerController,
    ) -> Result<()> {
        // Active player declares attackers
        let active_player = self.game.turn.active_player;
        let controller: &mut dyn PlayerController = if active_player == controller1.player_id() {
            controller1
        } else {
            controller2
        };

        // Get available creatures that can attack
        let available_creatures = self.get_available_attacker_creatures(active_player);

        if !available_creatures.is_empty() {
            // Ask controller to choose all attackers at once (v2 interface)
            let view = GameStateView::new(self.game, active_player);
            let attackers = controller.choose_attackers(&view, &available_creatures);

            // Log this choice point for snapshot/replay
            self.log_choice_point(active_player);

            // Declare each chosen attacker
            for attacker_id in attackers.iter() {
                // Use GameState::declare_attacker() which taps the creature (MTG Rules 508.1f)
                // NOT Combat::declare_attacker() which only adds to the attackers list
                if let Err(e) = self.game.declare_attacker(active_player, *attacker_id) {
                    if self.verbosity >= VerbosityLevel::Normal {
                        eprintln!("  Error declaring attacker: {e}");
                    }
                    continue;
                }

                if self.verbosity >= VerbosityLevel::Normal {
                    let card_name = self
                        .game
                        .cards
                        .get(*attacker_id)
                        .map(|c| c.name.as_str())
                        .unwrap_or("Unknown");

                    // Get power/toughness for more detail
                    if let Ok(card) = self.game.cards.get(*attacker_id) {
                        let power = card.power.unwrap_or(0);
                        let toughness = card.toughness.unwrap_or(0);
                        println!(
                            "  {} declares {} ({}) ({}/{}) as attacker",
                            self.get_player_name(active_player),
                            card_name,
                            attacker_id,
                            power,
                            toughness
                        );
                    }
                }
            }
        }

        // MTG Rules 508.4: After attackers are declared, players receive priority
        self.priority_round(controller1, controller2)?;

        Ok(())
    }

    fn declare_blockers_step(
        &mut self,
        controller1: &mut dyn PlayerController,
        controller2: &mut dyn PlayerController,
    ) -> Result<()> {
        // Defending player declares blockers
        let active_player = self.game.turn.active_player;
        let defending_player = self
            .game
            .get_other_player_id(active_player)
            .expect("Should have defending player");

        let controller: &mut dyn PlayerController = if defending_player == controller1.player_id() {
            controller1
        } else {
            controller2
        };

        // Get available blockers and attackers
        let available_blockers = self.get_available_blocker_creatures(defending_player);
        let attackers = self.get_current_attackers();

        if !available_blockers.is_empty() && !attackers.is_empty() {
            // Ask controller to choose all blocker assignments at once (v2 interface)
            let view = GameStateView::new(self.game, defending_player);
            let blocks = controller.choose_blockers(&view, &available_blockers, &attackers);

            // Log this choice point for snapshot/replay
            self.log_choice_point(defending_player);

            // Declare each blocking assignment
            for (blocker_id, attacker_id) in blocks.iter() {
                let mut attackers_vec = SmallVec::new();
                attackers_vec.push(*attacker_id);
                self.game.combat.declare_blocker(*blocker_id, attackers_vec);

                if self.verbosity >= VerbosityLevel::Verbose {
                    let blocker_name = self
                        .game
                        .cards
                        .get(*blocker_id)
                        .map(|c| c.name.as_str())
                        .unwrap_or("Unknown");
                    let attacker_name = self
                        .game
                        .cards
                        .get(*attacker_id)
                        .map(|c| c.name.as_str())
                        .unwrap_or("Unknown");
                    println!(
                        "  {} blocks {} with {}",
                        self.get_player_name(defending_player),
                        attacker_name,
                        blocker_name
                    );
                }
            }
        }

        // MTG Rules 509.4: After blockers are declared, players receive priority
        self.priority_round(controller1, controller2)?;

        Ok(())
    }

    fn combat_damage_step(
        &mut self,
        controller1: &mut dyn PlayerController,
        controller2: &mut dyn PlayerController,
    ) -> Result<()> {
        // Check if any attacking or blocking creature has first strike or double strike
        // MTG Rules 510.4: If so, we have two combat damage steps
        let has_first_strike = self.has_first_strike_combat();

        if has_first_strike {
            // First strike damage step
            if self.verbosity >= VerbosityLevel::Normal {
                println!("--- First Strike Combat Damage ---");
            }
            self.log_combat_damage(true)?;
            self.game
                .assign_combat_damage(controller1, controller2, true)?;
            self.priority_round(controller1, controller2)?;
        }

        // Normal combat damage step (or only step if no first strike)
        if self.verbosity >= VerbosityLevel::Normal && has_first_strike {
            println!("--- Normal Combat Damage ---");
        }
        self.log_combat_damage(false)?;
        self.game
            .assign_combat_damage(controller1, controller2, false)?;

        // After damage is dealt, players get priority
        self.priority_round(controller1, controller2)?;
        Ok(())
    }

    /// Check if any attacking or blocking creature has first strike or double strike
    fn has_first_strike_combat(&self) -> bool {
        // Check all attackers (using iterator to avoid Vec allocation)
        for attacker_id in self.game.combat.attackers_iter() {
            if let Ok(attacker) = self.game.cards.get(attacker_id) {
                if attacker.has_first_strike() || attacker.has_double_strike() {
                    return true;
                }
            }

            // Check all blockers of this attacker
            if self.game.combat.is_blocked(attacker_id) {
                let blockers = self.game.combat.get_blockers(attacker_id);
                for blocker_id in &blockers {
                    if let Ok(blocker) = self.game.cards.get(*blocker_id) {
                        if blocker.has_first_strike() || blocker.has_double_strike() {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// Log combat damage for debugging
    fn log_combat_damage(&self, first_strike_step: bool) -> Result<()> {
        if self.verbosity < VerbosityLevel::Normal {
            return Ok(());
        }

        let mut attackers = self.game.combat.get_attackers();
        // Sort for deterministic logging output
        attackers.sort_by_key(|id| id.as_u32());

        for attacker_id in &attackers {
            if let Ok(attacker) = self.game.cards.get(*attacker_id) {
                // Check if this attacker deals damage in this step
                let deals_damage = if first_strike_step {
                    attacker.has_first_strike() || attacker.has_double_strike()
                } else {
                    attacker.has_normal_strike()
                };

                if !deals_damage {
                    continue;
                }

                let power = attacker.current_power();
                let attacker_name = &attacker.name;

                if self.game.combat.is_blocked(*attacker_id) {
                    let mut blockers = self.game.combat.get_blockers(*attacker_id);
                    // Sort for deterministic logging output
                    blockers.sort_by_key(|id| id.as_u32());
                    for blocker_id in &blockers {
                        if let Ok(blocker) = self.game.cards.get(*blocker_id) {
                            // Check if blocker deals damage in this step
                            let blocker_deals_damage = if first_strike_step {
                                blocker.has_first_strike() || blocker.has_double_strike()
                            } else {
                                blocker.has_normal_strike()
                            };

                            if !blocker_deals_damage {
                                continue;
                            }

                            let blocker_power = blocker.current_power();
                            let blocker_name = &blocker.name;
                            println!(
                                "  Combat: {attacker_name} ({attacker_id}) ({power} damage) ↔ {blocker_name} ({blocker_id}) ({blocker_power} damage)"
                            );
                        }
                    }
                } else {
                    // Unblocked attacker
                    if let Some(defending_player) =
                        self.game.combat.get_defending_player(*attacker_id)
                    {
                        let defender_name = self.get_player_name(defending_player);
                        if power > 0 {
                            println!(
                                "  {attacker_name} ({attacker_id}) deals {power} damage to {defender_name}"
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn end_combat_step(
        &mut self,
        controller1: &mut dyn PlayerController,
        controller2: &mut dyn PlayerController,
    ) -> Result<()> {
        // Clear combat state at end of combat
        self.game.combat.clear();

        // Players get priority
        self.priority_round(controller1, controller2)?;
        Ok(())
    }

    fn end_step(
        &mut self,
        controller1: &mut dyn PlayerController,
        controller2: &mut dyn PlayerController,
    ) -> Result<()> {
        self.priority_round(controller1, controller2)?;
        Ok(())
    }

    /// Cleanup step - discard to hand size, remove damage
    fn cleanup_step(
        &mut self,
        controller1: &mut dyn PlayerController,
        controller2: &mut dyn PlayerController,
    ) -> Result<()> {
        let active_player = self.game.turn.active_player;

        // Get non-active player
        let non_active_player = self
            .game
            .get_other_player_id(active_player)
            .expect("Should have non-active player");

        // Process active player first, then non-active player
        for &player_id in &[active_player, non_active_player] {
            let hand_size = self
                .game
                .get_player_zones(player_id)
                .map(|z| z.hand.cards.len())
                .unwrap_or(0);

            let max_hand_size = self.game.get_player(player_id)?.max_hand_size;

            if hand_size > max_hand_size {
                let discard_count = hand_size - max_hand_size;

                log_if_verbose!(
                    self,
                    "{} must discard {} cards (hand size: {}, max: {})",
                    self.get_player_name(player_id),
                    discard_count,
                    hand_size,
                    max_hand_size
                );

                // Get the appropriate controller
                let controller: &mut dyn PlayerController = if player_id == controller1.player_id()
                {
                    controller1
                } else {
                    controller2
                };

                // Ask controller which cards to discard
                let view = GameStateView::new(self.game, player_id);
                let hand = view.hand();
                let cards_to_discard =
                    controller.choose_cards_to_discard(&view, hand, discard_count);

                // Log this choice point for snapshot/replay
                self.log_choice_point(player_id);

                // Verify correct number of cards
                if cards_to_discard.len() != discard_count {
                    return Err(crate::MtgError::InvalidAction(format!(
                        "Must discard exactly {discard_count} cards, got {}",
                        cards_to_discard.len()
                    )));
                }

                // Move cards to graveyard
                for card_id in cards_to_discard {
                    // Verify card is in hand before moving
                    if let Some(zones) = self.game.get_player_zones(player_id) {
                        if !zones.hand.contains(card_id) {
                            return Err(crate::MtgError::InvalidAction(format!(
                                "Card {card_id:?} not in player's hand"
                            )));
                        }
                    }

                    // Use move_card to properly log the action for undo
                    self.game.move_card(
                        card_id,
                        crate::zones::Zone::Hand,
                        crate::zones::Zone::Graveyard,
                        player_id,
                    )?;

                    log_if_verbose!(
                        self,
                        "{} discards {} ({})",
                        self.get_player_name(player_id),
                        self.game
                            .cards
                            .get(card_id)
                            .map(|c| c.name.as_str())
                            .unwrap_or("Unknown"),
                        card_id
                    );
                }
            }
        }

        // Empty mana pools
        for &player_id in &[active_player, non_active_player] {
            if let Ok(player) = self.game.get_player_mut(player_id) {
                player.mana_pool.clear();
            }
        }

        // TODO: Remove damage from creatures

        Ok(())
    }

    /// Resolve the top spell from the stack
    ///
    /// This removes the spell from the stack and executes its effects.
    /// Implements MTG Comprehensive Rules 608 (Resolving Spells and Abilities).
    fn resolve_top_spell_from_stack(&mut self, spell_id: CardId) -> Result<()> {
        // Look up the targets for this spell
        let targets = self
            .spell_targets
            .iter()
            .find(|(id, _)| *id == spell_id)
            .map(|(_, t)| t.clone())
            .unwrap_or_else(Vec::new);

        // Get card name and effects for logging (before resolution)
        let (card_name, card_effects, card_owner) = if let Ok(card) = self.game.cards.get(spell_id)
        {
            (card.name.to_string(), card.effects.clone(), card.owner)
        } else {
            return Err(crate::MtgError::EntityNotFound(spell_id.as_u32()));
        };

        if self.verbosity >= VerbosityLevel::Normal {
            println!("  {} ({}) resolves", card_name, spell_id);
        }

        // Resolve the spell
        self.game.resolve_spell(spell_id, &targets)?;

        // Log effects for instants/sorceries
        if self.verbosity >= VerbosityLevel::Normal {
            for effect in &card_effects {
                self.log_effect_execution(&card_name, spell_id, effect, card_owner);
            }

            // Check if it's a permanent entering battlefield
            if let Ok(card) = self.game.cards.get(spell_id) {
                if card.is_creature() {
                    println!(
                        "  {} ({}) enters the battlefield as a {}/{} creature",
                        card_name,
                        spell_id,
                        card.power.unwrap_or(0),
                        card.toughness.unwrap_or(0)
                    );
                }
            }
        }

        // Remove the spell from our targets tracking
        self.spell_targets.retain(|(id, _)| *id != spell_id);

        Ok(())
    }

    /// Priority round - players get chances to act until both pass
    ///
    /// This implements the priority system where players alternate making choices
    /// until both pass in succession, then resolves spells from the stack.
    ///
    /// ## MTG Rules Implementation
    /// - Gets all available spell abilities (lands, spells, abilities)
    /// - Calls controller.choose_spell_ability_to_play() for each priority window
    /// - Handles the chosen ability appropriately:
    ///   - PlayLand: Resolves directly (no stack)
    ///   - CastSpell: Puts spell on stack (MTG Rules 601)
    ///   - ActivateAbility: TODO - should go on stack for non-mana abilities
    /// - When both players pass with spells on stack, resolves top spell (MTG Rules 117.4)
    /// - Repeats until stack is empty and both players pass
    fn priority_round(
        &mut self,
        controller1: &mut dyn PlayerController,
        controller2: &mut dyn PlayerController,
    ) -> Result<()> {
        let active_player = self.game.turn.active_player;
        let non_active_player = self
            .game
            .get_other_player_id(active_player)
            .expect("Should have non-active player");

        // Outer loop: resolve stack until empty
        loop {
            // Active player gets priority first in each round
            let mut current_priority = active_player;
            let mut consecutive_passes = 0;
            let mut action_count = 0;
            const MAX_ACTIONS_PER_PRIORITY: usize = 1000;

            // Inner loop: pass priority until both players pass
            while consecutive_passes < 2 {
                // Safety check to prevent infinite loops
                action_count += 1;
                if action_count > MAX_ACTIONS_PER_PRIORITY {
                    return Err(crate::MtgError::InvalidAction(format!(
                    "Priority round exceeded max actions ({MAX_ACTIONS_PER_PRIORITY}), possible infinite loop"
                )));
                }

                // Get the appropriate controller
                let controller: &mut dyn PlayerController =
                    if current_priority == controller1.player_id() {
                        controller1
                    } else {
                        controller2
                    };

                // Get all available spell abilities for this player
                let available = self.get_available_spell_abilities(current_priority);

                // If no actions available, automatically pass priority without asking controller
                // Only invoke controller when there's an actual choice to make
                let choice = if available.is_empty() {
                    // No available actions - automatically pass priority
                    None
                } else {
                    // Ask controller to choose one (or None to pass)
                    let view = GameStateView::new(self.game, current_priority);
                    let choice = controller.choose_spell_ability_to_play(&view, &available);

                    // Log this choice point for snapshot/replay
                    self.log_choice_point(current_priority);

                    choice
                };

                match choice {
                    None => {
                        // Controller chose to pass priority
                        consecutive_passes += 1;
                        let view = GameStateView::new(self.game, current_priority);
                        controller.on_priority_passed(&view);

                        // Switch priority to other player
                        current_priority = if current_priority == active_player {
                            non_active_player
                        } else {
                            active_player
                        };
                    }
                    Some(ability) => {
                        // Controller chose an ability to play
                        consecutive_passes = 0; // Reset pass counter

                        match ability {
                            crate::core::SpellAbility::PlayLand { card_id } => {
                                // Play land - resolves directly (no stack)
                                if let Err(e) = self.game.play_land(current_priority, card_id) {
                                    if self.verbosity >= VerbosityLevel::Normal {
                                        eprintln!("  Error playing land: {e}");
                                    }
                                } else if self.verbosity >= VerbosityLevel::Normal {
                                    let card_name = self
                                        .game
                                        .cards
                                        .get(card_id)
                                        .map(|c| c.name.as_str())
                                        .unwrap_or("Unknown");
                                    println!(
                                        "  {} plays {} ({})",
                                        self.get_player_name(current_priority),
                                        card_name,
                                        card_id
                                    );
                                }
                            }
                            crate::core::SpellAbility::CastSpell { card_id } => {
                                // Cast spell using 8-step process
                                // Mana will be tapped during step 6 (NOT here!)

                                let card_name = self
                                    .game
                                    .cards
                                    .get(card_id)
                                    .map(|c| c.name.to_string())
                                    .unwrap_or_else(|_| "Unknown".to_string());

                                if self.verbosity >= VerbosityLevel::Normal {
                                    println!(
                                        "  {} casts {} ({}) (putting on stack)",
                                        self.get_player_name(current_priority),
                                        card_name,
                                        card_id
                                    );
                                }

                                // Get valid targets BEFORE calling cast_spell_8_step
                                // (we can't borrow controller inside the closure)
                                let valid_targets = self
                                    .game
                                    .get_valid_targets_for_spell(card_id)
                                    .unwrap_or_else(|_| SmallVec::new());

                                // Ask controller to choose targets (only if there are valid targets)
                                let chosen_targets_vec: Vec<CardId> = if !valid_targets.is_empty() {
                                    let view = GameStateView::new(self.game, current_priority);
                                    let chosen_targets =
                                        controller.choose_targets(&view, card_id, &valid_targets);

                                    // Log this choice point for snapshot/replay
                                    self.log_choice_point(current_priority);

                                    chosen_targets.into_iter().collect()
                                } else {
                                    // No targets needed - spell has no targeting effects
                                    Vec::new()
                                };

                                // Clone for closure (which will move it)
                                let targets_for_callback = chosen_targets_vec.clone();

                                // Create callbacks for targeting and mana payment
                                let targeting_callback =
                                    move |_game: &GameState, _spell_id: CardId| {
                                        // Return the pre-selected targets
                                        targets_for_callback.clone()
                                    };

                                let mana_callback =
                                    |game: &GameState, cost: &crate::core::ManaCost| {
                                        // For now, automatically choose mana sources
                                        // TODO: Call controller.choose_mana_sources_to_pay()
                                        let mut sources = Vec::new();
                                        let tappable = game
                                            .battlefield
                                            .cards
                                            .iter()
                                            .filter(|&&card_id| {
                                                if let Ok(card) = game.cards.get(card_id) {
                                                    card.owner == current_priority
                                                        && card.is_land()
                                                        && !card.tapped
                                                } else {
                                                    false
                                                }
                                            })
                                            .copied()
                                            .collect::<Vec<_>>();

                                        // Simple greedy algorithm: tap lands until we have enough
                                        for &land_id in &tappable {
                                            sources.push(land_id);
                                            if sources.len() >= cost.cmc() as usize {
                                                break;
                                            }
                                        }
                                        sources
                                    };

                                // Cast using 8-step process
                                if let Err(e) = self.game.cast_spell_8_step(
                                    current_priority,
                                    card_id,
                                    targeting_callback,
                                    mana_callback,
                                ) {
                                    if self.verbosity >= VerbosityLevel::Normal {
                                        eprintln!("  Error casting spell: {e}");
                                    }
                                } else {
                                    // Store targets for this spell (will be used when it resolves)
                                    self.spell_targets.push((card_id, chosen_targets_vec));

                                    // Spell is now on the stack - it will resolve later
                                    // when both players pass priority
                                }
                            }
                            crate::core::SpellAbility::ActivateAbility {
                                card_id,
                                ability_index,
                            } => {
                                // Activate ability from a permanent
                                // TODO(mtg-70): This should go on the stack for non-mana abilities

                                // Get the card and ability
                                let card_name =
                                    self.game.cards.get(card_id).ok().map(|c| c.name.clone());
                                let ability = self.game.cards.get(card_id).ok().and_then(|c| {
                                    c.activated_abilities.get(ability_index).cloned()
                                });

                                if let Some(ability) = ability {
                                    if self.verbosity >= VerbosityLevel::Normal {
                                        let name = card_name
                                            .as_ref()
                                            .map(|n| n.as_str())
                                            .unwrap_or("Unknown");
                                        println!(
                                            "  {} activates ability: {}",
                                            name, ability.description
                                        );
                                    }

                                    // Get valid targets for the ability (before paying costs)
                                    let valid_targets = self
                                        .game
                                        .get_valid_targets_for_ability(card_id, ability_index)
                                        .unwrap_or_else(|_| SmallVec::new());

                                    // Ask controller to choose targets (only if there are valid targets)
                                    let chosen_targets_vec: Vec<CardId> = if !valid_targets
                                        .is_empty()
                                    {
                                        let view = GameStateView::new(self.game, current_priority);
                                        let chosen_targets = controller.choose_targets(
                                            &view,
                                            card_id,
                                            &valid_targets,
                                        );

                                        // Log this choice point for snapshot/replay
                                        self.log_choice_point(current_priority);

                                        chosen_targets.into_iter().collect()
                                    } else {
                                        // No targets needed
                                        Vec::new()
                                    };

                                    // Pay costs
                                    if let Err(e) = self.game.pay_ability_cost(
                                        current_priority,
                                        card_id,
                                        &ability.cost,
                                    ) {
                                        if self.verbosity >= VerbosityLevel::Normal {
                                            eprintln!("    Failed to pay cost: {e}");
                                        }
                                        continue;
                                    }

                                    // Execute effects immediately (not on the stack)
                                    // TODO(mtg-70): Put non-mana abilities on the stack
                                    for effect in &ability.effects {
                                        // Fix placeholder player IDs and targets for effects
                                        let fixed_effect = match effect {
                                            crate::core::Effect::AddMana { player, mana }
                                                if player.as_u32() == 0 =>
                                            {
                                                // Replace placeholder with current player
                                                crate::core::Effect::AddMana {
                                                    player: current_priority,
                                                    mana: *mana,
                                                }
                                            }
                                            crate::core::Effect::GainLife { player, amount }
                                                if player.as_u32() == 0 =>
                                            {
                                                // Replace placeholder with current player
                                                crate::core::Effect::GainLife {
                                                    player: current_priority,
                                                    amount: *amount,
                                                }
                                            }
                                            crate::core::Effect::DrawCards { player, count }
                                                if player.as_u32() == 0 =>
                                            {
                                                // Replace placeholder with current player
                                                crate::core::Effect::DrawCards {
                                                    player: current_priority,
                                                    count: *count,
                                                }
                                            }
                                            // Replace placeholder targets with chosen targets
                                            crate::core::Effect::DestroyPermanent { target }
                                                if target.as_u32() == 0
                                                    && !chosen_targets_vec.is_empty() =>
                                            {
                                                // Use the first chosen target
                                                crate::core::Effect::DestroyPermanent {
                                                    target: chosen_targets_vec[0],
                                                }
                                            }
                                            crate::core::Effect::TapPermanent { target }
                                                if target.as_u32() == 0
                                                    && !chosen_targets_vec.is_empty() =>
                                            {
                                                crate::core::Effect::TapPermanent {
                                                    target: chosen_targets_vec[0],
                                                }
                                            }
                                            crate::core::Effect::UntapPermanent { target }
                                                if target.as_u32() == 0
                                                    && !chosen_targets_vec.is_empty() =>
                                            {
                                                crate::core::Effect::UntapPermanent {
                                                    target: chosen_targets_vec[0],
                                                }
                                            }
                                            crate::core::Effect::PumpCreature {
                                                target,
                                                power_bonus,
                                                toughness_bonus,
                                            } if target.as_u32() == 0
                                                && !chosen_targets_vec.is_empty() =>
                                            {
                                                crate::core::Effect::PumpCreature {
                                                    target: chosen_targets_vec[0],
                                                    power_bonus: *power_bonus,
                                                    toughness_bonus: *toughness_bonus,
                                                }
                                            }
                                            crate::core::Effect::PutCounter {
                                                target,
                                                counter_type,
                                                amount,
                                            } if target.as_u32() == 0
                                                && !chosen_targets_vec.is_empty() =>
                                            {
                                                crate::core::Effect::PutCounter {
                                                    target: chosen_targets_vec[0],
                                                    counter_type: *counter_type,
                                                    amount: *amount,
                                                }
                                            }
                                            crate::core::Effect::RemoveCounter {
                                                target,
                                                counter_type,
                                                amount,
                                            } if target.as_u32() == 0
                                                && !chosen_targets_vec.is_empty() =>
                                            {
                                                crate::core::Effect::RemoveCounter {
                                                    target: chosen_targets_vec[0],
                                                    counter_type: *counter_type,
                                                    amount: *amount,
                                                }
                                            }
                                            _ => effect.clone(),
                                        };

                                        if let Err(e) = self.game.execute_effect(&fixed_effect) {
                                            if self.verbosity >= VerbosityLevel::Normal {
                                                eprintln!("    Failed to execute effect: {e}");
                                            }
                                        }
                                    }
                                } else if self.verbosity >= VerbosityLevel::Normal {
                                    eprintln!("  Ability not found");
                                }
                            }
                        }

                        // After taking an action, switch priority to other player
                        current_priority = if current_priority == active_player {
                            non_active_player
                        } else {
                            active_player
                        };
                    }
                }
            }

            // Both players passed priority
            // Check if there are spells on the stack to resolve
            if self.game.stack.is_empty() {
                // Stack is empty, priority round is complete
                break;
            }

            // Resolve the top spell from the stack (MTG Rules 608: Resolving Spells and Abilities)
            // In MTG, the stack is LIFO (Last In, First Out)
            if let Some(&spell_id) = self.game.stack.cards.last() {
                self.resolve_top_spell_from_stack(spell_id)?;
                // After resolving a spell, players get priority again
                // Loop continues to give priority
            } else {
                // Stack was reported non-empty but has no cards (shouldn't happen)
                break;
            }
        }

        Ok(())
    }

    /// Get available attackers for a player
    #[allow(dead_code)] // Legacy v1 interface, will be removed
    fn get_available_attackers(&self, player_id: PlayerId) -> Vec<PlayerAction> {
        let mut actions = Vec::new();

        // Add finish action
        actions.push(PlayerAction::FinishDeclareAttackers);

        // Find creatures that can attack
        for &card_id in &self.game.battlefield.cards {
            if let Ok(card) = self.game.cards.get(card_id) {
                if card.controller == player_id
                    && card.is_creature()
                    && !card.tapped
                    && !self.game.combat.is_attacking(card_id)
                {
                    // TODO: Check for summoning sickness
                    actions.push(PlayerAction::DeclareAttacker(card_id));
                }
            }
        }

        actions
    }

    /// Get available blockers for a player
    #[allow(dead_code)] // Legacy v1 interface, will be removed
    fn get_available_blockers(&self, player_id: PlayerId) -> Vec<PlayerAction> {
        let mut actions = Vec::new();

        // Add finish action
        actions.push(PlayerAction::FinishDeclareBlockers);

        // Get all attacking creatures
        let attackers = self.game.combat.get_attackers();
        if attackers.is_empty() {
            return actions;
        }

        // Find creatures that can block
        for &card_id in &self.game.battlefield.cards {
            if let Ok(card) = self.game.cards.get(card_id) {
                if card.controller == player_id
                    && card.is_creature()
                    && !card.tapped
                    && !self.game.combat.is_blocking(card_id)
                {
                    // For each potential blocker, offer to block each attacker
                    // (For simplicity, we only support blocking one attacker at a time)
                    for &attacker in &attackers {
                        actions.push(PlayerAction::DeclareBlocker {
                            blocker: card_id,
                            attackers: vec![attacker],
                        });
                    }
                }
            }
        }

        actions
    }

    /// Get available actions for a player at current game state
    #[allow(dead_code)] // Legacy v1 interface, will be removed
    fn get_available_actions(&self, player_id: PlayerId) -> Vec<PlayerAction> {
        let mut actions = Vec::new();

        // Always can pass priority
        actions.push(PlayerAction::PassPriority);

        let current_step = self.game.turn.current_step;

        // Can play lands in main phases if player hasn't played one this turn
        if current_step.can_play_lands() {
            if let Ok(player) = self.game.get_player(player_id) {
                if player.can_play_land() {
                    // Find lands in hand
                    if let Some(zones) = self.game.get_player_zones(player_id) {
                        for &card_id in &zones.hand.cards {
                            if let Ok(card) = self.game.cards.get(card_id) {
                                if card.is_land() {
                                    actions.push(PlayerAction::PlayLand(card_id));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Can tap lands for mana
        for &card_id in &self.game.battlefield.cards {
            if let Ok(card) = self.game.cards.get(card_id) {
                if card.owner == player_id && card.is_land() && !card.tapped {
                    actions.push(PlayerAction::TapForMana(card_id));
                }
            }
        }

        // Can cast spells from hand
        if let Some(zones) = self.game.get_player_zones(player_id) {
            for &card_id in &zones.hand.cards {
                if let Ok(card) = self.game.cards.get(card_id) {
                    // Check if card is castable (not a land)
                    if !card.is_land() {
                        // Check if player has enough mana
                        if let Ok(player) = self.game.get_player(player_id) {
                            if player.mana_pool.can_pay(&card.mana_cost) {
                                actions.push(PlayerAction::CastSpell {
                                    card_id,
                                    targets: vec![],
                                });
                            }
                        }
                    }
                }
            }
        }

        actions
    }

    /// Get creatures that can attack for a player (v2 interface)
    fn get_available_attacker_creatures(&self, player_id: PlayerId) -> Vec<CardId> {
        let mut creatures = Vec::new();

        for &card_id in &self.game.battlefield.cards {
            if let Ok(card) = self.game.cards.get(card_id) {
                if card.controller == player_id
                    && card.is_creature()
                    && !card.tapped
                    && !self.game.combat.is_attacking(card_id)
                {
                    // TODO: Check for summoning sickness
                    creatures.push(card_id);
                }
            }
        }

        creatures
    }

    /// Get creatures that can block for a player (v2 interface)
    fn get_available_blocker_creatures(&self, player_id: PlayerId) -> Vec<CardId> {
        let mut creatures = Vec::new();

        for &card_id in &self.game.battlefield.cards {
            if let Ok(card) = self.game.cards.get(card_id) {
                if card.controller == player_id
                    && card.is_creature()
                    && !card.tapped
                    && !self.game.combat.is_blocking(card_id)
                {
                    creatures.push(card_id);
                }
            }
        }

        creatures
    }

    /// Get currently attacking creatures (v2 interface)
    fn get_current_attackers(&self) -> Vec<CardId> {
        self.game.combat.get_attackers()
    }

    /// Get lands in player's hand (v2 interface)
    fn get_lands_in_hand(&self, player_id: PlayerId) -> Vec<CardId> {
        let mut lands = Vec::new();

        if let Some(zones) = self.game.get_player_zones(player_id) {
            for &card_id in &zones.hand.cards {
                if let Ok(card) = self.game.cards.get(card_id) {
                    if card.is_land() {
                        lands.push(card_id);
                    }
                }
            }
        }

        lands
    }

    /// Get castable spells in player's hand (v2 interface)
    fn get_castable_spells(&self, player_id: PlayerId) -> Vec<CardId> {
        use crate::game::mana_engine::ManaEngine;

        let mut spells = Vec::new();

        // Create a mana engine and scan battlefield for available mana
        let mut mana_engine = ManaEngine::new(player_id);
        mana_engine.update(self.game);

        // Check if this is the active player (only active player can cast sorceries)
        let is_active_player = self.game.turn.active_player == player_id;

        // Check if it's sorcery speed (Main1 or Main2)
        let is_sorcery_speed = self.game.turn.current_step.is_sorcery_speed();

        if let Some(zones) = self.game.get_player_zones(player_id) {
            for &card_id in &zones.hand.cards {
                if let Ok(card) = self.game.cards.get(card_id) {
                    // Check if card is castable (not a land)
                    if !card.is_land() {
                        // Check timing restrictions
                        let can_cast_now = if card.is_instant() {
                            // Instants can be cast anytime with priority
                            true
                        } else {
                            // Creatures and sorceries require sorcery speed AND active player
                            is_sorcery_speed && is_active_player
                        };

                        if can_cast_now {
                            // Check if we can pay for this spell's mana cost
                            if mana_engine.can_pay(&card.mana_cost) {
                                spells.push(card_id);
                            }
                        }
                    }
                }
            }
        }

        spells
    }

    /// Get activatable abilities on player's permanents (v2 interface)
    fn get_activatable_abilities(&self, player_id: PlayerId) -> Vec<(CardId, usize)> {
        use crate::game::mana_engine::ManaEngine;

        let mut abilities = Vec::new();

        // Create a mana engine to check cost payability
        let mut mana_engine = ManaEngine::new(player_id);
        mana_engine.update(self.game);

        // Check all permanents controlled by this player
        for &card_id in &self.game.battlefield.cards {
            if let Ok(card) = self.game.cards.get(card_id) {
                // Only check permanents controlled by this player
                if card.controller != player_id {
                    continue;
                }

                // Check each activated ability on this card
                for (ability_index, ability) in card.activated_abilities.iter().enumerate() {
                    // Skip mana abilities for now (they'll be handled specially)
                    if ability.is_mana_ability {
                        continue;
                    }

                    // Check if cost can be paid
                    let mut can_activate = true;

                    // Check tap cost
                    if ability.cost.includes_tap() && card.tapped {
                        can_activate = false;
                    }

                    // Check mana cost
                    if let Some(mana_cost) = ability.cost.get_mana_cost() {
                        if !mana_engine.can_pay(mana_cost) {
                            can_activate = false;
                        }
                    }

                    // TODO: Check other cost types (sacrifice, discard, etc.)
                    // TODO: Check timing restrictions (sorcery speed abilities)
                    // TODO: Check activation limits

                    // TODO(mtg-70): Check if ability has valid targets
                    // For targeting abilities, check that there's at least one valid target
                    if can_activate {
                        // Check if this ability requires targets
                        let valid_targets = self
                            .game
                            .get_valid_targets_for_ability(card_id, ability_index)
                            .unwrap_or_else(|_| SmallVec::new());

                        // If get_valid_targets_for_ability returned an empty list,
                        // it might mean either:
                        // 1. The ability doesn't require targets (non-targeting ability)
                        // 2. The ability requires targets but none are available
                        //
                        // We need to distinguish between these cases.
                        // For now, check if the ability description contains "target"
                        let requires_targets =
                            ability.description.to_lowercase().contains("target");

                        if requires_targets && valid_targets.is_empty() {
                            // Ability requires targets but none are available
                            can_activate = false;
                        }
                    }

                    if can_activate {
                        abilities.push((card_id, ability_index));
                    }
                }
            }
        }

        abilities
    }

    /// Get all available spell abilities for a player
    ///
    /// This matches Java Forge's approach where lands, spells, and activated
    /// abilities are all represented as SpellAbility objects that can be
    /// chosen from a unified list.
    ///
    /// Returns a list of all abilities the player can currently play:
    /// - Land plays (if player can play lands and it's a main phase)
    /// - Castable spells (if player has mana and targeting is valid)
    /// - Activated abilities (TODO: not yet implemented)
    fn get_available_spell_abilities(&self, player_id: PlayerId) -> Vec<crate::core::SpellAbility> {
        use crate::core::SpellAbility;
        let mut abilities = Vec::new();

        // Add playable lands (only in main phases when player can play lands)
        if matches!(self.game.turn.current_step, Step::Main1 | Step::Main2) {
            if let Ok(player) = self.game.get_player(player_id) {
                if player.can_play_land() {
                    let lands = self.get_lands_in_hand(player_id);
                    for land_id in lands {
                        abilities.push(SpellAbility::PlayLand { card_id: land_id });
                    }
                }
            }
        }

        // Add castable spells
        let spells = self.get_castable_spells(player_id);
        for spell_id in spells {
            abilities.push(SpellAbility::CastSpell { card_id: spell_id });
        }

        // Add activated abilities
        let activatable = self.get_activatable_abilities(player_id);
        for (card_id, ability_index) in activatable {
            abilities.push(SpellAbility::ActivateAbility {
                card_id,
                ability_index,
            });
        }

        abilities
    }

    /// Execute a player action
    #[allow(dead_code)] // Legacy v1 interface, will be removed
    fn execute_action(&mut self, player_id: PlayerId, action: &PlayerAction) -> Result<()> {
        if !matches!(action, PlayerAction::PassPriority) {
            let player_name = self.get_player_name(player_id);
            let action_desc = self.describe_action(action);
            self.log_verbose(&format!("{player_name} {action_desc}"));
        }

        match action {
            PlayerAction::PlayLand(card_id) => {
                self.game.play_land(player_id, *card_id)?;
            }
            PlayerAction::TapForMana(card_id) => {
                self.game.tap_for_mana(player_id, *card_id)?;
            }
            PlayerAction::CastSpell { card_id, targets } => {
                // Show spell being cast (added to stack)
                log_if_verbose!(
                    self,
                    "{} casts {}",
                    self.get_player_name(player_id),
                    self.game
                        .cards
                        .get(*card_id)
                        .map(|c| c.name.as_str())
                        .unwrap_or("Unknown")
                );

                self.game.cast_spell(player_id, *card_id, targets.clone())?;

                // Immediately resolve spell (simplified - no stack interaction yet)
                // Legacy v1 path - no targets chosen, rely on auto-targeting
                self.game.resolve_spell(*card_id, &[])?;

                log_if_verbose!(
                    self,
                    "{} resolves",
                    self.game
                        .cards
                        .get(*card_id)
                        .map(|c| c.name.as_str())
                        .unwrap_or("Unknown")
                );
            }
            PlayerAction::DeclareAttacker(card_id) => {
                self.game.declare_attacker(player_id, *card_id)?;
            }
            PlayerAction::DeclareBlocker { blocker, attackers } => {
                self.game
                    .declare_blocker(player_id, *blocker, attackers.clone())?;
            }
            PlayerAction::FinishDeclareAttackers | PlayerAction::FinishDeclareBlockers => {
                // Handled by the combat step logic, not here
            }
            PlayerAction::PassPriority => {
                // Nothing to do
            }
        }
        Ok(())
    }

    /// Describe an action for verbose output
    #[allow(dead_code)] // Legacy v1 interface, will be removed
    fn describe_action(&self, action: &PlayerAction) -> String {
        match action {
            PlayerAction::PlayLand(card_id) => {
                let card_name = self
                    .game
                    .cards
                    .get(*card_id)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Unknown");
                format!("plays {card_name}")
            }
            PlayerAction::TapForMana(card_id) => {
                let card_name = self
                    .game
                    .cards
                    .get(*card_id)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Unknown");
                format!("taps {card_name} for mana")
            }
            PlayerAction::CastSpell { card_id, .. } => {
                let card_name = self
                    .game
                    .cards
                    .get(*card_id)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Unknown");
                format!("casts {card_name}")
            }
            PlayerAction::DeclareAttacker(card_id) => {
                let card_name = self
                    .game
                    .cards
                    .get(*card_id)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Unknown");
                format!("declares {card_name} as attacker")
            }
            PlayerAction::DeclareBlocker { blocker, attackers } => {
                let blocker_name = self
                    .game
                    .cards
                    .get(*blocker)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Unknown");
                let attacker_names: Vec<_> = attackers
                    .iter()
                    .filter_map(|id| self.game.cards.get(*id).ok().map(|c| c.name.as_str()))
                    .collect();
                format!("blocks with {blocker_name} (blocking {attacker_names:?})")
            }
            PlayerAction::FinishDeclareAttackers => "finishes declaring attackers".to_string(),
            PlayerAction::FinishDeclareBlockers => "finishes declaring blockers".to_string(),
            PlayerAction::PassPriority => "passes priority".to_string(),
        }
    }

    /// Check if the game has reached a win condition
    fn check_win_condition(&self) -> Option<GameResult> {
        // Check for player death (life <= 0)
        for player in &self.game.players {
            if player.life <= 0 {
                let winner = self.game.get_other_player_id(player.id);
                return Some(GameResult {
                    winner,
                    turns_played: self.turns_elapsed,
                    end_reason: GameEndReason::PlayerDeath(player.id),
                });
            }
        }

        // Check for decking (empty library when trying to draw)
        for player in &self.game.players {
            if let Some(zones) = self.game.get_player_zones(player.id) {
                if zones.library.is_empty() {
                    let winner = self.game.get_other_player_id(player.id);
                    return Some(GameResult {
                        winner,
                        turns_played: self.turns_elapsed,
                        end_reason: GameEndReason::Decking(player.id),
                    });
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_loop_creation() {
        let mut game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let _game_loop = GameLoop::new(&mut game);
    }

    #[test]
    fn test_untap_step() {
        let mut game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let alice = {
            game.players
                .iter()
                .map(|p| p.id)
                .next()
                .expect("Should have player 1")
        };

        // Create a tapped land on battlefield
        let land_id = game.next_card_id();
        let mut land = crate::core::Card::new(land_id, "Mountain".to_string(), alice);
        land.types.push(crate::core::CardType::Land);
        land.tap();
        game.cards.insert(land_id, land);
        game.battlefield.add(land_id);

        // Run untap step
        let mut game_loop = GameLoop::new(&mut game);
        game_loop.untap_step().unwrap();

        // Land should now be untapped
        let land = game.cards.get(land_id).unwrap();
        assert!(!land.tapped);
    }

    #[test]
    fn test_draw_step() {
        let mut game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let (alice, bob) = {
            let mut players_iter = game.players.iter().map(|p| p.id);
            (
                players_iter.next().expect("Should have player 1"),
                players_iter.next().expect("Should have player 2"),
            )
        };

        // Add a card to Alice's library
        let card_id = game.next_card_id();
        let card = crate::core::Card::new(card_id, "Test Card".to_string(), alice);
        game.cards.insert(card_id, card);
        if let Some(zones) = game.get_player_zones_mut(alice) {
            zones.library.add(card_id);
        }

        // Set turn to 2 (so draw happens)
        game.turn.turn_number = 2;

        // Create mock controllers
        let mut controller1 = crate::game::ZeroController::new(alice);
        let mut controller2 = crate::game::ZeroController::new(bob);

        // Run draw step
        let mut game_loop = GameLoop::new(&mut game);
        game_loop
            .draw_step(&mut controller1, &mut controller2)
            .unwrap();

        // Card should be in hand
        if let Some(zones) = game.get_player_zones(alice) {
            assert!(zones.hand.contains(card_id));
            assert!(!zones.library.contains(card_id));
        }
    }

    #[test]
    fn test_check_win_condition_life() {
        let mut game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let bob = {
            let mut players_iter = game.players.iter().map(|p| p.id);
            let _alice = players_iter.next().expect("Should have player 1");
            players_iter.next().expect("Should have player 2")
        };

        // Set Bob's life to 0
        game.get_player_mut(bob).unwrap().life = 0;

        let game_loop = GameLoop::new(&mut game);
        let result = game_loop.check_win_condition();

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.end_reason, GameEndReason::PlayerDeath(bob));
    }
}
