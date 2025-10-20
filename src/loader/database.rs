//! Card database for looking up card definitions
//!
//! Provides efficient lookup of card definitions by name

use crate::loader::card::{CardDefinition, CardLoader};
use crate::{MtgError, Result};
use std::collections::HashMap;
use std::path::Path;

/// Database of card definitions loaded from the cardsfolder
pub struct CardDatabase {
    cards: HashMap<String, CardDefinition>,
}

impl CardDatabase {
    /// Create an empty database
    pub fn new() -> Self {
        CardDatabase {
            cards: HashMap::new(),
        }
    }

    /// Load cards from a cardsfolder directory
    pub fn load_from_cardsfolder(cardsfolder_path: &Path) -> Result<Self> {
        let mut db = CardDatabase::new();

        // Recursively walk the cardsfolder directory
        if !cardsfolder_path.exists() {
            return Err(MtgError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Cardsfolder not found: {cardsfolder_path:?}"),
            )));
        }

        db.load_directory(cardsfolder_path)?;
        Ok(db)
    }

    /// Recursively load cards from a directory
    fn load_directory(&mut self, dir: &Path) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir).map_err(MtgError::IoError)? {
            let entry = entry.map_err(MtgError::IoError)?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively load subdirectories
                self.load_directory(&path)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("txt") {
                // Load .txt card files
                if let Ok(card_def) = CardLoader::load_from_file(&path) {
                    let name_lower = card_def.name.to_lowercase();
                    self.cards.insert(name_lower, card_def);
                }
                // Silently ignore cards that fail to parse for now
            }
        }

        Ok(())
    }

    /// Add a single card definition to the database
    pub fn add_card(&mut self, card_def: CardDefinition) {
        let name_lower = card_def.name.to_lowercase();
        self.cards.insert(name_lower, card_def);
    }

    /// Look up a card by name (case-insensitive)
    pub fn get_card(&self, name: &str) -> Option<&CardDefinition> {
        let name_lower = name.to_lowercase();
        self.cards.get(&name_lower)
    }

    /// Check if a card exists in the database
    pub fn contains(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        self.cards.contains_key(&name_lower)
    }

    /// Total number of cards in database
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// Check if database is empty
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

impl Default for CardDatabase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_database() {
        let db = CardDatabase::new();
        assert_eq!(db.len(), 0);
        assert!(db.is_empty());
        assert!(db.get_card("Lightning Bolt").is_none());
    }

    #[test]
    fn test_load_from_cardsfolder() {
        use std::path::PathBuf;

        let cardsfolder = PathBuf::from("cardsfolder");

        // Only run if cardsfolder exists
        if !cardsfolder.exists() {
            return;
        }

        let db = CardDatabase::load_from_cardsfolder(&cardsfolder).unwrap();

        // Should have loaded many cards
        assert!(!db.is_empty());

        // Check that Lightning Bolt is in the database
        assert!(db.contains("Lightning Bolt"));
        assert!(db.contains("lightning bolt")); // Case insensitive

        let bolt = db.get_card("Lightning Bolt").unwrap();
        assert_eq!(bolt.name.as_str(), "Lightning Bolt");
        assert_eq!(bolt.mana_cost.red, 1);
    }

    #[test]
    fn test_manual_add() {
        use crate::core::{CardName, CardType, Color, ManaCost};

        let mut db = CardDatabase::new();

        let def = CardDefinition {
            name: CardName::new("Test Card"),
            mana_cost: ManaCost::from_string("R"),
            types: vec![CardType::Instant],
            subtypes: vec![],
            colors: vec![Color::Red],
            power: None,
            toughness: None,
            oracle: "Test card".to_string(),
            raw_abilities: vec![],
        };

        db.add_card(def);

        assert_eq!(db.len(), 1);
        assert!(db.contains("Test Card"));
        assert!(db.contains("test card")); // Case insensitive

        let card = db.get_card("TEST CARD").unwrap();
        assert_eq!(card.name.as_str(), "Test Card");
    }
}
