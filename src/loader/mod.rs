//! Card and deck loaders
//!
//! Parsers for the Forge card format (.txt) and deck format (.dck)

pub mod card;
pub mod deck;

pub use card::CardLoader;
pub use deck::DeckLoader;
