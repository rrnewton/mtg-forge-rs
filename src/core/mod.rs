//! Core game types and entities

pub mod card;
pub mod entity;
pub mod mana;
pub mod player;

pub use card::{Card, CardType};
pub use entity::{EntityId, EntityStore, GameEntity};
pub use mana::{Color, ManaCost, ManaPool};
pub use player::Player;

// Type aliases for strongly-typed entity IDs
/// Strongly-typed ID for Player entities
pub type PlayerId = EntityId<Player>;

/// Strongly-typed ID for Card entities
pub type CardId = EntityId<Card>;
