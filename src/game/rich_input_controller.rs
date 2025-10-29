//! Rich input controller that parses textual commands
//!
//! This controller accepts rich text commands like "play swamp" or "cast black_knight"
//! and converts them to numeric choices based on available options.
//!
//! ## Command Syntax
//!
//! - **Verbs**: Play, Cast, Attack, Block, Discard, Pass (case-insensitive)
//! - **Card names**: Case-insensitive, spaces/underscores equivalent, prefix matching allowed
//! - **Quotes**: Optional for card names at end of command
//! - **Examples**: `play swamp`, `cast "Black Knight"`, `attack serra`
//!
//! ## Blocking Syntax
//!
//! Comma-separated clauses: `BlackKnight blocks WhiteKnight, SerraAngel blocks RoyalAssassin`

use crate::core::{CardId, ManaCost, PlayerId, SpellAbility};
use crate::game::controller::{GameStateView, PlayerController};
use smallvec::SmallVec;

/// Controller that parses rich text commands
pub struct RichInputController {
    player_id: PlayerId,
    /// Script of text commands (consumed from front)
    commands: Vec<String>,
    /// Current index in the command queue
    current_index: usize,
}

impl RichInputController {
    /// Create a new rich input controller
    ///
    /// # Arguments
    /// * `player_id` - The player ID this controller manages
    /// * `commands` - Vector of text commands to execute
    pub fn new(player_id: PlayerId, commands: Vec<String>) -> Self {
        RichInputController {
            player_id,
            commands,
            current_index: 0,
        }
    }

    /// Get the next command from the script
    fn next_command(&mut self) -> Option<String> {
        if self.current_index < self.commands.len() {
            let cmd = self.commands[self.current_index].clone();
            self.current_index += 1;
            Some(cmd)
        } else {
            None
        }
    }

    /// Normalize a string for comparison
    ///
    /// - Converts to lowercase
    /// - Removes spaces and underscores
    /// - Removes non-alphanumeric characters (except for basic punctuation)
    pub fn normalize(s: &str) -> String {
        s.chars()
            .filter(|c| !c.is_whitespace() && *c != '_')
            .collect::<String>()
            .to_lowercase()
    }

    /// Check if a card name matches a pattern (prefix matching)
    pub fn card_matches(card_name: &str, pattern: &str) -> bool {
        let normalized_card = Self::normalize(card_name);
        let normalized_pattern = Self::normalize(pattern);
        normalized_card.starts_with(&normalized_pattern)
    }

    /// Parse a quoted string or bare word at the end of a command
    ///
    /// Returns the extracted string and the remaining text
    fn extract_card_name(text: &str) -> Option<String> {
        let trimmed = text.trim();

        // Check for quoted string
        if trimmed.starts_with('"') {
            if let Some(end_quote) = trimmed[1..].find('"') {
                return Some(trimmed[1..=end_quote].to_string());
            }
        }

        // Otherwise, take everything after the verb as the card name
        Some(trimmed.to_string())
    }

    /// Parse a spell ability choice command
    ///
    /// Examples: "play swamp", "cast lightning bolt", "0", "pass"
    pub fn parse_spell_ability_choice(
        command: &str,
        view: &GameStateView,
        available: &[SpellAbility],
    ) -> Option<SpellAbility> {
        let cmd = command.trim().to_lowercase();

        // Handle numeric choice (legacy format)
        if let Ok(idx) = cmd.parse::<usize>() {
            if idx == 0 {
                return None; // Pass priority
            }
            let ability_idx = idx - 1;
            if ability_idx < available.len() {
                return Some(available[ability_idx].clone());
            } else {
                return None; // Out of bounds = pass
            }
        }

        // Handle "pass" or "p"
        if cmd == "pass" || cmd == "p" {
            return None;
        }

        // Parse verb + card name
        if cmd.starts_with("play ") {
            let card_pattern = &cmd[5..];
            // Find matching PlayLand ability
            for ability in available {
                if let SpellAbility::PlayLand { card_id } = ability {
                    if let Some(card_name) = view.card_name(*card_id) {
                        if Self::card_matches(&card_name, card_pattern) {
                            return Some(ability.clone());
                        }
                    }
                }
            }
        } else if cmd.starts_with("cast ") {
            let card_pattern = &cmd[5..];
            // Find matching CastSpell ability
            for ability in available {
                if let SpellAbility::CastSpell { card_id } = ability {
                    if let Some(card_name) = view.card_name(*card_id) {
                        if Self::card_matches(&card_name, card_pattern) {
                            return Some(ability.clone());
                        }
                    }
                }
            }
        }

        // Command not recognized or no match found - pass priority
        None
    }
}

impl PlayerController for RichInputController {
    fn player_id(&self) -> PlayerId {
        self.player_id
    }

    fn choose_spell_ability_to_play(
        &mut self,
        view: &GameStateView,
        available: &[SpellAbility],
    ) -> Option<SpellAbility> {
        if let Some(command) = self.next_command() {
            Self::parse_spell_ability_choice(&command, view, available)
        } else {
            // No more commands - pass priority
            None
        }
    }

    fn choose_targets(
        &mut self,
        _view: &GameStateView,
        _spell: CardId,
        valid_targets: &[CardId],
    ) -> SmallVec<[CardId; 4]> {
        if valid_targets.is_empty() {
            return SmallVec::new();
        }

        if valid_targets.len() == 1 {
            // Only one target - no choice needed
            let mut targets = SmallVec::new();
            targets.push(valid_targets[0]);
            return targets;
        }

        // For now, just take the first target
        // TODO: Implement rich syntax for target selection
        let mut targets = SmallVec::new();
        targets.push(valid_targets[0]);
        targets
    }

    fn choose_mana_sources_to_pay(
        &mut self,
        _view: &GameStateView,
        cost: &ManaCost,
        available_sources: &[CardId],
    ) -> SmallVec<[CardId; 8]> {
        // Simple greedy approach: take first N sources
        let mut sources = SmallVec::new();
        let needed = cost.cmc() as usize;

        for &source_id in available_sources.iter().take(needed) {
            sources.push(source_id);
        }

        sources
    }

    fn choose_attackers(
        &mut self,
        view: &GameStateView,
        available_creatures: &[CardId],
    ) -> SmallVec<[CardId; 8]> {
        if available_creatures.is_empty() {
            return SmallVec::new();
        }

        if let Some(command) = self.next_command() {
            let cmd = command.trim().to_lowercase();

            // Handle numeric choice (legacy format)
            if let Ok(num) = cmd.parse::<usize>() {
                let num_attackers = num.min(available_creatures.len());
                return available_creatures
                    .iter()
                    .take(num_attackers)
                    .copied()
                    .collect();
            }

            // Parse "attack X" commands
            let mut attackers = SmallVec::new();
            for clause in command.split(';') {
                let clause = clause.trim().to_lowercase();
                if clause.starts_with("attack ") {
                    let card_pattern = &clause[7..];
                    for &creature_id in available_creatures {
                        if let Some(card_name) = view.card_name(creature_id) {
                            if Self::card_matches(&card_name, card_pattern) {
                                if !attackers.contains(&creature_id) {
                                    attackers.push(creature_id);
                                }
                            }
                        }
                    }
                } else if clause == "done" {
                    break;
                }
            }

            attackers
        } else {
            // No more commands - don't attack
            SmallVec::new()
        }
    }

    fn choose_blockers(
        &mut self,
        view: &GameStateView,
        available_blockers: &[CardId],
        attackers: &[CardId],
    ) -> SmallVec<[(CardId, CardId); 8]> {
        if available_blockers.is_empty() || attackers.is_empty() {
            return SmallVec::new();
        }

        if let Some(command) = self.next_command() {
            let cmd = command.trim().to_lowercase();

            // Handle numeric choice (legacy format)
            if let Ok(num) = cmd.parse::<usize>() {
                let num_blockers = num.min(available_blockers.len());
                let mut blocks = SmallVec::new();
                for &blocker_id in available_blockers.iter().take(num_blockers) {
                    blocks.push((blocker_id, attackers[0]));
                }
                return blocks;
            }

            // Parse "X blocks Y" commands
            let mut blocks = SmallVec::new();
            for clause in command.split(';') {
                let clause = clause.trim().to_lowercase();
                if clause.contains(" blocks ") {
                    if let Some(blocks_pos) = clause.find(" blocks ") {
                        let blocker_pattern = &clause[..blocks_pos];
                        let attacker_pattern = &clause[blocks_pos + 8..];

                        // Find matching blocker
                        let mut blocker_id = None;
                        for &creature_id in available_blockers {
                            if let Some(card_name) = view.card_name(creature_id) {
                                if Self::card_matches(&card_name, blocker_pattern) {
                                    blocker_id = Some(creature_id);
                                    break;
                                }
                            }
                        }

                        // Find matching attacker
                        let mut attacker_id = None;
                        for &creature_id in attackers {
                            if let Some(card_name) = view.card_name(creature_id) {
                                if Self::card_matches(&card_name, attacker_pattern) {
                                    attacker_id = Some(creature_id);
                                    break;
                                }
                            }
                        }

                        if let (Some(blocker), Some(attacker)) = (blocker_id, attacker_id) {
                            blocks.push((blocker, attacker));
                        }
                    }
                } else if clause == "done" {
                    break;
                }
            }

            blocks
        } else {
            // No more commands - don't block
            SmallVec::new()
        }
    }

    fn choose_damage_assignment_order(
        &mut self,
        _view: &GameStateView,
        _attacker: CardId,
        blockers: &[CardId],
    ) -> SmallVec<[CardId; 4]> {
        // Keep original order (no reordering via rich input yet)
        blockers.iter().copied().collect()
    }

    fn choose_cards_to_discard(
        &mut self,
        _view: &GameStateView,
        hand: &[CardId],
        count: usize,
    ) -> SmallVec<[CardId; 7]> {
        // Simple: discard first N cards
        // TODO: Implement rich syntax for discard selection
        hand.iter().take(count).copied().collect()
    }

    fn on_priority_passed(&mut self, _view: &GameStateView) {
        // No action needed
    }

    fn on_game_end(&mut self, _view: &GameStateView, _won: bool) {
        // No action needed
    }

    fn get_snapshot_state(&self) -> Option<serde_json::Value> {
        // Serialize the controller state
        serde_json::to_value(self).ok()
    }

    fn has_more_choices(&self) -> bool {
        self.current_index < self.commands.len()
    }
}

// Implement serialization for snapshots
impl serde::Serialize for RichInputController {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("RichInputController", 3)?;
        state.serialize_field("player_id", &self.player_id)?;
        state.serialize_field("commands", &self.commands)?;
        state.serialize_field("current_index", &self.current_index)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for RichInputController {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct RichInputControllerData {
            player_id: PlayerId,
            commands: Vec<String>,
            current_index: usize,
        }

        let data = RichInputControllerData::deserialize(deserializer)?;
        Ok(RichInputController {
            player_id: data.player_id,
            commands: data.commands,
            current_index: data.current_index,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::EntityId;
    use crate::game::GameState;

    #[test]
    fn test_normalize() {
        assert_eq!(RichInputController::normalize("Black Knight"), "blackknight");
        assert_eq!(RichInputController::normalize("Serra_Angel"), "serraangel");
        assert_eq!(
            RichInputController::normalize("Royal  Assassin"),
            "royalassassin"
        );
    }

    #[test]
    fn test_card_matches() {
        assert!(RichInputController::card_matches("Black Knight", "black"));
        assert!(RichInputController::card_matches("Black Knight", "blackkn"));
        assert!(RichInputController::card_matches(
            "Serra Angel",
            "serra"
        ));
        assert!(!RichInputController::card_matches("Black Knight", "white"));
    }

    #[test]
    fn test_numeric_choice() {
        let player_id = EntityId::new(1);
        let mut controller = RichInputController::new(player_id, vec!["1".to_string()]);
        let game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let view = GameStateView::new(&game, player_id);

        let abilities = vec![SpellAbility::PlayLand {
            card_id: EntityId::new(10),
        }];

        let choice = controller.choose_spell_ability_to_play(&view, &abilities);
        assert!(choice.is_some());
    }

    #[test]
    fn test_pass_command() {
        let player_id = EntityId::new(1);
        let mut controller = RichInputController::new(player_id, vec!["pass".to_string()]);
        let game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);
        let view = GameStateView::new(&game, player_id);

        let choice = controller.choose_spell_ability_to_play(&view, &[]);
        assert!(choice.is_none());
    }
}
