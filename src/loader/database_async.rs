//! Async card database for on-demand and eager loading
//!
//! Supports two loading modes:
//! 1. Lazy loading: Load cards on-demand when requested (parallel I/O)
//! 2. Eager loading: Load all cards upfront from cardsfolder (parallel I/O)

use crate::loader::card::{CardDefinition, CardLoader};
use crate::{MtgError, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Instant;

/// Convert card name to file path
/// "Lightning Bolt" -> "cardsfolder/l/lightning_bolt.txt"
fn card_name_to_path(cardsfolder: &Path, card_name: &str) -> PathBuf {
    let normalized = card_name.to_lowercase().replace(' ', "_");
    let first_char = normalized.chars().next().unwrap_or('_');

    cardsfolder
        .join(first_char.to_string())
        .join(format!("{}.txt", normalized))
}

/// Async card database with lazy and eager loading support
pub struct CardDatabase {
    /// Base directory containing card files
    cardsfolder: PathBuf,
    /// Cache of loaded cards (shared, thread-safe)
    cards: Arc<RwLock<HashMap<String, CardDefinition>>>,
}

impl CardDatabase {
    /// Create a new database pointing at a cardsfolder
    pub fn new(cardsfolder: PathBuf) -> Self {
        CardDatabase {
            cardsfolder,
            cards: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load a single card by name (async, with caching)
    /// Returns None if card file doesn't exist
    pub async fn get_card(&self, name: &str) -> Result<Option<CardDefinition>> {
        let name_lower = name.to_lowercase();

        // Check cache first
        {
            let cards = self.cards.read().await;
            if let Some(card) = cards.get(&name_lower) {
                return Ok(Some(card.clone()));
            }
        }

        // Not in cache, try to load from disk
        let path = card_name_to_path(&self.cardsfolder, name);

        if !path.exists() {
            return Ok(None);
        }

        // Load asynchronously
        match Self::load_card_async(path).await {
            Ok(card_def) => {
                // Cache the loaded card
                let mut cards = self.cards.write().await;
                cards.insert(name_lower, card_def.clone());
                Ok(Some(card_def))
            }
            Err(e) => {
                // Card file exists but failed to parse
                eprintln!("Warning: Failed to parse card {}: {}", name, e);
                Ok(None)
            }
        }
    }

    /// Load multiple cards in parallel
    /// Returns timing information
    pub async fn load_cards(&self, names: &[String]) -> Result<(usize, std::time::Duration)> {
        let start = Instant::now();

        // Spawn tasks for all cards in parallel
        let mut tasks = Vec::new();
        for name in names {
            let name = name.clone();
            let db = self.clone_handle();
            tasks.push(tokio::spawn(async move {
                db.get_card(&name).await
            }));
        }

        // Wait for all to complete
        let mut loaded = 0;
        for task in tasks {
            if let Ok(Ok(Some(_))) = task.await {
                loaded += 1;
            }
        }

        let duration = start.elapsed();
        Ok((loaded, duration.into()))
    }

    /// Eagerly load all cards from cardsfolder (parallel)
    /// Returns (cards_loaded, duration)
    pub async fn eager_load(&self) -> Result<(usize, std::time::Duration)> {
        let start = Instant::now();

        // Recursively collect all .txt file paths
        let paths = Self::collect_card_paths(&self.cardsfolder)?;

        println!("Found {} card files, loading in parallel...", paths.len());

        // Load all cards in parallel
        let mut tasks = Vec::new();
        for path in paths {
            tasks.push(tokio::spawn(Self::load_card_async(path)));
        }

        // Collect results
        let mut cards_map = HashMap::new();
        for task in tasks {
            if let Ok(Ok(card_def)) = task.await {
                let name_lower = card_def.name.to_lowercase();
                cards_map.insert(name_lower, card_def);
            }
        }

        let count = cards_map.len();

        // Update cache
        let mut cards = self.cards.write().await;
        *cards = cards_map;

        let duration = start.elapsed();
        Ok((count, duration.into()))
    }

    /// Load a card from a file asynchronously
    async fn load_card_async(path: PathBuf) -> Result<CardDefinition> {
        let contents = tokio::fs::read_to_string(&path)
            .await
            .map_err(MtgError::IoError)?;

        CardLoader::parse(&contents)
    }

    /// Recursively collect all .txt file paths
    fn collect_card_paths(dir: &Path) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();
        Self::collect_card_paths_recursive(dir, &mut paths)?;
        Ok(paths)
    }

    fn collect_card_paths_recursive(dir: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir).map_err(MtgError::IoError)? {
            let entry = entry.map_err(MtgError::IoError)?;
            let path = entry.path();

            if path.is_dir() {
                Self::collect_card_paths_recursive(&path, paths)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("txt") {
                paths.push(path);
            }
        }

        Ok(())
    }

    /// Get a clone of the database handle (shares the cache)
    fn clone_handle(&self) -> Self {
        CardDatabase {
            cardsfolder: self.cardsfolder.clone(),
            cards: Arc::clone(&self.cards),
        }
    }

    /// Synchronous check if card exists in cache
    pub async fn contains(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        let cards = self.cards.read().await;
        cards.contains_key(&name_lower)
    }

    /// Get number of cards currently loaded
    pub async fn len(&self) -> usize {
        let cards = self.cards.read().await;
        cards.len()
    }

    /// Check if database is empty
    pub async fn is_empty(&self) -> bool {
        let cards = self.cards.read().await;
        cards.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_card_name_to_path() {
        let cardsfolder = PathBuf::from("cardsfolder");

        let path = card_name_to_path(&cardsfolder, "Lightning Bolt");
        assert_eq!(path, PathBuf::from("cardsfolder/l/lightning_bolt.txt"));

        let path = card_name_to_path(&cardsfolder, "Black Lotus");
        assert_eq!(path, PathBuf::from("cardsfolder/b/black_lotus.txt"));
    }

    #[tokio::test]
    async fn test_lazy_loading() {
        let cardsfolder = PathBuf::from("cardsfolder");
        if !cardsfolder.exists() {
            return;
        }

        let db = CardDatabase::new(cardsfolder);

        // Should start empty
        assert!(db.is_empty().await);

        // Load a card
        let card = db.get_card("Lightning Bolt").await.unwrap();
        assert!(card.is_some());

        // Should now have 1 card
        assert_eq!(db.len().await, 1);

        // Loading again should hit cache
        let card2 = db.get_card("Lightning Bolt").await.unwrap();
        assert!(card2.is_some());

        // Still only 1 card (hit cache)
        assert_eq!(db.len().await, 1);
    }

    #[tokio::test]
    async fn test_parallel_loading() {
        let cardsfolder = PathBuf::from("cardsfolder");
        if !cardsfolder.exists() {
            return;
        }

        let db = CardDatabase::new(cardsfolder);

        let cards = vec![
            "Lightning Bolt".to_string(),
            "Mountain".to_string(),
            "Forest".to_string(),
        ];

        let (loaded, duration) = db.load_cards(&cards).await.unwrap();
        assert_eq!(loaded, 3);
        println!("Loaded {} cards in {:?}", loaded, duration);
    }
}
