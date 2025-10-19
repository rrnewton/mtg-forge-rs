//! Core game types and entities

pub mod card;
pub mod effects;
pub mod entity;
pub mod mana;
pub mod player;
pub mod types;

pub use card::{Card, CardType};
pub use effects::{Effect, TargetRef};
pub use entity::{EntityId, EntityStore, GameEntity};
pub use mana::{Color, ManaCost, ManaPool};
pub use player::Player;
pub use types::{CardName, CounterType, PlayerName, Subtype};

// Type aliases for strongly-typed entity IDs
/// Strongly-typed ID for Player entities
pub type PlayerId = EntityId<Player>;

/// Strongly-typed ID for Card entities
pub type CardId = EntityId<Card>;
