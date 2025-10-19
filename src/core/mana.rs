//! Mana system for casting spells

use serde::{Deserialize, Serialize};
use std::fmt;

/// Mana colors in MTG
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
    Colorless,
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Color::White => write!(f, "W"),
            Color::Blue => write!(f, "U"),
            Color::Black => write!(f, "B"),
            Color::Red => write!(f, "R"),
            Color::Green => write!(f, "G"),
            Color::Colorless => write!(f, "C"),
        }
    }
}

/// Represents a mana cost (e.g., "2RR" = 2 generic + 2 red)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManaCost {
    pub generic: u8,
    pub white: u8,
    pub blue: u8,
    pub black: u8,
    pub red: u8,
    pub green: u8,
    pub colorless: u8,
}

impl ManaCost {
    pub fn new() -> Self {
        ManaCost {
            generic: 0,
            white: 0,
            blue: 0,
            black: 0,
            red: 0,
            green: 0,
            colorless: 0,
        }
    }

    /// Parse a mana cost string like "2RR" or "1UB"
    pub fn from_string(s: &str) -> Self {
        let mut cost = ManaCost::new();
        let mut generic_str = String::new();

        for c in s.chars() {
            match c {
                'W' => cost.white += 1,
                'U' => cost.blue += 1,
                'B' => cost.black += 1,
                'R' => cost.red += 1,
                'G' => cost.green += 1,
                'C' => cost.colorless += 1,
                '0'..='9' => generic_str.push(c),
                _ => {} // Ignore other characters
            }
        }

        if !generic_str.is_empty() {
            cost.generic = generic_str.parse().unwrap_or(0);
        }

        cost
    }

    /// Total converted mana cost
    pub fn cmc(&self) -> u8 {
        self.generic
            + self.white
            + self.blue
            + self.black
            + self.red
            + self.green
            + self.colorless
    }
}

impl Default for ManaCost {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ManaCost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.generic > 0 {
            write!(f, "{}", self.generic)?;
        }
        for _ in 0..self.white {
            write!(f, "W")?;
        }
        for _ in 0..self.blue {
            write!(f, "U")?;
        }
        for _ in 0..self.black {
            write!(f, "B")?;
        }
        for _ in 0..self.red {
            write!(f, "R")?;
        }
        for _ in 0..self.green {
            write!(f, "G")?;
        }
        for _ in 0..self.colorless {
            write!(f, "C")?;
        }
        Ok(())
    }
}

/// Mana pool for a player
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManaPool {
    pub white: u8,
    pub blue: u8,
    pub black: u8,
    pub red: u8,
    pub green: u8,
    pub colorless: u8,
}

impl ManaPool {
    pub fn new() -> Self {
        ManaPool {
            white: 0,
            blue: 0,
            black: 0,
            red: 0,
            green: 0,
            colorless: 0,
        }
    }

    pub fn add_color(&mut self, color: Color) {
        match color {
            Color::White => self.white += 1,
            Color::Blue => self.blue += 1,
            Color::Black => self.black += 1,
            Color::Red => self.red += 1,
            Color::Green => self.green += 1,
            Color::Colorless => self.colorless += 1,
        }
    }

    pub fn clear(&mut self) {
        self.white = 0;
        self.blue = 0;
        self.black = 0;
        self.red = 0;
        self.green = 0;
        self.colorless = 0;
    }

    /// Check if we can pay the given mana cost
    pub fn can_pay(&self, cost: &ManaCost) -> bool {
        // Check colored mana requirements
        if self.white < cost.white
            || self.blue < cost.blue
            || self.black < cost.black
            || self.red < cost.red
            || self.green < cost.green
            || self.colorless < cost.colorless
        {
            return false;
        }

        // Check if we have enough mana for generic cost
        let available = self.white + self.blue + self.black + self.red + self.green + self.colorless;
        let required = cost.cmc();
        available >= required
    }
}

impl Default for ManaPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mana_cost_parsing() {
        let cost = ManaCost::from_string("2RR");
        assert_eq!(cost.generic, 2);
        assert_eq!(cost.red, 2);
        assert_eq!(cost.cmc(), 4);

        let cost2 = ManaCost::from_string("1UB");
        assert_eq!(cost2.generic, 1);
        assert_eq!(cost2.blue, 1);
        assert_eq!(cost2.black, 1);
        assert_eq!(cost2.cmc(), 3);
    }

    #[test]
    fn test_mana_pool() {
        let mut pool = ManaPool::new();
        pool.add_color(Color::Red);
        pool.add_color(Color::Red);
        pool.add_color(Color::Blue);

        assert_eq!(pool.red, 2);
        assert_eq!(pool.blue, 1);

        // Can pay 1R (CMC 2) with our 3 mana
        let cost = ManaCost::from_string("1R");
        assert!(pool.can_pay(&cost));

        // Can pay 2R (CMC 3) with our 3 mana
        let cost2 = ManaCost::from_string("2R");
        assert!(pool.can_pay(&cost2));

        // Cannot pay 3R (CMC 4) with only 3 mana
        let cost3 = ManaCost::from_string("3R");
        assert!(!pool.can_pay(&cost3));

        // Cannot pay RRR (need 3 red, only have 2)
        let cost4 = ManaCost::from_string("RRR");
        assert!(!pool.can_pay(&cost4));
    }
}
