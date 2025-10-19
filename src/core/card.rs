//! Card types and definitions

use crate::core::{CardId, CardName, Color, CounterType, Effect, GameEntity, ManaCost, PlayerId, Subtype};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

/// Card types in MTG
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardType {
    Creature,
    Instant,
    Sorcery,
    Enchantment,
    Artifact,
    Land,
    Planeswalker,
}

/// Represents a card in the game
///
/// Cards have a unique CardId but many cards can share the same card definition.
/// This struct represents the instance of a card during gameplay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    /// Unique ID for this card instance
    pub id: CardId,

    /// Card name (e.g., "Lightning Bolt")
    pub name: CardName,

    /// Mana cost
    pub mana_cost: ManaCost,

    /// Card types (a card can be multiple types)
    pub types: SmallVec<[CardType; 2]>,

    /// Card subtypes (e.g., "Goblin", "Warrior")
    pub subtypes: SmallVec<[Subtype; 2]>,

    /// Colors of the card
    pub colors: SmallVec<[Color; 2]>,

    /// Power (for creatures)
    pub power: Option<i8>,

    /// Toughness (for creatures)
    pub toughness: Option<i8>,

    /// Oracle text
    pub text: String,

    /// Current zone owner (player who owns this card)
    pub owner: PlayerId,

    /// Current controller (can differ from owner)
    pub controller: PlayerId,

    /// Is the card tapped?
    pub tapped: bool,

    /// Counters on this card (using SmallVec for efficiency)
    /// Common counters: +1/+1, -1/-1, charge, loyalty
    pub counters: SmallVec<[(CounterType, u8); 2]>,

    /// Effects that execute when this card resolves
    /// For spells: effects execute when spell resolves
    /// For permanents: effects may be triggered or activated abilities
    pub effects: Vec<Effect>,
}

impl Card {
    pub fn new(id: CardId, name: impl Into<CardName>, owner: PlayerId) -> Self {
        Card {
            id,
            name: name.into(),
            mana_cost: ManaCost::new(),
            types: SmallVec::new(),
            subtypes: SmallVec::new(),
            colors: SmallVec::new(),
            power: None,
            toughness: None,
            text: String::new(),
            owner,
            controller: owner,
            tapped: false,
            counters: SmallVec::new(),
            effects: Vec::new(),
        }
    }

    pub fn is_type(&self, card_type: &CardType) -> bool {
        self.types.contains(card_type)
    }

    pub fn is_creature(&self) -> bool {
        self.is_type(&CardType::Creature)
    }

    pub fn is_land(&self) -> bool {
        self.is_type(&CardType::Land)
    }

    pub fn tap(&mut self) {
        self.tapped = true;
    }

    pub fn untap(&mut self) {
        self.tapped = false;
    }

    pub fn add_counter(&mut self, counter_type: impl Into<CounterType>, amount: u8) {
        let counter_type = counter_type.into();
        if let Some((_, count)) = self.counters.iter_mut().find(|(t, _)| t == &counter_type) {
            *count += amount;
        } else {
            self.counters.push((counter_type, amount));
        }
    }

    pub fn get_counter(&self, counter_type: impl Into<CounterType>) -> u8 {
        let counter_type = counter_type.into();
        self.counters
            .iter()
            .find(|(t, _)| t == &counter_type)
            .map(|(_, count)| *count)
            .unwrap_or(0)
    }

    /// Get current power (including counters)
    pub fn current_power(&self) -> i8 {
        let base = self.power.unwrap_or(0);
        let plus_counters = self.get_counter("+1/+1") as i8;
        let minus_counters = self.get_counter("-1/-1") as i8;
        base + plus_counters - minus_counters
    }

    /// Get current toughness (including counters)
    pub fn current_toughness(&self) -> i8 {
        let base = self.toughness.unwrap_or(0);
        let plus_counters = self.get_counter("+1/+1") as i8;
        let minus_counters = self.get_counter("-1/-1") as i8;
        base + plus_counters - minus_counters
    }
}

impl GameEntity<Card> for Card {
    fn id(&self) -> CardId {
        self.id
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_creation() {
        let id = CardId::new(1);
        let owner = PlayerId::new(100);
        let card = Card::new(id, "Lightning Bolt", owner);

        assert_eq!(card.id, id);
        assert_eq!(card.name.as_str(), "Lightning Bolt");
        assert_eq!(card.owner, owner);
        assert_eq!(card.controller, owner);
        assert!(!card.tapped);
    }

    #[test]
    fn test_card_counters() {
        let id = CardId::new(1);
        let owner = PlayerId::new(100);
        let mut card = Card::new(id, "Test Creature", owner);

        card.power = Some(2);
        card.toughness = Some(2);

        assert_eq!(card.current_power(), 2);
        assert_eq!(card.current_toughness(), 2);

        card.add_counter("+1/+1", 2);
        assert_eq!(card.current_power(), 4);
        assert_eq!(card.current_toughness(), 4);

        card.add_counter("-1/-1", 1);
        assert_eq!(card.current_power(), 3);
        assert_eq!(card.current_toughness(), 3);
    }
}
