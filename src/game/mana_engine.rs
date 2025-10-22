//! Mana availability computation
//!
//! This module provides efficient querying of whether a player can produce
//! enough mana to pay a given cost. It maintains cached state of available
//! mana sources partitioned into simple and complex sources.
//!
//! ## Design
//!
//! - **Simple sources**: Lands that produce a single specific color (e.g., Mountain → R)
//! - **Complex sources**: Lands with choices or conditional costs (e.g., City of Brass → any color)
//!
//! Simple sources can be cached as counters (WUBRGC). Complex sources require
//! search to determine payability.

use crate::core::{CardId, ManaCost, PlayerId};
use crate::game::GameState;

/// Maximum mana production capacity
///
/// Represents the maximum amount of mana of each color that can be produced
/// by tapping all available simple mana sources.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ManaCapacity {
    /// White mana
    pub white: u8,
    /// Blue mana
    pub blue: u8,
    /// Black mana
    pub black: u8,
    /// Red mana
    pub red: u8,
    /// Green mana
    pub green: u8,
    /// Colorless mana
    pub colorless: u8,
}

impl ManaCapacity {
    /// Create a new empty mana capacity
    pub fn new() -> Self {
        Self::default()
    }

    /// Get total mana available
    pub fn total(&self) -> u8 {
        self.white
            .saturating_add(self.blue)
            .saturating_add(self.black)
            .saturating_add(self.red)
            .saturating_add(self.green)
            .saturating_add(self.colorless)
    }

    /// Check if this capacity can pay for a mana cost
    ///
    /// For simple costs (only specific colors, no hybrid/phyrexian), this is
    /// a straightforward comparison. Returns false if the cost requires more
    /// mana of any color than we can produce.
    pub fn can_pay_simple(&self, cost: &ManaCost) -> bool {
        // Check specific color requirements
        if cost.white > self.white {
            return false;
        }
        if cost.blue > self.blue {
            return false;
        }
        if cost.black > self.black {
            return false;
        }
        if cost.red > self.red {
            return false;
        }
        if cost.green > self.green {
            return false;
        }
        if cost.colorless > self.colorless {
            return false;
        }

        // Check if we have enough total mana for generic requirement
        // Generic can be paid with any color or colorless mana
        let remaining_capacity = self
            .total()
            .saturating_sub(cost.white)
            .saturating_sub(cost.blue)
            .saturating_sub(cost.black)
            .saturating_sub(cost.red)
            .saturating_sub(cost.green)
            .saturating_sub(cost.colorless);

        remaining_capacity >= cost.generic
    }
}

/// Per-player mana engine
///
/// Maintains cached information about a player's mana-producing capabilities
/// and provides efficient queries for whether costs can be paid.
///
/// ## Usage
///
/// ```ignore
/// let mut engine = ManaEngine::new(player_id);
/// engine.update(&game); // Scan battlefield and cache mana sources
/// let can_cast = engine.can_pay(&mana_cost);
/// ```
pub struct ManaEngine {
    player_id: PlayerId,
    /// Simple mana sources (lands producing a single color)
    simple_sources: Vec<CardId>,
    /// Complex mana sources (lands with choices or conditions)
    complex_sources: Vec<CardId>,
    /// Cached capacity from simple sources
    simple_capacity: ManaCapacity,
}

impl ManaEngine {
    /// Create a new mana engine for a player
    pub fn new(player_id: PlayerId) -> Self {
        Self {
            player_id,
            simple_sources: Vec::new(),
            complex_sources: Vec::new(),
            simple_capacity: ManaCapacity::new(),
        }
    }

    /// Update the engine by scanning the battlefield for mana sources
    ///
    /// This should be called whenever:
    /// - A new permanent enters the battlefield
    /// - A permanent leaves the battlefield
    /// - A permanent becomes tapped/untapped
    pub fn update(&mut self, game: &GameState) {
        // Clear previous state
        self.simple_sources.clear();
        self.complex_sources.clear();
        self.simple_capacity = ManaCapacity::new();

        // Scan battlefield for untapped lands owned by this player
        for &card_id in &game.battlefield.cards {
            if let Ok(card) = game.cards.get(card_id) {
                // Check if this is an untapped land owned by this player
                if card.owner == self.player_id && card.is_land() && !card.tapped {
                    // Determine if this is a simple or complex mana source
                    if let Some(color) = get_simple_mana_color(card.name.as_str()) {
                        // Simple source - produces exactly one color
                        self.simple_sources.push(card_id);
                        match color {
                            'W' => self.simple_capacity.white += 1,
                            'U' => self.simple_capacity.blue += 1,
                            'B' => self.simple_capacity.black += 1,
                            'R' => self.simple_capacity.red += 1,
                            'G' => self.simple_capacity.green += 1,
                            'C' => self.simple_capacity.colorless += 1,
                            _ => {}
                        }
                    } else {
                        // Complex source - requires search
                        self.complex_sources.push(card_id);
                    }
                }
            }
        }
    }

    /// Check if the player can pay for a mana cost
    ///
    /// This considers all mana sources (simple and complex) and determines
    /// whether there exists a way to tap them to produce the required mana.
    pub fn can_pay(&self, cost: &ManaCost) -> bool {
        // Quick check: if we don't have any complex sources, use simple check
        if self.complex_sources.is_empty() {
            return self.simple_capacity.can_pay_simple(cost);
        }

        // TODO: Implement search algorithm for complex sources
        // For now, use pessimistic simple check (may reject valid combinations)
        todo!("Complex mana source handling not yet implemented. This will require a small search process to handle lands like City of Brass that can produce any color, or dual lands that produce {{R}} OR {{G}}.");
    }

    /// Get the current mana capacity from simple sources only
    pub fn simple_capacity(&self) -> ManaCapacity {
        self.simple_capacity
    }

    /// Get the list of simple mana sources
    pub fn simple_sources(&self) -> &[CardId] {
        &self.simple_sources
    }

    /// Get the list of complex mana sources
    pub fn complex_sources(&self) -> &[CardId] {
        &self.complex_sources
    }
}

/// Determine if a land is a simple mana source (produces exactly one color)
///
/// Returns the color character if it's a simple source: W, U, B, R, G, C
/// Returns None if it's a complex source or not a basic land.
fn get_simple_mana_color(land_name: &str) -> Option<char> {
    match land_name {
        "Plains" => Some('W'),
        "Island" => Some('U'),
        "Swamp" => Some('B'),
        "Mountain" => Some('R'),
        "Forest" => Some('G'),
        // Wastes produces colorless
        "Wastes" => Some('C'),
        // Any other land is considered complex for now
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Card, CardType};

    #[test]
    fn test_mana_capacity_total() {
        let capacity = ManaCapacity {
            white: 2,
            blue: 1,
            black: 0,
            red: 3,
            green: 1,
            colorless: 0,
        };
        assert_eq!(capacity.total(), 7);
    }

    #[test]
    fn test_can_pay_simple_exact() {
        let capacity = ManaCapacity {
            white: 2,
            blue: 1,
            black: 1,
            red: 2,
            green: 1,
            colorless: 0,
        };

        // Exact match
        let cost = ManaCost {
            generic: 0,
            white: 2,
            blue: 1,
            black: 1,
            red: 2,
            green: 1,
            colorless: 0,
        };
        assert!(capacity.can_pay_simple(&cost));
    }

    #[test]
    fn test_can_pay_simple_insufficient_color() {
        let capacity = ManaCapacity {
            white: 1,
            blue: 1,
            black: 1,
            red: 1,
            green: 1,
            colorless: 0,
        };

        // Requires 2 red, but we only have 1
        let cost = ManaCost {
            generic: 0,
            white: 0,
            blue: 0,
            black: 0,
            red: 2,
            green: 0,
            colorless: 0,
        };
        assert!(!capacity.can_pay_simple(&cost));
    }

    #[test]
    fn test_can_pay_simple_with_generic() {
        let capacity = ManaCapacity {
            white: 1,
            blue: 1,
            black: 1,
            red: 2,
            green: 1,
            colorless: 0,
        };

        // Cost: 1R (1 generic + 1 red)
        // We have 2 red, so 1 can be used for the red requirement
        // and we have 5 other mana for the generic
        let cost = ManaCost {
            generic: 1,
            white: 0,
            blue: 0,
            black: 0,
            red: 1,
            green: 0,
            colorless: 0,
        };
        assert!(capacity.can_pay_simple(&cost));
    }

    #[test]
    fn test_simple_mana_color_recognition() {
        assert_eq!(get_simple_mana_color("Plains"), Some('W'));
        assert_eq!(get_simple_mana_color("Island"), Some('U'));
        assert_eq!(get_simple_mana_color("Swamp"), Some('B'));
        assert_eq!(get_simple_mana_color("Mountain"), Some('R'));
        assert_eq!(get_simple_mana_color("Forest"), Some('G'));
        assert_eq!(get_simple_mana_color("Wastes"), Some('C'));
        assert_eq!(get_simple_mana_color("City of Brass"), None);
        assert_eq!(get_simple_mana_color("Taiga"), None);
    }

    #[test]
    fn test_mana_engine_update_simple_sources() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;

        // Add some basic lands to the battlefield
        let mountain_id = game.next_card_id();
        let mut mountain = Card::new(mountain_id, "Mountain".to_string(), p1_id);
        mountain.types.push(CardType::Land);
        mountain.controller = p1_id;
        game.cards.insert(mountain_id, mountain);
        game.battlefield.add(mountain_id);

        let island_id = game.next_card_id();
        let mut island = Card::new(island_id, "Island".to_string(), p1_id);
        island.types.push(CardType::Land);
        island.controller = p1_id;
        game.cards.insert(island_id, island);
        game.battlefield.add(island_id);

        // Create engine and update
        let mut engine = ManaEngine::new(p1_id);
        engine.update(&game);

        // Should have detected 2 simple sources
        assert_eq!(engine.simple_sources().len(), 2);
        assert_eq!(engine.complex_sources().len(), 0);

        // Should have correct capacity
        assert_eq!(engine.simple_capacity().red, 1);
        assert_eq!(engine.simple_capacity().blue, 1);
        assert_eq!(engine.simple_capacity().total(), 2);
    }

    #[test]
    fn test_mana_engine_can_pay() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;

        // Add 3 mountains
        for _ in 0..3 {
            let land_id = game.next_card_id();
            let mut land = Card::new(land_id, "Mountain".to_string(), p1_id);
            land.types.push(CardType::Land);
            land.controller = p1_id;
            game.cards.insert(land_id, land);
            game.battlefield.add(land_id);
        }

        let mut engine = ManaEngine::new(p1_id);
        engine.update(&game);

        // Should be able to pay for 2R (Lightning Bolt)
        let bolt_cost = ManaCost::from_string("2R");
        assert!(engine.can_pay(&bolt_cost));

        // Should not be able to pay for 4R
        let expensive_cost = ManaCost::from_string("4R");
        assert!(!engine.can_pay(&expensive_cost));

        // Should not be able to pay for 1U (requires blue)
        let blue_cost = ManaCost::from_string("1U");
        assert!(!engine.can_pay(&blue_cost));
    }
}
