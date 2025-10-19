//! Strongly-typed wrappers for game concepts
//!
//! This module provides newtypes to prevent type confusion and make the code
//! more self-documenting. Instead of using bare Strings for different concepts,
//! we wrap them in distinct types that cannot be mixed up.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Card subtype (creature type, artifact type, land type, etc.)
///
/// Examples: "Goblin", "Warrior", "Equipment", "Island"
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Subtype(String);

impl Subtype {
    pub fn new(s: impl Into<String>) -> Self {
        Subtype(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Subtype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Subtype {
    fn from(s: String) -> Self {
        Subtype(s)
    }
}

impl From<&str> for Subtype {
    fn from(s: &str) -> Self {
        Subtype(s.to_string())
    }
}

/// Counter type (e.g., "+1/+1", "-1/-1", "charge", "loyalty")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CounterType(String);

impl CounterType {
    pub fn new(s: impl Into<String>) -> Self {
        CounterType(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    // Common counter types as constants
    pub fn plus_one_plus_one() -> Self {
        CounterType("+1/+1".to_string())
    }

    pub fn minus_one_minus_one() -> Self {
        CounterType("-1/-1".to_string())
    }

    pub fn loyalty() -> Self {
        CounterType("loyalty".to_string())
    }

    pub fn charge() -> Self {
        CounterType("charge".to_string())
    }
}

impl fmt::Display for CounterType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for CounterType {
    fn from(s: String) -> Self {
        CounterType(s)
    }
}

impl From<&str> for CounterType {
    fn from(s: &str) -> Self {
        CounterType(s.to_string())
    }
}

/// Card name (distinct from other string types)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CardName(String);

impl CardName {
    pub fn new(s: impl Into<String>) -> Self {
        CardName(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_lowercase(&self) -> String {
        self.0.to_lowercase()
    }
}

impl fmt::Display for CardName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for CardName {
    fn from(s: String) -> Self {
        CardName(s)
    }
}

impl From<&str> for CardName {
    fn from(s: &str) -> Self {
        CardName(s.to_string())
    }
}

/// Player name (distinct from other string types)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerName(String);

impl PlayerName {
    pub fn new(s: impl Into<String>) -> Self {
        PlayerName(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PlayerName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for PlayerName {
    fn from(s: String) -> Self {
        PlayerName(s)
    }
}

impl From<&str> for PlayerName {
    fn from(s: &str) -> Self {
        PlayerName(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subtype() {
        let subtype = Subtype::new("Goblin");
        assert_eq!(subtype.as_str(), "Goblin");
        assert_eq!(subtype.to_string(), "Goblin");
    }

    #[test]
    fn test_counter_type() {
        let counter = CounterType::plus_one_plus_one();
        assert_eq!(counter.as_str(), "+1/+1");

        let custom = CounterType::new("my_counter");
        assert_eq!(custom.as_str(), "my_counter");
    }

    #[test]
    fn test_card_name() {
        let name = CardName::new("Lightning Bolt");
        assert_eq!(name.as_str(), "Lightning Bolt");
    }

    #[test]
    fn test_player_name() {
        let name = PlayerName::new("Alice");
        assert_eq!(name.as_str(), "Alice");
    }
}
