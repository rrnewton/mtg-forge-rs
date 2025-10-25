//! Heuristic AI controller - faithful port of Java Forge AI
//!
//! This implementation aims to faithfully reproduce the decision-making logic
//! of the Java Forge heuristic AI. It uses evaluation heuristics for creatures,
//! spells, and board states rather than simulation or Monte Carlo methods.
//!
//! Reference: forge-java/forge-ai/src/main/java/forge/ai/
//! - PlayerControllerAi.java (entry point)
//! - AiController.java (core logic)
//! - CreatureEvaluator.java (creature scoring)

use crate::core::{Card, CardId, Keyword, ManaCost, PlayerId, SpellAbility};
use crate::game::controller::{GameStateView, PlayerController};
use smallvec::SmallVec;

/// Heuristic AI controller that makes decisions using evaluation functions
/// rather than simulation. Aims to faithfully reproduce Java Forge AI behavior.
pub struct HeuristicController {
    player_id: PlayerId,
    #[allow(dead_code)] // Will be used for randomized decisions (bluffing, etc.)
    rng: Box<dyn rand::RngCore>,
    /// Aggression level for combat decisions (0 = defensive, 6 = all-in)
    /// Default is 3 (balanced). Matches Java's AiAttackController aggression.
    aggression_level: i32,
}

impl HeuristicController {
    /// Create a new heuristic controller with default settings
    pub fn new(player_id: PlayerId) -> Self {
        HeuristicController {
            player_id,
            rng: Box::new(rand::thread_rng()),
            aggression_level: 3, // Balanced aggression
        }
    }

    /// Create a heuristic controller with a seeded RNG (for deterministic testing)
    pub fn with_seed(player_id: PlayerId, seed: u64) -> Self {
        use rand::SeedableRng;
        HeuristicController {
            player_id,
            rng: Box::new(rand::rngs::StdRng::seed_from_u64(seed)),
            aggression_level: 3,
        }
    }

    /// Set the aggression level for combat decisions
    /// 0 = very defensive, 3 = balanced, 6 = very aggressive
    pub fn set_aggression(&mut self, level: i32) {
        self.aggression_level = level.clamp(0, 6);
    }

    /// Evaluate a creature's value using heuristics
    ///
    /// This is a faithful port of Java's CreatureEvaluator.evaluateCreature()
    /// Reference: forge-java/forge-ai/src/main/java/forge/ai/CreatureEvaluator.java:26
    ///
    /// Returns a score representing the creature's overall value.
    /// Higher scores indicate more valuable creatures.
    pub fn evaluate_creature(&self, card: &Card) -> i32 {
        self.evaluate_creature_impl(card, true, true)
    }

    /// Internal implementation of creature evaluation with optional P/T and CMC consideration
    ///
    /// Parameters:
    /// - consider_pt: Whether to factor in power/toughness
    /// - consider_cmc: Whether to factor in mana cost
    fn evaluate_creature_impl(&self, card: &Card, consider_pt: bool, consider_cmc: bool) -> i32 {
        let mut value = 80;

        // Tokens are worth less than actual cards
        // Java: if (!c.isToken()) { value += addValue(20, "non-token"); }
        // TODO: Add is_token flag to Card struct
        // For now, assume all cards are non-tokens
        value += 20;

        let power = card.power.unwrap_or(0) as i32;
        let toughness = card.toughness.unwrap_or(0) as i32;

        // Stats scoring
        if consider_pt {
            // Java: value += addValue(power * 15, "power");
            value += power * 15;
            // Java: value += addValue(toughness * 10, "toughness: " + toughness);
            value += toughness * 10;
        }

        if consider_cmc {
            // Java: value += addValue(c.getCMC() * 5, "cmc");
            let cmc = card.mana_cost.cmc() as i32;
            value += cmc * 5;
        }

        // Evasion keywords
        // Java: if (c.hasKeyword(Keyword.FLYING)) { value += addValue(power * 10, "flying"); }
        if card.has_flying() {
            value += power * 10;
        }

        // Horsemanship: Not implemented in Keyword enum yet, skip for now
        // TODO: Add Horsemanship to Keyword enum

        // Unblockable check
        // Java: if (StaticAbilityCantAttackBlock.cantBlockBy(c, null)) { value += addValue(power * 10, "unblockable"); }
        // For now, we'll check for explicit Other keyword
        // TODO: Implement full static ability check
        let is_unblockable = card.keywords.iter().any(|k| matches!(k, Keyword::Other(s) if s.contains("can't be blocked") || s.contains("unblockable")));

        if !is_unblockable {
            // Other evasion keywords - not yet in enum, check via Other variant
            // TODO: Add Fear, Intimidate, Skulk to Keyword enum
            let has_fear = card
                .keywords
                .iter()
                .any(|k| matches!(k, Keyword::Other(s) if s.contains("Fear")));
            let has_intimidate = card
                .keywords
                .iter()
                .any(|k| matches!(k, Keyword::Other(s) if s.contains("Intimidate")));
            let has_skulk = card
                .keywords
                .iter()
                .any(|k| matches!(k, Keyword::Other(s) if s.contains("Skulk")));

            if has_fear {
                value += power * 6;
            }
            if has_intimidate {
                value += power * 6;
            }
            // Java: if (c.hasKeyword(Keyword.MENACE)) { value += addValue(power * 4, "menace"); }
            if card.has_menace() {
                value += power * 4;
            }
            if has_skulk {
                value += power * 3;
            }
        } else {
            value += power * 10;
        }

        // Combat keywords (only relevant if creature has power)
        if power > 0 {
            // Java: if (c.hasKeyword(Keyword.DOUBLE_STRIKE)) { value += addValue(10 + (power * 15), "ds"); }
            if card.has_double_strike() {
                value += 10 + (power * 15);
            }
            // Java: else if (c.hasKeyword(Keyword.FIRST_STRIKE)) { value += addValue(10 + (power * 5), "fs"); }
            else if card.has_first_strike() {
                value += 10 + (power * 5);
            }

            // Java: if (c.hasKeyword(Keyword.DEATHTOUCH)) { value += addValue(25, "dt"); }
            if card.has_deathtouch() {
                value += 25;
            }

            // Java: if (c.hasKeyword(Keyword.LIFELINK)) { value += addValue(power * 10, "lifelink"); }
            if card.has_lifelink() {
                value += power * 10;
            }

            // Java: if (power > 1 && c.hasKeyword(Keyword.TRAMPLE)) { value += addValue((power - 1) * 5, "trample"); }
            if power > 1 && card.has_trample() {
                value += (power - 1) * 5;
            }

            // Java: if (c.hasKeyword(Keyword.VIGILANCE)) { value += addValue((power * 5) + (toughness * 5), "vigilance"); }
            if card.has_keyword(&Keyword::Vigilance) {
                value += (power * 5) + (toughness * 5);
            }

            // Infect, Wither: Not in Keyword enum yet, check via Other
            // TODO: Add Infect, Wither to Keyword enum
            let has_infect = card
                .keywords
                .iter()
                .any(|k| matches!(k, Keyword::Other(s) if s.contains("Infect")));
            let has_wither = card
                .keywords
                .iter()
                .any(|k| matches!(k, Keyword::Other(s) if s.contains("Wither")));

            if has_infect {
                value += power * 15;
            } else if has_wither {
                value += power * 10;
            }
        }

        // Defensive keywords
        // Java: if (c.hasKeyword(Keyword.REACH) && !c.hasKeyword(Keyword.FLYING)) { value += addValue(5, "reach"); }
        if card.has_reach() && !card.has_flying() {
            value += 5;
        }

        // Protection keywords
        // Java: if (c.hasKeyword(Keyword.INDESTRUCTIBLE)) { value += addValue(70, "darksteel"); }
        if card.has_indestructible() {
            value += 70;
        }

        // Java: if (c.hasKeyword(Keyword.HEXPROOF)) { value += addValue(35, "hexproof"); }
        if card.has_hexproof() {
            value += 35;
        }

        // Java: if (c.hasKeyword(Keyword.SHROUD)) { value += addValue(30, "shroud"); }
        if card.has_shroud() {
            value += 30;
        }

        // Negative keywords
        // Java: if (c.hasKeyword(Keyword.DEFENDER)) { value -= power * 9 + 40; }
        if card.has_defender() {
            value -= power * 9 + 40;
        }

        // Mana abilities add value
        // Java: if (!c.getManaAbilities().isEmpty()) { value += addValue(10, "mana"); }
        // TODO: Implement mana ability check
        // For now, check if it's a land with mana ability
        if card.is_land() {
            value += 10;
        }

        value
    }

    /// Get the best creature from a list based on evaluation score
    ///
    /// Reference: ComputerUtilCard.sortByEvaluateCreature() and getBestCreatureAI()
    fn get_best_creature<'a>(&self, creatures: &[&'a Card]) -> Option<&'a Card> {
        creatures
            .iter()
            .max_by_key(|card| self.evaluate_creature(card))
            .copied()
    }

    /// Get the worst creature from a list based on evaluation score
    #[allow(dead_code)] // Will be used for discard decisions
    fn get_worst_creature<'a>(&self, creatures: &[&'a Card]) -> Option<&'a Card> {
        creatures
            .iter()
            .min_by_key(|card| self.evaluate_creature(card))
            .copied()
    }

    /// Evaluate whether a land should be played
    ///
    /// Reference: AiController.java:1428-1446 (land play decision logic)
    ///
    /// The Java AI uses several checks:
    /// 1. Don't play lands that would deal lethal ETB damage
    /// 2. Sometimes hold land drop for Main 2 (bluffing/deception)
    /// 3. Prioritize playing lands early in the game
    ///
    /// For now, we use a simplified approach:
    /// - Always play lands (faithful to Java's default behavior)
    /// - TODO: Add ETB damage check
    /// - TODO: Add Main 2 hold logic (requires randomization based on AI profile)
    /// - TODO: Check for "PlayBeforeLandDrop" special cases
    fn should_play_land(&self, _view: &GameStateView) -> bool {
        // Basic check: always play lands
        // This matches Java's behavior when no special conditions apply

        // TODO(mtg-XX): Add ETB damage check
        // Java: (!player.canLoseLife() || player.cantLoseForZeroOrLessLife()
        //        || ComputerUtil.getDamageFromETB(player, land) < player.getLife())

        // TODO(mtg-XX): Add Main 2 hold logic for bluffing
        // Java: (!game.getPhaseHandler().is(PhaseType.MAIN1)
        //        || !isSafeToHoldLandDropForMain2(land))
        // This is a deception mechanism to hide information from opponents

        // For phase 1, always play lands
        true
    }

    /// Choose the best land to play from available lands
    ///
    /// Reference: AiController.java:500-724 (chooseBestLandToPlay)
    ///
    /// The Java AI scores lands based on:
    /// 1. Base evaluation score (from GameStateEvaluator.evaluateLand)
    /// 2. +25 points for new basic land types
    /// 3. Color production: (new_colors * 50) / (existing_colors + 1)
    /// 4. Preference for untapped lands when we have spells to cast
    /// 5. Color fixing for one-drops in hand
    ///
    /// For now, simplified version:
    /// - Prefer untapped lands
    /// - Prefer lands that produce colors we need
    /// - TODO: Full scoring algorithm from Java
    fn choose_best_land(&self, _view: &GameStateView, lands: &[CardId]) -> Option<CardId> {
        if lands.is_empty() {
            return None;
        }

        // For now, just return the first land
        // TODO(mtg-XX): Implement full land selection algorithm
        // - Score based on enters-tapped status
        // - Score based on color production vs colors in hand
        // - Score based on new basic land types
        // - Consider mana curve of hand

        Some(lands[0])
    }

    /// Choose the best spell to cast from available options
    ///
    /// This implements the core decision logic from AiController.chooseSpellAbilityToPlay()
    /// Reference: AiController.java:1415-1449
    ///
    /// Priority order (like Java):
    /// 1. Check for "PlayBeforeLandDrop" cards (special timing requirements)
    /// 2. Play land (if available and should play)
    /// 3. Cast creatures (best evaluation first)
    /// 4. Cast other spells (removal, pump, etc.)
    /// 5. Pass priority
    fn choose_best_spell(
        &mut self,
        view: &GameStateView,
        available: &[SpellAbility],
    ) -> Option<SpellAbility> {
        if available.is_empty() {
            return None;
        }

        // Phase 1: Check for "PlayBeforeLandDrop" cards
        // TODO(mtg-XX): Implement PlayBeforeLandDrop check
        // Java: CardLists.filter(player.getCardsIn(ZoneType.Hand),
        //                        CardPredicates.hasSVar("PlayBeforeLandDrop"))

        // Phase 2: Land play logic
        if self.should_play_land(view) {
            // Collect land play abilities
            let land_plays: Vec<&SpellAbility> = available
                .iter()
                .filter(|sa| matches!(sa, SpellAbility::PlayLand { .. }))
                .collect();

            if !land_plays.is_empty() {
                // Extract land card IDs
                let land_ids: Vec<CardId> = land_plays
                    .iter()
                    .filter_map(|sa| {
                        if let SpellAbility::PlayLand { card_id, .. } = sa {
                            Some(*card_id)
                        } else {
                            None
                        }
                    })
                    .collect();

                // Choose best land
                if let Some(best_land_id) = self.choose_best_land(view, &land_ids) {
                    // Find and return the corresponding land play ability
                    for ability in land_plays {
                        if let SpellAbility::PlayLand { card_id, .. } = ability {
                            if *card_id == best_land_id {
                                return Some((*ability).clone());
                            }
                        }
                    }
                }
            }
        }

        // Phase 3: Cast creatures (best evaluation first)
        // TODO(mtg-XX): Evaluate creature quality and choose best
        // For now, just cast the first creature we find
        for ability in available {
            if matches!(ability, SpellAbility::CastSpell { .. }) {
                return Some(ability.clone());
            }
        }

        // Phase 4: Cast other spells
        // TODO(mtg-XX): Evaluate removal, pump spells, card draw, etc.
        // Java: Has separate logic for each spell type with evaluation functions

        // Pass priority if nothing good to do
        None
    }
}

impl PlayerController for HeuristicController {
    fn player_id(&self) -> PlayerId {
        self.player_id
    }

    fn choose_spell_ability_to_play(
        &mut self,
        view: &GameStateView,
        available: &[SpellAbility],
    ) -> Option<SpellAbility> {
        if available.is_empty() {
            view.logger()
                .controller_choice("HEURISTIC", "chose to pass priority (no available actions)");
            return None;
        }

        let choice = self.choose_best_spell(view, available);

        if let Some(ref spell) = choice {
            view.logger().controller_choice(
                "HEURISTIC",
                &format!("chose to play spell/ability: {:?}", spell),
            );
        } else {
            view.logger().controller_choice(
                "HEURISTIC",
                &format!(
                    "chose to pass priority from {} available actions",
                    available.len()
                ),
            );
        }

        choice
    }

    fn choose_targets(
        &mut self,
        view: &GameStateView,
        spell: CardId,
        valid_targets: &[CardId],
    ) -> SmallVec<[CardId; 4]> {
        if valid_targets.is_empty() {
            return SmallVec::new();
        }

        // TODO: Implement intelligent targeting
        // For now, use simple heuristics:
        // - For removal: Target opponent's best creature
        // - For pump: Target our best creature
        // - For damage: Target opponent's best creature

        // Get the spell card to determine its type
        let spell_card = view.get_card(spell);
        let is_our_spell = spell_card
            .map(|c| c.owner == self.player_id)
            .unwrap_or(false);

        // Collect target cards
        let mut target_cards: Vec<&Card> = valid_targets
            .iter()
            .filter_map(|&id| view.get_card(id))
            .collect();

        if target_cards.is_empty() {
            // Fallback: just pick the first target
            let mut targets = SmallVec::new();
            targets.push(valid_targets[0]);
            return targets;
        }

        // For our own spells (pumps), target our best creature
        // For opponent spells (removal), target their best creature
        let target = if is_our_spell {
            // Target our best creature
            target_cards.retain(|c| c.owner == self.player_id);
            self.get_best_creature(&target_cards)
        } else {
            // Target opponent's best creature
            target_cards.retain(|c| c.owner != self.player_id);
            self.get_best_creature(&target_cards)
        };

        let mut targets = SmallVec::new();
        if let Some(target_card) = target {
            targets.push(target_card.id);
        } else if !valid_targets.is_empty() {
            // Fallback: just pick the first valid target
            targets.push(valid_targets[0]);
        }

        targets
    }

    fn choose_mana_sources_to_pay(
        &mut self,
        _view: &GameStateView,
        cost: &ManaCost,
        available_sources: &[CardId],
    ) -> SmallVec<[CardId; 8]> {
        // Simple greedy approach for now
        // TODO: Implement intelligent mana tapping order from ComputerUtilMana
        let mut sources = SmallVec::new();
        let needed = cost.cmc() as usize;

        for &source_id in available_sources.iter().take(needed) {
            sources.push(source_id);
        }

        sources
    }

    fn choose_attackers(
        &mut self,
        view: &GameStateView,
        available_creatures: &[CardId],
    ) -> SmallVec<[CardId; 8]> {
        // TODO: Implement full attack logic from AiAttackController
        // For now, use simple heuristic based on aggression level

        let mut attackers = SmallVec::new();

        // Get creature cards
        let creatures: Vec<&Card> = available_creatures
            .iter()
            .filter_map(|&id| view.get_card(id))
            .collect();

        // Simple heuristic: Attack with creatures that have evasion or high power
        for creature in creatures {
            let has_evasion = creature.has_flying()
                || creature.has_menace()
                || creature.keywords.iter().any(|k| matches!(k, Keyword::Other(s) if s.contains("can't be blocked") || s.contains("unblockable")));

            let power = creature.power.unwrap_or(0);

            // Aggressive: attack with evasive creatures or creatures with power >= 2
            if has_evasion || power >= 2 {
                attackers.push(creature.id);
            }
        }

        if !attackers.is_empty() {
            view.logger().controller_choice(
                "HEURISTIC",
                &format!(
                    "chose {} attackers from {} available creatures",
                    attackers.len(),
                    available_creatures.len()
                ),
            );
        }

        attackers
    }

    fn choose_blockers(
        &mut self,
        view: &GameStateView,
        available_blockers: &[CardId],
        attackers: &[CardId],
    ) -> SmallVec<[(CardId, CardId); 8]> {
        // TODO: Implement full block logic from AiBlockController
        // For now, use simple heuristic

        let mut blocks = SmallVec::new();

        if attackers.is_empty() || available_blockers.is_empty() {
            return blocks;
        }

        // Simple heuristic: Block the biggest attackers with our best blockers
        let mut attacker_cards: Vec<&Card> = attackers
            .iter()
            .filter_map(|&id| view.get_card(id))
            .collect();

        let mut blocker_cards: Vec<&Card> = available_blockers
            .iter()
            .filter_map(|&id| view.get_card(id))
            .collect();

        // Sort attackers by power (descending)
        attacker_cards.sort_by_key(|c| -(c.power.unwrap_or(0)));

        // Sort blockers by toughness (descending)
        blocker_cards.sort_by_key(|c| -(c.toughness.unwrap_or(0)));

        // Assign blockers to attackers
        for (blocker, attacker) in blocker_cards.iter().zip(attacker_cards.iter()) {
            blocks.push((blocker.id, attacker.id));
        }

        if !blocks.is_empty() {
            view.logger()
                .controller_choice("HEURISTIC", &format!("chose {} blockers", blocks.len()));
        }

        blocks
    }

    fn choose_damage_assignment_order(
        &mut self,
        _view: &GameStateView,
        _attacker: CardId,
        blockers: &[CardId],
    ) -> SmallVec<[CardId; 4]> {
        // For now, just return the blockers in order
        // TODO: Implement intelligent ordering to kill blockers efficiently
        blockers.iter().copied().collect()
    }

    fn choose_cards_to_discard(
        &mut self,
        view: &GameStateView,
        hand: &[CardId],
        count: usize,
    ) -> SmallVec<[CardId; 7]> {
        // Simple heuristic: Discard lands first, then worst creatures
        let mut hand_cards: Vec<&Card> = hand.iter().filter_map(|&id| view.get_card(id)).collect();

        // Sort by value (ascending) - discard worst cards first
        hand_cards.sort_by_key(|c| {
            if c.is_land() {
                0 // Discard lands first
            } else if c.is_creature() {
                self.evaluate_creature(c)
            } else {
                100 // Keep spells
            }
        });

        hand_cards.iter().take(count).map(|c| c.id).collect()
    }

    fn on_priority_passed(&mut self, _view: &GameStateView) {
        // Could track game state here for future decisions
    }

    fn on_game_end(&mut self, _view: &GameStateView, _won: bool) {
        // Could collect statistics here
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::EntityId;

    #[test]
    fn test_heuristic_controller_creation() {
        let player_id = EntityId::new(1);
        let controller = HeuristicController::new(player_id);
        assert_eq!(controller.player_id(), player_id);
        assert_eq!(controller.aggression_level, 3);
    }

    #[test]
    fn test_seeded_controller() {
        let player_id = EntityId::new(1);
        let controller = HeuristicController::with_seed(player_id, 42);
        assert_eq!(controller.player_id(), player_id);
    }

    #[test]
    fn test_aggression_setting() {
        let player_id = EntityId::new(1);
        let mut controller = HeuristicController::new(player_id);

        controller.set_aggression(0);
        assert_eq!(controller.aggression_level, 0);

        controller.set_aggression(6);
        assert_eq!(controller.aggression_level, 6);

        // Test clamping
        controller.set_aggression(10);
        assert_eq!(controller.aggression_level, 6);

        controller.set_aggression(-5);
        assert_eq!(controller.aggression_level, 0);
    }
}
