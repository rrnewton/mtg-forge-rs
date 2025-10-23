//! Cost system for activated abilities
//!
//! Represents the various costs players can pay to activate abilities,
//! such as tapping, paying mana, sacrificing permanents, etc.

use crate::core::{CardId, ManaCost};
use serde::{Deserialize, Serialize};

/// A cost that must be paid to activate an ability
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cost {
    /// Tap the permanent (T)
    Tap,

    /// Untap the permanent (Q or Untap)
    Untap,

    /// Pay mana cost
    Mana(ManaCost),

    /// Tap and pay mana (combined cost like "2 T")
    TapAndMana(ManaCost),

    /// Sacrifice a permanent
    Sacrifice { card_id: CardId },

    /// Pay life
    PayLife { amount: i32 },

    /// Discard a card
    Discard { card_id: CardId },

    /// Composite cost (multiple costs combined)
    Composite(Vec<Cost>),
}

impl Cost {
    /// Parse a cost string from card data (e.g., "T", "2 T", "PayLife<2>")
    pub fn parse(cost_str: &str) -> Option<Self> {
        let trimmed = cost_str.trim();

        // Simple tap cost
        if trimmed == "T" || trimmed == "Tap" {
            return Some(Cost::Tap);
        }

        // Simple untap cost
        if trimmed == "Q" || trimmed == "Untap" {
            return Some(Cost::Untap);
        }

        // PayLife cost (e.g., "PayLife<2>") - check before mana parsing
        if trimmed.starts_with("PayLife<") && trimmed.ends_with('>') {
            if let Some(amount_str) = trimmed
                .strip_prefix("PayLife<")
                .and_then(|s| s.strip_suffix('>'))
            {
                if let Ok(amount) = amount_str.parse::<i32>() {
                    return Some(Cost::PayLife { amount });
                }
            }
        }

        // Check for tap + mana combo (e.g., "2 T", "1 R T", "W T")
        if trimmed.contains(" T") || trimmed.contains(" Tap") {
            // Split and parse the mana part
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            let mana_parts: Vec<&str> = parts
                .iter()
                .filter(|&&p| p != "T" && p != "Tap")
                .copied()
                .collect();

            if !mana_parts.is_empty() {
                let mana_str = mana_parts.join(" ");
                let mana_cost = ManaCost::from_string(&mana_str);
                return Some(Cost::TapAndMana(mana_cost));
            }
        }

        // Pure mana cost (no tap)
        // Try to parse as mana cost if it contains numbers or color letters
        if trimmed
            .chars()
            .any(|c| c.is_ascii_digit() || "WUBRG".contains(c))
        {
            let mana_cost = ManaCost::from_string(trimmed);
            // Only treat as mana cost if it's not empty
            if mana_cost.cmc() > 0 {
                return Some(Cost::Mana(mana_cost));
            }
        }

        // If we can't parse it, return None
        None
    }

    /// Check if this cost includes a tap
    pub fn includes_tap(&self) -> bool {
        match self {
            Cost::Tap | Cost::TapAndMana(_) => true,
            Cost::Composite(costs) => costs.iter().any(|c| c.includes_tap()),
            _ => false,
        }
    }

    /// Check if this cost includes mana payment
    pub fn includes_mana(&self) -> bool {
        match self {
            Cost::Mana(_) | Cost::TapAndMana(_) => true,
            Cost::Composite(costs) => costs.iter().any(|c| c.includes_mana()),
            _ => false,
        }
    }

    /// Get the mana cost component if present
    pub fn get_mana_cost(&self) -> Option<&ManaCost> {
        match self {
            Cost::Mana(mana) | Cost::TapAndMana(mana) => Some(mana),
            Cost::Composite(costs) => costs.iter().find_map(|c| c.get_mana_cost()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tap_cost() {
        let cost = Cost::parse("T").unwrap();
        assert_eq!(cost, Cost::Tap);
        assert!(cost.includes_tap());
        assert!(!cost.includes_mana());
    }

    #[test]
    fn test_parse_tap_and_mana() {
        let cost = Cost::parse("2 T").unwrap();
        match &cost {
            Cost::TapAndMana(mana) => {
                assert_eq!(mana.generic, 2);
                assert!(mana.white == 0);
            }
            _ => panic!("Expected TapAndMana cost"),
        }
        assert!(cost.includes_tap());
        assert!(cost.includes_mana());
    }

    #[test]
    fn test_parse_colored_mana_tap() {
        let cost = Cost::parse("1 R T").unwrap();
        match &cost {
            Cost::TapAndMana(mana) => {
                assert_eq!(mana.generic, 1);
                assert_eq!(mana.red, 1);
            }
            _ => panic!("Expected TapAndMana cost"),
        }
    }

    #[test]
    fn test_parse_pure_mana() {
        let cost = Cost::parse("3").unwrap();
        match &cost {
            Cost::Mana(mana) => {
                assert_eq!(mana.generic, 3);
            }
            _ => panic!("Expected Mana cost"),
        }
        assert!(!cost.includes_tap());
        assert!(cost.includes_mana());
    }

    #[test]
    fn test_parse_pay_life() {
        let cost = Cost::parse("PayLife<2>").unwrap();
        assert_eq!(cost, Cost::PayLife { amount: 2 });
    }
}
