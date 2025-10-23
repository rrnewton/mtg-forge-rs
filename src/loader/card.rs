//! Card file loader (.txt format)
//!
//! Loads card definitions from Forge's cardsfolder format

use crate::core::{
    Card, CardName, CardType, Color, Keyword, ManaCost, Subtype, Trigger, TriggerEvent,
};
use crate::{MtgError, Result};
use smallvec::SmallVec;
use std::fs;
use std::path::Path;

/// Card loader for .txt files
pub struct CardLoader;

impl CardLoader {
    /// Load a card from a .txt file
    pub fn load_from_file(path: &Path) -> Result<CardDefinition> {
        let content = fs::read_to_string(path).map_err(MtgError::IoError)?;
        Self::parse(&content)
    }

    /// Parse a card from its text content
    pub fn parse(content: &str) -> Result<CardDefinition> {
        let mut name = None;
        let mut mana_cost = ManaCost::new();
        let mut types = Vec::new();
        let mut subtypes = Vec::new();
        let mut colors = Vec::new();
        let mut power = None;
        let mut toughness = None;
        let mut oracle = String::new();
        let mut raw_abilities = Vec::new();
        let mut raw_keywords = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();

                match key {
                    "Name" => name = Some(CardName::new(value)),
                    "ManaCost" => mana_cost = ManaCost::from_string(value),
                    "Types" => {
                        for part in value.split_whitespace() {
                            match part {
                                "Creature" => types.push(CardType::Creature),
                                "Instant" => types.push(CardType::Instant),
                                "Sorcery" => types.push(CardType::Sorcery),
                                "Enchantment" => types.push(CardType::Enchantment),
                                "Artifact" => types.push(CardType::Artifact),
                                "Land" => types.push(CardType::Land),
                                "Planeswalker" => types.push(CardType::Planeswalker),
                                _ => subtypes.push(Subtype::new(part)),
                            }
                        }
                    }
                    "PT" => {
                        if let Some((p, t)) = value.split_once('/') {
                            power = p.trim().parse().ok();
                            toughness = t.trim().parse().ok();
                        }
                    }
                    "Oracle" => oracle = value.to_string(),
                    // Keyword lines (K:)
                    "K" => {
                        raw_keywords.push(value.to_string());
                    }
                    // Ability lines (A:, S:, T:, etc.)
                    "A" | "S" | "T" => {
                        raw_abilities.push(format!("{key}:{value}"));
                    }
                    _ => {} // Ignore other fields for now
                }
            }
        }

        // Derive colors from mana cost
        if mana_cost.white > 0 {
            colors.push(Color::White);
        }
        if mana_cost.blue > 0 {
            colors.push(Color::Blue);
        }
        if mana_cost.black > 0 {
            colors.push(Color::Black);
        }
        if mana_cost.red > 0 {
            colors.push(Color::Red);
        }
        if mana_cost.green > 0 {
            colors.push(Color::Green);
        }
        if colors.is_empty() {
            colors.push(Color::Colorless);
        }

        let name = name.ok_or(MtgError::InvalidCardFormat("Missing card name".to_string()))?;

        Ok(CardDefinition {
            name,
            mana_cost,
            types,
            subtypes,
            colors,
            power,
            toughness,
            oracle,
            raw_abilities,
            raw_keywords,
        })
    }
}

/// Card definition (not yet instantiated in a game)
#[derive(Debug, Clone)]
pub struct CardDefinition {
    pub name: CardName,
    pub mana_cost: ManaCost,
    pub types: Vec<CardType>,
    pub subtypes: Vec<Subtype>,
    pub colors: Vec<Color>,
    pub power: Option<i8>,
    pub toughness: Option<i8>,
    pub oracle: String,
    /// Raw ability scripts from the card file (A:, S:, T: lines)
    /// We'll parse these into actual effects later
    pub raw_abilities: Vec<String>,
    /// Raw keyword scripts from the card file (K: lines)
    pub raw_keywords: Vec<String>,
}

impl CardDefinition {
    /// Create a Card instance from this definition
    pub fn instantiate(&self, id: crate::core::CardId, owner: crate::core::PlayerId) -> Card {
        let mut card = Card::new(id, self.name.clone(), owner);
        card.mana_cost = self.mana_cost.clone();
        card.types = SmallVec::from_vec(self.types.clone());
        card.subtypes = SmallVec::from_vec(self.subtypes.clone());
        card.colors = SmallVec::from_vec(self.colors.clone());
        card.power = self.power;
        card.toughness = self.toughness;
        card.text = self.oracle.clone();

        // Parse keywords
        card.keywords = self.parse_keywords();

        // Parse abilities into effects (simplified parser for common cases)
        card.effects = self.parse_effects();

        // Parse triggered abilities
        card.triggers = self.parse_triggers();

        card
    }

    /// Parse raw keywords into Keyword objects
    fn parse_keywords(&self) -> Vec<Keyword> {
        let mut keywords = Vec::new();

        for keyword_str in &self.raw_keywords {
            // Check if keyword has a parameter (colon separated)
            if let Some((kw, param)) = keyword_str.split_once(':') {
                let kw = kw.trim();
                let param = param.trim();

                // Keywords with parameters
                let keyword = match kw {
                    "Madness" => Keyword::Madness(param.to_string()),
                    "Flashback" => Keyword::Flashback(param.to_string()),
                    "Enchant" => Keyword::Enchant(param.to_string()),
                    _ => Keyword::Other(keyword_str.clone()),
                };
                keywords.push(keyword);
            } else {
                // Simple keywords (no parameters)
                let kw = keyword_str.trim();
                let keyword = match kw {
                    "Flying" => Keyword::Flying,
                    "First Strike" => Keyword::FirstStrike,
                    "Double Strike" => Keyword::DoubleStrike,
                    "Deathtouch" => Keyword::Deathtouch,
                    "Haste" => Keyword::Haste,
                    "Hexproof" => Keyword::Hexproof,
                    "Indestructible" => Keyword::Indestructible,
                    "Lifelink" => Keyword::Lifelink,
                    "Menace" => Keyword::Menace,
                    "Reach" => Keyword::Reach,
                    "Trample" => Keyword::Trample,
                    "Vigilance" => Keyword::Vigilance,
                    "Defender" => Keyword::Defender,
                    "Shroud" => Keyword::Shroud,
                    "Choose a Background" => Keyword::ChooseABackground,
                    // Protection variants
                    "Protection from red" => Keyword::ProtectionFromRed,
                    "Protection from blue" => Keyword::ProtectionFromBlue,
                    "Protection from black" => Keyword::ProtectionFromBlack,
                    "Protection from white" => Keyword::ProtectionFromWhite,
                    "Protection from green" => Keyword::ProtectionFromGreen,
                    _ => Keyword::Other(keyword_str.clone()),
                };
                keywords.push(keyword);
            }
        }

        keywords
    }

    /// Parse raw abilities into Effect objects (simplified)
    /// This is a temporary solution until we have a full ability parser
    fn parse_effects(&self) -> Vec<crate::core::Effect> {
        use crate::core::{Effect, PlayerId, TargetRef};

        let mut effects = Vec::new();

        for ability in &self.raw_abilities {
            // Parse DealDamage abilities
            // Format: "A:SP$ DealDamage | ValidTgts$ Any | NumDmg$ 3 | ..."
            if ability.contains("DealDamage") {
                // Extract damage amount
                if let Some(dmg_str) = ability.split("NumDmg$").nth(1) {
                    if let Some(dmg_part) = dmg_str.trim().split(['|', ' ']).next() {
                        if let Ok(amount) = dmg_part.trim().parse::<i32>() {
                            // For now, use TargetRef::None - targeting will be filled in at cast time
                            effects.push(Effect::DealDamage {
                                target: TargetRef::None,
                                amount,
                            });
                        }
                    }
                }
            }

            // Parse Draw abilities
            // Format: "A:SP$ Draw | NumCards$ 3 | ValidTgts$ Player | ..."
            // Format: "A:SP$ Draw | NumCards$ 1 | Defined$ You | ..." (draw yourself)
            if ability.contains("SP$ Draw") {
                // Extract number of cards to draw
                if let Some(cards_str) = ability.split("NumCards$").nth(1) {
                    if let Some(cards_part) = cards_str.trim().split(['|', ' ']).next() {
                        if let Ok(count) = cards_part.trim().parse::<u8>() {
                            // For now, use a placeholder player ID - will be filled in at cast time
                            // Check if it targets a player or is self-draw
                            effects.push(Effect::DrawCards {
                                player: PlayerId::new(0), // Placeholder, will be set during resolution
                                count,
                            });
                        }
                    }
                }
            }

            // Parse Destroy abilities
            // Format: "A:SP$ Destroy | ValidTgts$ Creature | ..."
            // Format: "A:SP$ Destroy | ValidTgts$ Creature.nonArtifact+nonBlack | ..."
            if ability.contains("SP$ Destroy") {
                // Destroy effects target a permanent
                // Use placeholder card ID 0 - will be filled in at cast time
                use crate::core::CardId;
                effects.push(Effect::DestroyPermanent {
                    target: CardId::new(0), // Placeholder, will be set during resolution
                });
            }

            // Parse GainLife abilities
            // Format: "A:SP$ GainLife | LifeAmount$ 7 | ..."
            // Format: "DB$ GainLife | ValidTgts$ Player | LifeAmount$ 3 | ..." (with targeting)
            if ability.contains("GainLife") {
                // Extract life amount
                if let Some(life_str) = ability.split("LifeAmount$").nth(1) {
                    if let Some(life_part) = life_str.trim().split(['|', ' ']).next() {
                        if let Ok(amount) = life_part.trim().parse::<i32>() {
                            // Use placeholder player ID 0 - will be filled in at cast time
                            effects.push(Effect::GainLife {
                                player: PlayerId::new(0), // Placeholder, will be set during resolution
                                amount,
                            });
                        }
                    }
                }
            }

            // Parse Pump abilities
            // Format: "A:SP$ Pump | ValidTgts$ Creature | NumAtt$ +3 | NumDef$ +3 | ..."
            if ability.contains("SP$ Pump") {
                let mut power_bonus = 0;
                let mut toughness_bonus = 0;

                // Extract power bonus (NumAtt$)
                if let Some(att_str) = ability.split("NumAtt$").nth(1) {
                    if let Some(att_part) = att_str.trim().split(['|', ' ']).next() {
                        // Handle +X or -X format
                        let att_trimmed = att_part.trim().trim_start_matches('+');
                        if let Ok(bonus) = att_trimmed.parse::<i32>() {
                            power_bonus = bonus;
                        }
                    }
                }

                // Extract toughness bonus (NumDef$)
                if let Some(def_str) = ability.split("NumDef$").nth(1) {
                    if let Some(def_part) = def_str.trim().split(['|', ' ']).next() {
                        // Handle +X or -X format
                        let def_trimmed = def_part.trim().trim_start_matches('+');
                        if let Ok(bonus) = def_trimmed.parse::<i32>() {
                            toughness_bonus = bonus;
                        }
                    }
                }

                // Only add the effect if we successfully parsed at least one bonus
                if power_bonus != 0 || toughness_bonus != 0 {
                    use crate::core::CardId;
                    effects.push(Effect::PumpCreature {
                        target: CardId::new(0), // Placeholder, will be set during resolution
                        power_bonus,
                        toughness_bonus,
                    });
                }
            }

            // Parse Tap abilities
            // Format: "A:SP$ Tap | ValidTgts$ Creature | ..."
            if ability.contains("SP$ Tap") && !ability.contains("TapAll") {
                use crate::core::CardId;
                effects.push(Effect::TapPermanent {
                    target: CardId::new(0), // Placeholder, will be set during resolution
                });
            }

            // Parse Untap abilities
            // Format: "A:SP$ Untap | ValidTgts$ Land | ..."
            if ability.contains("SP$ Untap") {
                use crate::core::CardId;
                effects.push(Effect::UntapPermanent {
                    target: CardId::new(0), // Placeholder, will be set during resolution
                });
            }

            // Parse Mill abilities
            // Format: "A:SP$ Mill | NumCards$ 3 | ValidTgts$ Player | ..."
            if ability.contains("SP$ Mill") {
                if let Some(cards_str) = ability.split("NumCards$").nth(1) {
                    if let Some(cards_part) = cards_str.trim().split(['|', ' ']).next() {
                        if let Ok(count) = cards_part.trim().parse::<u8>() {
                            use crate::core::PlayerId;
                            effects.push(Effect::Mill {
                                player: PlayerId::new(0), // Placeholder, will be set during resolution
                                count,
                            });
                        }
                    }
                }
            }
        }

        effects
    }

    /// Parse triggered abilities (T: lines)
    /// This is a simplified parser for common ETB triggers
    fn parse_triggers(&self) -> Vec<Trigger> {
        use crate::core::{Effect, PlayerId};

        let mut triggers = Vec::new();

        for ability in &self.raw_abilities {
            // Only process T: lines (triggered abilities)
            if !ability.starts_with("T:") {
                continue;
            }

            // Parse ETB triggers
            // Format: "T:Mode$ ChangesZone | Origin$ Any | Destination$ Battlefield | ValidCard$ Card.Self | Execute$ TrigDraw | TriggerDescription$ When..."
            if ability.contains("Mode$ ChangesZone")
                && ability.contains("Destination$ Battlefield")
                && ability.contains("ValidCard$ Card.Self")
            {
                // Extract the Execute$ parameter to determine what effects to apply
                let mut effects = Vec::new();

                // Check if this is a draw trigger
                if ability.contains("Execute$ TrigDraw") || ability.contains("Draw") {
                    // Look for NumCards in the ability string
                    // For now, default to drawing 1 card if we can't find the number
                    let count = if let Some(cards_str) = ability.split("NumCards$").nth(1) {
                        cards_str
                            .trim()
                            .split(['|', ' ', '\n'])
                            .next()
                            .and_then(|s| s.trim().parse::<u8>().ok())
                            .unwrap_or(1)
                    } else {
                        1
                    };

                    effects.push(Effect::DrawCards {
                        player: PlayerId::new(0), // Placeholder - will be filled when triggered
                        count,
                    });
                }

                // Check if this is a damage trigger
                if ability.contains("Execute$ TrigDealDamage") || ability.contains("DealDamage") {
                    // Look for NumDmg in the ability string
                    let amount = if let Some(dmg_str) = ability.split("NumDmg$").nth(1) {
                        dmg_str
                            .trim()
                            .split(['|', ' ', '\n'])
                            .next()
                            .and_then(|s| s.trim().parse::<i32>().ok())
                            .unwrap_or(1)
                    } else {
                        1
                    };

                    effects.push(Effect::DealDamage {
                        target: crate::core::TargetRef::None, // Will be filled when triggered
                        amount,
                    });
                }

                // Check if this is a gain life trigger
                if ability.contains("Execute$ TrigGainLife")
                    || (ability.contains("GainLife") && !ability.contains("DealDamage"))
                {
                    // Look for LifeAmount in the ability string
                    let amount = if let Some(life_str) = ability.split("LifeAmount$").nth(1) {
                        life_str
                            .trim()
                            .split(['|', ' ', '\n'])
                            .next()
                            .and_then(|s| s.trim().parse::<i32>().ok())
                            .unwrap_or(1)
                    } else {
                        1
                    };

                    effects.push(Effect::GainLife {
                        player: PlayerId::new(0), // Placeholder - will be filled when triggered
                        amount,
                    });
                }

                // Check if this is a destroy trigger
                if ability.contains("Execute$ TrigDestroy") || ability.contains("Destroy") {
                    use crate::core::CardId;
                    effects.push(Effect::DestroyPermanent {
                        target: CardId::new(0), // Placeholder - will be filled when triggered
                    });
                }

                // Check if this is a pump trigger
                if ability.contains("Execute$ TrigPump") || ability.contains("Pump") {
                    // Look for NumAtt and NumDef in the ability string
                    let power_bonus = if let Some(att_str) = ability.split("NumAtt$").nth(1) {
                        att_str
                            .trim()
                            .split(['|', ' ', '\n'])
                            .next()
                            .and_then(|s| {
                                s.trim()
                                    .strip_prefix('+')
                                    .unwrap_or(s.trim())
                                    .parse::<i32>()
                                    .ok()
                            })
                            .unwrap_or(0)
                    } else {
                        0
                    };

                    let toughness_bonus = if let Some(def_str) = ability.split("NumDef$").nth(1) {
                        def_str
                            .trim()
                            .split(['|', ' ', '\n'])
                            .next()
                            .and_then(|s| {
                                s.trim()
                                    .strip_prefix('+')
                                    .unwrap_or(s.trim())
                                    .parse::<i32>()
                                    .ok()
                            })
                            .unwrap_or(0)
                    } else {
                        0
                    };

                    if power_bonus != 0 || toughness_bonus != 0 {
                        use crate::core::CardId;
                        effects.push(Effect::PumpCreature {
                            target: CardId::new(0), // Placeholder - will be filled when triggered
                            power_bonus,
                            toughness_bonus,
                        });
                    }
                }

                if !effects.is_empty() {
                    // Extract description from TriggerDescription$ if available
                    let description =
                        if let Some(desc_str) = ability.split("TriggerDescription$").nth(1) {
                            desc_str.trim().to_string()
                        } else {
                            "When this enters the battlefield".to_string()
                        };

                    triggers.push(Trigger::new(
                        TriggerEvent::EntersBattlefield,
                        effects,
                        description,
                    ));
                }
            }
        }

        triggers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lightning_bolt() {
        let content = r#"
Name:Lightning Bolt
ManaCost:R
Types:Instant
A:SP$ DealDamage | ValidTgts$ Any | NumDmg$ 3 | SpellDescription$ CARDNAME deals 3 damage to any target.
Oracle:Lightning Bolt deals 3 damage to any target.
"#;

        let def = CardLoader::parse(content).unwrap();
        assert_eq!(def.name.as_str(), "Lightning Bolt");
        assert_eq!(def.mana_cost.red, 1);
        assert_eq!(def.types.len(), 1);
        assert!(def.types.contains(&CardType::Instant));
        assert!(def.colors.contains(&Color::Red));

        // Check that the effect is parsed
        let effects = def.parse_effects();
        assert_eq!(effects.len(), 1, "Lightning Bolt should have 1 effect");

        use crate::core::{Effect, TargetRef};
        match &effects[0] {
            Effect::DealDamage { target, amount } => {
                assert_eq!(*amount, 3, "Should deal 3 damage");
                assert!(
                    matches!(target, TargetRef::None),
                    "Target should be None initially"
                );
            }
            _ => panic!("Expected DealDamage effect"),
        }
    }

    #[test]
    fn test_parse_creature() {
        let content = r#"
Name:Grizzly Bears
ManaCost:1G
Types:Creature Bear
PT:2/2
Oracle:
"#;

        let def = CardLoader::parse(content).unwrap();
        assert_eq!(def.name.as_str(), "Grizzly Bears");
        assert_eq!(def.mana_cost.generic, 1);
        assert_eq!(def.mana_cost.green, 1);
        assert!(def.types.contains(&CardType::Creature));
        assert!(def.subtypes.contains(&Subtype::new("Bear")));
        assert_eq!(def.power, Some(2));
        assert_eq!(def.toughness, Some(2));
    }

    #[test]
    fn test_load_from_cardsfolder() {
        use std::path::PathBuf;

        // Try to load Lightning Bolt from the cardsfolder
        let path = PathBuf::from("cardsfolder/l/lightning_bolt.txt");

        // Only run this test if the cardsfolder exists
        if !path.exists() {
            return;
        }

        let def = CardLoader::load_from_file(&path).unwrap();
        assert_eq!(def.name.as_str(), "Lightning Bolt");
        assert_eq!(def.mana_cost.red, 1);
        assert!(def.types.contains(&CardType::Instant));
        assert!(def.colors.contains(&Color::Red));
        assert_eq!(def.raw_abilities.len(), 1);
        assert!(def.raw_abilities[0].contains("DealDamage"));
    }

    #[test]
    fn test_parse_with_abilities() {
        let content = r#"
Name:Lightning Bolt
ManaCost:R
Types:Instant
A:SP$ DealDamage | ValidTgts$ Any | NumDmg$ 3 | SpellDescription$ CARDNAME deals 3 damage to any target.
Oracle:Lightning Bolt deals 3 damage to any target.
"#;

        let def = CardLoader::parse(content).unwrap();
        assert_eq!(def.name.as_str(), "Lightning Bolt");
        assert_eq!(def.raw_abilities.len(), 1);
        assert!(def.raw_abilities[0].starts_with("A:"));
        assert!(def.raw_abilities[0].contains("DealDamage"));
    }

    #[test]
    fn test_parse_draw_spell() {
        let content = r#"
Name:Ancestral Recall
ManaCost:U
Types:Instant
A:SP$ Draw | NumCards$ 3 | ValidTgts$ Player | TgtPrompt$ Select target player | SpellDescription$ Target player draws three cards.
Oracle:Target player draws three cards.
"#;

        let def = CardLoader::parse(content).unwrap();
        assert_eq!(def.name.as_str(), "Ancestral Recall");
        assert_eq!(def.mana_cost.blue, 1);
        assert!(def.types.contains(&CardType::Instant));
        assert!(def.colors.contains(&Color::Blue));

        // Check that the effect is parsed
        let effects = def.parse_effects();
        assert_eq!(effects.len(), 1, "Ancestral Recall should have 1 effect");

        use crate::core::Effect;
        match &effects[0] {
            Effect::DrawCards { player: _, count } => {
                assert_eq!(*count, 3, "Should draw 3 cards");
            }
            _ => panic!("Expected DrawCards effect, got {:?}", effects[0]),
        }
    }

    #[test]
    fn test_parse_destroy_spell() {
        let content = r#"
Name:Terror
ManaCost:1 B
Types:Instant
A:SP$ Destroy | ValidTgts$ Creature.nonArtifact+nonBlack | TgtPrompt$ Select target nonartifact, nonblack creature | NoRegen$ True | SpellDescription$ Destroy target nonartifact, nonblack creature. It can't be regenerated.
Oracle:Destroy target nonartifact, nonblack creature. It can't be regenerated.
"#;

        let def = CardLoader::parse(content).unwrap();
        assert_eq!(def.name.as_str(), "Terror");
        assert_eq!(def.mana_cost.generic, 1);
        assert_eq!(def.mana_cost.black, 1);
        assert!(def.types.contains(&CardType::Instant));
        assert!(def.colors.contains(&Color::Black));

        // Check that the effect is parsed
        let effects = def.parse_effects();
        assert_eq!(effects.len(), 1, "Terror should have 1 effect");

        use crate::core::Effect;
        match &effects[0] {
            Effect::DestroyPermanent { target: _ } => {
                // Success - correct effect type
            }
            _ => panic!("Expected DestroyPermanent effect, got {:?}", effects[0]),
        }
    }

    #[test]
    fn test_parse_gainlife_spell() {
        let content = r#"
Name:Angel's Mercy
ManaCost:2 W W
Types:Instant
A:SP$ GainLife | LifeAmount$ 7 | SpellDescription$ You gain 7 life.
Oracle:You gain 7 life.
"#;

        let def = CardLoader::parse(content).unwrap();
        assert_eq!(def.name.as_str(), "Angel's Mercy");
        assert_eq!(def.mana_cost.generic, 2);
        assert_eq!(def.mana_cost.white, 2);
        assert!(def.types.contains(&CardType::Instant));
        assert!(def.colors.contains(&Color::White));

        // Check that the effect is parsed
        let effects = def.parse_effects();
        assert_eq!(effects.len(), 1, "Angel's Mercy should have 1 effect");

        use crate::core::Effect;
        match &effects[0] {
            Effect::GainLife { player: _, amount } => {
                assert_eq!(*amount, 7, "Should gain 7 life");
            }
            _ => panic!("Expected GainLife effect, got {:?}", effects[0]),
        }
    }
}
