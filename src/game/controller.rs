//! Player controller interface
//!
//! This module defines the PlayerController trait that AI and UI implementations
//! must implement. The design matches Java Forge's PlayerController.java where
//! the controller chooses from available spell abilities (lands, spells, abilities)
//! and makes decisions during the spell casting process.
//!
//! ## Key Differences from Previous Design
//!
//! - **Unified Spell Ability Selection**: Instead of separate methods for lands
//!   and spells, `choose_spell_ability_to_play()` returns any playable ability
//! - **Correct Mana Timing**: Mana is tapped during step 6 of 8 in the casting
//!   process, AFTER the spell is on the stack
//! - **Callback-Based Casting**: Controller provides callbacks for targeting and
//!   mana payment during the casting sequence

use crate::core::{CardId, ManaCost, PlayerId, SpellAbility};
use crate::game::{GameState, Step};
use crate::zones::Zone;
use smallvec::SmallVec;

/// Format available spell/ability choices as a menu
///
/// This creates a standardized menu showing all available actions for a player.
/// The format is: "<PLAYERNAME> available actions: [0] Play land: Swamp..."
///
/// All controllers should use this function when showing choices to maintain
/// a consistent format across the codebase.
pub fn format_choice_menu(view: &GameStateView, available: &[SpellAbility]) -> String {
    let mut output = String::new();
    let player_name = view.player_name();

    output.push_str(&format!("\n{} available actions:\n", player_name));

    for (idx, ability) in available.iter().enumerate() {
        match ability {
            SpellAbility::PlayLand { card_id } => {
                let name = view.card_name(*card_id).unwrap_or_default();
                output.push_str(&format!("  [{}] Play land: {}\n", idx, name));
            }
            SpellAbility::CastSpell { card_id } => {
                let name = view.card_name(*card_id).unwrap_or_default();
                output.push_str(&format!("  [{}] Cast spell: {}\n", idx, name));
            }
            SpellAbility::ActivateAbility { card_id, .. } => {
                let name = view.card_name(*card_id).unwrap_or_default();
                output.push_str(&format!("  [{}] Activate ability: {}\n", idx, name));
            }
        }
    }

    output
}

/// Read-only view of game state for controllers
///
/// This provides access to game information without allowing mutation.
/// Controllers should only inspect this view to make decisions.
pub struct GameStateView<'a> {
    game: &'a GameState,
    player_id: PlayerId,
}

impl<'a> GameStateView<'a> {
    /// Create a new view of the game state from a player's perspective
    pub fn new(game: &'a GameState, player_id: PlayerId) -> Self {
        GameStateView { game, player_id }
    }

    /// Get the player ID this view is for
    pub fn player_id(&self) -> PlayerId {
        self.player_id
    }

    /// Get cards in this player's hand
    pub fn hand(&self) -> &[CardId] {
        self.game
            .get_player_zones(self.player_id)
            .map(|zones| zones.hand.cards.as_slice())
            .unwrap_or(&[])
    }

    /// Get cards in a specific player's hand
    pub fn player_hand(&self, player_id: PlayerId) -> &[CardId] {
        self.game
            .get_player_zones(player_id)
            .map(|zones| zones.hand.cards.as_slice())
            .unwrap_or(&[])
    }

    /// Get cards on the battlefield
    pub fn battlefield(&self) -> &[CardId] {
        &self.game.battlefield.cards
    }

    /// Get cards in this player's graveyard
    pub fn graveyard(&self) -> &[CardId] {
        self.game
            .get_player_zones(self.player_id)
            .map(|zones| zones.graveyard.cards.as_slice())
            .unwrap_or(&[])
    }

    /// Get cards in a specific player's graveyard
    pub fn player_graveyard(&self, player_id: PlayerId) -> &[CardId] {
        self.game
            .get_player_zones(player_id)
            .map(|zones| zones.graveyard.cards.as_slice())
            .unwrap_or(&[])
    }

    /// Check if a card is in a specific zone
    pub fn is_card_in_zone(&self, card_id: CardId, zone: Zone) -> bool {
        match zone {
            Zone::Hand => self
                .game
                .get_player_zones(self.player_id)
                .map(|z| z.hand.contains(card_id))
                .unwrap_or(false),
            Zone::Battlefield => self.game.battlefield.contains(card_id),
            Zone::Graveyard => self
                .game
                .get_player_zones(self.player_id)
                .map(|z| z.graveyard.contains(card_id))
                .unwrap_or(false),
            Zone::Library => self
                .game
                .get_player_zones(self.player_id)
                .map(|z| z.library.contains(card_id))
                .unwrap_or(false),
            Zone::Stack => self.game.stack.contains(card_id),
            Zone::Exile => self
                .game
                .get_player_zones(self.player_id)
                .map(|z| z.exile.contains(card_id))
                .unwrap_or(false),
            Zone::Command => false, // Command zone not yet implemented
        }
    }

    /// Get a card's name
    pub fn card_name(&self, card_id: CardId) -> Option<String> {
        self.game.cards.get(card_id).ok().map(|c| c.name.to_string())
    }

    /// Check if a card is a land
    pub fn is_land(&self, card_id: CardId) -> bool {
        self.game.cards.get(card_id).map(|c| c.is_land()).unwrap_or(false)
    }

    /// Get the current step
    pub fn current_step(&self) -> Step {
        self.game.turn.current_step
    }

    /// Get the current turn number
    pub fn turn_number(&self) -> u32 {
        self.game.turn.turn_number
    }

    /// Get a card's name (convenience method)
    pub fn get_card_name(&self, card_id: CardId) -> Option<String> {
        self.card_name(card_id)
    }

    /// Get a reference to a card
    ///
    /// This allows controllers to inspect card properties for decision-making
    pub fn get_card(&self, card_id: CardId) -> Option<&crate::core::Card> {
        self.game.cards.get(card_id).ok()
    }

    /// Check if a card is tapped
    pub fn is_tapped(&self, card_id: CardId) -> bool {
        self.game.cards.get(card_id).map(|c| c.tapped).unwrap_or(false)
    }

    /// Get access to the game logger
    ///
    /// This allows controllers and other game components to log messages
    /// at appropriate verbosity levels without needing to track verbosity themselves.
    pub fn logger(&self) -> &crate::game::GameLogger {
        &self.game.logger
    }

    /// Get player's current life total
    pub fn life(&self) -> i32 {
        self.game.get_player(self.player_id).ok().map(|p| p.life).unwrap_or(0)
    }

    /// Get player's name
    pub fn player_name(&self) -> String {
        self.game
            .get_player(self.player_id)
            .ok()
            .map(|p| p.name.to_string())
            .unwrap_or_else(|| {
                // Use 1-based indexing for human-readable player numbers
                let player_num = self.player_id.as_u32() + 1;
                format!("Player {}", player_num)
            })
    }

    /// Get a specific player's life total
    pub fn player_life(&self, player_id: PlayerId) -> i32 {
        self.game.get_player(player_id).ok().map(|p| p.life).unwrap_or(0)
    }

    /// Get all opponent player IDs
    ///
    /// Returns an iterator over all players except the current player.
    /// Useful for multiplayer games.
    pub fn opponents(&self) -> impl Iterator<Item = PlayerId> + '_ {
        self.game
            .players
            .iter()
            .map(|p| p.id)
            .filter(move |&id| id != self.player_id)
    }

    /// Get opponent life total in a 2-player game
    ///
    /// For 2-player games, returns the opponent's life total.
    /// For multiplayer, returns the total life of all opponents combined.
    pub fn opponent_life(&self) -> i32 {
        self.opponents().map(|id| self.player_life(id)).sum()
    }

    /// Get player's mana pool
    pub fn available_mana(&self) -> (u8, u8, u8, u8, u8, u8) {
        // Returns (white, blue, black, red, green, colorless)
        self.game
            .get_player(self.player_id)
            .ok()
            .map(|p| {
                (
                    p.mana_pool.white,
                    p.mana_pool.blue,
                    p.mana_pool.black,
                    p.mana_pool.red,
                    p.mana_pool.green,
                    p.mana_pool.colorless,
                )
            })
            .unwrap_or((0, 0, 0, 0, 0, 0))
    }

    /// Check if player can play lands this turn
    pub fn can_play_land(&self) -> bool {
        self.game
            .get_player(self.player_id)
            .ok()
            .map(|p| p.can_play_land())
            .unwrap_or(false)
    }
}

/// Player controller interface
///
/// This trait defines the decision-making interface for players (AI or human).
/// The design matches Java Forge's PlayerController where the controller:
/// 1. Chooses spell abilities to play from a unified list (lands, spells, abilities)
/// 2. Provides callbacks during the spell casting process for targeting and mana payment
/// 3. Makes combat decisions
/// 4. Handles cleanup and notifications
///
/// ## Mana Payment Timing
///
/// Unlike the previous design, mana is NOT tapped during priority rounds.
/// Instead, when a spell is cast, the game follows the 8-step casting process
/// (MTG Rules 601.2), and mana sources are tapped during step 6, which happens
/// AFTER the spell is already on the stack.
pub trait PlayerController {
    /// Get the player ID this controller is responsible for
    fn player_id(&self) -> PlayerId;

    /// Choose a spell ability to play
    ///
    /// This is the main decision point during priority. The controller receives
    /// a list of all available spell abilities:
    /// - Land plays (if can play lands this turn)
    /// - Castable spells (if have mana and in appropriate phase)
    /// - Activated abilities (if can activate)
    ///
    /// Returns the chosen ability, or None to pass priority.
    ///
    /// Controllers that need randomness should maintain their own RNG
    /// (seeded independently from the game engine's RNG).
    ///
    /// ## Java Forge Equivalent
    /// This matches `PlayerController.chooseSpellAbilityToPlay()` which returns
    /// a list of SpellAbilities to play (usually just one, but can be multiple
    /// for special cases like multiple lands in one turn).
    fn choose_spell_ability_to_play(
        &mut self,
        view: &GameStateView,
        available: &[SpellAbility],
    ) -> Option<SpellAbility>;

    /// Choose targets for a spell or ability
    ///
    /// Called during step 3 of casting a spell (MTG Rules 601.2c).
    /// The controller must choose valid targets from the provided list.
    ///
    /// For spells with no targets, this may not be called, or valid_targets
    /// will be empty.
    ///
    /// ## Java Forge Equivalent
    /// Matches `PlayerController.chooseTargetsFor(SpellAbility)`
    fn choose_targets(
        &mut self,
        view: &GameStateView,
        spell: CardId,
        valid_targets: &[CardId],
    ) -> SmallVec<[CardId; 4]>;

    /// Choose which mana sources to tap to pay a cost
    ///
    /// Called during step 6 of casting a spell (MTG Rules 601.2g).
    /// At this point, the spell is already on the stack.
    ///
    /// The controller must choose which permanents to tap for mana to pay
    /// the given cost. Returns the card IDs to tap in order.
    ///
    /// ## Java Forge Equivalent
    /// This is part of `PlayerController.payManaCost(...)` which the AI
    /// implements with `ComputerUtilMana.payManaCost()`.
    fn choose_mana_sources_to_pay(
        &mut self,
        view: &GameStateView,
        cost: &ManaCost,
        available_sources: &[CardId],
    ) -> SmallVec<[CardId; 8]>;

    /// Choose which creatures to declare as attackers
    ///
    /// Called during the declare attackers step.
    /// Returns a list of creature card IDs that should attack.
    fn choose_attackers(&mut self, view: &GameStateView, available_creatures: &[CardId]) -> SmallVec<[CardId; 8]>;

    /// Choose how to block attacking creatures
    ///
    /// Called during the declare blockers step.
    /// Returns pairs of (blocker, attacker) indicating which creature
    /// blocks which attacker.
    fn choose_blockers(
        &mut self,
        view: &GameStateView,
        available_blockers: &[CardId],
        attackers: &[CardId],
    ) -> SmallVec<[(CardId, CardId); 8]>;

    /// Choose the damage assignment order for blockers
    ///
    /// Called during combat damage step when an attacker is blocked by multiple creatures.
    /// The attacker's controller chooses the order in which damage will be assigned to blockers.
    /// MTG Rules 509.2: The attacking player announces the damage assignment order.
    ///
    /// Returns the blockers in the order damage should be assigned. All blockers must be included.
    fn choose_damage_assignment_order(
        &mut self,
        view: &GameStateView,
        attacker: CardId,
        blockers: &[CardId],
    ) -> SmallVec<[CardId; 4]>;

    /// Choose cards to discard to maximum hand size
    ///
    /// Called during cleanup step if hand size exceeds maximum.
    fn choose_cards_to_discard(&mut self, view: &GameStateView, hand: &[CardId], count: usize)
        -> SmallVec<[CardId; 7]>;

    /// Notification that priority was passed
    ///
    /// Called when this controller passes priority, allowing it to track
    /// game flow or update internal state.
    fn on_priority_passed(&mut self, view: &GameStateView);

    /// Notification that the game has ended
    ///
    /// Called when the game is over, with a boolean indicating whether
    /// this player won.
    fn on_game_end(&mut self, view: &GameStateView, won: bool);

    /// Get serializable snapshot state for this controller
    ///
    /// Returns controller-specific state that should be preserved across snapshot/resume.
    /// Most controllers (Random, Heuristic, etc.) return None since they don't have
    /// state to preserve. FixedScriptController returns its current script position.
    ///
    /// This method is called by the snapshot system to capture controller state.
    fn get_snapshot_state(&self) -> Option<serde_json::Value> {
        None // Default implementation returns None
    }

    /// Check if controller has more choices available
    ///
    /// Used for `--stop-when-fixed-exhausted` flag. Returns true if the controller
    /// has more choices to make, false if exhausted (only relevant for FixedScriptController).
    ///
    /// Default implementation returns true (infinite choices for AI/human controllers).
    fn has_more_choices(&self) -> bool {
        true
    }

    /// Get the controller type for snapshot persistence
    ///
    /// Returns the controller type so snapshots can record which controller
    /// was active, even for stateless controllers like Heuristic and Zero.
    /// This is critical for snapshot/resume functionality - without this,
    /// stateless controllers would be incorrectly restored as Zero controllers.
    fn get_controller_type(&self) -> crate::game::snapshot::ControllerType;
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    include!("controller_tests.rs");
}
