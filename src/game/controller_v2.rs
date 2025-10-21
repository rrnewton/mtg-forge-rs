//! Two-layer controller architecture
//!
//! See CONTROLLER_DESIGN.md for full architecture documentation.
//!
//! ## Layer 1: PlayerController (Specific Callbacks)
//!
//! Provides specific callback methods for each type of game decision.
//! Uses zero-copy patterns (slices, SmallVec) for efficiency.
//! Better suited for UI implementation and heuristic AI.
//!
//! Example:
//! ```ignore
//! impl PlayerController for MyAI {
//!     fn choose_land_to_play(&mut self, view: &GameStateView, lands: &[CardId]) -> Option<CardId> {
//!         // Pick first land
//!         lands.first().copied()
//!     }
//!     // ... implement other methods
//! }
//! ```
//!
//! ## Layer 2: DecisionMaker (Generic Choices)
//!
//! Reduces all decisions to "pick option 0-N" with string descriptions.
//! Better suited for tree search and MCTS algorithms.
//!
//! Example:
//! ```ignore
//! impl DecisionMaker for SearchAgent {
//!     fn make_choice(&mut self, prompt: &str, options: &[&str]) -> usize {
//!         // Use MCTS to evaluate each option
//!         mcts_search(prompt, options)
//!     }
//! }
//! ```
//!
//! ## GameLoop Integration
//!
//! The GameLoop can work with either interface:
//! - For PlayerController: Calls specific methods with typed options
//! - For DecisionMaker: Converts options to strings, calls make_choice(), maps index back
//!
//! See examples in CONTROLLER_DESIGN.md.

use crate::core::CardId;
use crate::game::GameStateView;
use smallvec::SmallVec;

/// Layer 1: Specific callback interface for game decisions
///
/// This trait provides type-safe, specific methods for each kind of decision
/// a player must make. Implementations should use the GameStateView to inspect
/// game state and make informed choices.
///
/// All methods use zero-copy patterns:
/// - Accept slices (&[CardId]) instead of Vec
/// - Return SmallVec to avoid heap allocation in common cases
/// - Borrow strings from existing game state where possible
pub trait PlayerController {
    /// Get the player ID this controller is responsible for
    fn player_id(&self) -> crate::core::PlayerId;

    /// Choose which land to play from hand (if any)
    ///
    /// Called during main phases when the player can play lands.
    /// Returns None to decline playing a land.
    fn choose_land_to_play(
        &mut self,
        view: &GameStateView,
        lands_in_hand: &[CardId],
    ) -> Option<CardId>;

    /// Choose which spell to cast from hand (if any)
    ///
    /// Returns the spell CardId and a SmallVec of targets.
    /// Returns None to decline casting a spell.
    fn choose_spell_to_cast(
        &mut self,
        view: &GameStateView,
        castable_spells: &[CardId],
    ) -> Option<(CardId, SmallVec<[CardId; 4]>)>;

    /// Choose which permanent to tap for mana (if any)
    ///
    /// Called when the player needs mana or chooses to tap for mana.
    /// Returns None to decline tapping for mana.
    fn choose_card_to_tap_for_mana(
        &mut self,
        view: &GameStateView,
        tappable_cards: &[CardId],
    ) -> Option<CardId>;

    /// Choose which creatures to declare as attackers
    ///
    /// Called during declare attackers step.
    /// Returns a SmallVec of CardIds to attack with (may be empty).
    fn choose_attackers(
        &mut self,
        view: &GameStateView,
        available_creatures: &[CardId],
    ) -> SmallVec<[CardId; 8]>;

    /// Choose which creatures block which attackers
    ///
    /// Called during declare blockers step.
    /// Returns pairs of (blocker, attacker) assignments.
    fn choose_blockers(
        &mut self,
        view: &GameStateView,
        available_blockers: &[CardId],
        attackers: &[CardId],
    ) -> SmallVec<[(CardId, CardId); 8]>;

    /// Choose cards to discard from hand
    ///
    /// Called during cleanup step when hand size exceeds maximum.
    /// Must return exactly `count` cards from `hand`.
    fn choose_cards_to_discard(
        &mut self,
        view: &GameStateView,
        hand: &[CardId],
        count: usize,
    ) -> SmallVec<[CardId; 7]>;

    /// Decide whether to take an action during priority
    ///
    /// Called repeatedly during priority rounds.
    /// Return true to pass priority, false to take an action.
    /// If false, the game will call other choose_* methods to determine the action.
    fn wants_to_pass_priority(&mut self, view: &GameStateView) -> bool;

    /// Optional: Called when priority is passed (for logging/debugging)
    fn on_priority_passed(&mut self, _view: &GameStateView) {}

    /// Optional: Called when the game ends (for cleanup/logging)
    fn on_game_end(&mut self, _view: &GameStateView, _won: bool) {}
}

/// Layer 2: Generic decision interface for tree search
///
/// This trait reduces all game decisions to a simple "pick option 0 to N-1" choice.
/// Each choice is presented as a prompt string and a list of option strings.
///
/// This abstraction is ideal for:
/// - Game tree search algorithms
/// - Monte Carlo Tree Search (MCTS)
/// - Recording/replaying games
/// - Testing determinism
///
/// The lack of strong types is intentional - it forces the implementation to
/// work purely from descriptions, which is necessary for general search algorithms.
pub trait DecisionMaker {
    /// Get the player ID this decision maker is responsible for
    fn player_id(&self) -> crate::core::PlayerId;

    /// Make a choice from available options
    ///
    /// # Arguments
    /// * `prompt` - Description of what decision is being made
    /// * `options` - Descriptions of each option (indexed 0 to N-1)
    ///
    /// # Returns
    /// Index of the chosen option (0 to options.len()-1)
    ///
    /// # Panics
    /// Implementations may panic or return invalid results if the returned
    /// index is out of bounds. The game engine will validate the choice.
    fn make_choice(&mut self, prompt: &str, options: &[&str]) -> usize;
}

/// Adapter to wrap a PlayerController as a DecisionMaker
///
/// This adapter translates the specific callback methods of PlayerController
/// into generic "pick option N" choices. Card names and other descriptions
/// are extracted from the game state to avoid string allocation where possible.
pub struct DecisionTreeAdapter<C: PlayerController> {
    controller: C,
}

impl<C: PlayerController> DecisionTreeAdapter<C> {
    pub fn new(controller: C) -> Self {
        DecisionTreeAdapter { controller }
    }

    pub fn into_inner(self) -> C {
        self.controller
    }
}

impl<C: PlayerController> DecisionMaker for DecisionTreeAdapter<C> {
    fn player_id(&self) -> crate::core::PlayerId {
        self.controller.player_id()
    }

    fn make_choice(&mut self, _prompt: &str, _options: &[&str]) -> usize {
        // For now, this is a placeholder that delegates back to the controller
        // In a full implementation, we would need to:
        // 1. Parse the prompt to determine decision type
        // 2. Parse options to extract CardIds or other data
        // 3. Call the appropriate PlayerController method
        // 4. Map the result back to an option index
        //
        // This is intentionally left as a TODO since it requires careful design
        // to avoid allocations while parsing descriptions back into CardIds.
        todo!("DecisionTreeAdapter needs context to translate choices back to CardIds")
    }
}

#[cfg(test)]
mod tests {
    // Tests will be added as we implement concrete controllers
}
