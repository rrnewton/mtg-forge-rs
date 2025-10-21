//! Async deck loading utilities
//!
//! Helpers for loading decks with async card database

use crate::loader::database_async::CardDatabase as AsyncCardDatabase;
use crate::loader::deck::DeckList;
use crate::Result;
use std::collections::HashSet;
use std::time::Duration;

/// Load unique cards from a deck in parallel
/// Returns (cards_loaded, duration)
pub async fn load_deck_cards(
    db: &AsyncCardDatabase,
    deck: &DeckList,
) -> Result<(usize, Duration)> {
    // Collect unique card names
    let mut unique_names = HashSet::new();
    for entry in &deck.main_deck {
        unique_names.insert(entry.card_name.clone());
    }
    for entry in &deck.sideboard {
        unique_names.insert(entry.card_name.clone());
    }

    let names: Vec<String> = unique_names.into_iter().collect();
    db.load_cards(&names).await
}
