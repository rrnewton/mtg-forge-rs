//! Game loop implementation
//!
//! Manages the main game loop, turn progression, and priority system

use crate::core::PlayerId;
use crate::game::controller::{GameStateView, PlayerAction, PlayerController};
use crate::game::phase::Step;
use crate::game::GameState;
use crate::{MtgError, Result};

/// Verbosity level for game output
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
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
    /// Verbosity level for output
    pub verbosity: VerbosityLevel,
    /// Track if current step header has been printed (for lazy printing)
    step_header_printed: bool,
}

impl<'a> GameLoop<'a> {
    /// Create a new game loop for the given game state
    pub fn new(game: &'a mut GameState) -> Self {
        GameLoop {
            game,
            max_turns: 1000, // Default maximum turns
            turns_elapsed: 0,
            verbosity: VerbosityLevel::default(),
            step_header_printed: false,
        }
    }

    /// Set maximum turns before forcing a draw
    pub fn with_max_turns(mut self, max_turns: u32) -> Self {
        self.max_turns = max_turns;
        self
    }

    /// Set verbosity level for output
    pub fn with_verbosity(mut self, verbosity: VerbosityLevel) -> Self {
        self.verbosity = verbosity;
        self
    }

    /// Enable verbose output (deprecated, use with_verbosity)
    #[deprecated(note = "Use with_verbosity instead")]
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbosity = if verbose {
            VerbosityLevel::Verbose
        } else {
            VerbosityLevel::Silent
        };
        self
    }

    /// Run the game loop with the given player controllers
    ///
    /// Returns when the game reaches a win condition or turn limit
    pub fn run_game(
        &mut self,
        controller1: &mut dyn PlayerController,
        controller2: &mut dyn PlayerController,
    ) -> Result<GameResult> {
        // Verify controllers match players
        let players: Vec<_> = self.game.players.iter().map(|(id, _)| *id).collect();
        if players.len() != 2 {
            return Err(MtgError::InvalidAction(
                "Game loop requires exactly 2 players".to_string(),
            ));
        }

        let player1_id = players[0];
        let player2_id = players[1];

        if controller1.player_id() != player1_id || controller2.player_id() != player2_id {
            return Err(MtgError::InvalidAction(
                "Controller player IDs don't match game players".to_string(),
            ));
        }

        // Main game loop
        loop {
            // Check win conditions
            if let Some(result) = self.check_win_condition() {
                // Notify controllers of game end
                let winner_id = result.winner;
                controller1.on_game_end(
                    &GameStateView::new(self.game, player1_id),
                    winner_id == Some(player1_id),
                );
                controller2.on_game_end(
                    &GameStateView::new(self.game, player2_id),
                    winner_id == Some(player2_id),
                );
                return Ok(result);
            }

            // Check turn limit
            if self.turns_elapsed >= self.max_turns {
                return Ok(GameResult {
                    winner: None,
                    turns_played: self.turns_elapsed,
                    end_reason: GameEndReason::TurnLimit,
                });
            }

            // Run one turn
            self.run_turn(controller1, controller2)?;
            self.turns_elapsed += 1;
        }
    }

    /// Run a single turn
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

            // Print battlefield state at start of turn
            if let Ok(player) = self.game.players.get(active_player) {
                println!("  Life: {}", player.life);
                let hand_size = self
                    .game
                    .get_player_zones(active_player)
                    .map(|z| z.hand.cards.len())
                    .unwrap_or(0);
                println!("  Hand: {} cards", hand_size);

                let battlefield_cards = self.game.battlefield.cards.len();
                println!("  Battlefield: {} cards", battlefield_cards);
            }
            println!("========================================");
        }

        // Reset turn-based state
        self.reset_turn_state(active_player)?;

        // Run through all steps of the turn
        loop {
            // Execute the step
            self.execute_step(controller1, controller2)?;

            // Try to advance to next step
            if !self.game.turn.advance_step() {
                // End of turn reached
                break;
            }
        }

        // Move to next player's turn
        let players: Vec<_> = self.game.players.iter().map(|(id, _)| *id).collect();
        let next_player = if active_player == players[0] {
            players[1]
        } else {
            players[0]
        };

        self.game.turn.next_turn(next_player);

        Ok(())
    }

    /// Get player name for display
    fn get_player_name(&self, player_id: PlayerId) -> String {
        self.game
            .players
            .get(player_id)
            .map(|p| p.name.to_string())
            .unwrap_or_else(|_| format!("Player {:?}", player_id))
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
            println!("  {}", message);
        }
    }

    /// Log a message at Verbose verbosity level (with lazy step header)
    /// Used for detailed action-by-action logging
    fn log_verbose(&mut self, message: &str) {
        if self.verbosity >= VerbosityLevel::Verbose {
            self.print_step_header_if_needed();
            println!("  {}", message);
        }
    }

    /// Log a message at Minimal verbosity level (no step header needed)
    /// Used for major game events like outcomes
    #[allow(dead_code)]
    fn log_minimal(&mut self, message: &str) {
        if self.verbosity >= VerbosityLevel::Minimal {
            println!("{}", message);
        }
    }

    /// Reset turn-based state for the active player
    fn reset_turn_state(&mut self, active_player: PlayerId) -> Result<()> {
        // Reset lands played this turn
        if let Ok(player) = self.game.players.get_mut(active_player) {
            player.reset_lands_played();
        }

        // Empty mana pools at start of turn
        let player_ids: Vec<_> = self.game.players.iter().map(|(id, _)| *id).collect();
        for player_id in player_ids {
            if let Ok(player) = self.game.players.get_mut(player_id) {
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
            Step::Draw => self.draw_step(),
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
        let cards_to_untap: Vec<_> = self
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
    fn draw_step(&mut self) -> Result<()> {
        let active_player = self.game.turn.active_player;

        // Skip draw on first turn (player going first doesn't draw)
        if self.game.turn.turn_number == 1 {
            self.log_normal("(First turn - no draw)");
            return Ok(());
        }

        // Draw a card
        self.game.draw_card(active_player)?;

        let player_name = self.get_player_name(active_player);
        if let Some(zones) = self.game.get_player_zones(active_player) {
            if let Some(&card_id) = zones.hand.cards.last() {
                if let Ok(card) = self.game.cards.get(card_id) {
                    self.log_normal(&format!("{} draws {}", player_name, card.name));
                    return Ok(());
                }
            }
        }
        self.log_normal(&format!("{} draws a card", player_name));

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

        // Loop: let active player declare attackers until they finish
        loop {
            // Get available actions (attackers they can declare)
            let available_actions = self.get_available_attackers(active_player);

            if available_actions.is_empty() {
                // No creatures to attack with, skip
                break;
            }

            let view = GameStateView::new(self.game, active_player);
            let action = controller.choose_action(&view, &available_actions);

            match action {
                Some(PlayerAction::DeclareAttacker(card_id)) => {
                    self.execute_action(active_player, &PlayerAction::DeclareAttacker(card_id))?;
                }
                Some(PlayerAction::FinishDeclareAttackers)
                | Some(PlayerAction::PassPriority)
                | None => {
                    // Done declaring attackers
                    break;
                }
                _ => {
                    // Invalid action during declare attackers
                    break;
                }
            }
        }

        Ok(())
    }

    fn declare_blockers_step(
        &mut self,
        controller1: &mut dyn PlayerController,
        controller2: &mut dyn PlayerController,
    ) -> Result<()> {
        // Defending player declares blockers
        let active_player = self.game.turn.active_player;
        let players: Vec<_> = self.game.players.iter().map(|(id, _)| *id).collect();
        let defending_player = if active_player == players[0] {
            players[1]
        } else {
            players[0]
        };

        let controller: &mut dyn PlayerController = if defending_player == controller1.player_id() {
            controller1
        } else {
            controller2
        };

        // Loop: let defending player declare blockers until they finish
        loop {
            // Get available actions (blockers they can declare)
            let available_actions = self.get_available_blockers(defending_player);

            if available_actions.is_empty() {
                // No creatures to block with, skip
                break;
            }

            let view = GameStateView::new(self.game, defending_player);
            let action = controller.choose_action(&view, &available_actions);

            match action {
                Some(PlayerAction::DeclareBlocker { blocker, attackers }) => {
                    self.execute_action(
                        defending_player,
                        &PlayerAction::DeclareBlocker { blocker, attackers },
                    )?;
                }
                Some(PlayerAction::FinishDeclareBlockers)
                | Some(PlayerAction::PassPriority)
                | None => {
                    // Done declaring blockers
                    break;
                }
                _ => {
                    // Invalid action during declare blockers
                    break;
                }
            }
        }

        Ok(())
    }

    fn combat_damage_step(
        &mut self,
        controller1: &mut dyn PlayerController,
        controller2: &mut dyn PlayerController,
    ) -> Result<()> {
        // Assign and deal combat damage (this is automatic, no player choices)
        self.game.assign_combat_damage()?;

        // After damage is dealt, players get priority
        self.priority_round(controller1, controller2)?;
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

        // Active player discards to hand size first
        let player_ids = [active_player];

        // Then non-active players (in a 2-player game, just one other player)
        let all_players: Vec<_> = self.game.players.iter().map(|(id, _)| *id).collect();
        let other_players: Vec<_> = all_players
            .iter()
            .filter(|&&p| p != active_player)
            .copied()
            .collect();

        // Process active player first, then others
        for &player_id in player_ids.iter().chain(other_players.iter()) {
            let hand_size = self
                .game
                .get_player_zones(player_id)
                .map(|z| z.hand.cards.len())
                .unwrap_or(0);

            let max_hand_size = self.game.players.get(player_id)?.max_hand_size;

            if hand_size > max_hand_size {
                let discard_count = hand_size - max_hand_size;

                let player_name = self.get_player_name(player_id);
                self.log_normal(&format!(
                    "{} must discard {} cards (hand size: {}, max: {})",
                    player_name, discard_count, hand_size, max_hand_size
                ));

                // Get the appropriate controller
                let controller: &mut dyn PlayerController = if player_id == controller1.player_id()
                {
                    controller1
                } else {
                    controller2
                };

                // Ask controller which cards to discard
                let view = GameStateView::new(self.game, player_id);
                let cards_to_discard = controller.choose_cards_to_discard(&view, discard_count);

                // Verify correct number of cards
                if cards_to_discard.len() != discard_count {
                    return Err(crate::MtgError::InvalidAction(format!(
                        "Must discard exactly {} cards, got {}",
                        discard_count,
                        cards_to_discard.len()
                    )));
                }

                // Move cards to graveyard
                for card_id in cards_to_discard {
                    if let Some(zones) = self.game.get_player_zones_mut(player_id) {
                        if zones.hand.contains(card_id) {
                            zones.hand.remove(card_id);
                            zones.graveyard.add(card_id);

                            let card_name = self
                                .game
                                .cards
                                .get(card_id)
                                .map(|c| c.name.as_str())
                                .unwrap_or("Unknown");
                            let player_name = self.get_player_name(player_id);
                            self.log_normal(&format!("{} discards {}", player_name, card_name));
                        } else {
                            return Err(crate::MtgError::InvalidAction(format!(
                                "Card {:?} not in player's hand",
                                card_id
                            )));
                        }
                    }
                }
            }
        }

        // Empty mana pools
        for player_id in all_players {
            if let Ok(player) = self.game.players.get_mut(player_id) {
                player.mana_pool.clear();
            }
        }

        // TODO: Remove damage from creatures

        Ok(())
    }

    /// Priority round - players get chances to act until both pass
    fn priority_round(
        &mut self,
        controller1: &mut dyn PlayerController,
        controller2: &mut dyn PlayerController,
    ) -> Result<()> {
        let active_player = self.game.turn.active_player;
        let players: Vec<_> = self.game.players.iter().map(|(id, _)| *id).collect();
        let non_active_player = if active_player == players[0] {
            players[1]
        } else {
            players[0]
        };

        // Active player gets priority first
        let mut current_priority = active_player;
        let mut consecutive_passes = 0;
        let mut action_count = 0;
        const MAX_ACTIONS_PER_PRIORITY: usize = 1000; // Safety limit (allows many land taps + spells)

        while consecutive_passes < 2 {
            // Safety check to prevent infinite loops
            action_count += 1;
            if action_count > MAX_ACTIONS_PER_PRIORITY {
                return Err(crate::MtgError::InvalidAction(format!(
                    "Priority round exceeded max actions ({}), possible infinite loop",
                    MAX_ACTIONS_PER_PRIORITY
                )));
            }

            // Get the appropriate controller
            let controller: &mut dyn PlayerController =
                if current_priority == controller1.player_id() {
                    controller1
                } else {
                    controller2
                };

            // Get available actions for current priority player
            let available_actions = self.get_available_actions(current_priority);

            // Let controller choose an action
            let view = GameStateView::new(self.game, current_priority);
            let action = controller.choose_action(&view, &available_actions);

            match action {
                Some(action) => {
                    // Execute the action
                    self.execute_action(current_priority, &action)?;
                    consecutive_passes = 0; // Reset pass counter
                }
                None => {
                    // Pass priority
                    consecutive_passes += 1;
                    controller.on_priority_passed(&view);
                }
            }

            // Switch priority to other player
            current_priority = if current_priority == active_player {
                non_active_player
            } else {
                active_player
            };
        }

        Ok(())
    }

    /// Get available attackers for a player
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
    fn get_available_actions(&self, player_id: PlayerId) -> Vec<PlayerAction> {
        let mut actions = Vec::new();

        // Always can pass priority
        actions.push(PlayerAction::PassPriority);

        let current_step = self.game.turn.current_step;

        // Can play lands in main phases if player hasn't played one this turn
        if current_step.can_play_lands() {
            if let Ok(player) = self.game.players.get(player_id) {
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
                        if let Ok(player) = self.game.players.get(player_id) {
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

    /// Execute a player action
    fn execute_action(&mut self, player_id: PlayerId, action: &PlayerAction) -> Result<()> {
        if !matches!(action, PlayerAction::PassPriority) {
            let player_name = self.get_player_name(player_id);
            let action_desc = self.describe_action(action);
            self.log_verbose(&format!("{} {}", player_name, action_desc));
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
                let player_name = self.get_player_name(player_id);
                let card_name = self
                    .game
                    .cards
                    .get(*card_id)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Unknown");
                self.log_normal(&format!("{} casts {}", player_name, card_name));

                self.game.cast_spell(player_id, *card_id, targets.clone())?;

                // Immediately resolve spell (simplified - no stack interaction yet)
                self.game.resolve_spell(*card_id)?;

                let card_name = self
                    .game
                    .cards
                    .get(*card_id)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Unknown");
                self.log_normal(&format!("{} resolves", card_name));
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
    fn describe_action(&self, action: &PlayerAction) -> String {
        match action {
            PlayerAction::PlayLand(card_id) => {
                let card_name = self
                    .game
                    .cards
                    .get(*card_id)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Unknown");
                format!("plays {}", card_name)
            }
            PlayerAction::TapForMana(card_id) => {
                let card_name = self
                    .game
                    .cards
                    .get(*card_id)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Unknown");
                format!("taps {} for mana", card_name)
            }
            PlayerAction::CastSpell { card_id, .. } => {
                let card_name = self
                    .game
                    .cards
                    .get(*card_id)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Unknown");
                format!("casts {}", card_name)
            }
            PlayerAction::DeclareAttacker(card_id) => {
                let card_name = self
                    .game
                    .cards
                    .get(*card_id)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Unknown");
                format!("declares {} as attacker", card_name)
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
                format!(
                    "blocks with {} (blocking {:?})",
                    blocker_name, attacker_names
                )
            }
            PlayerAction::FinishDeclareAttackers => "finishes declaring attackers".to_string(),
            PlayerAction::FinishDeclareBlockers => "finishes declaring blockers".to_string(),
            PlayerAction::PassPriority => "passes priority".to_string(),
        }
    }

    /// Check if the game has reached a win condition
    fn check_win_condition(&self) -> Option<GameResult> {
        let players: Vec<_> = self.game.players.iter().map(|(id, p)| (*id, p)).collect();

        // Check for player death (life <= 0)
        for (player_id, player) in &players {
            if player.life <= 0 {
                let winner = players
                    .iter()
                    .find(|(id, _)| id != player_id)
                    .map(|(id, _)| *id);
                return Some(GameResult {
                    winner,
                    turns_played: self.turns_elapsed,
                    end_reason: GameEndReason::PlayerDeath(*player_id),
                });
            }
        }

        // Check for decking (empty library when trying to draw)
        for (player_id, _) in &players {
            if let Some(zones) = self.game.get_player_zones(*player_id) {
                if zones.library.is_empty() {
                    let winner = players
                        .iter()
                        .find(|(id, _)| id != player_id)
                        .map(|(id, _)| *id);
                    return Some(GameResult {
                        winner,
                        turns_played: self.turns_elapsed,
                        end_reason: GameEndReason::Decking(*player_id),
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
        let players: Vec<_> = game.players.iter().map(|(id, _)| *id).collect();
        let alice = players[0];

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
        let players: Vec<_> = game.players.iter().map(|(id, _)| *id).collect();
        let alice = players[0];

        // Add a card to Alice's library
        let card_id = game.next_card_id();
        let card = crate::core::Card::new(card_id, "Test Card".to_string(), alice);
        game.cards.insert(card_id, card);
        if let Some(zones) = game.get_player_zones_mut(alice) {
            zones.library.add(card_id);
        }

        // Set turn to 2 (so draw happens)
        game.turn.turn_number = 2;

        // Run draw step
        let mut game_loop = GameLoop::new(&mut game);
        game_loop.draw_step().unwrap();

        // Card should be in hand
        if let Some(zones) = game.get_player_zones(alice) {
            assert!(zones.hand.contains(card_id));
            assert!(!zones.library.contains(card_id));
        }
    }

    #[test]
    fn test_check_win_condition_life() {
        let mut game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let players: Vec<_> = game.players.iter().map(|(id, _)| *id).collect();
        let bob = players[1];

        // Set Bob's life to 0
        game.players.get_mut(bob).unwrap().life = 0;

        let game_loop = GameLoop::new(&mut game);
        let result = game_loop.check_win_condition();

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.end_reason, GameEndReason::PlayerDeath(bob));
    }
}
