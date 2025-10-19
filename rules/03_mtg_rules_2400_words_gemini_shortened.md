---
created: 2025-10-19_12:17
updated: 2025-10-19_12:21
---
# Summary of Magic: The Gathering Core Rules

This document provides a summary of the core rules for a standard, two-player game of Magic: The Gathering. The citations included, such as `(Rules 101.1)`, refer to the specific rule numbers in the official Magic Comprehensive Rules document dated September 19, 2025. This summary omits rules for multiplayer, casual variants (like Commander), and other non-standard game elements to focus on the foundational concepts.

## Table of Contents
1.  [Game Concepts](#1-game-concepts)
2.  [Parts of a Card](#2-parts-of-a-card)
3.  [Card Types](#3-card-types)
4.  [Zones](#4-zones)
5.  [Turn Structure](#5-turn-structure)
6.  [Spells, Abilities, and Effects](#6-spells-abilities-and-effects)

---

## 1. Game Concepts

### 1.1 The Goal of the Game
A game of Magic is typically played between two players. Each player starts with 20 life `(Rules 103.4)`. The primary goal is to win the game, which usually happens when your opponent's life total becomes 0 or less `(Rules 104.3b)`.

### 1.2 Deck Construction
* Each player needs their own deck of at least 60 cards `(Rules 100.2a)`. There is no maximum deck size `(Rules 100.5)`.
* With the exception of basic lands (Plains, Island, Swamp, Mountain, Forest), a deck may not contain more than four copies of any single card, based on its English name `(Rules 100.2a)`.
* Players may also have a sideboard of up to 15 cards, which can be used to modify their deck between games in a match `(Rules 100.4a)`.

### 1.3 The Magic Golden Rules
* **Rule 101.1:** If a card's text directly contradicts a game rule, the card takes precedence `(Rules 101.1)`.
* **Rule 101.2:** If one effect says something "can" happen and another says it "can't," the "can't" effect wins `(Rules 101.2)`.
* **Rule 101.4 (APNAP Order):** If multiple players must make choices or take actions at the same time, the **A**ctive **P**layer (the player whose turn it is) makes their choices first, followed by the **N**on-**A**ctive **P**layer. Then, all actions occur simultaneously `(Rules 101.4)`.

### 1.4 Starting the Game
1.  For the first game of a match, players use a random method (like flipping a coin) to decide who goes first. In subsequent games, the loser of the previous game decides who goes first `(Rules 103.1)`.
2.  Each player shuffles their deck, which then becomes their library `(Rules 103.3)`.
3.  Each player's starting life total is 20 `(Rules 103.4)`.
4.  Each player draws an opening hand of seven cards `(Rules 103.5)`.
5.  A player who is unsatisfied with their hand may take a **mulligan**. To do so, the player shuffles their hand back into their library, draws a new hand of seven cards, and then puts a number of cards from their new hand on the bottom of their library equal to the number of times they have taken a mulligan this game. This process can be repeated until the player keeps a hand `(Rules 103.5)`.
6.  The player who takes the first turn skips the draw step of that turn `(Rules 103.8a)`.

### 1.5 Ending the Game
A game can end in one of the following ways:
* **A player loses if:**
    * Their life total is 0 or less `(Rules 104.3b)`.
    * They attempt to draw a card from a library with no cards in it `(Rules 104.3c)`.
    * They have ten or more poison counters `(Rules 104.3d)`.
    * An effect states that they lose the game `(Rules 104.3e)`.
    * They concede `(Rules 104.3a)`.
* **A player wins if:**
    * Their opponent loses the game `(Rules 104.2a)`.
    * An effect states that they win the game `(Rules 104.2b)`.
* **The game is a draw if:**
    * Both players would lose the game simultaneously `(Rules 104.4a)`.
    * The game enters a "loop" of mandatory actions that cannot be stopped `(Rules 104.4b)`.

### 1.6 Mana and Colors
* There are five colors in Magic: white (W), blue (U), black (B), red (R), and green (G) `(Rules 105.1)`. An object's color is determined by the mana symbols in its mana cost `(Rules 202.2)`. Objects with no colored mana symbols are colorless `(Rules 202.2b)`.
* Mana is the primary resource, used to cast spells and activate abilities. There are six types of mana: the five colors and colorless `(Rules 106.1b)`.
* When an effect produces mana, it is added to a player's **mana pool** `(Rules 106.4)`. This mana pool empties at the end of every step and phase `(Rules 500.4)`.

---

## 2. Parts of a Card

A card's characteristics are its name, mana cost, color, type line, abilities, power, and toughness. The official wording of a card is its Oracle text, which can be found in the Gatherer database at [Gatherer.Wizards.com](https://Gatherer.Wizards.com) `(Rules 108.1)`.

* **Name (Rules 201):** The card's name, printed at the top.
* **Mana Cost & Mana Value (Rules 202):** The mana symbols in the upper corner represent the mana cost `(Rules 202.1)`. The **mana value** is the total amount of mana in the cost, regardless of color `(Rules 202.3)`. If an object has no mana cost, its mana value is 0 `(Rules 202.3a)`. For costs with an {X}, X is treated as 0 everywhere except on the stack, where it is the value chosen for it `(Rules 202.3e)`.
* **Type Line (Rules 205):** Located below the art, it displays the card's type(s), supertype(s) (such as Basic or Legendary), and subtype(s) (such as Goblin or Wizard).
* **Text Box (Rules 207):** Contains the card's rules text, which defines its abilities. It may also contain italicized *reminder text* or *flavor text*, which have no effect on gameplay `(Rules 207.2)`.
* **Power/Toughness (P/T) (Rules 208):** Found on creature cards in the bottom right corner as two numbers separated by a slash (e.g., 2/3). The first number is power (damage dealt in combat), and the second is toughness (damage required to destroy it) `(Rules 208.1)`.

---

## 3. Card Types

There are several card types, which fall into two main categories: permanents and non-permanents.

### 3.1 Permanents
Permanents are cards or tokens that exist on the battlefield `(Rules 110.1)`. They are typically cast during your main phase when the stack is empty `(Rules 301.1, 302.1, 303.1, 306.1)`.
* **Creature (Rules 302):** Represent creatures that can attack and block. Creatures are affected by "summoning sickness": a creature cannot attack or use abilities with the tap symbol ({T}) unless its controller has controlled it continuously since the beginning of their most recent turn `(Rules 302.6)`.
* **Land (Rules 305):** Lands are played, not cast, and do not use the stack `(Rules 305.1)`. A player may normally play only one land per turn, during their main phase when the stack is empty `(Rules 305.2)`. The five basic land types (Plains, Island, Swamp, Mountain, Forest) have an intrinsic ability to tap for one mana of their corresponding color `(Rules 305.6)`.
* **Artifact (Rules 301):** Represent magical items, machines, or constructs. Many are colorless.
* **Enchantment (Rules 303):** Represent persistent magical effects. Some are **Auras**, which are attached to another object or player when they enter the battlefield `(Rules 303.4)`.
* **Planeswalker (Rules 306):** Represent powerful allies. They enter the battlefield with a number of loyalty counters equal to the number in their lower right corner `(Rules 306.5b)`. Players can activate one of their loyalty abilities per turn `(Rules 306.5d)`. Planeswalkers can be attacked by creatures `(Rules 306.6)`.

### 3.2 Non-Permanents (Spells)
When a non-permanent spell resolves, you follow its instructions, and then it is put into its owner's graveyard `(Rules 304.2, 307.2)`.
* **Sorcery (Rules 307):** Can only be cast by the active player during one of their main phases when the stack is empty `(Rules 307.1)`.
* **Instant (Rules 304):** Can be cast any time a player has priority `(Rules 304.1)`.

---

## 4. Zones

Objects in the game exist in one of several zones. When a card moves from one zone to another, it becomes a new object with no memory of its previous existence, with a few exceptions `(Rules 400.7)`.

* **Library (Rules 401):** Each player's deck, kept face down. Players draw cards from the top of their library.
* **Hand (Rules 402):** The cards a player has drawn but not yet played. This is a hidden zone `(Rules 400.2)`. The maximum hand size is normally seven; players must discard down to this number at the end of their turn `(Rules 402.2)`.
* **Battlefield (Rules 403):** The shared area where permanents (creatures, lands, artifacts, etc.) exist.
* **Graveyard (Rules 404):** A player's discard pile. Cards here are face-up and public information `(Rules 404.2)`. Used spells, destroyed permanents, and discarded cards go here.
* **The Stack (Rules 405):** A shared zone where spells and abilities wait to resolve. It functions on a "Last-In, First-Out" (LIFO) basis: the last spell or ability added to the stack is the first to resolve `(Rules 405.5)`. After an object on the stack resolves, players get a chance to add more objects before the next one resolves.
* **Exile (Rules 406):** A zone for cards that have been removed from the game by an effect `(Rules 406.1)`. Cards in exile are normally face-up.

---

## 5. Turn Structure

A turn is divided into five phases, and some phases are divided into steps.

1.  **Beginning Phase `(Rules 501)`:**
    * **Untap Step (Rules 502):** The active player untaps all of their tapped permanents simultaneously `(Rules 502.3)`. No player receives priority `(Rules 502.4)`.
    * **Upkeep Step (Rules 503):** Abilities that trigger "at the beginning of your upkeep" are put on the stack. Players receive priority `(Rules 503.1)`.
    * **Draw Step (Rules 504):** The active player draws a card (unless it is the first turn of the game) `(Rules 504.1, 103.8a)`. Players receive priority `(Rules 504.2)`.

2.  **Precombat Main Phase (Rules 505):** The active player receives priority and may cast spells (creatures, sorceries, etc.) and play a land `(Rules 505.6)`.

3.  **Combat Phase (Rules 506):**
    * **Beginning of Combat Step (Rules 507):** Players receive priority and can cast instants or activate abilities before attackers are declared.
    * **Declare Attackers Step (Rules 508):** The active player declares which of their untapped creatures are attacking. Attacking creatures become tapped `(Rules 508.1f)` unless they have the **Vigilance** ability `(Rules 702.20b)`.
    * **Declare Blockers Step (Rules 509):** The defending player declares which of their untapped creatures will block which attacking creatures.
    * **Combat Damage Step (Rules 510):** Attacking and blocking creatures deal combat damage equal to their power simultaneously `(Rules 510.2)`. Unblocked creatures deal damage to the player or planeswalker they are attacking. Blocked creatures deal damage to the creatures blocking them, and vice versa. Creatures with **First Strike** or **Double Strike** deal damage in an additional, earlier combat damage step `(Rules 510.4)`.
    * **End of Combat Step (Rules 511):** A final chance for players to cast instants and activate abilities before combat ends.

4.  **Postcombat Main Phase (Rules 505):** Same as the precombat main phase. The active player can cast spells and play a land if they haven't already done so.

5.  **Ending Phase `(Rules 512)`:**
    * **End Step (Rules 513):** Abilities that trigger "at the beginning of the end step" go on the stack. Players receive priority.
    * **Cleanup Step (Rules 514):** The active player discards cards from their hand until they have no more than their maximum hand size (usually seven) `(Rules 514.1)`. Then, all damage marked on creatures is removed, and all effects that last "until end of turn" expire `(Rules 514.2)`.

---

## 6. Spells, Abilities, and Effects

### 6.1 Casting Spells (Rules 601)
To cast a spell, a player follows a sequence `(Rules 601.2)`:
1.  Move the card from its current zone (usually the hand) to the stack.
2.  Announce choices, such as modes, values for X, and whether to pay additional costs like Kicker.
3.  Choose targets for the spell.
4.  Determine the total cost to cast the spell, applying any cost increases or reductions.
5.  Activate mana abilities if needed.
6.  Pay the total cost.
Once these steps are complete, the spell is considered "cast," and any abilities that trigger on a spell being cast will trigger `(Rules 601.2i)`.

### 6.2 Types of Abilities (Rules 113)
* **Activated Abilities:** Written as "[Cost]: [Effect]." A player can activate these any time they have priority, provided they can pay the cost `(Rules 113.3b)`.
* **Triggered Abilities:** Written as "When/Whenever/At [event], [effect]." These trigger automatically when the specified event occurs and are put on the stack the next time a player gets priority `(Rules 113.3c)`.
* **Static Abilities:** Written as statements (e.g., "Creatures you control have flying"). They are continuously active as long as the object with the ability is in the appropriate zone `(Rules 113.3d)`.
* **Mana Abilities:** Activated or triggered abilities that add mana to a player's mana pool, don't have targets, and are not loyalty abilities. They do not use the stack and resolve immediately `(Rules 605, 605.3b)`.

### 6.3 The Stack and Resolving Spells
When a spell is cast or an ability is activated or triggered, it is put on top of the stack. Players can then respond by casting instants or activating abilities, which go on top of the stack above the original spell/ability. When all players pass priority in succession, the top object on the stack resolves `(Rules 405.5)`.

When a spell or ability resolves, it checks if its targets are still legal. If all of its targets have become illegal, the spell or ability is removed from the stack and has no effect. Otherwise, its controller follows its instructions `(Rules 608.2b)`.

### 6.4 Continuous Effects and Layers (Rules 613)
If multiple continuous effects (from static abilities, spells, etc.) are active at the same time, they are applied in a specific sequence of "layers" to determine an object's final characteristics. The general order is `(Rules 613.1)`:
1.  Copy effects.
2.  Control-changing effects.
3.  Text-changing effects.
4.  Type-changing effects.
5.  Color-changing effects.
6.  Ability-adding/removing effects.
7.  Power/Toughness-changing effects.

Within a layer, effects are usually applied in **timestamp order**, meaning the newest effect is applied last `(Rules 613.7)`.

### 6.5 Replacement and Prevention Effects (Rules 614, 615)
These are continuous effects that watch for a specific event and modify it.
* **Replacement Effects** use words like "instead" or "skip" to replace an event with a different one (e.g., "If you would draw a card, instead you gain 2 life") `(Rules 614.1a, 614.1b)`.
* **Prevention Effects** use the word "prevent" to stop some or all of a damage event (e.g., "Prevent the next 3 damage that would be dealt to target creature") `(Rules 615.1a)`.

If multiple replacement/prevention effects could apply to the same event, the controller of the affected object or the affected player chooses which one to apply first `(Rules 616.1)`.