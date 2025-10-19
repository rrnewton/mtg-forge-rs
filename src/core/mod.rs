//! Core game types and entities

pub mod entity;
pub mod card;
pub mod player;
pub mod mana;

pub use entity::{EntityId, GameEntity, EntityStore};
pub use card::{Card, CardType};
pub use player::Player;
pub use mana::{ManaCost, ManaPool, Color};
