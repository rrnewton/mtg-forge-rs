//! Card effects and ability system

use crate::core::{CardId, PlayerId};
use serde::{Deserialize, Serialize};

/// Target reference for effects
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetRef {
    /// Target a player
    Player(PlayerId),
    /// Target a creature or other permanent
    Permanent(CardId),
    /// No target (e.g., "each player", "all creatures")
    None,
}

/// Keyword abilities in MTG
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Keyword {
    // Evergreen keywords (no parameters)
    Flying,
    FirstStrike,
    DoubleStrike,
    Deathtouch,
    Haste,
    Hexproof,
    Indestructible,
    Lifelink,
    Menace,
    Reach,
    Trample,
    Vigilance,
    Defender,

    // Protection
    ProtectionFromRed,
    ProtectionFromBlue,
    ProtectionFromBlack,
    ProtectionFromWhite,
    ProtectionFromGreen,

    // Shroud
    Shroud,

    // Keywords with parameters (stored as raw strings for now)
    /// Madness cost (e.g., "Madness:1 R")
    Madness(String),
    /// Flashback cost (e.g., "Flashback:3 R")
    Flashback(String),
    /// Enchant type (e.g., "Enchant:Creature")
    Enchant(String),

    // Commander-specific
    ChooseABackground,

    // Catch-all for other keywords
    Other(String),
}

/// Basic card effects that can be executed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    /// Deal damage to a target
    /// Example: "Lightning Bolt deals 3 damage to any target"
    DealDamage { target: TargetRef, amount: i32 },

    /// Draw cards
    /// Example: "Draw a card"
    DrawCards { player: PlayerId, count: u8 },

    /// Gain life
    /// Example: "You gain 3 life"
    GainLife { player: PlayerId, amount: i32 },

    /// Destroy a permanent
    /// Example: "Destroy target creature"
    DestroyPermanent { target: CardId },

    /// Tap a permanent
    /// Example: "Tap target creature"
    TapPermanent { target: CardId },

    /// Untap a permanent
    /// Example: "Untap target land"
    UntapPermanent { target: CardId },

    /// Pump (temporary stat boost) until end of turn
    /// Example: "Target creature gets +3/+3 until end of turn"
    PumpCreature {
        target: CardId,
        power_bonus: i32,
        toughness_bonus: i32,
    },

    /// Mill cards from library to graveyard
    /// Example: "Target player mills 3 cards"
    Mill { player: PlayerId, count: u8 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_creation() {
        let player_id = PlayerId::new(1);
        let card_id = CardId::new(100);

        let damage_effect = Effect::DealDamage {
            target: TargetRef::Player(player_id),
            amount: 3,
        };

        match damage_effect {
            Effect::DealDamage { target, amount } => {
                assert_eq!(amount, 3);
                assert_eq!(target, TargetRef::Player(player_id));
            }
            _ => panic!("Wrong effect type"),
        }

        let draw_effect = Effect::DrawCards {
            player: player_id,
            count: 2,
        };

        match draw_effect {
            Effect::DrawCards { player, count } => {
                assert_eq!(player, player_id);
                assert_eq!(count, 2);
            }
            _ => panic!("Wrong effect type"),
        }

        let destroy_effect = Effect::DestroyPermanent { target: card_id };

        match destroy_effect {
            Effect::DestroyPermanent { target } => {
                assert_eq!(target, card_id);
            }
            _ => panic!("Wrong effect type"),
        }
    }
}
