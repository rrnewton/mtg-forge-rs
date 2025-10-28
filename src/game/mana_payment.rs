//! Mana payment resolution system
//!
//! This module provides the interface and implementations for determining
//! how to pay mana costs using available mana sources.
//!
//! # Architecture
//!
//! The system is designed with a clean interface that allows multiple
//! implementation strategies:
//!
//! - **SimpleManaResolver**: Handles basic lands (Mountains, Islands, etc.)
//! - **GreedyManaResolver**: Java Forge-style greedy algorithm for complex sources
//! - **BacktrackingResolver**: Complete search for optimal solutions (future)
//! - **OptimalResolver**: Graph-based optimal solver (future)
//!
//! # Example
//!
//! ```ignore
//! use mtg_forge_rs::game::mana_payment::{ManaSource, ManaPaymentResolver, SimpleManaResolver};
//! use mtg_forge_rs::core::ManaCost;
//!
//! let resolver = SimpleManaResolver::new();
//! let sources = vec![/* ... */];
//! let cost = ManaCost::from_string("2R");
//!
//! if resolver.can_pay(&cost, &sources) {
//!     let tap_order = resolver.compute_tap_order(&cost, &sources).unwrap();
//!     // Use tap_order to actually tap the lands
//! }
//! ```

use crate::core::{CardId, ManaCost};

/// Represents a single mana-producing source (land or creature)
///
/// This struct captures all the information needed to determine what mana
/// a permanent can produce and under what conditions.
#[derive(Debug, Clone)]
pub struct ManaSource {
    /// The card producing the mana
    pub card_id: CardId,

    /// The type of mana this source produces
    pub production: ManaProduction,

    /// Whether this source is currently tapped
    pub is_tapped: bool,

    /// Whether this source has summoning sickness (for creatures)
    pub has_summoning_sickness: bool,
}

/// What mana a source can produce
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManaProduction {
    /// Produces exactly one specific color (e.g., Mountain → {R})
    Fixed(ManaColor),

    /// Can produce one of several colors (e.g., Taiga → {R} or {G})
    Choice(Vec<ManaColor>),

    /// Can produce any color (e.g., City of Brass)
    AnyColor,

    /// Produces colorless mana
    Colorless,
}

/// Represents a color of mana
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ManaColor {
    White,
    Blue,
    Black,
    Red,
    Green,
}

impl ManaColor {
    /// Convert to single-character representation (W, U, B, R, G)
    pub fn to_char(self) -> char {
        match self {
            ManaColor::White => 'W',
            ManaColor::Blue => 'U',
            ManaColor::Black => 'B',
            ManaColor::Red => 'R',
            ManaColor::Green => 'G',
        }
    }

    /// Parse from single-character representation
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'W' | 'w' => Some(ManaColor::White),
            'U' | 'u' => Some(ManaColor::Blue),
            'B' | 'b' => Some(ManaColor::Black),
            'R' | 'r' => Some(ManaColor::Red),
            'G' | 'g' => Some(ManaColor::Green),
            _ => None,
        }
    }
}

/// Trait for mana payment resolution strategies
///
/// Different implementations can provide different algorithms for determining
/// how to pay mana costs. The interface is kept minimal to allow flexibility.
pub trait ManaPaymentResolver {
    /// Check if a mana cost can be paid with the given sources
    ///
    /// This should be a relatively fast check that doesn't necessarily compute
    /// the full tap order.
    fn can_pay(&self, cost: &ManaCost, sources: &[ManaSource]) -> bool;

    /// Compute the actual tap order for paying a cost
    ///
    /// Returns `Some(vec)` with the CardIds to tap in order if payment is
    /// possible, or `None` if the cost cannot be paid.
    ///
    /// The returned vector should contain exactly the cards needed to pay
    /// the cost, in the order they should be tapped.
    fn compute_tap_order(&self, cost: &ManaCost, sources: &[ManaSource]) -> Option<Vec<CardId>>;
}

/// Simple resolver for basic lands only
///
/// This is the initial implementation that only handles lands that produce
/// a single fixed color (Plains, Island, Swamp, Mountain, Forest, Wastes).
///
/// This resolver uses a straightforward algorithm:
/// 1. Count available mana of each color
/// 2. Match specific color requirements first
/// 3. Use remaining sources for generic costs
pub struct SimpleManaResolver;

impl SimpleManaResolver {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleManaResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl ManaPaymentResolver for SimpleManaResolver {
    fn can_pay(&self, cost: &ManaCost, sources: &[ManaSource]) -> bool {
        // Count available mana by color (only untapped, fixed-color sources)
        let mut white = 0u8;
        let mut blue = 0u8;
        let mut black = 0u8;
        let mut red = 0u8;
        let mut green = 0u8;
        let mut colorless = 0u8;

        for source in sources {
            if source.is_tapped || source.has_summoning_sickness {
                continue;
            }

            match &source.production {
                ManaProduction::Fixed(color) => match color {
                    ManaColor::White => white += 1,
                    ManaColor::Blue => blue += 1,
                    ManaColor::Black => black += 1,
                    ManaColor::Red => red += 1,
                    ManaColor::Green => green += 1,
                },
                ManaProduction::Colorless => colorless += 1,
                _ => {
                    // SimpleManaResolver doesn't handle complex sources
                    // If we encounter any, we conservatively return false
                    return false;
                }
            }
        }

        // Check specific color requirements
        if cost.white > white {
            return false;
        }
        if cost.blue > blue {
            return false;
        }
        if cost.black > black {
            return false;
        }
        if cost.red > red {
            return false;
        }
        if cost.green > green {
            return false;
        }
        if cost.colorless > colorless {
            return false;
        }

        // Check if we have enough total mana for generic requirement
        let total = white
            .saturating_add(blue)
            .saturating_add(black)
            .saturating_add(red)
            .saturating_add(green)
            .saturating_add(colorless);

        let used = cost
            .white
            .saturating_add(cost.blue)
            .saturating_add(cost.black)
            .saturating_add(cost.red)
            .saturating_add(cost.green)
            .saturating_add(cost.colorless);

        let remaining = total.saturating_sub(used);

        remaining >= cost.generic
    }

    fn compute_tap_order(&self, cost: &ManaCost, sources: &[ManaSource]) -> Option<Vec<CardId>> {
        // First check if payment is possible
        if !self.can_pay(cost, sources) {
            return None;
        }

        let mut tap_order = Vec::new();
        let mut remaining_cost = *cost;

        // Helper to tap sources of a specific color
        let mut tap_color = |color: ManaColor, amount: u8, sources: &[ManaSource]| {
            let mut tapped = 0;
            for source in sources {
                if tapped >= amount {
                    break;
                }
                if source.is_tapped
                    || source.has_summoning_sickness
                    || tap_order.contains(&source.card_id)
                {
                    continue;
                }
                if let ManaProduction::Fixed(c) = source.production {
                    if c == color {
                        tap_order.push(source.card_id);
                        tapped += 1;
                    }
                }
            }
        };

        // Tap sources for specific color requirements first
        tap_color(ManaColor::White, remaining_cost.white, sources);
        remaining_cost.white = 0;

        tap_color(ManaColor::Blue, remaining_cost.blue, sources);
        remaining_cost.blue = 0;

        tap_color(ManaColor::Black, remaining_cost.black, sources);
        remaining_cost.black = 0;

        tap_color(ManaColor::Red, remaining_cost.red, sources);
        remaining_cost.red = 0;

        tap_color(ManaColor::Green, remaining_cost.green, sources);
        remaining_cost.green = 0;

        // Tap colorless sources for colorless requirement
        let mut tapped_colorless = 0;
        for source in sources {
            if tapped_colorless >= remaining_cost.colorless {
                break;
            }
            if source.is_tapped
                || source.has_summoning_sickness
                || tap_order.contains(&source.card_id)
            {
                continue;
            }
            if source.production == ManaProduction::Colorless {
                tap_order.push(source.card_id);
                tapped_colorless += 1;
            }
        }
        remaining_cost.colorless = 0;

        // Tap any remaining sources for generic cost
        let mut tapped_generic = 0;
        for source in sources {
            if tapped_generic >= remaining_cost.generic {
                break;
            }
            if source.is_tapped
                || source.has_summoning_sickness
                || tap_order.contains(&source.card_id)
            {
                continue;
            }
            // Can use any untapped source for generic
            match source.production {
                ManaProduction::Fixed(_) | ManaProduction::Colorless => {
                    tap_order.push(source.card_id);
                    tapped_generic += 1;
                }
                _ => {} // Skip complex sources
            }
        }

        Some(tap_order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mana_color_conversion() {
        assert_eq!(ManaColor::White.to_char(), 'W');
        assert_eq!(ManaColor::Blue.to_char(), 'U');
        assert_eq!(ManaColor::from_char('R'), Some(ManaColor::Red));
        assert_eq!(ManaColor::from_char('g'), Some(ManaColor::Green));
        assert_eq!(ManaColor::from_char('X'), None);
    }

    #[test]
    fn test_simple_resolver_exact_match() {
        let resolver = SimpleManaResolver::new();

        let sources = vec![
            ManaSource {
                card_id: CardId::new(1),
                production: ManaProduction::Fixed(ManaColor::Red),
                is_tapped: false,
                has_summoning_sickness: false,
            },
            ManaSource {
                card_id: CardId::new(2),
                production: ManaProduction::Fixed(ManaColor::Red),
                is_tapped: false,
                has_summoning_sickness: false,
            },
            ManaSource {
                card_id: CardId::new(3),
                production: ManaProduction::Fixed(ManaColor::Red),
                is_tapped: false,
                has_summoning_sickness: false,
            },
        ];

        // Cost: 2R requires 1 red + 2 generic (can pay with 3 red)
        let cost = ManaCost {
            generic: 2,
            white: 0,
            blue: 0,
            black: 0,
            red: 1,
            green: 0,
            colorless: 0,
        };

        assert!(resolver.can_pay(&cost, &sources));

        let tap_order = resolver.compute_tap_order(&cost, &sources).unwrap();
        assert_eq!(tap_order.len(), 3); // Should tap all 3 mountains
    }

    #[test]
    fn test_simple_resolver_insufficient_color() {
        let resolver = SimpleManaResolver::new();

        let sources = vec![ManaSource {
            card_id: CardId::new(1),
            production: ManaProduction::Fixed(ManaColor::Red),
            is_tapped: false,
            has_summoning_sickness: false,
        }];

        // Cost: 1U requires blue mana
        let cost = ManaCost {
            generic: 1,
            white: 0,
            blue: 1,
            black: 0,
            red: 0,
            green: 0,
            colorless: 0,
        };

        assert!(!resolver.can_pay(&cost, &sources));
        assert!(resolver.compute_tap_order(&cost, &sources).is_none());
    }

    #[test]
    fn test_simple_resolver_rejects_complex_sources() {
        let resolver = SimpleManaResolver::new();

        let sources = vec![
            ManaSource {
                card_id: CardId::new(1),
                production: ManaProduction::Fixed(ManaColor::Red),
                is_tapped: false,
                has_summoning_sickness: false,
            },
            ManaSource {
                card_id: CardId::new(2),
                production: ManaProduction::AnyColor,
                is_tapped: false,
                has_summoning_sickness: false,
            },
        ];

        let cost = ManaCost {
            generic: 1,
            white: 0,
            blue: 0,
            black: 0,
            red: 0,
            green: 0,
            colorless: 0,
        };

        // SimpleManaResolver conservatively rejects when complex sources present
        assert!(!resolver.can_pay(&cost, &sources));
    }
}
