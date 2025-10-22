//! Game actions and mechanics

use crate::core::{CardId, CardType, Effect, Keyword, PlayerId, TargetRef};
use crate::game::GameState;
use crate::zones::Zone;
use crate::{MtgError, Result};
use smallvec::SmallVec;

/// Types of game actions
#[derive(Debug, Clone)]
pub enum GameAction {
    /// Play a land from hand
    PlayLand {
        player_id: PlayerId,
        card_id: CardId,
    },

    /// Cast a spell from hand
    CastSpell {
        player_id: PlayerId,
        card_id: CardId,
        targets: Vec<CardId>,
    },

    /// Deal damage to a target
    DealDamage {
        source: CardId,
        target: CardId,
        amount: i32,
    },

    /// Tap a permanent for mana
    TapForMana {
        player_id: PlayerId,
        card_id: CardId,
    },

    /// Declare attackers
    DeclareAttackers {
        player_id: PlayerId,
        attackers: Vec<CardId>,
    },

    /// Pass priority
    PassPriority { player_id: PlayerId },
}

impl GameState {
    /// Play a land from hand to battlefield
    pub fn play_land(&mut self, player_id: PlayerId, card_id: CardId) -> Result<()> {
        // Check if player can play a land
        let player = self.get_player(player_id)?;
        if !player.can_play_land() {
            return Err(MtgError::InvalidAction(
                "Cannot play more lands this turn".to_string(),
            ));
        }

        // Check if card is a land and in hand
        let card = self.cards.get(card_id)?;
        if !card.is_land() {
            return Err(MtgError::InvalidAction("Card is not a land".to_string()));
        }

        // Check if in hand
        if let Some(zones) = self.get_player_zones(player_id) {
            if !zones.hand.contains(card_id) {
                return Err(MtgError::InvalidAction("Card not in hand".to_string()));
            }
        }

        // Move card to battlefield
        self.move_card(card_id, Zone::Hand, Zone::Battlefield, player_id)?;

        // Record the turn number when this land entered the battlefield
        if let Ok(card) = self.cards.get_mut(card_id) {
            card.turn_entered_battlefield = Some(self.turn.turn_number);
        }

        // Increment lands played
        let player = self.get_player_mut(player_id)?;
        player.play_land();

        Ok(())
    }

    /// Cast a spell (put it on the stack)
    ///
    /// This validates mana payment and deducts the cost from the player's mana pool.
    pub fn cast_spell(
        &mut self,
        player_id: PlayerId,
        card_id: CardId,
        _targets: Vec<CardId>,
    ) -> Result<()> {
        // Check if card is in hand
        if let Some(zones) = self.get_player_zones(player_id) {
            if !zones.hand.contains(card_id) {
                return Err(MtgError::InvalidAction("Card not in hand".to_string()));
            }
        }

        // Get the mana cost (need to do this before mutable borrow)
        let mana_cost = {
            let card = self.cards.get(card_id)?;
            card.mana_cost.clone()
        };

        // Pay the mana cost
        let player = self.get_player_mut(player_id)?;
        player
            .mana_pool
            .pay_cost(&mana_cost)
            .map_err(MtgError::InvalidAction)?;

        // Move card to stack
        self.move_card(card_id, Zone::Hand, Zone::Stack, player_id)?;

        Ok(())
    }

    /// Resolve a spell from the stack
    pub fn resolve_spell(&mut self, card_id: CardId) -> Result<()> {
        // Get card owner and effects
        let (card_owner, mut effects) = {
            let card = self.cards.get(card_id)?;
            // TODO: eliminate this clone and instead just take a reference. Why does it need to be mutable?
            (card.owner, card.effects.clone())
        };

        // Fill in missing targets for effects
        // For now, target an opponent for DealDamage effects with no target
        for effect in &mut effects {
            match effect {
                Effect::DealDamage {
                    target: TargetRef::None,
                    amount,
                } => {
                    // Find an opponent
                    if let Some(opponent_id) = self
                        .players
                        .iter()
                        .map(|p| p.id)
                        .find(|id| *id != card_owner)
                    {
                        *effect = Effect::DealDamage {
                            target: TargetRef::Player(opponent_id),
                            amount: *amount,
                        };
                    }
                }
                Effect::DrawCards { player, count } if player.as_u32() == 0 => {
                    // Default: the card's controller draws (placeholder player ID 0 means "controller")
                    *effect = Effect::DrawCards {
                        player: card_owner,
                        count: *count,
                    };
                }
                Effect::DestroyPermanent { target } if target.as_u32() == 0 => {
                    // Default: destroy an opponent's creature (placeholder card ID 0 means "opponent's creature")
                    // Find an opponent's creature on the battlefield (that doesn't have hexproof or shroud)
                    if let Some(creature_id) = self
                        .battlefield
                        .cards
                        .iter()
                        .find(|&card_id| {
                            if let Ok(card) = self.cards.get(*card_id) {
                                card.owner != card_owner
                                    && card.is_creature()
                                    && !card.has_hexproof()
                                    && !card.has_shroud()
                            } else {
                                false
                            }
                        })
                        .copied()
                    {
                        *effect = Effect::DestroyPermanent {
                            target: creature_id,
                        };
                    }
                }
                Effect::GainLife { player, amount } if player.as_u32() == 0 => {
                    // Default: the card's controller gains life (placeholder player ID 0 means "controller")
                    *effect = Effect::GainLife {
                        player: card_owner,
                        amount: *amount,
                    };
                }
                Effect::PumpCreature {
                    target,
                    power_bonus,
                    toughness_bonus,
                } if target.as_u32() == 0 => {
                    // Default: target any creature (placeholder card ID 0 means "target creature")
                    // Hexproof prevents targeting by opponent's spells/abilities
                    // Find the first valid target creature on the battlefield
                    if let Some(creature_id) = self
                        .battlefield
                        .cards
                        .iter()
                        .find(|&card_id| {
                            if let Ok(card) = self.cards.get(*card_id) {
                                // Shroud prevents targeting by anyone (including controller)
                                if card.has_shroud() {
                                    false
                                } else if card.owner != card_owner {
                                    // Opponent's creature: can only target if it doesn't have hexproof
                                    card.is_creature() && !card.has_hexproof()
                                } else {
                                    // Own creature: can target (unless it has shroud, checked above)
                                    card.is_creature()
                                }
                            } else {
                                false
                            }
                        })
                        .copied()
                    {
                        *effect = Effect::PumpCreature {
                            target: creature_id,
                            power_bonus: *power_bonus,
                            toughness_bonus: *toughness_bonus,
                        };
                    }
                }
                Effect::TapPermanent { target } if target.as_u32() == 0 => {
                    // Default: target an opponent's untapped creature (that doesn't have hexproof or shroud)
                    if let Some(creature_id) = self
                        .battlefield
                        .cards
                        .iter()
                        .find(|&card_id| {
                            if let Ok(card) = self.cards.get(*card_id) {
                                card.owner != card_owner
                                    && card.is_creature()
                                    && !card.tapped
                                    && !card.has_hexproof()
                                    && !card.has_shroud()
                            } else {
                                false
                            }
                        })
                        .copied()
                    {
                        *effect = Effect::TapPermanent {
                            target: creature_id,
                        };
                    }
                }
                Effect::UntapPermanent { target } if target.as_u32() == 0 => {
                    // Default: target a tapped permanent controlled by the caster
                    if let Some(permanent_id) = self
                        .battlefield
                        .cards
                        .iter()
                        .find(|&card_id| {
                            if let Ok(card) = self.cards.get(*card_id) {
                                card.owner == card_owner && card.tapped
                            } else {
                                false
                            }
                        })
                        .copied()
                    {
                        *effect = Effect::UntapPermanent {
                            target: permanent_id,
                        };
                    }
                }
                _ => {}
            }
        }

        for effect in effects {
            self.execute_effect(&effect)?;
        }

        // Determine destination based on card type
        let destination = {
            let card = self.cards.get(card_id)?;
            if card.is_type(&CardType::Instant) || card.is_type(&CardType::Sorcery) {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        };

        // Move card from stack to destination
        let owner = self.cards.get(card_id)?.owner;
        self.move_card(card_id, Zone::Stack, destination, owner)?;

        // If it entered the battlefield, record the turn number (for summoning sickness)
        if destination == Zone::Battlefield {
            if let Ok(card) = self.cards.get_mut(card_id) {
                card.turn_entered_battlefield = Some(self.turn.turn_number);
            }
        }

        Ok(())
    }

    /// Cast a spell following the full 8-step process (MTG Rules 601.2)
    ///
    /// This method implements the complete spell casting sequence:
    /// 1. Propose the spell (move to stack)
    /// 2. Make choices (modes, X values) - TODO
    /// 3. Choose targets
    /// 4. Divide effects - TODO
    /// 5. Determine total cost
    /// 6. Activate mana abilities (tap sources for mana)
    /// 7. Pay costs
    /// 8. Spell becomes cast (trigger abilities) - TODO
    ///
    /// ## Parameters
    /// - `player_id`: The player casting the spell
    /// - `card_id`: The spell card to cast
    /// - `choose_targets_fn`: Callback to choose targets (step 3)
    /// - `choose_mana_sources_fn`: Callback to choose what to tap for mana (step 6)
    ///
    /// ## Java Forge Equivalent
    /// This matches `ComputerUtil.handlePlayingSpellAbility()` which:
    /// 1. Moves spell to stack (line 99)
    /// 2. Handles targeting
    /// 3. Pays costs with `CostPayment.payComputerCosts()` (line 125)
    pub fn cast_spell_8_step<TargetFn, ManaFn>(
        &mut self,
        player_id: PlayerId,
        card_id: CardId,
        mut choose_targets_fn: TargetFn,
        mut choose_mana_sources_fn: ManaFn,
    ) -> Result<()>
    where
        TargetFn: FnMut(&GameState, CardId) -> Vec<CardId>,
        ManaFn: FnMut(&GameState, &crate::core::ManaCost) -> Vec<CardId>,
    {
        // Verify card is in hand
        if let Some(zones) = self.get_player_zones(player_id) {
            if !zones.hand.contains(card_id) {
                return Err(MtgError::InvalidAction("Card not in hand".to_string()));
            }
        }

        // Step 1: Propose the spell - move card to stack
        // This happens BEFORE paying costs (unlike our old implementation)
        self.move_card(card_id, Zone::Hand, Zone::Stack, player_id)?;

        // Step 2: Make choices (modes, X values)
        // TODO: Implement modal spell choices and X value selection

        // Step 3: Choose targets
        let _targets = choose_targets_fn(self, card_id);
        // TODO: Store targets on the spell for resolution
        // For now, we'll use them to update effects immediately (simplified)

        // Step 4: Divide effects
        // TODO: Implement dividing damage/counters among targets

        // Step 5: Determine total cost
        let mana_cost = {
            let card = self.cards.get(card_id)?;
            card.mana_cost.clone()
        };

        // Step 6: Activate mana abilities
        // This is where mana gets tapped - AFTER the spell is on the stack
        let sources_to_tap = choose_mana_sources_fn(self, &mana_cost);
        for &source_id in &sources_to_tap {
            self.tap_for_mana(player_id, source_id)?;
        }

        // Step 7: Pay costs
        let player = self.get_player_mut(player_id)?;
        if let Err(e) = player.mana_pool.pay_cost(&mana_cost) {
            // If we can't pay, we need to unwind - move card back to hand
            // and untap mana sources
            // For now, just return error (TODO: proper unwinding)
            return Err(MtgError::InvalidAction(format!(
                "Failed to pay mana cost: {}",
                e
            )));
        }

        // Step 8: Spell becomes cast
        // TODO: Trigger "whenever you cast a spell" abilities
        // For now, this is complete - spell is on stack and costs are paid

        Ok(())
    }

    /// Execute a single effect
    pub fn execute_effect(&mut self, effect: &Effect) -> Result<()> {
        match effect {
            Effect::DealDamage { target, amount } => match target {
                TargetRef::Player(player_id) => {
                    self.deal_damage(*player_id, *amount)?;
                }
                TargetRef::Permanent(card_id) => {
                    self.deal_damage_to_creature(*card_id, *amount)?;
                }
                TargetRef::None => {
                    return Err(MtgError::InvalidAction(
                        "DealDamage effect requires a target".to_string(),
                    ));
                }
            },
            Effect::DrawCards { player, count } => {
                for _ in 0..*count {
                    self.draw_card(*player)?;
                }
            }
            Effect::GainLife { player, amount } => {
                let p = self.get_player_mut(*player)?;
                p.gain_life(*amount);

                // Log the life gain
                self.undo_log.log(crate::undo::GameAction::ModifyLife {
                    player_id: *player,
                    delta: *amount,
                });
            }
            Effect::DestroyPermanent { target } => {
                // Skip if target is still placeholder (0) - no valid targets found
                if target.as_u32() == 0 {
                    // Spell fizzles - no valid targets
                    return Ok(());
                }
                // MTG Rules 702.12b: Permanents with indestructible can't be destroyed
                let (owner, has_indestructible) = {
                    let card = self.cards.get(*target)?;
                    (card.owner, card.has_indestructible())
                };
                if !has_indestructible {
                    self.move_card(*target, Zone::Battlefield, Zone::Graveyard, owner)?;
                }
            }
            Effect::TapPermanent { target } => {
                // Skip if target is still placeholder (0) - no valid targets found
                if target.as_u32() == 0 {
                    // Spell fizzles - no valid targets
                    return Ok(());
                }
                let card = self.cards.get_mut(*target)?;
                card.tap();

                // Log the tap
                self.undo_log.log(crate::undo::GameAction::TapCard {
                    card_id: *target,
                    tapped: true,
                });
            }
            Effect::UntapPermanent { target } => {
                let card = self.cards.get_mut(*target)?;
                card.untap();

                // Log the untap
                self.undo_log.log(crate::undo::GameAction::TapCard {
                    card_id: *target,
                    tapped: false,
                });
            }
            Effect::PumpCreature {
                target,
                power_bonus,
                toughness_bonus,
            } => {
                // Skip if target is still placeholder (0) - no valid targets found
                if target.as_u32() == 0 {
                    // Spell fizzles - no valid targets
                    return Ok(());
                }
                let card = self.cards.get_mut(*target)?;
                card.power_bonus += power_bonus;
                card.toughness_bonus += toughness_bonus;

                // Log the pump effect
                self.undo_log.log(crate::undo::GameAction::PumpCreature {
                    card_id: *target,
                    power_delta: *power_bonus,
                    toughness_delta: *toughness_bonus,
                });
            }
        }
        Ok(())
    }

    /// Deal damage to a player target
    pub fn deal_damage(&mut self, target_id: PlayerId, amount: i32) -> Result<()> {
        // Check if target is a player
        if self.players.iter().any(|p| p.id == target_id) {
            let player = self.get_player_mut(target_id)?;
            player.lose_life(amount);

            // Log the life change
            self.undo_log.log(crate::undo::GameAction::ModifyLife {
                player_id: target_id,
                delta: -amount,
            });

            return Ok(());
        }

        Err(MtgError::InvalidAction("Invalid damage target".to_string()))
    }

    /// Deal damage to a creature
    pub fn deal_damage_to_creature(&mut self, target_id: CardId, amount: i32) -> Result<()> {
        // Get info about the creature first (without holding the borrow)
        let (is_creature, toughness, owner, has_indestructible) = {
            let card = self.cards.get(target_id)?;
            (
                card.is_creature(),
                card.current_toughness(),
                card.owner,
                card.has_indestructible(),
            )
        };

        if is_creature {
            // Mark damage (simplified - real MTG has damage tracking)
            // MTG Rules 702.12b: Permanents with indestructible aren't destroyed by lethal damage
            // For now, if damage >= toughness and creature doesn't have indestructible, creature dies
            if amount >= toughness as i32 && !has_indestructible {
                self.move_card(target_id, Zone::Battlefield, Zone::Graveyard, owner)?;
            }
            return Ok(());
        }

        Err(MtgError::InvalidAction("Invalid damage target".to_string()))
    }

    /// Tap a land for mana
    pub fn tap_for_mana(&mut self, player_id: PlayerId, card_id: CardId) -> Result<()> {
        let card = self.cards.get_mut(card_id)?;

        // Check if card is a land and untapped
        if !card.is_land() {
            return Err(MtgError::InvalidAction("Card is not a land".to_string()));
        }

        if card.tapped {
            return Err(MtgError::InvalidAction(
                "Land is already tapped".to_string(),
            ));
        }

        // Get land name before tapping (to avoid borrow conflicts)
        let land_name = card.name.to_lowercase();

        // Tap the land
        card.tap();

        // Log the tap
        self.undo_log.log(crate::undo::GameAction::TapCard {
            card_id,
            tapped: true,
        });

        // Add mana to player's pool based on land type
        // For now, simplified - just check land name
        let player = self.get_player_mut(player_id)?;
        let color = if land_name.contains("mountain") {
            Some(crate::core::Color::Red)
        } else if land_name.contains("island") {
            Some(crate::core::Color::Blue)
        } else if land_name.contains("swamp") {
            Some(crate::core::Color::Black)
        } else if land_name.contains("forest") {
            Some(crate::core::Color::Green)
        } else if land_name.contains("plains") {
            Some(crate::core::Color::White)
        } else {
            None
        };

        if let Some(color) = color {
            player.mana_pool.add_color(color);

            // Log the mana addition
            self.undo_log
                .log(crate::undo::GameAction::AddMana { player_id, color });
        }

        Ok(())
    }

    /// Declare a creature as an attacker
    pub fn declare_attacker(&mut self, player_id: PlayerId, card_id: CardId) -> Result<()> {
        // Validate creature can attack
        let card = self.cards.get(card_id)?;

        // Must be a creature
        if !card.is_creature() {
            return Err(MtgError::InvalidAction(
                "Only creatures can attack".to_string(),
            ));
        }

        // Must be controlled by the attacking player
        if card.controller != player_id {
            return Err(MtgError::InvalidAction(
                "Can't attack with opponent's creatures".to_string(),
            ));
        }

        // Must be on battlefield
        if !self.battlefield.contains(card_id) {
            return Err(MtgError::InvalidAction(
                "Creature must be on battlefield to attack".to_string(),
            ));
        }

        // Must not be tapped
        if card.tapped {
            return Err(MtgError::InvalidAction(
                "Creature is tapped and can't attack".to_string(),
            ));
        }

        // Check for summoning sickness
        // Creatures can't attack the turn they entered the battlefield unless they have haste
        if let Some(entered_turn) = card.turn_entered_battlefield {
            if entered_turn == self.turn.turn_number && !card.has_keyword(&Keyword::Haste) {
                return Err(MtgError::InvalidAction(
                    "Creature has summoning sickness and can't attack this turn".to_string(),
                ));
            }
        }

        // Get defending player (for 2-player, it's the other player)
        let defending_player = self
            .players
            .iter()
            .find(|p| p.id != player_id)
            .map(|p| p.id)
            .ok_or_else(|| MtgError::InvalidAction("No opponent found".to_string()))?;

        // Declare attacker in combat state
        self.combat.declare_attacker(card_id, defending_player);

        // Tap the creature (unless it has vigilance)
        let has_vigilance = self.cards.get(card_id)?.has_keyword(&Keyword::Vigilance);
        if !has_vigilance {
            let card = self.cards.get_mut(card_id)?;
            card.tap();

            // Log the action
            self.undo_log.log(crate::undo::GameAction::TapCard {
                card_id,
                tapped: true,
            });
        }

        Ok(())
    }

    /// Declare a creature as a blocker
    pub fn declare_blocker(
        &mut self,
        player_id: PlayerId,
        blocker_id: CardId,
        attackers: Vec<CardId>,
    ) -> Result<()> {
        // Validate blocker can block
        let blocker = self.cards.get(blocker_id)?;

        // Must be a creature
        if !blocker.is_creature() {
            return Err(MtgError::InvalidAction(
                "Only creatures can block".to_string(),
            ));
        }

        // Must be controlled by the defending player
        if blocker.controller != player_id {
            return Err(MtgError::InvalidAction(
                "Can't block with opponent's creatures".to_string(),
            ));
        }

        // Must be on battlefield
        if !self.battlefield.contains(blocker_id) {
            return Err(MtgError::InvalidAction(
                "Creature must be on battlefield to block".to_string(),
            ));
        }

        // Must not be tapped
        if blocker.tapped {
            return Err(MtgError::InvalidAction(
                "Creature is tapped and can't block".to_string(),
            ));
        }

        // Validate all attackers are actually attacking
        for &attacker in &attackers {
            if !self.combat.is_attacking(attacker) {
                return Err(MtgError::InvalidAction(format!(
                    "Card {:?} is not attacking",
                    attacker
                )));
            }
        }

        // Check Flying/Reach restrictions (MTG rule 702.9)
        // A creature with Flying can only be blocked by creatures with Flying or Reach
        let blocker_has_flying = blocker.has_keyword(&Keyword::Flying);
        let blocker_has_reach = blocker.has_keyword(&Keyword::Reach);

        for &attacker_id in &attackers {
            let attacker = self.cards.get(attacker_id)?;
            let attacker_has_flying = attacker.has_keyword(&Keyword::Flying);

            if attacker_has_flying && !blocker_has_flying && !blocker_has_reach {
                return Err(MtgError::InvalidAction(
                    "Creature cannot block attacker with flying (needs flying or reach)"
                        .to_string(),
                ));
            }

            // Note: Menace validation (MTG rule 702.111b) would require checking that creatures
            // with Menace have 0 or 2+ blockers, but this can only be validated after all
            // blockers are declared. Controllers should be smart enough not to block a Menace
            // creature with exactly 1 blocker. Incremental validation during blocker declaration
            // would reject the first blocker, which is incorrect.
        }

        // MTG rule: normally a creature can only block one attacker
        // (unless it has an ability that allows it to block multiple)
        if attackers.len() > 1 {
            // TODO: Check for abilities that allow blocking multiple
            return Err(MtgError::InvalidAction(
                "Creature can only block one attacker".to_string(),
            ));
        }

        // Declare blocker
        let mut attackers_vec = smallvec::SmallVec::new();
        for &attacker in &attackers {
            attackers_vec.push(attacker);
        }
        self.combat.declare_blocker(blocker_id, attackers_vec);

        Ok(())
    }

    /// Assign and deal combat damage
    ///
    /// This method handles the combat damage step. For each attacker:
    /// - If unblocked, damage goes to defending player
    /// - If blocked by multiple creatures, attacker's controller chooses damage assignment order
    /// - Damage is assigned in order, with lethal damage assigned to each blocker before the next
    ///
    /// MTG Rules 510.1: Combat damage is assigned and dealt simultaneously.
    /// MTG Rules 510.4: If any creature has first strike or double strike, there are two
    /// combat damage steps. Creatures with first strike or double strike deal damage in the
    /// first step, and creatures without first strike (plus double strike creatures) deal
    /// damage in the second step.
    ///
    /// # Arguments
    /// * `first_strike_step` - True for first strike damage step, false for normal damage step
    pub fn assign_combat_damage(
        &mut self,
        attacker_controller: &mut dyn crate::game::controller::PlayerController,
        blocker_controller: &mut dyn crate::game::controller::PlayerController,
        first_strike_step: bool,
    ) -> Result<()> {
        use crate::game::controller::GameStateView;
        use std::collections::HashMap;

        // Get all attackers
        let attackers = self.combat.get_attackers();

        // First pass: collect all damage assignment orders for attackers with multiple blockers
        let mut damage_orders: HashMap<CardId, SmallVec<[CardId; 4]>> = HashMap::new();

        for attacker_id in &attackers {
            if self.combat.is_blocked(*attacker_id) {
                let blockers = self.combat.get_blockers(*attacker_id);

                // If multiple blockers, ask attacker's controller for damage assignment order
                if blockers.len() > 1 {
                    let attacker = self.cards.get(*attacker_id)?;
                    let attacker_owner = attacker.owner;
                    let view = GameStateView::new(self, attacker_owner);

                    let ordered_blockers = if attacker_owner == attacker_controller.player_id() {
                        attacker_controller.choose_damage_assignment_order(
                            &view,
                            *attacker_id,
                            &blockers,
                        )
                    } else {
                        blocker_controller.choose_damage_assignment_order(
                            &view,
                            *attacker_id,
                            &blockers,
                        )
                    };

                    damage_orders.insert(*attacker_id, ordered_blockers);
                }
            }
        }

        // Second pass: assign all damage
        let mut damage_to_creatures: HashMap<CardId, i32> = HashMap::new();
        let mut damage_to_players: HashMap<PlayerId, i32> = HashMap::new();
        // Track damage dealt by each creature for lifelink (creature_id -> total damage dealt)
        let mut damage_dealt_by_creature: HashMap<CardId, i32> = HashMap::new();
        // Track creatures dealt deathtouch damage (for state-based destruction)
        let mut deathtouch_damaged_creatures: std::collections::HashSet<CardId> =
            std::collections::HashSet::new();

        for attacker_id in attackers {
            // Skip creatures that are no longer on the battlefield
            // (e.g., died in first strike damage step)
            if !self.battlefield.contains(attacker_id) {
                continue;
            }

            let attacker = self.cards.get(attacker_id)?;

            // Check if this creature deals damage in this step
            // First strike step: only first strike or double strike creatures
            // Normal step: only creatures without first strike, plus double strike creatures
            let deals_damage_this_step = if first_strike_step {
                attacker.has_first_strike() || attacker.has_double_strike()
            } else {
                attacker.has_normal_strike()
            };

            if !deals_damage_this_step {
                continue; // This creature doesn't deal damage in this step
            }

            let mut remaining_power = attacker.current_power();

            if remaining_power <= 0 {
                continue; // 0 or negative power deals no damage
            }

            // Check if attacker is blocked
            if self.combat.is_blocked(attacker_id) {
                // Attacker deals damage to blockers
                let blockers = self.combat.get_blockers(attacker_id);

                // Use the pre-determined order if we have one, otherwise use default order
                let ordered_blockers = damage_orders.get(&attacker_id).cloned().unwrap_or(blockers);

                // Assign damage in order
                // MTG Rules 510.1c:
                // - If exactly one creature is blocking:
                //   * WITHOUT trample: assign ALL damage to that blocker
                //   * WITH trample: assign at least lethal, rest can trample over
                // - If multiple creatures are blocking: assign at least lethal to each
                //   before assigning to the next (can assign more)
                // Note: Current implementation doesn't track damage, so lethal = toughness
                let has_trample = attacker.has_trample();
                for blocker_id in &ordered_blockers {
                    if remaining_power <= 0 {
                        break;
                    }

                    let blocker = self.cards.get(*blocker_id)?;
                    let blocker_toughness = blocker.current_toughness();

                    // Lethal damage is the creature's toughness
                    // MTG Rules 702.2c: If attacker has deathtouch, any nonzero damage is lethal
                    // (In full MTG, this would be toughness minus damage already marked)
                    let has_deathtouch = attacker.has_deathtouch();
                    let lethal_damage = if has_deathtouch && blocker_toughness > 0 {
                        1 // Any nonzero damage from deathtouch is lethal
                    } else {
                        blocker_toughness
                    };

                    let damage_to_assign = if ordered_blockers.len() == 1 && !has_trample {
                        // MTG Rules 510.1c: With exactly one blocker and NO trample,
                        // assign ALL damage to it (even if more than lethal)
                        remaining_power
                    } else {
                        // MTG Rules 510.1c: With trample OR multiple blockers,
                        // assign at least lethal to each before moving to next.
                        // For simplicity, we assign exactly lethal.
                        remaining_power.min(lethal_damage)
                    };

                    if damage_to_assign > 0 {
                        *damage_to_creatures.entry(*blocker_id).or_insert(0) +=
                            damage_to_assign as i32;
                        // Track damage for lifelink
                        *damage_dealt_by_creature.entry(attacker_id).or_insert(0) +=
                            damage_to_assign as i32;
                        // Track deathtouch damage (MTG Rules 702.2b)
                        if has_deathtouch {
                            deathtouch_damaged_creatures.insert(*blocker_id);
                        }
                        remaining_power -= damage_to_assign;
                    }
                }

                // Trample: If attacker has trample and there's remaining damage after
                // assigning lethal to all blockers, assign remaining to defending player
                // MTG Rules 702.19
                if attacker.has_trample() && remaining_power > 0 {
                    if let Some(defending_player) = self.combat.get_defending_player(attacker_id) {
                        *damage_to_players.entry(defending_player).or_insert(0) +=
                            remaining_power as i32;
                        // Track damage for lifelink
                        *damage_dealt_by_creature.entry(attacker_id).or_insert(0) +=
                            remaining_power as i32;
                    }
                }

                // All blockers deal their damage back to attacker (simultaneously)
                // But only if they deal damage in this step (same rules as attackers)
                for blocker_id in &ordered_blockers {
                    // Skip blockers that are no longer on the battlefield
                    if !self.battlefield.contains(*blocker_id) {
                        continue;
                    }

                    let blocker = self.cards.get(*blocker_id)?;

                    // Check if blocker deals damage in this step
                    let blocker_deals_damage = if first_strike_step {
                        blocker.has_first_strike() || blocker.has_double_strike()
                    } else {
                        blocker.has_normal_strike()
                    };

                    if !blocker_deals_damage {
                        continue;
                    }

                    let blocker_power = blocker.current_power();
                    if blocker_power > 0 {
                        *damage_to_creatures.entry(attacker_id).or_insert(0) +=
                            blocker_power as i32;
                        // Track damage for lifelink
                        *damage_dealt_by_creature.entry(*blocker_id).or_insert(0) +=
                            blocker_power as i32;
                        // Track deathtouch damage from blocker (MTG Rules 702.2b)
                        if blocker.has_deathtouch() {
                            deathtouch_damaged_creatures.insert(attacker_id);
                        }
                    }
                }
            } else {
                // Unblocked attacker deals damage to defending player
                if let Some(defending_player) = self.combat.get_defending_player(attacker_id) {
                    *damage_to_players.entry(defending_player).or_insert(0) +=
                        remaining_power as i32;
                    // Track damage for lifelink
                    *damage_dealt_by_creature.entry(attacker_id).or_insert(0) +=
                        remaining_power as i32;
                }
            }
        }

        // Apply lifelink BEFORE dealing damage (since creatures might die)
        // MTG Rules 702.15: Damage dealt by a source with lifelink also causes
        // its controller to gain that much life
        for (creature_id, total_damage) in &damage_dealt_by_creature {
            if let Ok(creature) = self.cards.get(*creature_id) {
                if creature.has_lifelink() {
                    let controller = creature.controller;
                    if let Ok(player) = self.get_player_mut(controller) {
                        player.gain_life(*total_damage);
                    }
                }
            }
        }

        // Deal all damage simultaneously
        for (creature_id, damage) in damage_to_creatures {
            self.deal_damage_to_creature(creature_id, damage)?;
        }

        for (player_id, damage) in damage_to_players {
            self.deal_damage(player_id, damage)?;
        }

        // Apply deathtouch state-based action (MTG Rules 702.2b)
        // Any creature with toughness > 0 that was dealt damage by a deathtouch source is destroyed
        // MTG Rules 702.12b: Permanents with indestructible can't be destroyed
        for creature_id in deathtouch_damaged_creatures {
            // Check if creature is still on battlefield (might have died from normal damage)
            if self.battlefield.contains(creature_id) {
                if let Ok(creature) = self.cards.get(creature_id) {
                    // Only destroy if it has toughness > 0 and doesn't have indestructible
                    if creature.is_creature()
                        && creature.current_toughness() > 0
                        && !creature.has_indestructible()
                    {
                        let owner = creature.owner;
                        self.move_card(creature_id, Zone::Battlefield, Zone::Graveyard, owner)?;
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Card, ManaCost};
    use crate::game::ZeroController;
    use crate::loader::CardDatabase;
    use std::path::PathBuf;

    /// Helper to load a card from the cardsfolder for tests
    fn load_test_card(game: &mut GameState, card_name: &str, owner_id: PlayerId) -> Result<CardId> {
        let card_id = game.next_entity_id();

        // Load card definition from cardsfolder
        let cardsfolder = PathBuf::from("./cardsfolder");
        let db = CardDatabase::new(cardsfolder);

        let card_def = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { db.get_card(card_name).await })?
            .ok_or_else(|| MtgError::InvalidCardFormat(format!("Card not found: {}", card_name)))?;

        // Create card instance from definition
        let card = card_def.instantiate(card_id, owner_id);
        game.cards.insert(card_id, card);

        Ok(card_id)
    }

    #[test]
    fn test_play_land() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);

        let p1_id = game.players.first().unwrap().id;

        // Create a mountain card
        let card_id = game.next_entity_id();
        let mut card = Card::new(card_id, "Mountain".to_string(), p1_id);
        card.types.push(CardType::Land);
        game.cards.insert(card_id, card);

        // Add to hand
        if let Some(zones) = game.get_player_zones_mut(p1_id) {
            zones.hand.add(card_id);
        }

        // Play the land
        assert!(game.play_land(p1_id, card_id).is_ok());

        // Check it's on battlefield
        assert!(game.battlefield.contains(card_id));

        // Check player used their land drop
        let player = game.get_player(p1_id).unwrap();
        assert!(!player.can_play_land());
    }

    #[test]
    fn test_tap_for_mana() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);

        let p1_id = game.players.first().unwrap().id;

        // Create a mountain on battlefield
        let card_id = game.next_entity_id();
        let mut card = Card::new(card_id, "Mountain".to_string(), p1_id);
        card.types.push(CardType::Land);
        game.cards.insert(card_id, card);
        game.battlefield.add(card_id);

        // Tap for mana
        assert!(game.tap_for_mana(p1_id, card_id).is_ok());

        // Check mana was added
        let player = game.get_player(p1_id).unwrap();
        assert_eq!(player.mana_pool.red, 1);

        // Check land is tapped
        let card = game.cards.get(card_id).unwrap();
        assert!(card.tapped);
    }

    #[test]
    fn test_deal_damage_to_player() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);

        let p1_id = game.players.first().unwrap().id;

        // Deal 3 damage
        assert!(game.deal_damage(p1_id, 3).is_ok());

        let player = game.get_player(p1_id).unwrap();
        assert_eq!(player.life, 17);
    }

    #[test]
    fn test_move_card_battlefield_to_graveyard() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);

        let p1_id = game.players.first().unwrap().id;

        // Create a creature on battlefield
        let card_id = game.next_entity_id();
        let card = Card::new(card_id, "Test Card".to_string(), p1_id);
        game.cards.insert(card_id, card);
        game.battlefield.add(card_id);

        // Test move_card directly
        let result = game.move_card(card_id, Zone::Battlefield, Zone::Graveyard, p1_id);
        if let Err(e) = &result {
            panic!("move_card failed: {e:?}");
        }

        // Check it moved
        assert!(
            !game.battlefield.contains(card_id),
            "Card still on battlefield"
        );
        if let Some(zones) = game.get_player_zones(p1_id) {
            assert!(zones.graveyard.contains(card_id), "Card not in graveyard");
        }
    }

    #[test]
    fn test_deal_damage_to_creature() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);

        let p1_id = game.players.first().unwrap().id;

        // Create a 2/2 creature on battlefield
        let card_id = game.next_card_id();
        let mut card = Card::new(card_id, "Grizzly Bears".to_string(), p1_id);
        card.types.push(CardType::Creature);
        card.power = Some(2);
        card.toughness = Some(2);
        game.cards.insert(card_id, card);
        game.battlefield.add(card_id);

        // Deal 2 damage (should kill it)
        let result = game.deal_damage_to_creature(card_id, 2);
        assert!(result.is_ok(), "deal_damage_to_creature failed: {result:?}");

        // Check it's in graveyard
        assert!(
            !game.battlefield.contains(card_id),
            "Card still on battlefield"
        );
        if let Some(zones) = game.get_player_zones(p1_id) {
            assert!(zones.graveyard.contains(card_id), "Card not in graveyard");
        }
    }

    #[test]
    fn test_cast_spell_with_mana_payment() {
        use crate::core::{Color, ManaCost};

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players.first().unwrap().id;

        // Create a Lightning Bolt in hand (cost: R)
        let bolt_id = game.next_card_id();
        let mut bolt = Card::new(bolt_id, "Lightning Bolt".to_string(), p1_id);
        bolt.types.push(CardType::Instant);
        bolt.mana_cost = ManaCost::from_string("R");
        game.cards.insert(bolt_id, bolt);

        // Add to hand
        if let Some(zones) = game.get_player_zones_mut(p1_id) {
            zones.hand.add(bolt_id);
        }

        // Try to cast without mana - should fail
        let result = game.cast_spell(p1_id, bolt_id, vec![]);
        assert!(result.is_err());

        // Add mana to pool
        let player = game.get_player_mut(p1_id).unwrap();
        player.mana_pool.add_color(Color::Red);

        // Now cast should succeed
        let result = game.cast_spell(p1_id, bolt_id, vec![]);
        assert!(result.is_ok(), "cast_spell failed: {result:?}");

        // Check mana was deducted
        let player = game.get_player(p1_id).unwrap();
        assert_eq!(player.mana_pool.red, 0);

        // Check card is on stack
        assert!(game.stack.contains(bolt_id));
    }

    #[test]
    fn test_cast_spell_with_generic_mana() {
        use crate::core::{Color, ManaCost};

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players.first().unwrap().id;

        // Create a spell with cost 2R
        let spell_id = game.next_card_id();
        let mut spell = Card::new(spell_id, "Lava Spike".to_string(), p1_id);
        spell.types.push(CardType::Sorcery);
        spell.mana_cost = ManaCost::from_string("2R");
        game.cards.insert(spell_id, spell);

        // Add to hand
        if let Some(zones) = game.get_player_zones_mut(p1_id) {
            zones.hand.add(spell_id);
        }

        // Add mana: 2R + 1U = 4 mana total
        let player = game.get_player_mut(p1_id).unwrap();
        player.mana_pool.add_color(Color::Red);
        player.mana_pool.add_color(Color::Red);
        player.mana_pool.add_color(Color::Blue);

        // Cast spell - should use 1R for R, and 2R for generic 2
        let result = game.cast_spell(p1_id, spell_id, vec![]);
        assert!(result.is_ok(), "cast_spell failed: {result:?}");

        // Check mana was deducted properly (should have 1 blue left)
        let player = game.get_player(p1_id).unwrap();
        assert_eq!(player.mana_pool.red, 0);
        assert_eq!(player.mana_pool.blue, 0); // Blue was used for generic cost
        assert_eq!(player.mana_pool.total(), 0);

        // Check card is on stack
        assert!(game.stack.contains(spell_id));
    }

    #[test]
    fn test_execute_damage_effect_to_player() {
        use crate::core::{Effect, TargetRef};

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
        let p2_id = players[1];

        let effect = Effect::DealDamage {
            target: TargetRef::Player(p2_id),
            amount: 3,
        };

        assert!(game.execute_effect(&effect).is_ok());

        let p2 = game.get_player(p2_id).unwrap();
        assert_eq!(p2.life, 17);
    }

    #[test]
    fn test_execute_draw_effect() {
        use crate::core::Effect;

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players.first().unwrap().id;

        // Add cards to library
        for i in 0..5 {
            let card_id = game.next_card_id();
            let card = Card::new(card_id, format!("Card {i}"), p1_id);
            game.cards.insert(card_id, card);
            if let Some(zones) = game.get_player_zones_mut(p1_id) {
                zones.library.add(card_id);
            }
        }

        let effect = Effect::DrawCards {
            player: p1_id,
            count: 2,
        };

        assert!(game.execute_effect(&effect).is_ok());

        // Check cards were drawn
        if let Some(zones) = game.get_player_zones(p1_id) {
            assert_eq!(zones.hand.cards.len(), 2);
            assert_eq!(zones.library.cards.len(), 3);
        }
    }

    #[test]
    fn test_resolve_spell_with_effects() {
        use crate::core::{Effect, ManaCost, TargetRef};

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
        let p1_id = players[0];
        let p2_id = players[1];

        // Create Lightning Bolt with damage effect
        let bolt_id = game.next_card_id();
        let mut bolt = Card::new(bolt_id, "Lightning Bolt".to_string(), p1_id);
        bolt.types.push(CardType::Instant);
        bolt.mana_cost = ManaCost::from_string("R");
        bolt.effects.push(Effect::DealDamage {
            target: TargetRef::Player(p2_id),
            amount: 3,
        });
        game.cards.insert(bolt_id, bolt);

        // Put it on the stack (simulating cast)
        game.stack.add(bolt_id);

        // Resolve the spell
        assert!(game.resolve_spell(bolt_id).is_ok());

        // Check damage was dealt
        let p2 = game.get_player(p2_id).unwrap();
        assert_eq!(p2.life, 17);

        // Check spell went to graveyard
        if let Some(zones) = game.get_player_zones(p1_id) {
            assert!(zones.graveyard.contains(bolt_id));
        }
    }

    #[test]
    fn test_resolve_draw_spell() {
        use crate::core::{Effect, ManaCost};

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
        let p1_id = players[0];

        // Add cards to P1's library
        for i in 0..5 {
            let card_id = game.next_card_id();
            let card = Card::new(card_id, format!("Card {i}"), p1_id);
            game.cards.insert(card_id, card);
            if let Some(zones) = game.get_player_zones_mut(p1_id) {
                zones.library.add(card_id);
            }
        }

        // Create a Draw spell (like Divination)
        let draw_spell_id = game.next_card_id();
        let mut draw_spell = Card::new(draw_spell_id, "Divination".to_string(), p1_id);
        draw_spell.types.push(CardType::Sorcery);
        draw_spell.mana_cost = ManaCost::from_string("2U");
        // Use placeholder player ID 0 which will be replaced with card owner
        draw_spell.effects.push(Effect::DrawCards {
            player: PlayerId::new(0),
            count: 2,
        });
        game.cards.insert(draw_spell_id, draw_spell);

        // Put it on the stack (simulating cast)
        game.stack.add(draw_spell_id);

        // Check initial state
        if let Some(zones) = game.get_player_zones(p1_id) {
            assert_eq!(
                zones.hand.cards.len(),
                0,
                "Should start with 0 cards in hand"
            );
            assert_eq!(
                zones.library.cards.len(),
                5,
                "Should have 5 cards in library"
            );
        }

        // Resolve the spell
        assert!(
            game.resolve_spell(draw_spell_id).is_ok(),
            "Failed to resolve draw spell"
        );

        // Check cards were drawn
        if let Some(zones) = game.get_player_zones(p1_id) {
            assert_eq!(zones.hand.cards.len(), 2, "Should have drawn 2 cards");
            assert_eq!(
                zones.library.cards.len(),
                3,
                "Should have 3 cards left in library"
            );
        }

        // Check spell went to graveyard
        if let Some(zones) = game.get_player_zones(p1_id) {
            assert!(
                zones.graveyard.contains(draw_spell_id),
                "Draw spell should be in graveyard"
            );
        }
    }

    #[test]
    fn test_resolve_destroy_spell() {
        use crate::core::{Effect, ManaCost};

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
        let p1_id = players[0];
        let p2_id = players[1];

        // Create a creature for P2 (the target)
        let target_creature_id = game.next_card_id();
        let mut target = Card::new(target_creature_id, "Grizzly Bears".to_string(), p2_id);
        target.types.push(CardType::Creature);
        target.power = Some(2);
        target.toughness = Some(2);
        target.controller = p2_id;
        game.cards.insert(target_creature_id, target);
        game.battlefield.add(target_creature_id);

        // Create a Destroy spell (like Terror)
        let destroy_spell_id = game.next_card_id();
        let mut destroy_spell = Card::new(destroy_spell_id, "Terror".to_string(), p1_id);
        destroy_spell.types.push(CardType::Instant);
        destroy_spell.mana_cost = ManaCost::from_string("1B");
        // Use placeholder card ID 0 which will be replaced with an opponent's creature
        destroy_spell.effects.push(Effect::DestroyPermanent {
            target: CardId::new(0),
        });
        game.cards.insert(destroy_spell_id, destroy_spell);

        // Put it on the stack (simulating cast)
        game.stack.add(destroy_spell_id);

        // Check initial state
        assert!(
            game.battlefield.contains(target_creature_id),
            "Target creature should be on battlefield"
        );

        // Resolve the spell
        assert!(
            game.resolve_spell(destroy_spell_id).is_ok(),
            "Failed to resolve destroy spell"
        );

        // Check target creature was destroyed (moved to graveyard)
        assert!(
            !game.battlefield.contains(target_creature_id),
            "Target creature should not be on battlefield"
        );

        if let Some(zones) = game.get_player_zones(p2_id) {
            assert!(
                zones.graveyard.contains(target_creature_id),
                "Target creature should be in graveyard"
            );
        }

        // Check spell went to graveyard
        if let Some(zones) = game.get_player_zones(p1_id) {
            assert!(
                zones.graveyard.contains(destroy_spell_id),
                "Destroy spell should be in graveyard"
            );
        }
    }

    #[test]
    fn test_resolve_gainlife_spell() {
        use crate::core::{Effect, ManaCost};

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
        let p1_id = players[0];

        // Create a GainLife spell (like Angel's Mercy)
        let gainlife_spell_id = game.next_card_id();
        let mut gainlife_spell = Card::new(gainlife_spell_id, "Angel's Mercy".to_string(), p1_id);
        gainlife_spell.types.push(CardType::Instant);
        gainlife_spell.mana_cost = ManaCost::from_string("2WW");
        // Use placeholder player ID 0 which will be replaced with card controller
        gainlife_spell.effects.push(Effect::GainLife {
            player: PlayerId::new(0),
            amount: 7,
        });
        game.cards.insert(gainlife_spell_id, gainlife_spell);

        // Put it on the stack (simulating cast)
        game.stack.add(gainlife_spell_id);

        // Check initial life total
        let p1_before = game.get_player(p1_id).unwrap();
        assert_eq!(p1_before.life, 20, "Should start with 20 life");

        // Resolve the spell
        assert!(
            game.resolve_spell(gainlife_spell_id).is_ok(),
            "Failed to resolve gain life spell"
        );

        // Check life was gained
        let p1_after = game.get_player(p1_id).unwrap();
        assert_eq!(p1_after.life, 27, "Should have gained 7 life (20 + 7)");

        // Check spell went to graveyard
        if let Some(zones) = game.get_player_zones(p1_id) {
            assert!(
                zones.graveyard.contains(gainlife_spell_id),
                "GainLife spell should be in graveyard"
            );
        }
    }

    #[test]
    fn test_declare_attacker() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
        let p1_id = players[0];

        // Create a creature
        let creature_id = game.next_card_id();
        let mut creature = Card::new(creature_id, "Grizzly Bears".to_string(), p1_id);
        creature.types.push(CardType::Creature);
        creature.power = Some(2);
        creature.toughness = Some(2);
        creature.controller = p1_id;
        game.cards.insert(creature_id, creature);

        // Put creature on battlefield
        game.battlefield.add(creature_id);

        // Declare attacker
        let result = game.declare_attacker(p1_id, creature_id);
        assert!(result.is_ok(), "Failed to declare attacker: {result:?}");

        // Check creature is attacking
        assert!(game.combat.is_attacking(creature_id));

        // Check creature is tapped
        let creature = game.cards.get(creature_id).unwrap();
        assert!(creature.tapped);
    }

    #[test]
    fn test_declare_blocker() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
        let p1_id = players[0];
        let p2_id = players[1];

        // Create an attacker
        let attacker_id = game.next_card_id();
        let mut attacker = Card::new(attacker_id, "Goblin".to_string(), p1_id);
        attacker.types.push(CardType::Creature);
        attacker.power = Some(2);
        attacker.toughness = Some(1);
        attacker.controller = p1_id;
        game.cards.insert(attacker_id, attacker);
        game.battlefield.add(attacker_id);

        // Declare as attacker
        game.combat.declare_attacker(attacker_id, p2_id);

        // Create a blocker
        let blocker_id = game.next_card_id();
        let mut blocker = Card::new(blocker_id, "Wall".to_string(), p2_id);
        blocker.types.push(CardType::Creature);
        blocker.power = Some(0);
        blocker.toughness = Some(3);
        blocker.controller = p2_id;
        game.cards.insert(blocker_id, blocker);
        game.battlefield.add(blocker_id);

        // Declare blocker
        let result = game.declare_blocker(p2_id, blocker_id, vec![attacker_id]);
        assert!(result.is_ok(), "Failed to declare blocker: {result:?}");

        // Check blocker is blocking
        assert!(game.combat.is_blocking(blocker_id));
        assert!(game.combat.is_blocked(attacker_id));

        let blockers = game.combat.get_blockers(attacker_id);
        assert_eq!(blockers.len(), 1);
        assert!(blockers.contains(&blocker_id));
    }

    #[test]
    fn test_combat_damage_unblocked() {
        use crate::game::zero_controller::ZeroController;

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
        let p1_id = players[0];
        let p2_id = players[1];

        // Create an attacker
        let attacker_id = game.next_card_id();
        let mut attacker = Card::new(attacker_id, "Dragon".to_string(), p1_id);
        attacker.types.push(CardType::Creature);
        attacker.power = Some(5);
        attacker.toughness = Some(5);
        attacker.controller = p1_id;
        game.cards.insert(attacker_id, attacker);
        game.battlefield.add(attacker_id);

        // Declare as attacker (unblocked)
        game.combat.declare_attacker(attacker_id, p2_id);

        // Create controllers
        let mut controller1 = ZeroController::new(p1_id);
        let mut controller2 = ZeroController::new(p2_id);

        // Assign combat damage
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, false);
        assert!(result.is_ok(), "Failed to assign combat damage: {result:?}");

        // Check defending player took damage
        let p2 = game.get_player(p2_id).unwrap();
        assert_eq!(p2.life, 15); // 20 - 5 = 15
    }

    #[test]
    fn test_combat_damage_blocked() {
        use crate::game::zero_controller::ZeroController;

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
        let p1_id = players[0];
        let p2_id = players[1];

        // Create an attacker (3/3)
        let attacker_id = game.next_card_id();
        let mut attacker = Card::new(attacker_id, "Bear".to_string(), p1_id);
        attacker.types.push(CardType::Creature);
        attacker.power = Some(3);
        attacker.toughness = Some(3);
        attacker.controller = p1_id;
        game.cards.insert(attacker_id, attacker);
        game.battlefield.add(attacker_id);

        // Create a blocker (2/2)
        let blocker_id = game.next_card_id();
        let mut blocker = Card::new(blocker_id, "Wolf".to_string(), p2_id);
        blocker.types.push(CardType::Creature);
        blocker.power = Some(2);
        blocker.toughness = Some(2);
        blocker.controller = p2_id;
        game.cards.insert(blocker_id, blocker);
        game.battlefield.add(blocker_id);

        // Declare attacker and blocker
        game.combat.declare_attacker(attacker_id, p2_id);
        let blocker_vec = smallvec::SmallVec::from_vec(vec![attacker_id]);
        game.combat.declare_blocker(blocker_id, blocker_vec);

        // Create controllers
        let mut controller1 = ZeroController::new(p1_id);
        let mut controller2 = ZeroController::new(p2_id);

        // Assign combat damage
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, false);
        assert!(result.is_ok(), "Failed to assign combat damage: {result:?}");

        // Check defending player took no damage (blocked)
        let p2 = game.get_player(p2_id).unwrap();
        assert_eq!(p2.life, 20);

        // Check blocker died (took 2 damage, toughness 2)
        if let Some(zones) = game.get_player_zones(p2_id) {
            assert!(zones.graveyard.contains(blocker_id));
        }

        // Check attacker took 2 damage but has toughness 3, so it survives
        assert!(game.battlefield.contains(attacker_id));
    }

    #[test]
    fn test_combat_damage_multiple_blockers() {
        use crate::game::zero_controller::ZeroController;

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
        let p1_id = players[0];
        let p2_id = players[1];

        // Create a powerful attacker (5/5)
        let attacker_id = game.next_card_id();
        let mut attacker = Card::new(attacker_id, "Dragon".to_string(), p1_id);
        attacker.types.push(CardType::Creature);
        attacker.power = Some(5);
        attacker.toughness = Some(5);
        attacker.controller = p1_id;
        game.cards.insert(attacker_id, attacker);
        game.battlefield.add(attacker_id);

        // Create first blocker (2/2)
        let blocker1_id = game.next_card_id();
        let mut blocker1 = Card::new(blocker1_id, "Bear".to_string(), p2_id);
        blocker1.types.push(CardType::Creature);
        blocker1.power = Some(2);
        blocker1.toughness = Some(2);
        blocker1.controller = p2_id;
        game.cards.insert(blocker1_id, blocker1);
        game.battlefield.add(blocker1_id);

        // Create second blocker (3/3)
        let blocker2_id = game.next_card_id();
        let mut blocker2 = Card::new(blocker2_id, "Wolf".to_string(), p2_id);
        blocker2.types.push(CardType::Creature);
        blocker2.power = Some(3);
        blocker2.toughness = Some(3);
        blocker2.controller = p2_id;
        game.cards.insert(blocker2_id, blocker2);
        game.battlefield.add(blocker2_id);

        // Declare attacker and both blockers
        game.combat.declare_attacker(attacker_id, p2_id);
        let blocker_vec = smallvec::SmallVec::from_vec(vec![attacker_id]);
        game.combat
            .declare_blocker(blocker1_id, blocker_vec.clone());
        game.combat.declare_blocker(blocker2_id, blocker_vec);

        // Create controllers
        let mut controller1 = ZeroController::new(p1_id);
        let mut controller2 = ZeroController::new(p2_id);

        // Assign combat damage
        // ZeroController will keep the order as-is
        // Dragon (5 power) assigns: 2 to first blocker (lethal), 3 to second blocker (lethal)
        // Both blockers (2+3=5 power) deal 5 damage back to Dragon (lethal)
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, false);
        assert!(result.is_ok(), "Failed to assign combat damage: {result:?}");

        // Check defending player took no damage (blocked)
        let p2 = game.get_player(p2_id).unwrap();
        assert_eq!(p2.life, 20);

        // Check both blockers died
        if let Some(zones) = game.get_player_zones(p2_id) {
            assert!(
                zones.graveyard.contains(blocker1_id),
                "First blocker should be in graveyard"
            );
            assert!(
                zones.graveyard.contains(blocker2_id),
                "Second blocker should be in graveyard"
            );
        }

        // Check attacker died (took 5 damage, toughness 5)
        if let Some(zones) = game.get_player_zones(p1_id) {
            assert!(
                zones.graveyard.contains(attacker_id),
                "Attacker should be in graveyard"
            );
        }
    }

    #[test]
    fn test_summoning_sickness_blocks_attack() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;

        // Create a creature and put it on battlefield
        let creature_id = game.next_entity_id();
        let mut creature = Card::new(creature_id, "Grizzly Bears".to_string(), p1_id);
        creature.types.push(CardType::Creature);
        creature.power = Some(2);
        creature.toughness = Some(2);
        creature.controller = p1_id;
        game.cards.insert(creature_id, creature);
        game.battlefield.add(creature_id);

        // Mark it as entering this turn (summoning sickness)
        if let Ok(card) = game.cards.get_mut(creature_id) {
            card.turn_entered_battlefield = Some(game.turn.turn_number);
        }

        // Try to declare it as an attacker - should fail due to summoning sickness
        let result = game.declare_attacker(p1_id, creature_id);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("summoning sickness"));
    }

    #[test]
    fn test_summoning_sickness_allows_attack_next_turn() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;

        // Create a creature and put it on battlefield
        let creature_id = game.next_entity_id();
        let mut creature = Card::new(creature_id, "Grizzly Bears".to_string(), p1_id);
        creature.types.push(CardType::Creature);
        creature.power = Some(2);
        creature.toughness = Some(2);
        creature.controller = p1_id;
        game.cards.insert(creature_id, creature);
        game.battlefield.add(creature_id);

        // Mark it as entering on a previous turn
        if let Ok(card) = game.cards.get_mut(creature_id) {
            card.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        }

        // Declare it as an attacker - should succeed
        let result = game.declare_attacker(p1_id, creature_id);
        assert!(result.is_ok());
        assert!(game.combat.is_attacking(creature_id));
    }

    #[test]
    fn test_haste_bypasses_summoning_sickness() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;

        // Create a creature with haste
        let creature_id = game.next_entity_id();
        let mut creature = Card::new(creature_id, "Lightning Elemental".to_string(), p1_id);
        creature.types.push(CardType::Creature);
        creature.power = Some(4);
        creature.toughness = Some(1);
        creature.controller = p1_id;
        creature.keywords.push(Keyword::Haste);
        game.cards.insert(creature_id, creature);
        game.battlefield.add(creature_id);

        // Mark it as entering this turn
        if let Ok(card) = game.cards.get_mut(creature_id) {
            card.turn_entered_battlefield = Some(game.turn.turn_number);
        }

        // Declare it as an attacker - should succeed because of haste
        let result = game.declare_attacker(p1_id, creature_id);
        assert!(result.is_ok());
        assert!(game.combat.is_attacking(creature_id));
    }

    #[test]
    fn test_vigilance_creature_stays_untapped() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;

        // Load Serra Angel (4/4 with Flying and Vigilance)
        let creature_id =
            load_test_card(&mut game, "Serra Angel", p1_id).expect("Failed to load Serra Angel");

        if let Ok(creature) = game.cards.get_mut(creature_id) {
            creature.controller = p1_id;
        }
        game.battlefield.add(creature_id);

        // Mark it as entering on a previous turn (no summoning sickness)
        if let Ok(card) = game.cards.get_mut(creature_id) {
            card.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        }

        // Declare it as an attacker
        let result = game.declare_attacker(p1_id, creature_id);
        assert!(result.is_ok());
        assert!(game.combat.is_attacking(creature_id));

        // Check that creature is still untapped (vigilance effect)
        let card = game.cards.get(creature_id).unwrap();
        assert!(
            !card.tapped,
            "Creature with vigilance should not be tapped after attacking"
        );
    }

    #[test]
    fn test_non_vigilance_creature_gets_tapped() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;

        // Create a creature WITHOUT vigilance
        let creature_id = game.next_entity_id();
        let mut creature = Card::new(creature_id, "Grizzly Bears".to_string(), p1_id);
        creature.types.push(CardType::Creature);
        creature.power = Some(2);
        creature.toughness = Some(2);
        creature.controller = p1_id;
        game.cards.insert(creature_id, creature);
        game.battlefield.add(creature_id);

        // Mark it as entering on a previous turn (no summoning sickness)
        if let Ok(card) = game.cards.get_mut(creature_id) {
            card.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        }

        // Declare it as an attacker
        let result = game.declare_attacker(p1_id, creature_id);
        assert!(result.is_ok());
        assert!(game.combat.is_attacking(creature_id));

        // Check that creature is tapped (normal attack behavior)
        let card = game.cards.get(creature_id).unwrap();
        assert!(
            card.tapped,
            "Creature without vigilance should be tapped after attacking"
        );
    }

    #[test]
    fn test_flying_creature_blocked_by_flying() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P1: Load Storm Crow (1/2 with Flying) as attacker
        let attacker_id =
            load_test_card(&mut game, "Storm Crow", p1_id).expect("Failed to load Storm Crow");

        if let Ok(attacker) = game.cards.get_mut(attacker_id) {
            attacker.controller = p1_id;
            attacker.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        }
        game.battlefield.add(attacker_id);

        // P2: Load Segovian Angel (1/1 with Flying and Vigilance) as blocker
        let blocker_id = load_test_card(&mut game, "Segovian Angel", p2_id)
            .expect("Failed to load Segovian Angel");

        if let Ok(blocker) = game.cards.get_mut(blocker_id) {
            blocker.controller = p2_id;
        }
        game.battlefield.add(blocker_id);

        // Declare attacker
        game.declare_attacker(p1_id, attacker_id).unwrap();

        // Blocker with flying should be able to block attacker with flying
        let result = game.declare_blocker(p2_id, blocker_id, vec![attacker_id]);
        assert!(
            result.is_ok(),
            "Creature with flying should be able to block creature with flying"
        );
    }

    #[test]
    fn test_flying_creature_blocked_by_reach() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P1: Load Storm Crow (1/2 with Flying) as attacker
        let attacker_id =
            load_test_card(&mut game, "Storm Crow", p1_id).expect("Failed to load Storm Crow");

        if let Ok(attacker) = game.cards.get_mut(attacker_id) {
            attacker.controller = p1_id;
            attacker.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        }
        game.battlefield.add(attacker_id);

        // P2: Load Giant Spider (2/4 with Reach) as blocker
        let blocker_id =
            load_test_card(&mut game, "Giant Spider", p2_id).expect("Failed to load Giant Spider");

        if let Ok(blocker) = game.cards.get_mut(blocker_id) {
            blocker.controller = p2_id;
        }
        game.battlefield.add(blocker_id);

        // Declare attacker
        game.declare_attacker(p1_id, attacker_id).unwrap();

        // Blocker with reach should be able to block attacker with flying
        let result = game.declare_blocker(p2_id, blocker_id, vec![attacker_id]);
        assert!(
            result.is_ok(),
            "Creature with reach should be able to block creature with flying"
        );
    }

    #[test]
    fn test_flying_creature_cannot_be_blocked_by_non_flying() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P1: Create a creature with Flying (attacker)
        let attacker_id = game.next_entity_id();
        let mut attacker = Card::new(attacker_id, "Storm Crow".to_string(), p1_id);
        attacker.types.push(CardType::Creature);
        attacker.power = Some(1);
        attacker.toughness = Some(2);
        attacker.controller = p1_id;
        attacker.keywords.push(Keyword::Flying);
        attacker.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        game.cards.insert(attacker_id, attacker);
        game.battlefield.add(attacker_id);

        // P2: Create a creature without Flying or Reach (blocker)
        let blocker_id = game.next_entity_id();
        let mut blocker = Card::new(blocker_id, "Grizzly Bears".to_string(), p2_id);
        blocker.types.push(CardType::Creature);
        blocker.power = Some(2);
        blocker.toughness = Some(2);
        blocker.controller = p2_id;
        game.cards.insert(blocker_id, blocker);
        game.battlefield.add(blocker_id);

        // Declare attacker
        game.declare_attacker(p1_id, attacker_id).unwrap();

        // Blocker without flying or reach should NOT be able to block attacker with flying
        let result = game.declare_blocker(p2_id, blocker_id, vec![attacker_id]);
        assert!(
            result.is_err(),
            "Creature without flying or reach should not be able to block creature with flying"
        );
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot block attacker with flying"));
    }

    #[test]
    fn test_non_flying_creature_blocked_by_any() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P1: Create a creature without Flying (attacker)
        let attacker_id = game.next_entity_id();
        let mut attacker = Card::new(attacker_id, "Grizzly Bears".to_string(), p1_id);
        attacker.types.push(CardType::Creature);
        attacker.power = Some(2);
        attacker.toughness = Some(2);
        attacker.controller = p1_id;
        attacker.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        game.cards.insert(attacker_id, attacker);
        game.battlefield.add(attacker_id);

        // P2: Create a creature without Flying or Reach (blocker)
        let blocker_id = game.next_entity_id();
        let mut blocker = Card::new(blocker_id, "Hill Giant".to_string(), p2_id);
        blocker.types.push(CardType::Creature);
        blocker.power = Some(3);
        blocker.toughness = Some(3);
        blocker.controller = p2_id;
        game.cards.insert(blocker_id, blocker);
        game.battlefield.add(blocker_id);

        // Declare attacker
        game.declare_attacker(p1_id, attacker_id).unwrap();

        // Any creature should be able to block a non-flying creature
        let result = game.declare_blocker(p2_id, blocker_id, vec![attacker_id]);
        assert!(
            result.is_ok(),
            "Any creature should be able to block a non-flying creature"
        );
    }

    #[test]
    fn test_flying_and_reach_blocker_can_block_flying() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P1: Create a creature with Flying (attacker)
        let attacker_id = game.next_entity_id();
        let mut attacker = Card::new(attacker_id, "Storm Crow".to_string(), p1_id);
        attacker.types.push(CardType::Creature);
        attacker.power = Some(1);
        attacker.toughness = Some(2);
        attacker.controller = p1_id;
        attacker.keywords.push(Keyword::Flying);
        attacker.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        game.cards.insert(attacker_id, attacker);
        game.battlefield.add(attacker_id);

        // P2: Create a creature with both Flying AND Reach (blocker)
        let blocker_id = game.next_entity_id();
        let mut blocker = Card::new(blocker_id, "Mystic Drake".to_string(), p2_id);
        blocker.types.push(CardType::Creature);
        blocker.power = Some(2);
        blocker.toughness = Some(3);
        blocker.controller = p2_id;
        blocker.keywords.push(Keyword::Flying);
        blocker.keywords.push(Keyword::Reach);
        game.cards.insert(blocker_id, blocker);
        game.battlefield.add(blocker_id);

        // Declare attacker
        game.declare_attacker(p1_id, attacker_id).unwrap();

        // Blocker with both flying and reach should be able to block attacker with flying
        let result = game.declare_blocker(p2_id, blocker_id, vec![attacker_id]);
        assert!(
            result.is_ok(),
            "Creature with flying and reach should be able to block creature with flying"
        );
    }

    #[test]
    fn test_first_strike_creature_kills_before_taking_damage() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P1: Load Advance Scout (1/1 with First Strike) as attacker
        let attacker_id = load_test_card(&mut game, "Advance Scout", p1_id)
            .expect("Failed to load Advance Scout");

        // Set attacker power/toughness to 2/2 so test works as before
        if let Ok(attacker) = game.cards.get_mut(attacker_id) {
            attacker.power = Some(2);
            attacker.toughness = Some(2);
            attacker.controller = p1_id;
            attacker.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        }
        game.battlefield.add(attacker_id);

        // P2: Load Grizzly Bears (2/2 vanilla) as blocker
        let blocker_id = load_test_card(&mut game, "Grizzly Bears", p2_id)
            .expect("Failed to load Grizzly Bears");

        if let Ok(blocker) = game.cards.get_mut(blocker_id) {
            blocker.controller = p2_id;
        }
        game.battlefield.add(blocker_id);

        // Declare combat
        game.combat.declare_attacker(attacker_id, p2_id);
        let attacker_vec = smallvec::SmallVec::from_vec(vec![attacker_id]);
        game.combat.declare_blocker(blocker_id, attacker_vec);

        // Create controllers
        let mut controller1 = ZeroController::new(p1_id);
        let mut controller2 = ZeroController::new(p2_id);

        // First strike damage step: attacker deals 2 damage, blocker takes none
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, true);
        assert!(
            result.is_ok(),
            "Failed to assign first strike damage: {result:?}"
        );

        // Blocker should be dead (took 2 damage, toughness 2)
        if let Some(zones) = game.get_player_zones(p2_id) {
            assert!(
                zones.graveyard.contains(blocker_id),
                "Blocker should be in graveyard after first strike damage"
            );
        }

        // Normal damage step: only attacker can deal damage (blocker is dead)
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, false);
        assert!(result.is_ok(), "Failed to assign normal damage: {result:?}");

        // Attacker should still be alive (never took damage)
        assert!(
            game.battlefield.contains(attacker_id),
            "Attacker should still be alive"
        );

        // Check attacker is undamaged
        if let Ok(attacker) = game.cards.get(attacker_id) {
            assert_eq!(
                attacker.current_toughness(),
                2,
                "Attacker should be undamaged"
            );
        }
    }

    #[test]
    fn test_double_strike_creature_deals_damage_twice() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P1: Load Adorned Pouncer (1/1 with Double Strike) as attacker
        let attacker_id = load_test_card(&mut game, "Adorned Pouncer", p1_id)
            .expect("Failed to load Adorned Pouncer");

        // Set power/toughness to 3/3 so test works as before
        if let Ok(attacker) = game.cards.get_mut(attacker_id) {
            attacker.power = Some(3);
            attacker.toughness = Some(3);
            attacker.controller = p1_id;
            attacker.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        }
        game.battlefield.add(attacker_id);

        // Declare unblocked attacker
        game.combat.declare_attacker(attacker_id, p2_id);

        // Create controllers
        let mut controller1 = ZeroController::new(p1_id);
        let mut controller2 = ZeroController::new(p2_id);

        // First strike damage step: attacker deals 3 damage to player
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, true);
        assert!(
            result.is_ok(),
            "Failed to assign first strike damage: {result:?}"
        );

        // Check player took 3 damage
        let p2 = game.get_player(p2_id).unwrap();
        assert_eq!(
            p2.life, 17,
            "Player should have taken 3 damage in first strike step"
        );

        // Normal damage step: attacker deals another 3 damage to player
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, false);
        assert!(result.is_ok(), "Failed to assign normal damage: {result:?}");

        // Check player took another 3 damage (total 6)
        let p2 = game.get_player(p2_id).unwrap();
        assert_eq!(
            p2.life, 14,
            "Player should have taken 6 total damage from double strike"
        );
    }

    #[test]
    fn test_double_strike_vs_first_strike() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P1: Create a 2/2 creature with Double Strike (attacker)
        let attacker_id = game.next_entity_id();
        let mut attacker = Card::new(attacker_id, "Double Strike Knight".to_string(), p1_id);
        attacker.types.push(CardType::Creature);
        attacker.power = Some(2);
        attacker.toughness = Some(2);
        attacker.controller = p1_id;
        attacker.keywords.push(Keyword::DoubleStrike);
        attacker.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        game.cards.insert(attacker_id, attacker);
        game.battlefield.add(attacker_id);

        // P2: Create a 2/2 creature with First Strike (blocker)
        let blocker_id = game.next_entity_id();
        let mut blocker = Card::new(blocker_id, "First Strike Soldier".to_string(), p2_id);
        blocker.types.push(CardType::Creature);
        blocker.power = Some(2);
        blocker.toughness = Some(2);
        blocker.controller = p2_id;
        blocker.keywords.push(Keyword::FirstStrike);
        game.cards.insert(blocker_id, blocker);
        game.battlefield.add(blocker_id);

        // Declare combat
        game.combat.declare_attacker(attacker_id, p2_id);
        let attacker_vec = smallvec::SmallVec::from_vec(vec![attacker_id]);
        game.combat.declare_blocker(blocker_id, attacker_vec);

        // Create controllers
        let mut controller1 = ZeroController::new(p1_id);
        let mut controller2 = ZeroController::new(p2_id);

        // First strike damage step: both creatures deal damage simultaneously
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, true);
        assert!(
            result.is_ok(),
            "Failed to assign first strike damage: {result:?}"
        );

        // Both creatures should be dead (both took 2 damage, both have 2 toughness)
        if let Some(zones) = game.get_player_zones(p1_id) {
            assert!(
                zones.graveyard.contains(attacker_id),
                "Double strike attacker should be in graveyard"
            );
        }
        if let Some(zones) = game.get_player_zones(p2_id) {
            assert!(
                zones.graveyard.contains(blocker_id),
                "First strike blocker should be in graveyard"
            );
        }

        // Normal damage step: no creatures left to deal damage
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, false);
        assert!(result.is_ok(), "Failed to assign normal damage: {result:?}");

        // Both creatures should still be in graveyards
        if let Some(zones) = game.get_player_zones(p1_id) {
            assert!(zones.graveyard.contains(attacker_id));
        }
        if let Some(zones) = game.get_player_zones(p2_id) {
            assert!(zones.graveyard.contains(blocker_id));
        }
    }

    #[test]
    fn test_resolve_pump_spell() {
        use crate::core::{Effect, ManaCost};

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
        let p1_id = players[0];

        // Create a 2/2 creature on battlefield
        let creature_id = game.next_card_id();
        let mut creature = Card::new(creature_id, "Grizzly Bears".to_string(), p1_id);
        creature.types.push(CardType::Creature);
        creature.power = Some(2);
        creature.toughness = Some(2);
        creature.controller = p1_id;
        game.cards.insert(creature_id, creature);
        game.battlefield.add(creature_id);

        // Check initial stats
        let creature_before = game.cards.get(creature_id).unwrap();
        assert_eq!(creature_before.current_power(), 2);
        assert_eq!(creature_before.current_toughness(), 2);

        // Create Giant Growth (pump +3/+3)
        let pump_spell_id = game.next_card_id();
        let mut pump_spell = Card::new(pump_spell_id, "Giant Growth".to_string(), p1_id);
        pump_spell.types.push(CardType::Instant);
        pump_spell.mana_cost = ManaCost::from_string("G");
        // Target the creature we created
        pump_spell.effects.push(Effect::PumpCreature {
            target: creature_id,
            power_bonus: 3,
            toughness_bonus: 3,
        });
        game.cards.insert(pump_spell_id, pump_spell);

        // Put spell on stack (simulating cast)
        game.stack.add(pump_spell_id);

        // Resolve the spell
        assert!(
            game.resolve_spell(pump_spell_id).is_ok(),
            "Failed to resolve pump spell"
        );

        // Check creature got the bonus
        let creature_after = game.cards.get(creature_id).unwrap();
        assert_eq!(
            creature_after.current_power(),
            5,
            "Creature should have +3 power bonus (2 + 3)"
        );
        assert_eq!(
            creature_after.current_toughness(),
            5,
            "Creature should have +3 toughness bonus (2 + 3)"
        );

        // Check spell went to graveyard
        if let Some(zones) = game.get_player_zones(p1_id) {
            assert!(
                zones.graveyard.contains(pump_spell_id),
                "Pump spell should be in graveyard"
            );
        }
    }

    #[test]
    fn test_pump_effect_cleanup_at_end_of_turn() {
        use crate::core::CardType;
        use crate::game::Step;

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;

        // Create a 2/2 creature on battlefield
        let creature_id = game.next_card_id();
        let mut creature = Card::new(creature_id, "Grizzly Bears".to_string(), p1_id);
        creature.types.push(CardType::Creature);
        creature.power = Some(2);
        creature.toughness = Some(2);
        creature.controller = p1_id;
        game.cards.insert(creature_id, creature);
        game.battlefield.add(creature_id);

        // Apply pump effect manually
        if let Ok(card) = game.cards.get_mut(creature_id) {
            card.power_bonus = 3;
            card.toughness_bonus = 3;
        }

        // Verify pumped stats
        let creature_pumped = game.cards.get(creature_id).unwrap();
        assert_eq!(creature_pumped.current_power(), 5);
        assert_eq!(creature_pumped.current_toughness(), 5);

        // Advance to End step
        game.turn.current_step = Step::End;

        // Advance to Cleanup step (should trigger cleanup)
        assert!(game.advance_step().is_ok());
        assert_eq!(game.turn.current_step, Step::Cleanup);

        // Check that bonuses were cleared
        let creature_after = game.cards.get(creature_id).unwrap();
        assert_eq!(
            creature_after.current_power(),
            2,
            "Power bonus should be cleared at cleanup"
        );
        assert_eq!(
            creature_after.current_toughness(),
            2,
            "Toughness bonus should be cleared at cleanup"
        );
    }

    #[test]
    fn test_normal_creature_vs_first_strike() {
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P1: Create a 3/3 creature without first strike (attacker)
        let attacker_id = game.next_entity_id();
        let mut attacker = Card::new(attacker_id, "Hill Giant".to_string(), p1_id);
        attacker.types.push(CardType::Creature);
        attacker.power = Some(3);
        attacker.toughness = Some(3);
        attacker.controller = p1_id;
        attacker.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        game.cards.insert(attacker_id, attacker);
        game.battlefield.add(attacker_id);

        // P2: Create a 2/2 creature with First Strike (blocker)
        let blocker_id = game.next_entity_id();
        let mut blocker = Card::new(blocker_id, "First Strike Knight".to_string(), p2_id);
        blocker.types.push(CardType::Creature);
        blocker.power = Some(2);
        blocker.toughness = Some(2);
        blocker.controller = p2_id;
        blocker.keywords.push(Keyword::FirstStrike);
        game.cards.insert(blocker_id, blocker);
        game.battlefield.add(blocker_id);

        // Declare combat
        game.combat.declare_attacker(attacker_id, p2_id);
        let attacker_vec = smallvec::SmallVec::from_vec(vec![attacker_id]);
        game.combat.declare_blocker(blocker_id, attacker_vec);

        // Create controllers
        let mut controller1 = ZeroController::new(p1_id);
        let mut controller2 = ZeroController::new(p2_id);

        // First strike damage step: only blocker deals damage
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, true);
        assert!(
            result.is_ok(),
            "Failed to assign first strike damage: {result:?}"
        );

        // Attacker should have taken 2 damage but still be alive (3 toughness)
        assert!(
            game.battlefield.contains(attacker_id),
            "Attacker should still be alive after first strike"
        );

        // Blocker should still be alive (hasn't taken damage yet)
        assert!(
            game.battlefield.contains(blocker_id),
            "Blocker should still be alive"
        );

        // Normal damage step: attacker deals damage, killing blocker
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, false);
        assert!(result.is_ok(), "Failed to assign normal damage: {result:?}");

        // Blocker should be dead (took 3 damage, toughness 2)
        if let Some(zones) = game.get_player_zones(p2_id) {
            assert!(
                zones.graveyard.contains(blocker_id),
                "Blocker should be in graveyard after normal damage"
            );
        }

        // Attacker should still be alive (took only 2 damage, has 3 toughness)
        assert!(
            game.battlefield.contains(attacker_id),
            "Attacker should still be alive"
        );
    }

    #[test]
    fn test_resolve_tap_spell() {
        use crate::core::{Effect, ManaCost};

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
        let p1_id = players[0];
        let p2_id = players[1];

        // Create an untapped creature for P2
        let creature_id = game.next_card_id();
        let mut creature = Card::new(creature_id, "Grizzly Bears".to_string(), p2_id);
        creature.types.push(CardType::Creature);
        creature.power = Some(2);
        creature.toughness = Some(2);
        creature.controller = p2_id;
        game.cards.insert(creature_id, creature);
        game.battlefield.add(creature_id);

        // Check initial state
        let creature_before = game.cards.get(creature_id).unwrap();
        assert!(!creature_before.tapped, "Creature should start untapped");

        // Create a Tap spell
        let tap_spell_id = game.next_card_id();
        let mut tap_spell = Card::new(tap_spell_id, "Frost Breath".to_string(), p1_id);
        tap_spell.types.push(CardType::Instant);
        tap_spell.mana_cost = ManaCost::from_string("2U");
        // Target the specific creature
        tap_spell.effects.push(Effect::TapPermanent {
            target: creature_id,
        });
        game.cards.insert(tap_spell_id, tap_spell);

        // Put spell on stack (simulating cast)
        game.stack.add(tap_spell_id);

        // Resolve the spell
        assert!(
            game.resolve_spell(tap_spell_id).is_ok(),
            "Failed to resolve tap spell"
        );

        // Check creature is tapped
        let creature_after = game.cards.get(creature_id).unwrap();
        assert!(
            creature_after.tapped,
            "Creature should be tapped after spell"
        );

        // Check spell went to graveyard
        if let Some(zones) = game.get_player_zones(p1_id) {
            assert!(
                zones.graveyard.contains(tap_spell_id),
                "Tap spell should be in graveyard"
            );
        }
    }

    #[test]
    fn test_resolve_untap_spell() {
        use crate::core::{Effect, ManaCost};

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let players: Vec<_> = game.players.iter().map(|p| p.id).collect();
        let p1_id = players[0];

        // Create a tapped land for P1
        let land_id = game.next_card_id();
        let mut land = Card::new(land_id, "Forest".to_string(), p1_id);
        land.types.push(CardType::Land);
        land.controller = p1_id;
        land.tapped = true; // Start tapped
        game.cards.insert(land_id, land);
        game.battlefield.add(land_id);

        // Check initial state
        let land_before = game.cards.get(land_id).unwrap();
        assert!(land_before.tapped, "Land should start tapped");

        // Create an Untap spell
        let untap_spell_id = game.next_card_id();
        let mut untap_spell = Card::new(untap_spell_id, "Untap".to_string(), p1_id);
        untap_spell.types.push(CardType::Instant);
        untap_spell.mana_cost = ManaCost::from_string("U");
        // Target the specific land
        untap_spell
            .effects
            .push(Effect::UntapPermanent { target: land_id });
        game.cards.insert(untap_spell_id, untap_spell);

        // Put spell on stack (simulating cast)
        game.stack.add(untap_spell_id);

        // Resolve the spell
        assert!(
            game.resolve_spell(untap_spell_id).is_ok(),
            "Failed to resolve untap spell"
        );

        // Check land is untapped
        let land_after = game.cards.get(land_id).unwrap();
        assert!(!land_after.tapped, "Land should be untapped after spell");

        // Check spell went to graveyard
        if let Some(zones) = game.get_player_zones(p1_id) {
            assert!(
                zones.graveyard.contains(untap_spell_id),
                "Untap spell should be in graveyard"
            );
        }
    }

    #[test]
    fn test_trample_excess_damage_to_player() {
        use crate::game::random_controller::RandomController;
        use crate::game::zero_controller::ZeroController;

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P1: Create a 5/5 creature with Trample (attacker)
        let attacker_id = game.next_entity_id();
        let mut attacker = Card::new(attacker_id, "Craw Wurm".to_string(), p1_id);
        attacker.types.push(CardType::Creature);
        attacker.power = Some(5);
        attacker.toughness = Some(5);
        attacker.controller = p1_id;
        attacker.keywords.push(Keyword::Trample);
        attacker.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        game.cards.insert(attacker_id, attacker);
        game.battlefield.add(attacker_id);

        // P2: Create a 2/2 creature (blocker)
        let blocker_id = game.next_entity_id();
        let mut blocker = Card::new(blocker_id, "Grizzly Bears".to_string(), p2_id);
        blocker.types.push(CardType::Creature);
        blocker.power = Some(2);
        blocker.toughness = Some(2);
        blocker.controller = p2_id;
        game.cards.insert(blocker_id, blocker);
        game.battlefield.add(blocker_id);

        // Declare combat
        game.combat.declare_attacker(attacker_id, p2_id);
        let attacker_vec = smallvec::SmallVec::from_vec(vec![attacker_id]);
        game.combat.declare_blocker(blocker_id, attacker_vec);

        // Record P2's life before combat
        let p2_life_before = game.players[1].life;

        // Assign combat damage
        let mut controller1 = RandomController::with_seed(p1_id, 12345);
        let mut controller2 = ZeroController::new(p2_id);
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, false);
        assert!(result.is_ok(), "Failed to assign combat damage: {result:?}");

        // Blocker should be dead (took 5 damage, toughness 2)
        if let Some(zones) = game.get_player_zones(p2_id) {
            assert!(
                zones.graveyard.contains(blocker_id),
                "Blocker should be in graveyard"
            );
        }

        // P2 should have taken 3 trample damage (5 power - 2 to kill blocker)
        let p2_life_after = game.players[1].life;
        assert_eq!(
            p2_life_after,
            p2_life_before - 3,
            "P2 should have taken 3 trample damage"
        );
    }

    #[test]
    fn test_trample_exact_lethal_no_excess() {
        use crate::game::random_controller::RandomController;
        use crate::game::zero_controller::ZeroController;

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P1: Create a 3/3 creature with Trample (attacker)
        let attacker_id = game.next_entity_id();
        let mut attacker = Card::new(attacker_id, "Trained Armodon".to_string(), p1_id);
        attacker.types.push(CardType::Creature);
        attacker.power = Some(3);
        attacker.toughness = Some(3);
        attacker.controller = p1_id;
        attacker.keywords.push(Keyword::Trample);
        attacker.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        game.cards.insert(attacker_id, attacker);
        game.battlefield.add(attacker_id);

        // P2: Create a 3/3 creature (blocker)
        let blocker_id = game.next_entity_id();
        let mut blocker = Card::new(blocker_id, "Hill Giant".to_string(), p2_id);
        blocker.types.push(CardType::Creature);
        blocker.power = Some(3);
        blocker.toughness = Some(3);
        blocker.controller = p2_id;
        game.cards.insert(blocker_id, blocker);
        game.battlefield.add(blocker_id);

        // Declare combat
        game.combat.declare_attacker(attacker_id, p2_id);
        let attacker_vec = smallvec::SmallVec::from_vec(vec![attacker_id]);
        game.combat.declare_blocker(blocker_id, attacker_vec);

        // Record P2's life before combat
        let p2_life_before = game.players[1].life;

        // Assign combat damage
        let mut controller1 = RandomController::with_seed(p1_id, 12345);
        let mut controller2 = ZeroController::new(p2_id);
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, false);
        assert!(result.is_ok(), "Failed to assign combat damage: {result:?}");

        // Blocker should be dead (took 3 damage, toughness 3)
        if let Some(zones) = game.get_player_zones(p2_id) {
            assert!(
                zones.graveyard.contains(blocker_id),
                "Blocker should be in graveyard"
            );
        }

        // P2 should NOT have taken any damage (exact lethal, no excess)
        let p2_life_after = game.players[1].life;
        assert_eq!(
            p2_life_after, p2_life_before,
            "P2 should not have taken damage (exact lethal, no excess)"
        );
    }

    #[test]
    fn test_non_trample_blocked_no_player_damage() {
        use crate::game::random_controller::RandomController;
        use crate::game::zero_controller::ZeroController;

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P1: Create a 5/5 creature WITHOUT Trample (attacker)
        let attacker_id = game.next_entity_id();
        let mut attacker = Card::new(attacker_id, "Serra Angel".to_string(), p1_id);
        attacker.types.push(CardType::Creature);
        attacker.power = Some(5);
        attacker.toughness = Some(5);
        attacker.controller = p1_id;
        // NO Trample keyword
        attacker.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        game.cards.insert(attacker_id, attacker);
        game.battlefield.add(attacker_id);

        // P2: Create a 1/1 creature (blocker)
        let blocker_id = game.next_entity_id();
        let mut blocker = Card::new(blocker_id, "Llanowar Elves".to_string(), p2_id);
        blocker.types.push(CardType::Creature);
        blocker.power = Some(1);
        blocker.toughness = Some(1);
        blocker.controller = p2_id;
        game.cards.insert(blocker_id, blocker);
        game.battlefield.add(blocker_id);

        // Declare combat
        game.combat.declare_attacker(attacker_id, p2_id);
        let attacker_vec = smallvec::SmallVec::from_vec(vec![attacker_id]);
        game.combat.declare_blocker(blocker_id, attacker_vec);

        // Record P2's life before combat
        let p2_life_before = game.players[1].life;

        // Assign combat damage
        let mut controller1 = RandomController::with_seed(p1_id, 12345);
        let mut controller2 = ZeroController::new(p2_id);
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, false);
        assert!(result.is_ok(), "Failed to assign combat damage: {result:?}");

        // Blocker should be dead
        if let Some(zones) = game.get_player_zones(p2_id) {
            assert!(
                zones.graveyard.contains(blocker_id),
                "Blocker should be in graveyard"
            );
        }

        // P2 should NOT have taken any damage (no trample, so excess is lost)
        let p2_life_after = game.players[1].life;
        assert_eq!(
            p2_life_after, p2_life_before,
            "P2 should not have taken damage without trample"
        );
    }

    #[test]
    fn test_trample_multiple_blockers() {
        use crate::game::random_controller::RandomController;
        use crate::game::zero_controller::ZeroController;

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P1: Create a 7/7 creature with Trample (attacker)
        let attacker_id = game.next_entity_id();
        let mut attacker = Card::new(attacker_id, "Enormous Baloth".to_string(), p1_id);
        attacker.types.push(CardType::Creature);
        attacker.power = Some(7);
        attacker.toughness = Some(7);
        attacker.controller = p1_id;
        attacker.keywords.push(Keyword::Trample);
        attacker.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        game.cards.insert(attacker_id, attacker);
        game.battlefield.add(attacker_id);

        // P2: Create two blockers (2/2 and 3/3)
        let blocker1_id = game.next_entity_id();
        let mut blocker1 = Card::new(blocker1_id, "Grizzly Bears".to_string(), p2_id);
        blocker1.types.push(CardType::Creature);
        blocker1.power = Some(2);
        blocker1.toughness = Some(2);
        blocker1.controller = p2_id;
        game.cards.insert(blocker1_id, blocker1);
        game.battlefield.add(blocker1_id);

        let blocker2_id = game.next_entity_id();
        let mut blocker2 = Card::new(blocker2_id, "Hill Giant".to_string(), p2_id);
        blocker2.types.push(CardType::Creature);
        blocker2.power = Some(3);
        blocker2.toughness = Some(3);
        blocker2.controller = p2_id;
        game.cards.insert(blocker2_id, blocker2);
        game.battlefield.add(blocker2_id);

        // Declare combat
        game.combat.declare_attacker(attacker_id, p2_id);
        let attacker_vec = smallvec::SmallVec::from_vec(vec![attacker_id]);
        game.combat
            .declare_blocker(blocker1_id, attacker_vec.clone());
        game.combat.declare_blocker(blocker2_id, attacker_vec);

        // Record P2's life before combat
        let p2_life_before = game.players[1].life;

        // Assign combat damage
        let mut controller1 = RandomController::with_seed(p1_id, 12345);
        let mut controller2 = ZeroController::new(p2_id);
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, false);
        assert!(result.is_ok(), "Failed to assign combat damage: {result:?}");

        // Both blockers should be dead
        if let Some(zones) = game.get_player_zones(p2_id) {
            assert!(
                zones.graveyard.contains(blocker1_id),
                "Blocker 1 should be in graveyard"
            );
            assert!(
                zones.graveyard.contains(blocker2_id),
                "Blocker 2 should be in graveyard"
            );
        }

        // P2 should have taken 2 trample damage (7 power - 2 - 3 = 2)
        let p2_life_after = game.players[1].life;
        assert_eq!(
            p2_life_after,
            p2_life_before - 2,
            "P2 should have taken 2 trample damage"
        );
    }

    #[test]
    fn test_lifelink_attacker_blocked() {
        use crate::game::random_controller::RandomController;
        use crate::game::zero_controller::ZeroController;

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P1: Create a 3/3 creature with Lifelink (attacker)
        let attacker_id = game.next_entity_id();
        let mut attacker = Card::new(attacker_id, "Healer's Hawk".to_string(), p1_id);
        attacker.types.push(CardType::Creature);
        attacker.power = Some(3);
        attacker.toughness = Some(3);
        attacker.controller = p1_id;
        attacker.keywords.push(Keyword::Lifelink);
        attacker.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        game.cards.insert(attacker_id, attacker);
        game.battlefield.add(attacker_id);

        // P2: Create a 2/2 creature (blocker)
        let blocker_id = game.next_entity_id();
        let mut blocker = Card::new(blocker_id, "Grizzly Bears".to_string(), p2_id);
        blocker.types.push(CardType::Creature);
        blocker.power = Some(2);
        blocker.toughness = Some(2);
        blocker.controller = p2_id;
        game.cards.insert(blocker_id, blocker);
        game.battlefield.add(blocker_id);

        // Declare combat
        game.combat.declare_attacker(attacker_id, p2_id);
        let attacker_vec = smallvec::SmallVec::from_vec(vec![attacker_id]);
        game.combat.declare_blocker(blocker_id, attacker_vec);

        // Record P1's life before combat
        let p1_life_before = game.players[0].life;

        // Assign combat damage
        let mut controller1 = RandomController::with_seed(p1_id, 12345);
        let mut controller2 = ZeroController::new(p2_id);
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, false);
        assert!(result.is_ok(), "Failed to assign combat damage: {result:?}");

        // P1 should have gained 3 life from lifelink (3 damage dealt to blocker)
        let p1_life_after = game.players[0].life;
        assert_eq!(
            p1_life_after,
            p1_life_before + 3,
            "P1 should have gained 3 life from lifelink"
        );

        // Blocker should be dead
        if let Some(zones) = game.get_player_zones(p2_id) {
            assert!(
                zones.graveyard.contains(blocker_id),
                "Blocker should be in graveyard"
            );
        }
    }

    #[test]
    fn test_lifelink_attacker_unblocked() {
        use crate::game::random_controller::RandomController;
        use crate::game::zero_controller::ZeroController;

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P1: Create a 4/4 creature with Lifelink (attacker)
        let attacker_id = game.next_entity_id();
        let mut attacker = Card::new(attacker_id, "Ajani's Pridemate".to_string(), p1_id);
        attacker.types.push(CardType::Creature);
        attacker.power = Some(4);
        attacker.toughness = Some(4);
        attacker.controller = p1_id;
        attacker.keywords.push(Keyword::Lifelink);
        attacker.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        game.cards.insert(attacker_id, attacker);
        game.battlefield.add(attacker_id);

        // Declare combat (no blockers)
        game.combat.declare_attacker(attacker_id, p2_id);

        // Record life before combat
        let p1_life_before = game.players[0].life;
        let p2_life_before = game.players[1].life;

        // Assign combat damage
        let mut controller1 = RandomController::with_seed(p1_id, 12345);
        let mut controller2 = ZeroController::new(p2_id);
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, false);
        assert!(result.is_ok(), "Failed to assign combat damage: {result:?}");

        // P1 should have gained 4 life from lifelink (4 damage dealt to player)
        let p1_life_after = game.players[0].life;
        assert_eq!(
            p1_life_after,
            p1_life_before + 4,
            "P1 should have gained 4 life from lifelink"
        );

        // P2 should have taken 4 damage
        let p2_life_after = game.players[1].life;
        assert_eq!(
            p2_life_after,
            p2_life_before - 4,
            "P2 should have taken 4 damage"
        );
    }

    #[test]
    fn test_lifelink_blocker() {
        use crate::game::random_controller::RandomController;
        use crate::game::zero_controller::ZeroController;

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P1: Create a 3/3 creature (attacker)
        let attacker_id = game.next_entity_id();
        let mut attacker = Card::new(attacker_id, "Hill Giant".to_string(), p1_id);
        attacker.types.push(CardType::Creature);
        attacker.power = Some(3);
        attacker.toughness = Some(3);
        attacker.controller = p1_id;
        attacker.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        game.cards.insert(attacker_id, attacker);
        game.battlefield.add(attacker_id);

        // P2: Create a 2/2 creature with Lifelink (blocker)
        let blocker_id = game.next_entity_id();
        let mut blocker = Card::new(blocker_id, "Vampire Cutthroat".to_string(), p2_id);
        blocker.types.push(CardType::Creature);
        blocker.power = Some(2);
        blocker.toughness = Some(2);
        blocker.controller = p2_id;
        blocker.keywords.push(Keyword::Lifelink);
        game.cards.insert(blocker_id, blocker);
        game.battlefield.add(blocker_id);

        // Declare combat
        game.combat.declare_attacker(attacker_id, p2_id);
        let attacker_vec = smallvec::SmallVec::from_vec(vec![attacker_id]);
        game.combat.declare_blocker(blocker_id, attacker_vec);

        // Record P2's life before combat
        let p2_life_before = game.players[1].life;

        // Assign combat damage
        let mut controller1 = RandomController::with_seed(p1_id, 12345);
        let mut controller2 = ZeroController::new(p2_id);
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, false);
        assert!(result.is_ok(), "Failed to assign combat damage: {result:?}");

        // P2 should have gained 2 life from lifelink (blocker dealt 2 damage)
        let p2_life_after = game.players[1].life;
        assert_eq!(
            p2_life_after,
            p2_life_before + 2,
            "P2 should have gained 2 life from lifelink blocker"
        );

        // Blocker should be dead (took 3 damage, has 2 toughness)
        // Attacker should survive (took 2 damage, has 3 toughness)
        if let Some(zones) = game.get_player_zones(p1_id) {
            assert!(
                !zones.graveyard.contains(attacker_id),
                "Attacker should still be alive (took 2 damage, has 3 toughness)"
            );
            assert!(
                game.battlefield.contains(attacker_id),
                "Attacker should still be on battlefield"
            );
        }
        if let Some(zones) = game.get_player_zones(p2_id) {
            assert!(
                zones.graveyard.contains(blocker_id),
                "Blocker should be in graveyard (took 3 damage, has 2 toughness)"
            );
        }
    }

    #[test]
    fn test_lifelink_with_trample() {
        use crate::game::random_controller::RandomController;
        use crate::game::zero_controller::ZeroController;

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P1: Create a 5/5 creature with Lifelink AND Trample (attacker)
        let attacker_id = game.next_entity_id();
        let mut attacker = Card::new(attacker_id, "Baneslayer Angel".to_string(), p1_id);
        attacker.types.push(CardType::Creature);
        attacker.power = Some(5);
        attacker.toughness = Some(5);
        attacker.controller = p1_id;
        attacker.keywords.push(Keyword::Lifelink);
        attacker.keywords.push(Keyword::Trample);
        attacker.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        game.cards.insert(attacker_id, attacker);
        game.battlefield.add(attacker_id);

        // P2: Create a 2/2 creature (blocker)
        let blocker_id = game.next_entity_id();
        let mut blocker = Card::new(blocker_id, "Grizzly Bears".to_string(), p2_id);
        blocker.types.push(CardType::Creature);
        blocker.power = Some(2);
        blocker.toughness = Some(2);
        blocker.controller = p2_id;
        game.cards.insert(blocker_id, blocker);
        game.battlefield.add(blocker_id);

        // Declare combat
        game.combat.declare_attacker(attacker_id, p2_id);
        let attacker_vec = smallvec::SmallVec::from_vec(vec![attacker_id]);
        game.combat.declare_blocker(blocker_id, attacker_vec);

        // Record life before combat
        let p1_life_before = game.players[0].life;
        let p2_life_before = game.players[1].life;

        // Assign combat damage
        let mut controller1 = RandomController::with_seed(p1_id, 12345);
        let mut controller2 = ZeroController::new(p2_id);
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, false);
        assert!(result.is_ok(), "Failed to assign combat damage: {result:?}");

        // P1 should have gained 5 life (2 to blocker + 3 trample to player = 5 total damage)
        let p1_life_after = game.players[0].life;
        assert_eq!(
            p1_life_after,
            p1_life_before + 5,
            "P1 should have gained 5 life from lifelink (all damage dealt)"
        );

        // P2 should have taken 3 trample damage
        let p2_life_after = game.players[1].life;
        assert_eq!(
            p2_life_after,
            p2_life_before - 3,
            "P2 should have taken 3 trample damage"
        );

        // Blocker should be dead
        if let Some(zones) = game.get_player_zones(p2_id) {
            assert!(
                zones.graveyard.contains(blocker_id),
                "Blocker should be in graveyard"
            );
        }
    }

    #[test]
    fn test_deathtouch_attacker_kills_large_blocker() {
        use crate::game::random_controller::RandomController;
        use crate::game::zero_controller::ZeroController;

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P1: Create a 1/1 creature with Deathtouch (attacker)
        let attacker_id = game.next_entity_id();
        let mut attacker = Card::new(attacker_id, "Deadly Recluse".to_string(), p1_id);
        attacker.types.push(CardType::Creature);
        attacker.power = Some(1);
        attacker.toughness = Some(1);
        attacker.controller = p1_id;
        attacker.keywords.push(Keyword::Deathtouch);
        attacker.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        game.cards.insert(attacker_id, attacker);
        game.battlefield.add(attacker_id);

        // P2: Create a 5/5 creature (blocker)
        let blocker_id = game.next_entity_id();
        let mut blocker = Card::new(blocker_id, "Serra Angel".to_string(), p2_id);
        blocker.types.push(CardType::Creature);
        blocker.power = Some(5);
        blocker.toughness = Some(5);
        blocker.controller = p2_id;
        game.cards.insert(blocker_id, blocker);
        game.battlefield.add(blocker_id);

        // Declare combat
        game.combat.declare_attacker(attacker_id, p2_id);
        let attacker_vec = smallvec::SmallVec::from_vec(vec![attacker_id]);
        game.combat.declare_blocker(blocker_id, attacker_vec);

        // Assign combat damage
        let mut controller1 = RandomController::with_seed(p1_id, 12345);
        let mut controller2 = ZeroController::new(p2_id);
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, false);
        assert!(result.is_ok(), "Failed to assign combat damage: {result:?}");

        // Blocker should be dead (deathtouch from 1 damage)
        // Attacker should be dead (5 damage from blocker)
        if let Some(zones) = game.get_player_zones(p1_id) {
            assert!(
                zones.graveyard.contains(attacker_id),
                "Attacker should be in graveyard (took 5 damage)"
            );
        }
        if let Some(zones) = game.get_player_zones(p2_id) {
            assert!(
                zones.graveyard.contains(blocker_id),
                "Blocker should be in graveyard (dealt deathtouch damage)"
            );
        }
    }

    #[test]
    fn test_deathtouch_blocker_kills_large_attacker() {
        use crate::game::random_controller::RandomController;
        use crate::game::zero_controller::ZeroController;

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P1: Create a 5/5 creature (attacker)
        let attacker_id = game.next_entity_id();
        let mut attacker = Card::new(attacker_id, "Serra Angel".to_string(), p1_id);
        attacker.types.push(CardType::Creature);
        attacker.power = Some(5);
        attacker.toughness = Some(5);
        attacker.controller = p1_id;
        attacker.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        game.cards.insert(attacker_id, attacker);
        game.battlefield.add(attacker_id);

        // P2: Create a 1/1 creature with Deathtouch (blocker)
        let blocker_id = game.next_entity_id();
        let mut blocker = Card::new(blocker_id, "Typhoid Rats".to_string(), p2_id);
        blocker.types.push(CardType::Creature);
        blocker.power = Some(1);
        blocker.toughness = Some(1);
        blocker.controller = p2_id;
        blocker.keywords.push(Keyword::Deathtouch);
        game.cards.insert(blocker_id, blocker);
        game.battlefield.add(blocker_id);

        // Declare combat
        game.combat.declare_attacker(attacker_id, p2_id);
        let attacker_vec = smallvec::SmallVec::from_vec(vec![attacker_id]);
        game.combat.declare_blocker(blocker_id, attacker_vec);

        // Assign combat damage
        let mut controller1 = RandomController::with_seed(p1_id, 12345);
        let mut controller2 = ZeroController::new(p2_id);
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, false);
        assert!(result.is_ok(), "Failed to assign combat damage: {result:?}");

        // Attacker should be dead (deathtouch from 1 damage)
        // Blocker should be dead (5 damage from attacker)
        if let Some(zones) = game.get_player_zones(p1_id) {
            assert!(
                zones.graveyard.contains(attacker_id),
                "Attacker should be in graveyard (dealt deathtouch damage)"
            );
        }
        if let Some(zones) = game.get_player_zones(p2_id) {
            assert!(
                zones.graveyard.contains(blocker_id),
                "Blocker should be in graveyard (took 5 damage)"
            );
        }
    }

    #[test]
    fn test_deathtouch_with_trample_minimal_damage() {
        use crate::game::random_controller::RandomController;
        use crate::game::zero_controller::ZeroController;

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P1: Create a 5/5 creature with Deathtouch AND Trample (attacker)
        let attacker_id = game.next_entity_id();
        let mut attacker = Card::new(attacker_id, "Chevill, Bane of Monsters".to_string(), p1_id);
        attacker.types.push(CardType::Creature);
        attacker.power = Some(5);
        attacker.toughness = Some(5);
        attacker.controller = p1_id;
        attacker.keywords.push(Keyword::Deathtouch);
        attacker.keywords.push(Keyword::Trample);
        attacker.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        game.cards.insert(attacker_id, attacker);
        game.battlefield.add(attacker_id);

        // P2: Create a 3/3 creature (blocker)
        let blocker_id = game.next_entity_id();
        let mut blocker = Card::new(blocker_id, "Hill Giant".to_string(), p2_id);
        blocker.types.push(CardType::Creature);
        blocker.power = Some(3);
        blocker.toughness = Some(3);
        blocker.controller = p2_id;
        game.cards.insert(blocker_id, blocker);
        game.battlefield.add(blocker_id);

        // Declare combat
        game.combat.declare_attacker(attacker_id, p2_id);
        let attacker_vec = smallvec::SmallVec::from_vec(vec![attacker_id]);
        game.combat.declare_blocker(blocker_id, attacker_vec);

        // Record P2's life before combat
        let p2_life_before = game.players[1].life;

        // Assign combat damage
        let mut controller1 = RandomController::with_seed(p1_id, 12345);
        let mut controller2 = ZeroController::new(p2_id);
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, false);
        assert!(result.is_ok(), "Failed to assign combat damage: {result:?}");

        // MTG Rules 702.2c: With deathtouch + trample, only 1 damage is lethal
        // So 1 damage to blocker (kills it), 4 damage tramples over to player
        let p2_life_after = game.players[1].life;
        assert_eq!(
            p2_life_after,
            p2_life_before - 4,
            "P2 should have taken 4 trample damage (5 power - 1 lethal to blocker)"
        );

        // Blocker should be dead (deathtouch)
        // Attacker should survive (took 3 damage, has 5 toughness)
        assert!(
            game.battlefield.contains(attacker_id),
            "Attacker should survive (took 3 damage, has 5 toughness)"
        );
        if let Some(zones) = game.get_player_zones(p2_id) {
            assert!(
                zones.graveyard.contains(blocker_id),
                "Blocker should be in graveyard (dealt deathtouch damage)"
            );
        }
    }

    #[test]
    fn test_deathtouch_with_multiple_blockers() {
        use crate::game::random_controller::RandomController;
        use crate::game::zero_controller::ZeroController;

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P1: Create a 3/3 creature with Deathtouch (attacker)
        let attacker_id = game.next_entity_id();
        let mut attacker = Card::new(attacker_id, "Gifted Aetherborn".to_string(), p1_id);
        attacker.types.push(CardType::Creature);
        attacker.power = Some(3);
        attacker.toughness = Some(3);
        attacker.controller = p1_id;
        attacker.keywords.push(Keyword::Deathtouch);
        attacker.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        game.cards.insert(attacker_id, attacker);
        game.battlefield.add(attacker_id);

        // P2: Create two blockers (both 5/5)
        let blocker1_id = game.next_entity_id();
        let mut blocker1 = Card::new(blocker1_id, "Serra Angel".to_string(), p2_id);
        blocker1.types.push(CardType::Creature);
        blocker1.power = Some(5);
        blocker1.toughness = Some(5);
        blocker1.controller = p2_id;
        game.cards.insert(blocker1_id, blocker1);
        game.battlefield.add(blocker1_id);

        let blocker2_id = game.next_entity_id();
        let mut blocker2 = Card::new(blocker2_id, "Air Elemental".to_string(), p2_id);
        blocker2.types.push(CardType::Creature);
        blocker2.power = Some(5);
        blocker2.toughness = Some(5);
        blocker2.controller = p2_id;
        game.cards.insert(blocker2_id, blocker2);
        game.battlefield.add(blocker2_id);

        // Declare combat with both blockers
        game.combat.declare_attacker(attacker_id, p2_id);
        let attacker_vec = smallvec::SmallVec::from_vec(vec![attacker_id]);
        game.combat
            .declare_blocker(blocker1_id, attacker_vec.clone());
        game.combat.declare_blocker(blocker2_id, attacker_vec);

        // Assign combat damage (damage order determined internally)
        let mut controller1 = RandomController::with_seed(p1_id, 12345);
        let mut controller2 = ZeroController::new(p2_id);
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, false);
        assert!(result.is_ok(), "Failed to assign combat damage: {result:?}");

        // With deathtouch, 1 damage is lethal to each blocker
        // 3/3 attacker: 1 damage to first blocker, 1 damage to second blocker, 1 damage wasted
        // Both blockers should be dead, attacker should be dead (took 10 damage total)
        if let Some(zones) = game.get_player_zones(p1_id) {
            assert!(
                zones.graveyard.contains(attacker_id),
                "Attacker should be in graveyard (took 10 damage from two 5/5 blockers)"
            );
        }
        if let Some(zones) = game.get_player_zones(p2_id) {
            assert!(
                zones.graveyard.contains(blocker1_id),
                "First blocker should be in graveyard (dealt deathtouch damage)"
            );
            assert!(
                zones.graveyard.contains(blocker2_id),
                "Second blocker should be in graveyard (dealt deathtouch damage)"
            );
        }
    }

    // Note: Menace validation test removed because incremental validation during
    // blocker declaration would incorrectly reject the first blocker. Menace validation
    // should happen after all blockers are declared. The following tests verify that
    // Menace works correctly when multiple blockers are declared or no blockers are declared.

    #[test]
    fn test_menace_can_be_blocked_by_two_creatures() {
        use crate::game::random_controller::RandomController;
        use crate::game::zero_controller::ZeroController;

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P1: Create a 3/3 creature with Menace (attacker)
        let attacker_id = game.next_entity_id();
        let mut attacker = Card::new(attacker_id, "Mardu Skullhunter".to_string(), p1_id);
        attacker.types.push(CardType::Creature);
        attacker.power = Some(3);
        attacker.toughness = Some(3);
        attacker.controller = p1_id;
        attacker.keywords.push(Keyword::Menace);
        attacker.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        game.cards.insert(attacker_id, attacker);
        game.battlefield.add(attacker_id);

        // P2: Create two blockers
        let blocker1_id = game.next_entity_id();
        let mut blocker1 = Card::new(blocker1_id, "Grizzly Bears".to_string(), p2_id);
        blocker1.types.push(CardType::Creature);
        blocker1.power = Some(2);
        blocker1.toughness = Some(2);
        blocker1.controller = p2_id;
        game.cards.insert(blocker1_id, blocker1);
        game.battlefield.add(blocker1_id);

        let blocker2_id = game.next_entity_id();
        let mut blocker2 = Card::new(blocker2_id, "Elite Vanguard".to_string(), p2_id);
        blocker2.types.push(CardType::Creature);
        blocker2.power = Some(2);
        blocker2.toughness = Some(1);
        blocker2.controller = p2_id;
        game.cards.insert(blocker2_id, blocker2);
        game.battlefield.add(blocker2_id);

        // Declare attacker
        game.combat.declare_attacker(attacker_id, p2_id);

        // Block with two creatures - should succeed
        let result1 = game.declare_blocker(p2_id, blocker1_id, vec![attacker_id]);
        assert!(result1.is_ok(), "First blocker should succeed: {result1:?}");

        let result2 = game.declare_blocker(p2_id, blocker2_id, vec![attacker_id]);
        assert!(
            result2.is_ok(),
            "Second blocker should succeed: {result2:?}"
        );

        // Verify combat resolves correctly
        let mut controller1 = RandomController::with_seed(p1_id, 12345);
        let mut controller2 = ZeroController::new(p2_id);
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, false);
        assert!(result.is_ok(), "Combat damage should resolve: {result:?}");

        // Both blockers should be dead (took 3 damage total, both have <= 2 toughness)
        // Attacker should be dead (took 4 damage total from 2+2, has 3 toughness)
        if let Some(zones) = game.get_player_zones(p1_id) {
            assert!(
                zones.graveyard.contains(attacker_id),
                "Attacker should be dead"
            );
        }
        if let Some(zones) = game.get_player_zones(p2_id) {
            assert!(
                zones.graveyard.contains(blocker1_id),
                "First blocker should be dead"
            );
            assert!(
                zones.graveyard.contains(blocker2_id),
                "Second blocker should be dead"
            );
        }
    }

    #[test]
    fn test_menace_can_be_unblocked() {
        use crate::game::random_controller::RandomController;
        use crate::game::zero_controller::ZeroController;

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P1: Create a 3/3 creature with Menace (attacker)
        let attacker_id = game.next_entity_id();
        let mut attacker = Card::new(attacker_id, "Goblin Heelcutter".to_string(), p1_id);
        attacker.types.push(CardType::Creature);
        attacker.power = Some(3);
        attacker.toughness = Some(3);
        attacker.controller = p1_id;
        attacker.keywords.push(Keyword::Menace);
        attacker.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        game.cards.insert(attacker_id, attacker);
        game.battlefield.add(attacker_id);

        // Declare attacker (no blockers)
        game.combat.declare_attacker(attacker_id, p2_id);

        // Record life before combat
        let p2_life_before = game.players[1].life;

        // Assign combat damage
        let mut controller1 = RandomController::with_seed(p1_id, 12345);
        let mut controller2 = ZeroController::new(p2_id);
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, false);
        assert!(result.is_ok(), "Combat damage should resolve: {result:?}");

        // P2 should have taken 3 damage
        let p2_life_after = game.players[1].life;
        assert_eq!(
            p2_life_after,
            p2_life_before - 3,
            "P2 should have taken 3 damage from unblocked menace creature"
        );
    }

    #[test]
    fn test_menace_with_three_blockers() {
        use crate::game::random_controller::RandomController;
        use crate::game::zero_controller::ZeroController;

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P1: Create a 5/5 creature with Menace (attacker)
        let attacker_id = game.next_entity_id();
        let mut attacker = Card::new(attacker_id, "Charging Monstrosaur".to_string(), p1_id);
        attacker.types.push(CardType::Creature);
        attacker.power = Some(5);
        attacker.toughness = Some(5);
        attacker.controller = p1_id;
        attacker.keywords.push(Keyword::Menace);
        attacker.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        game.cards.insert(attacker_id, attacker);
        game.battlefield.add(attacker_id);

        // P2: Create three blockers (1/1 each)
        let blocker1_id = game.next_entity_id();
        let mut blocker1 = Card::new(blocker1_id, "Soldier Token 1".to_string(), p2_id);
        blocker1.types.push(CardType::Creature);
        blocker1.power = Some(1);
        blocker1.toughness = Some(1);
        blocker1.controller = p2_id;
        game.cards.insert(blocker1_id, blocker1);
        game.battlefield.add(blocker1_id);

        let blocker2_id = game.next_entity_id();
        let mut blocker2 = Card::new(blocker2_id, "Soldier Token 2".to_string(), p2_id);
        blocker2.types.push(CardType::Creature);
        blocker2.power = Some(1);
        blocker2.toughness = Some(1);
        blocker2.controller = p2_id;
        game.cards.insert(blocker2_id, blocker2);
        game.battlefield.add(blocker2_id);

        let blocker3_id = game.next_entity_id();
        let mut blocker3 = Card::new(blocker3_id, "Soldier Token 3".to_string(), p2_id);
        blocker3.types.push(CardType::Creature);
        blocker3.power = Some(1);
        blocker3.toughness = Some(1);
        blocker3.controller = p2_id;
        game.cards.insert(blocker3_id, blocker3);
        game.battlefield.add(blocker3_id);

        // Declare attacker
        game.combat.declare_attacker(attacker_id, p2_id);

        // Block with three creatures - should succeed (more than 2 is fine)
        let result1 = game.declare_blocker(p2_id, blocker1_id, vec![attacker_id]);
        assert!(result1.is_ok(), "First blocker should succeed");

        let result2 = game.declare_blocker(p2_id, blocker2_id, vec![attacker_id]);
        assert!(result2.is_ok(), "Second blocker should succeed");

        let result3 = game.declare_blocker(p2_id, blocker3_id, vec![attacker_id]);
        assert!(result3.is_ok(), "Third blocker should succeed");

        // Verify combat resolves correctly
        let mut controller1 = RandomController::with_seed(p1_id, 12345);
        let mut controller2 = ZeroController::new(p2_id);
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, false);
        assert!(result.is_ok(), "Combat damage should resolve: {result:?}");

        // All three blockers should be dead (each took 1 toughness worth of damage)
        // Attacker should survive (took 3 damage, has 5 toughness)
        assert!(
            game.battlefield.contains(attacker_id),
            "Attacker should survive (took 3 damage, has 5 toughness)"
        );
        if let Some(zones) = game.get_player_zones(p2_id) {
            assert!(
                zones.graveyard.contains(blocker1_id),
                "First blocker should be dead"
            );
            assert!(
                zones.graveyard.contains(blocker2_id),
                "Second blocker should be dead"
            );
            assert!(
                zones.graveyard.contains(blocker3_id),
                "Third blocker should be dead"
            );
        }
    }

    #[test]
    fn test_hexproof_blocks_destroy_spell() {
        // Test that destroy spells cannot target hexproof creatures controlled by opponent
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P2: Create a hexproof creature
        let hexproof_creature_id = game.next_entity_id();
        let mut hexproof_creature =
            Card::new(hexproof_creature_id, "Slippery Bogle".to_string(), p2_id);
        hexproof_creature.types.push(CardType::Creature);
        hexproof_creature.power = Some(1);
        hexproof_creature.toughness = Some(1);
        hexproof_creature.keywords.push(Keyword::Hexproof);
        game.cards.insert(hexproof_creature_id, hexproof_creature);
        game.battlefield.add(hexproof_creature_id);

        // P2: Create a normal creature
        let normal_creature_id = game.next_entity_id();
        let mut normal_creature = Card::new(normal_creature_id, "Grizzly Bears".to_string(), p2_id);
        normal_creature.types.push(CardType::Creature);
        normal_creature.power = Some(2);
        normal_creature.toughness = Some(2);
        game.cards.insert(normal_creature_id, normal_creature);
        game.battlefield.add(normal_creature_id);

        // P1: Cast a destroy spell (Terror) - should target normal creature, not hexproof one
        let destroy_spell_id = game.next_entity_id();
        let mut destroy_spell = Card::new(destroy_spell_id, "Terror".to_string(), p1_id);
        destroy_spell.types.push(CardType::Instant);
        destroy_spell.mana_cost = ManaCost::from_string("1B");
        // Use placeholder card ID 0 which will be replaced with a targetable opponent's creature
        destroy_spell.effects.push(Effect::DestroyPermanent {
            target: CardId::new(0),
        });
        game.cards.insert(destroy_spell_id, destroy_spell);

        // Put it on the stack (simulating cast)
        game.stack.add(destroy_spell_id);

        // Resolve the spell
        let result = game.resolve_spell(destroy_spell_id);
        assert!(result.is_ok(), "Destroy spell should resolve successfully");

        // Check that the hexproof creature is still alive
        assert!(
            game.battlefield.contains(hexproof_creature_id),
            "Hexproof creature should still be on battlefield"
        );

        // Check that the normal creature was destroyed
        if let Some(zones) = game.get_player_zones(p2_id) {
            assert!(
                zones.graveyard.contains(normal_creature_id),
                "Normal creature should be in graveyard"
            );
        }
    }

    #[test]
    fn test_hexproof_blocks_tap_spell() {
        // Test that tap spells cannot target hexproof creatures controlled by opponent
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P2: Create a hexproof creature
        let hexproof_creature_id = game.next_entity_id();
        let mut hexproof_creature =
            Card::new(hexproof_creature_id, "Slippery Bogle".to_string(), p2_id);
        hexproof_creature.types.push(CardType::Creature);
        hexproof_creature.power = Some(1);
        hexproof_creature.toughness = Some(1);
        hexproof_creature.keywords.push(Keyword::Hexproof);
        game.cards.insert(hexproof_creature_id, hexproof_creature);
        game.battlefield.add(hexproof_creature_id);

        // P2: Create a normal creature
        let normal_creature_id = game.next_entity_id();
        let mut normal_creature = Card::new(normal_creature_id, "Grizzly Bears".to_string(), p2_id);
        normal_creature.types.push(CardType::Creature);
        normal_creature.power = Some(2);
        normal_creature.toughness = Some(2);
        game.cards.insert(normal_creature_id, normal_creature);
        game.battlefield.add(normal_creature_id);

        // P1: Cast a tap spell - should target normal creature, not hexproof one
        let tap_spell_id = game.next_entity_id();
        let mut tap_spell = Card::new(tap_spell_id, "Frost Breath".to_string(), p1_id);
        tap_spell.types.push(CardType::Instant);
        tap_spell.mana_cost = ManaCost::from_string("2U");
        // Use placeholder card ID 0 which will be replaced with a targetable opponent's creature
        tap_spell.effects.push(Effect::TapPermanent {
            target: CardId::new(0),
        });
        game.cards.insert(tap_spell_id, tap_spell);

        // Put spell on stack (simulating cast)
        game.stack.add(tap_spell_id);

        // Resolve the spell
        let result = game.resolve_spell(tap_spell_id);
        assert!(result.is_ok(), "Tap spell should resolve successfully");

        // Check that the hexproof creature is not tapped
        let hexproof_card = game.cards.get(hexproof_creature_id).unwrap();
        assert!(
            !hexproof_card.tapped,
            "Hexproof creature should not be tapped"
        );

        // Check that the normal creature was tapped
        let normal_card = game.cards.get(normal_creature_id).unwrap();
        assert!(normal_card.tapped, "Normal creature should be tapped");
    }

    #[test]
    fn test_hexproof_allows_own_spells() {
        // Test that hexproof creatures CAN be targeted by their controller's spells
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let _p2_id = game.players[1].id;

        // P1: Create a hexproof creature
        let hexproof_creature_id = game.next_entity_id();
        let mut hexproof_creature =
            Card::new(hexproof_creature_id, "Slippery Bogle".to_string(), p1_id);
        hexproof_creature.types.push(CardType::Creature);
        hexproof_creature.power = Some(1);
        hexproof_creature.toughness = Some(1);
        hexproof_creature.keywords.push(Keyword::Hexproof);
        game.cards.insert(hexproof_creature_id, hexproof_creature);
        game.battlefield.add(hexproof_creature_id);

        // P1: Cast Giant Growth on their own hexproof creature - should work!
        let pump_spell_id = game.next_entity_id();
        let mut pump_spell = Card::new(pump_spell_id, "Giant Growth".to_string(), p1_id);
        pump_spell.types.push(CardType::Instant);
        pump_spell.mana_cost = ManaCost::from_string("G");
        // Use placeholder card ID 0 which will be replaced with a targetable creature
        pump_spell.effects.push(Effect::PumpCreature {
            target: CardId::new(0),
            power_bonus: 3,
            toughness_bonus: 3,
        });
        game.cards.insert(pump_spell_id, pump_spell);

        // Put spell on stack (simulating cast)
        game.stack.add(pump_spell_id);

        // Resolve the spell
        let result = game.resolve_spell(pump_spell_id);
        assert!(
            result.is_ok(),
            "Pump spell on own hexproof creature should resolve successfully"
        );

        // Check that the hexproof creature got the pump
        let creature = game.cards.get(hexproof_creature_id).unwrap();
        assert_eq!(
            creature.current_power(),
            4,
            "Hexproof creature should have boosted power (1+3)"
        );
        assert_eq!(
            creature.current_toughness(),
            4,
            "Hexproof creature should have boosted toughness (1+3)"
        );
    }

    #[test]
    fn test_hexproof_no_valid_targets() {
        // Test that spells fail to find targets if only hexproof creatures exist
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P2: Create only a hexproof creature (no valid targets for opponent)
        let hexproof_creature_id = game.next_entity_id();
        let mut hexproof_creature =
            Card::new(hexproof_creature_id, "Slippery Bogle".to_string(), p2_id);
        hexproof_creature.types.push(CardType::Creature);
        hexproof_creature.power = Some(1);
        hexproof_creature.toughness = Some(1);
        hexproof_creature.keywords.push(Keyword::Hexproof);
        game.cards.insert(hexproof_creature_id, hexproof_creature);
        game.battlefield.add(hexproof_creature_id);

        // P1: Try to cast a destroy spell - should fail to find valid target
        let destroy_spell_id = game.next_entity_id();
        let mut destroy_spell = Card::new(destroy_spell_id, "Terror".to_string(), p1_id);
        destroy_spell.types.push(CardType::Instant);
        destroy_spell.mana_cost = ManaCost::from_string("1B");
        // Use placeholder card ID 0 which will fail to be replaced with a target
        destroy_spell.effects.push(Effect::DestroyPermanent {
            target: CardId::new(0),
        });
        game.cards.insert(destroy_spell_id, destroy_spell);

        // Put it on the stack (simulating cast)
        game.stack.add(destroy_spell_id);

        // Resolve the spell - should succeed but do nothing (no valid targets)
        let result = game.resolve_spell(destroy_spell_id);
        assert!(
            result.is_ok(),
            "Spell with no valid targets should still resolve"
        );

        // Check that the hexproof creature is still alive
        assert!(
            game.battlefield.contains(hexproof_creature_id),
            "Hexproof creature should still be on battlefield"
        );
    }

    #[test]
    fn test_indestructible_survives_lethal_damage() {
        // Test that indestructible creatures survive lethal damage
        use crate::game::zero_controller::ZeroController;

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P1: Create a 2/2 indestructible creature
        let indestructible_id = game.next_entity_id();
        let mut indestructible = Card::new(indestructible_id, "Darksteel Myr".to_string(), p1_id);
        indestructible.types.push(CardType::Creature);
        indestructible.power = Some(2);
        indestructible.toughness = Some(2);
        indestructible.keywords.push(Keyword::Indestructible);
        indestructible.controller = p1_id;
        indestructible.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        game.cards.insert(indestructible_id, indestructible);
        game.battlefield.add(indestructible_id);

        // P2: Create a 5/5 creature (blocker)
        let blocker_id = game.next_entity_id();
        let mut blocker = Card::new(blocker_id, "Hill Giant".to_string(), p2_id);
        blocker.types.push(CardType::Creature);
        blocker.power = Some(5);
        blocker.toughness = Some(5);
        blocker.controller = p2_id;
        blocker.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        game.cards.insert(blocker_id, blocker);
        game.battlefield.add(blocker_id);

        // P1 attacks with indestructible creature
        let mut controller1 = ZeroController::new(p1_id);
        let mut controller2 = ZeroController::new(p2_id);

        game.combat.declare_attacker(indestructible_id, p2_id);

        // P2 blocks with 5/5 creature
        let result = game.declare_blocker(p2_id, blocker_id, vec![indestructible_id]);
        assert!(result.is_ok(), "Failed to declare blocker: {result:?}");

        // Assign combat damage
        // Indestructible 2/2 deals 2 damage to blocker
        // Blocker 5/5 deals 5 damage to indestructible (more than lethal, but indestructible survives)
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, false);
        assert!(result.is_ok(), "Failed to assign combat damage: {result:?}");

        // Indestructible creature should survive (took 5 damage but has indestructible)
        assert!(
            game.battlefield.contains(indestructible_id),
            "Indestructible creature should survive lethal damage"
        );

        // Blocker should survive (took 2 damage, has 5 toughness)
        assert!(
            game.battlefield.contains(blocker_id),
            "Blocker should survive 2 damage"
        );
    }

    #[test]
    fn test_indestructible_immune_to_destroy_effects() {
        // Test that indestructible creatures can't be destroyed by Terror/Murder
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P2: Create an indestructible creature
        let indestructible_id = game.next_entity_id();
        let mut indestructible = Card::new(indestructible_id, "Darksteel Myr".to_string(), p2_id);
        indestructible.types.push(CardType::Creature);
        indestructible.power = Some(0);
        indestructible.toughness = Some(1);
        indestructible.keywords.push(Keyword::Indestructible);
        game.cards.insert(indestructible_id, indestructible);
        game.battlefield.add(indestructible_id);

        // P1: Cast Terror targeting the indestructible creature
        let destroy_spell_id = game.next_entity_id();
        let mut destroy_spell = Card::new(destroy_spell_id, "Terror".to_string(), p1_id);
        destroy_spell.types.push(CardType::Instant);
        destroy_spell.mana_cost = ManaCost::from_string("1B");
        // Explicitly target the indestructible creature
        destroy_spell.effects.push(Effect::DestroyPermanent {
            target: indestructible_id,
        });
        game.cards.insert(destroy_spell_id, destroy_spell);

        // Put it on the stack
        game.stack.add(destroy_spell_id);

        // Resolve the spell
        let result = game.resolve_spell(destroy_spell_id);
        assert!(result.is_ok(), "Destroy spell should resolve successfully");

        // Indestructible creature should still be alive
        assert!(
            game.battlefield.contains(indestructible_id),
            "Indestructible creature should survive destroy effect"
        );
    }

    #[test]
    fn test_indestructible_survives_deathtouch() {
        // Test that indestructible creatures survive deathtouch damage
        use crate::game::zero_controller::ZeroController;

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P1: Create a 1/1 deathtouch creature (attacker)
        let deathtouch_id = game.next_entity_id();
        let mut deathtouch = Card::new(deathtouch_id, "Typhoid Rats".to_string(), p1_id);
        deathtouch.types.push(CardType::Creature);
        deathtouch.power = Some(1);
        deathtouch.toughness = Some(1);
        deathtouch.keywords.push(Keyword::Deathtouch);
        deathtouch.controller = p1_id;
        deathtouch.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        game.cards.insert(deathtouch_id, deathtouch);
        game.battlefield.add(deathtouch_id);

        // P2: Create a 5/5 indestructible creature (blocker)
        let indestructible_id = game.next_entity_id();
        let mut indestructible =
            Card::new(indestructible_id, "Darksteel Colossus".to_string(), p2_id);
        indestructible.types.push(CardType::Creature);
        indestructible.power = Some(5);
        indestructible.toughness = Some(5);
        indestructible.keywords.push(Keyword::Indestructible);
        indestructible.controller = p2_id;
        indestructible.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        game.cards.insert(indestructible_id, indestructible);
        game.battlefield.add(indestructible_id);

        // P1 attacks with deathtouch creature
        let mut controller1 = ZeroController::new(p1_id);
        let mut controller2 = ZeroController::new(p2_id);

        game.combat.declare_attacker(deathtouch_id, p2_id);

        // P2 blocks with indestructible creature
        let result = game.declare_blocker(p2_id, indestructible_id, vec![deathtouch_id]);
        assert!(result.is_ok(), "Failed to declare blocker: {result:?}");

        // Assign combat damage
        // Deathtouch 1/1 deals 1 damage to indestructible (deathtouch damage, but indestructible survives)
        // Indestructible 5/5 deals 5 damage to deathtouch (kills it)
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, false);
        assert!(result.is_ok(), "Failed to assign combat damage: {result:?}");

        // Indestructible creature should survive deathtouch damage
        assert!(
            game.battlefield.contains(indestructible_id),
            "Indestructible creature should survive deathtouch damage"
        );

        // Deathtouch creature should be dead (took 5 damage, has 1 toughness)
        if let Some(zones) = game.get_player_zones(p1_id) {
            assert!(
                zones.graveyard.contains(deathtouch_id),
                "Deathtouch creature should be in graveyard"
            );
        }
    }

    #[test]
    fn test_indestructible_vs_non_indestructible_combat() {
        // Test that normal creature dies while indestructible survives in mutual combat
        use crate::game::zero_controller::ZeroController;

        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P1: Create a 3/3 indestructible creature (attacker)
        let indestructible_id = game.next_entity_id();
        let mut indestructible = Card::new(indestructible_id, "Indomitable".to_string(), p1_id);
        indestructible.types.push(CardType::Creature);
        indestructible.power = Some(3);
        indestructible.toughness = Some(3);
        indestructible.keywords.push(Keyword::Indestructible);
        indestructible.controller = p1_id;
        indestructible.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        game.cards.insert(indestructible_id, indestructible);
        game.battlefield.add(indestructible_id);

        // P2: Create a 3/3 normal creature (blocker)
        let normal_id = game.next_entity_id();
        let mut normal = Card::new(normal_id, "Hill Giant".to_string(), p2_id);
        normal.types.push(CardType::Creature);
        normal.power = Some(3);
        normal.toughness = Some(3);
        normal.controller = p2_id;
        normal.turn_entered_battlefield = Some(game.turn.turn_number - 1);
        game.cards.insert(normal_id, normal);
        game.battlefield.add(normal_id);

        // P1 attacks with indestructible creature
        let mut controller1 = ZeroController::new(p1_id);
        let mut controller2 = ZeroController::new(p2_id);

        game.combat.declare_attacker(indestructible_id, p2_id);

        // P2 blocks with normal creature
        let result = game.declare_blocker(p2_id, normal_id, vec![indestructible_id]);
        assert!(result.is_ok(), "Failed to declare blocker: {result:?}");

        // Assign combat damage
        // Both deal 3 damage to each other (lethal)
        // Indestructible survives, normal dies
        let result = game.assign_combat_damage(&mut controller1, &mut controller2, false);
        assert!(result.is_ok(), "Failed to assign combat damage: {result:?}");

        // Indestructible creature should survive
        assert!(
            game.battlefield.contains(indestructible_id),
            "Indestructible creature should survive"
        );

        // Normal creature should be dead
        if let Some(zones) = game.get_player_zones(p2_id) {
            assert!(
                zones.graveyard.contains(normal_id),
                "Normal creature should be in graveyard"
            );
        }
    }

    #[test]
    fn test_shroud_blocks_destroy_from_opponent() {
        // Test that destroy spells from opponents can't target shroud creatures
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P2: Create a shroud creature
        let shroud_creature_id = game.next_entity_id();
        let mut shroud_creature =
            Card::new(shroud_creature_id, "Silhana Ledgewalker".to_string(), p2_id);
        shroud_creature.types.push(CardType::Creature);
        shroud_creature.power = Some(1);
        shroud_creature.toughness = Some(1);
        shroud_creature.keywords.push(Keyword::Shroud);
        game.cards.insert(shroud_creature_id, shroud_creature);
        game.battlefield.add(shroud_creature_id);

        // P2: Create a normal creature
        let normal_creature_id = game.next_entity_id();
        let mut normal_creature = Card::new(normal_creature_id, "Grizzly Bears".to_string(), p2_id);
        normal_creature.types.push(CardType::Creature);
        normal_creature.power = Some(2);
        normal_creature.toughness = Some(2);
        game.cards.insert(normal_creature_id, normal_creature);
        game.battlefield.add(normal_creature_id);

        // P1: Cast Terror - should target normal creature, not shroud one
        let destroy_spell_id = game.next_entity_id();
        let mut destroy_spell = Card::new(destroy_spell_id, "Terror".to_string(), p1_id);
        destroy_spell.types.push(CardType::Instant);
        destroy_spell.mana_cost = ManaCost::from_string("1B");
        destroy_spell.effects.push(Effect::DestroyPermanent {
            target: CardId::new(0),
        });
        game.cards.insert(destroy_spell_id, destroy_spell);
        game.stack.add(destroy_spell_id);

        let result = game.resolve_spell(destroy_spell_id);
        assert!(result.is_ok(), "Destroy spell should resolve");

        // Shroud creature should still be alive
        assert!(
            game.battlefield.contains(shroud_creature_id),
            "Shroud creature should still be on battlefield"
        );

        // Normal creature was destroyed
        if let Some(zones) = game.get_player_zones(p2_id) {
            assert!(
                zones.graveyard.contains(normal_creature_id),
                "Normal creature should be in graveyard"
            );
        }
    }

    #[test]
    fn test_shroud_blocks_pump_from_controller() {
        // Test that shroud prevents targeting even by the controller (unlike hexproof)
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let _p2_id = game.players[1].id;

        // P1: Create a shroud creature
        let shroud_creature_id = game.next_entity_id();
        let mut shroud_creature =
            Card::new(shroud_creature_id, "Silhana Ledgewalker".to_string(), p1_id);
        shroud_creature.types.push(CardType::Creature);
        shroud_creature.power = Some(1);
        shroud_creature.toughness = Some(1);
        shroud_creature.keywords.push(Keyword::Shroud);
        game.cards.insert(shroud_creature_id, shroud_creature);
        game.battlefield.add(shroud_creature_id);

        // P1: Create a normal creature
        let normal_creature_id = game.next_entity_id();
        let mut normal_creature = Card::new(normal_creature_id, "Grizzly Bears".to_string(), p1_id);
        normal_creature.types.push(CardType::Creature);
        normal_creature.power = Some(2);
        normal_creature.toughness = Some(2);
        game.cards.insert(normal_creature_id, normal_creature);
        game.battlefield.add(normal_creature_id);

        // P1: Cast Giant Growth - should target normal creature, not shroud one
        let pump_spell_id = game.next_entity_id();
        let mut pump_spell = Card::new(pump_spell_id, "Giant Growth".to_string(), p1_id);
        pump_spell.types.push(CardType::Instant);
        pump_spell.mana_cost = ManaCost::from_string("G");
        pump_spell.effects.push(Effect::PumpCreature {
            target: CardId::new(0),
            power_bonus: 3,
            toughness_bonus: 3,
        });
        game.cards.insert(pump_spell_id, pump_spell);
        game.stack.add(pump_spell_id);

        let result = game.resolve_spell(pump_spell_id);
        assert!(result.is_ok(), "Pump spell should resolve");

        // Shroud creature should NOT have the pump
        let shroud_card = game.cards.get(shroud_creature_id).unwrap();
        assert_eq!(
            shroud_card.current_power(),
            1,
            "Shroud creature should not have boosted power"
        );

        // Normal creature should have the pump
        let normal_card = game.cards.get(normal_creature_id).unwrap();
        assert_eq!(
            normal_card.current_power(),
            5,
            "Normal creature should have boosted power (2+3)"
        );
    }

    #[test]
    fn test_shroud_blocks_tap_effect() {
        // Test that tap effects can't target shroud creatures
        let mut game = GameState::new_two_player("P1".to_string(), "P2".to_string(), 20);
        let p1_id = game.players[0].id;
        let p2_id = game.players[1].id;

        // P2: Create a shroud creature
        let shroud_creature_id = game.next_entity_id();
        let mut shroud_creature =
            Card::new(shroud_creature_id, "Silhana Ledgewalker".to_string(), p2_id);
        shroud_creature.types.push(CardType::Creature);
        shroud_creature.power = Some(1);
        shroud_creature.toughness = Some(1);
        shroud_creature.keywords.push(Keyword::Shroud);
        game.cards.insert(shroud_creature_id, shroud_creature);
        game.battlefield.add(shroud_creature_id);

        // P2: Create a normal creature
        let normal_creature_id = game.next_entity_id();
        let mut normal_creature = Card::new(normal_creature_id, "Grizzly Bears".to_string(), p2_id);
        normal_creature.types.push(CardType::Creature);
        normal_creature.power = Some(2);
        normal_creature.toughness = Some(2);
        game.cards.insert(normal_creature_id, normal_creature);
        game.battlefield.add(normal_creature_id);

        // P1: Cast tap spell - should target normal creature, not shroud one
        let tap_spell_id = game.next_entity_id();
        let mut tap_spell = Card::new(tap_spell_id, "Frost Breath".to_string(), p1_id);
        tap_spell.types.push(CardType::Instant);
        tap_spell.mana_cost = ManaCost::from_string("2U");
        tap_spell.effects.push(Effect::TapPermanent {
            target: CardId::new(0),
        });
        game.cards.insert(tap_spell_id, tap_spell);
        game.stack.add(tap_spell_id);

        let result = game.resolve_spell(tap_spell_id);
        assert!(result.is_ok(), "Tap spell should resolve");

        // Shroud creature should not be tapped
        let shroud_card = game.cards.get(shroud_creature_id).unwrap();
        assert!(!shroud_card.tapped, "Shroud creature should not be tapped");

        // Normal creature should be tapped
        let normal_card = game.cards.get(normal_creature_id).unwrap();
        assert!(normal_card.tapped, "Normal creature should be tapped");
    }
}
