

# **A Condensed Guide to the Comprehensive Rules of Magic: The Gathering**

## **Introduction**

### **Purpose and Scope**

This document serves as a detailed summary of the *Magic: The Gathering Comprehensive Rules*, focusing on the foundational principles of the standard two-player game. It is designed for players who wish to deepen their understanding of the game's formal structure beyond a basic level, providing an authoritative reference for the core mechanics that govern gameplay. The content is structured to build from foundational concepts to the intricacies of timing, interaction, and effect resolution, creating a logical path through the game's legal framework.

### **Citation Format**

Throughout this guide, references to the official *Magic: The Gathering Comprehensive Rules* (effective as of September 19, 2025\) will be made using the format (Rules X.Y), where X is the rule number and Y is the sub-rule. This allows for direct cross-referencing with the source document for players seeking the full, unabridged text on a particular interaction.

### **Omissions**

This guide intentionally omits rules pertaining to multiplayer formats (Section 800), casual variants like Commander and Planechase (Section 900), and numerous highly specific or obscure card interactions to maintain clarity and focus on the core gameplay experience. A detailed list of omissions is provided in the conclusion.

## **Section 1: Core Game Concepts**

This section establishes the foundational principles and components of the game, forming the bedrock upon which all other rules are built.

### **1.1. The Golden Rules of Magic**

The entire game of *Magic* is governed by a few fundamental principles that resolve conflicts between card effects and the game's core rules. Understanding this hierarchy is the first step to mastering the rules.

* **Card Text Supersedes Rules:** The single most important rule in *Magic* is that if a card's text directly contradicts a game rule, the card's text takes precedence. The card overrides only the specific rule that applies to that situation. The only absolute rule that a card cannot override is a player's ability to concede the game at any time (Rules 101.1). This principle establishes cards as the primary agents of change and exception within the game's structured environment, allowing for constant innovation in design.  
* **"Can't" Overrules "Can":** If one effect allows or directs a player to do something, and another effect states that they can't, the "can't" effect always takes precedence (Rules 101.2). This is a crucial conflict resolution tool that prevents paradoxical game states. For example, if one effect says, "You may play an additional land this turn," and another says, "You can't play lands this turn," the player is unable to play lands.  
* **Impossible Actions:** Any part of an instruction that is impossible to perform is simply ignored. The rest of the effect is carried out to the extent possible. In many cases, a card will specify consequences for being unable to perform an action; if it does not, there is no further effect (Rules 101.3).  
* **APNAP Order:** If multiple players must make choices or take actions at the same time, the **A**ctive **P**layer (the player whose turn it is) makes all of their required choices first. Then, each **N**on-**A**ctive **P**layer does the same in turn order. After all choices have been made, all the actions happen simultaneously (Rules 101.4). This is known as APNAP order. For example, if an effect reads, "Each player sacrifices a creature," the active player chooses a creature they control, then the non-active player chooses a creature they control. After both choices are made, the two creatures are sacrificed at the same time. This procedural solution ensures that simultaneous actions are handled in a fair, consistent, and deterministic sequence.

These golden rules establish a clear hierarchy of authority: a card's text is paramount, restrictive effects ("can't") are absolute, and procedural order (APNAP) governs simultaneous decisions. This framework ensures that even the most complex and novel interactions have a clear path to resolution.

### **1.2. Game Objects and Characteristics**

The game uses specific terminology to refer to the pieces and their properties.

* An **object** is an ability on the stack, a card, a copy of a card, a token, a spell, a permanent, or an emblem (Rules 109.1). Essentially, any component that exists within the game's framework is an object.  
* An object's **characteristics** are its fundamental properties: name, mana cost, color, color indicator, card type, subtype, supertype, rules text, abilities, power, toughness, loyalty, and defense (Rules 109.3). An object's status—such as whether it is tapped, face down, or flipped—is not a characteristic.  
* **Owner vs. Controller:** These are two distinct and important concepts. The **owner** of a card is the player who started the game with it in their deck. This never changes. The **controller** of a spell is the player who cast it, and the controller of a permanent is, by default, the player under whose control it entered the battlefield (Rules 108.3, 110.2). Control of permanents can change during the game through various effects. If a rule needs to know the controller of an object that doesn't have one (like a card in a graveyard), it uses the object's owner instead (Rules 108.4a).

### **1.3. The Five Colors**

Color is a central theme and mechanical element of *Magic*.

* The five colors are **White** ($W$), **Blue** ($U$), **Black** ($B$), **Red** ($R$), and **Green** ($G$) (Rules 105.1).  
* An object's color is determined primarily by the colored mana symbols in its mana cost. For example, a card with a mana cost of {$2$}{$W$} is white (Rules 202.2).  
  * An object with no colored mana symbols in its cost is **colorless**.  
  * An object with exactly one color is **monocolored**.  
  * An object with two or more colors is **multicolored**.  
* A **color indicator**—a colored dot that appears on the type line of some cards—can also define an object's color. This is most common on cards that have no mana cost but are intended to be colored, such as the back faces of some double-faced cards (Rules 204).

### **1.4. Mana: The Game's Primary Resource**

Mana is the energy players use to cast spells and activate abilities.

* There are six types of mana: the five colored types (White, Blue, Black, Red, Green) and **colorless** (Rules 106.1b).  
* **Mana Pool:** When a spell or ability produces mana, that mana is added to a player's **mana pool**. This is a temporary holding area for mana. Mana in the pool can be spent immediately to pay costs. Any mana that is not spent is lost at the end of each step and phase of the turn; it does not carry over (Rules 106.4).  
* **Mana Symbols:** Costs are represented by mana symbols.  
  * **Colored Mana Symbols** ({$W$}, {$U$}, {$B$}, {$R$}, {$G$}) require one mana of that specific color.  
  * **Generic Mana Symbols** ({$0$}, {$1$}, {$2$}, etc.) can be paid with mana of any type, colored or colorless.  
  * **Colorless Mana Symbol** ({$C$}) represents a cost that must be paid with one colorless mana. This is distinct from generic mana (Rules 107.4).  
  * **Hybrid Mana Symbols** (e.g., {$W/U$}) represent a cost that can be paid with one mana of either of its component colors. An object with a hybrid symbol in its cost is all of its component colors (Rules 107.4e).  
  * **Phyrexian Mana Symbols** (e.g., {$W/P$}) represent a cost that can be paid with either one mana of its color or by paying 2 life (Rules 107.4f).

### **1.5. The Seven Game Zones**

Every object in the game exists in one of seven zones. The movement of objects between these zones is what constitutes the game's action.

* **Library:** Each player's deck becomes their library at the start of the game. It is kept face down, and the order of cards is unknown and may not be changed unless an effect specifically permits it. The library is a hidden zone (Rules 401).  
* **Hand:** The set of cards a player has drawn but not yet played. A player's hand is hidden from their opponent (Rules 402).  
* **Battlefield:** The shared, primary area of play where permanents reside. Objects on the battlefield are public information. This zone was formerly called the "in-play" zone (Rules 403).  
* **Graveyard:** A player's discard pile. Cards are placed here when they are discarded, destroyed, sacrificed, or countered, and instant and sorcery spells are put here after they resolve. The graveyard is a public zone, and its contents are face up (Rules 404).  
* **The Stack:** A shared zone where spells and abilities are placed after being cast or activated. They wait on the stack to resolve. The stack is a public zone and operates on a "last in, first out" basis (Rules 405).  
* **Exile:** A zone for objects that have been removed from the game, either temporarily or permanently. Cards in the exile zone are public information and are typically kept face up (Rules 406).  
* **Command:** A zone for specific game objects that are not on the battlefield but can still affect the game, such as emblems (Rules 408).

The division of these zones into **public** (Battlefield, Graveyard, Stack, Exile, Command) and **hidden** (Library, Hand) is a cornerstone of strategic gameplay. This structure dictates what information is freely available to all players and what must be inferred, tracked, or revealed through game actions. A card's journey defines the game's narrative: moving from the Library (hidden) to the Hand (hidden) is a draw, a private gain of resources. Moving from the Hand (hidden) to the Stack (public) is the act of casting a spell, a public declaration of intent that opens the door for interaction. This constant transition between hidden and public states creates a balance between perfect information (the current board state) and imperfect information (an opponent's available resources), which is essential for strategic depth, enabling bluffing, prediction, and skillful play.

## **Section 2: The Anatomy of a Magic Card**

This section deconstructs a standard *Magic* card, explaining how each component contributes to its function within the game rules. The official wording of a card is its Oracle text, which can be found in the Gatherer card database; this text is used to resolve any discrepancies with the printed card (Rules 108.1).

### **2.1. Name**

A card's name, printed in its upper left corner, is its primary identifier (Rules 201.1). The game's rules, such as the "four-of" limit in deck construction and the "legend rule," are based on a card's unique English name (Rules 100.2a, 704.5j). When a card's rules text refers to itself by name (e.g., "When *Llanowar Elves* enters the battlefield..."), it is referring only to that specific object on the battlefield or stack, not to any other object that happens to share the same name (Rules 201.5). This self-referential shorthand prevents ambiguity and makes card text more concise.

### **2.2. Mana Cost and Mana Value**

* **Mana Cost:** The sequence of mana symbols in the top right corner of a card indicates the cost to cast it. This cost also determines the card's color(s) (Rules 202.1). Objects with no mana symbols in their cost, such as lands, are colorless by default and have an unpayable mana cost (Rules 202.1b).  
* **Mana Value (MV):** The total amount of mana in a card's mana cost, regardless of color, represented as a single number. For example, a card with a mana cost of {$1$}{$U$}{$B$} has a mana value of 3\. Any {$X$} in a mana cost is treated as 0 in all zones except the stack. While on the stack, {$X$} is the value chosen for it by the player casting the spell. The mana value of a card with no mana cost is 0 (Rules 202.3). This value, formerly known as "converted mana cost," is frequently referenced by card effects.

### **2.3. The Type Line: Supertypes, Card Types, and Subtypes**

The type line, located directly below the illustration, defines what an object is and how it functions within the game (Rules 205.1). It is read from left to right: Supertype — Card Type — Subtype.

* **Supertypes:** These are special designators that appear before the card type. The most common are:  
  * **Basic:** Found on basic lands (Plains, Island, Swamp, Mountain, Forest, and their snow-covered variants). A deck may contain any number of basic land cards (Rules 205.4c).  
  * **Legendary:** A permanent with this supertype is subject to the "legend rule." If a player controls two or more legendary permanents with the same name, that player chooses one to keep and puts the rest into their owner's graveyard as a state-based action (Rules 205.4d, 704.5j).  
* **Card Types:** These are the fundamental categories of cards. The seven primary types are **Artifact, Creature, Enchantment, Instant, Land, Planeswalker,** and **Sorcery** (Rules 205.2a).  
  * Cards of the types Artifact, Creature, Enchantment, Land, and Planeswalker are **permanent** types. When on the battlefield, they are referred to as permanents (Rules 110.1).  
  * Instants and Sorceries are **non-permanent** types. They create an effect and then go to the graveyard; they can never exist on the battlefield (Rules 304.4, 307.4).  
* **Subtypes:** These appear after a long dash and provide further classification. Subtypes are always associated with a card type. For example, "Goblin" and "Wizard" are creature types; "Equipment" is an artifact type; and "Forest" is a land type (Rules 205.3). An object can have multiple subtypes.

### **2.4. The Text Box: Abilities and Flavor**

The text box contains the rules text that defines an object's abilities (Rules 207.1).

* **Rules Text:** Each paragraph break in a card's rules text typically denotes a separate ability (Rules 113.2c). There are three main categories of abilities:  
  * **Static Abilities:** These are written as statements of fact and are continuously active as long as the object is in the appropriate zone (usually the battlefield). They don't use the stack. Example: "Creatures you control have flying" (Rules 113.3d).  
  * **Triggered Abilities:** These begin with the words "When," "Whenever," or "At." They trigger automatically when a specific game event occurs. After triggering, they are put onto the stack the next time a player would receive priority. Example: "Whenever another creature enters the battlefield, you gain 1 life" (Rules 113.3c).  
  * **Activated Abilities:** These are always written in the format "\[Cost\]: \[Effect\]." A player must have priority to choose to pay the cost and activate the ability. Once activated, the ability is put on the stack. Example: "{$1$}, {$T$}: Draw a card" (Rules 113.3b).  
* **Flavor Text:** Italicized text below the rules text that provides story context or a quote. It has no effect on gameplay (Rules 207.2b).

### **2.5. Power and Toughness (P/T)**

Found only on creature cards in the bottom right corner, formatted as two numbers separated by a slash (e.g., 2/3) (Rules 208.1).

* **Power (the first number):** The amount of combat damage the creature deals.  
* **Toughness (the second number):** The amount of damage that must be dealt to the creature in a single turn to destroy it.

If a creature's power or toughness is represented by a star (\*), its value is determined by a **characteristic-defining ability** printed in its text box. This ability functions in all zones. For example, a creature with \*/\* and the ability "This creature's power and toughness are each equal to the number of creatures you control" will have its P/T constantly updated as the number of creatures changes (Rules 208.2a).

## **Section 3: A Guide to Card Types**

This section details the rules governing the seven primary card types found in a standard 1v1 game, outlining how they are played and how they behave.

### **3.1. Lands**

Lands are the foundation of a player's resources.

* **Playing a Land:** Lands are not cast; they are **played**. Playing a land is a special action that does not use the stack, meaning it cannot be responded to or countered. A player simply puts the land from their hand onto the battlefield (Rules 305.1).  
* **Timing Restrictions:** A player may normally play only one land during their turn, and only during one of their main phases while the stack is empty and they have priority (Rules 305.2). Effects from other cards can modify this, allowing a player to play additional lands.  
* **Basic Land Types:** Lands with a basic land type (Plains, Island, Swamp, Mountain, or Forest) have an intrinsic mana ability. For example, a land with the subtype "Mountain" has the inherent ability "{$T$}: Add {$R$}" even if it is not printed on the card (Rules 305.6).

### **3.2. Creatures**

Creatures are the primary means of attacking and blocking.

* **Casting Creatures:** A creature spell can be cast by the active player during their main phase when the stack is empty (Rules 302.1). When the spell resolves, the creature card is put onto the battlefield.  
* **"Summoning Sickness":** A creature is affected by "summoning sickness" if its controller has not continuously controlled it since the beginning of their most recent turn. A creature with summoning sickness cannot attack, nor can it activate its abilities that include the tap symbol ({$T$}) or the untap symbol ({$Q$}) in their cost (Rules 302.6).

### **3.3. Artifacts, Enchantments, and Planeswalkers**

These are other types of permanents that provide a wide range of effects.

* **Casting:** Like creatures, these are permanent spells that are typically cast during a player's main phase when the stack is empty (Rules 301.1, 303.1, 306.1).  
* **Auras:** Auras are a special subtype of enchantment that must be attached to an object or player. An Aura spell is the only permanent spell type that requires a target when it is cast. If the object an Aura is attached to leaves the battlefield, the Aura is put into its owner's graveyard (Rules 303.4).  
* **Planeswalkers:** These powerful permanents enter the battlefield with a number of **loyalty counters** equal to the number printed in their lower right corner (Rules 306.5b).  
  * **Loyalty Abilities:** Their activated abilities are paid for by adding or removing loyalty counters. A player may only activate one loyalty ability of a given planeswalker per turn, and only during their main phase when the stack is empty (informally, "at sorcery speed") (Rules 606.3).  
  * **Vulnerability:** Planeswalkers can be attacked by creatures (Rules 306.6). Damage dealt to a planeswalker results in that many loyalty counters being removed from it (Rules 306.8). If a planeswalker's loyalty becomes 0, it is put into its owner's graveyard (Rules 306.9).

### **3.4. Instants and Sorceries**

These are spells that produce a one-time effect and are then put into their owner's graveyard upon resolution (Rules 304.2, 307.2). They never enter the battlefield.

* **Sorceries:** These can only be cast by the active player during one of their main phases when the stack is empty (Rules 307.1). This timing restriction makes them primarily proactive spells used to develop a player's board state or deal with existing threats on their own turn.  
* **Instants:** These are the most flexible spells. They can be cast any time a player has priority, including during an opponent's turn or in response to another spell or ability (Rules 304.1). This flexibility makes them the core of reactive and interactive gameplay.

The different card types create a spectrum of timing restrictions that dictates the pace and flow of the game. This spectrum runs from the most restrictive (Lands) to the least restrictive (Instants). Lands have the tightest restriction—one per turn, during a main phase, with an empty stack—ensuring a gradual and predictable buildup of resources. Sorceries, creatures, and other permanents share the "main phase, empty stack" restriction, defining the proactive, "building" portion of a turn. Activated abilities can generally be used anytime a player has priority, opening up more reactive gameplay possibilities. Finally, instants are the least restrictive, usable anytime a player has priority, forming the foundation of reactive and interactive gameplay. This deliberate hierarchy of timing is fundamental to *Magic's* strategic depth, creating distinct windows of opportunity and forcing players to manage their actions across different phases and turns.

## **Section 4: The Structure of a Turn**

A turn is a rigid sequence of five phases and their respective steps. Understanding this structure is critical for knowing when actions can be taken and when abilities will trigger.

### **4.1. Overview of the Five Phases**

The turn proceeds in a fixed order: **1\. Beginning Phase, 2\. Precombat Main Phase, 3\. Combat Phase, 4\. Postcombat Main Phase, 5\. Ending Phase** (Rules 500.1). At the end of every phase and step, any unused mana in a player's mana pool empties and is lost (Rules 500.4).

| Phase | Step | Key Turn-Based Actions | When Players Receive Priority |
| :---- | :---- | :---- | :---- |
| **Beginning Phase** | **Untap Step** | Phasing occurs, then the active player untaps their tapped permanents. | No player receives priority. |
|  | **Upkeep Step** | None. | Yes, after "beginning of upkeep" triggers are put on the stack. |
|  | **Draw Step** | The active player draws a card. | Yes, after the card is drawn. |
| **Precombat Main Phase** | (No steps) | Saga counters are added. | Yes. The active player can play a land and cast any spell type. |
| **Combat Phase** | **Beginning of Combat** | None. | Yes. Last chance to act before attackers are declared. |
|  | **Declare Attackers** | The active player declares attackers. | Yes, after attackers are declared. |
|  | **Declare Blockers** | The defending player declares blockers. | Yes, after blockers are declared. |
|  | **Combat Damage** | Creatures deal combat damage. | Yes, after damage is dealt. |
|  | **End of Combat** | None. | Yes. Last chance to act during combat. |
| **Postcombat Main Phase** | (No steps) | None. | Yes. The active player can play a land (if they haven't) and cast any spell type. |
| **Ending Phase** | **End Step** | None. | Yes, after "beginning of the end step" triggers are put on the stack. |
|  | **Cleanup Step** | Active player discards to hand size; damage is removed; "until end of turn" effects end. | Only if an ability triggers or a state-based action occurs. |

### **4.2. The Beginning Phase**

This phase sets up the player for their turn.

* **Untap Step:** The active player untaps all of their tapped permanents simultaneously. This is a turn-based action that does not use the stack. No player receives priority during this step, meaning no spells can be cast or abilities activated (Rules 502.3, 502.4).  
* **Upkeep Step:** This is the first time in a turn that a player receives priority. Any abilities that trigger "at the beginning of your upkeep" are put onto the stack before the active player gets priority (Rules 503.1). This step is a common window for casting instants or activating abilities before the player draws their card for the turn.  
* **Draw Step:** The active player draws a card from their library. This is a turn-based action that does not use the stack. After the card is drawn, players receive priority and can cast spells and activate abilities (Rules 504). The player who takes the first turn of the game skips the draw step of their first turn (Rules 103.8a).

### **4.3. The Main Phases (Precombat and Postcombat)**

A player takes most of their proactive actions during their main phases.

* During their main phase, if the stack is empty and they have priority, the active player may play a land (if they have not already done so this turn) and may cast any type of spell (creatures, sorceries, artifacts, etc.) (Rules 505.6).  
* Having two main phases, one before combat and one after, provides significant strategic flexibility. A player can cast a creature before combat to attack with it (if it has haste), or they can wait until after combat to see how the board state has changed before committing more resources to the battlefield.

### **4.4. The Combat Phase: A Step-by-Step Breakdown**

The combat phase is where creatures can attack and block. It is the most complex phase, broken down into five distinct steps.

* **Beginning of Combat Step:** This step is the final opportunity for players to cast instants or activate abilities before attackers are declared. For example, a player might tap a potential attacker to activate an ability, making it unable to attack (Rules 507).  
* **Declare Attackers Step:** The active player declares which of their eligible creatures are attacking and which player or planeswalker each is attacking. This is a turn-based action that does not use the stack. Immediately after attackers are declared, any abilities that trigger "whenever a creature attacks" are put onto the stack, and then players receive priority (Rules 508).  
* **Declare Blockers Step:** The defending player declares which of their untapped creatures will block which attacking creatures. This is also a turn-based action that does not use the stack. A single creature can be blocked by multiple creatures, but a single creature cannot block multiple attackers (unless an effect allows it). After blockers are declared, players receive priority (Rules 509).  
* **Combat Damage Step:** All combat damage is dealt simultaneously. An attacking creature deals damage equal to its power to the creature(s) blocking it or to the player/planeswalker it is attacking if unblocked. A blocking creature deals damage equal to its power to the creature it is blocking. This is a turn-based action that does not use the stack. After damage is dealt, players receive priority (Rules 510).  
* **End of Combat Step:** This is the final step of the combat phase and the last chance to cast spells or activate abilities that specifically refer to combat (e.g., "at end of combat"). At the conclusion of this step, all creatures and planeswalkers are removed from combat (Rules 511).

### **4.5. The Ending Phase**

This phase concludes the turn.

* **End Step:** Abilities that trigger "at the beginning of the end step" are put onto the stack. This step provides the last main window for players to cast instants or activate abilities before the turn ends (Rules 513).  
* **Cleanup Step:** This step involves two primary turn-based actions that do not use the stack. First, the active player discards cards from their hand until they have no more than their maximum hand size (normally seven). Second, all damage marked on permanents is removed, and all effects with a duration of "until end of turn" or "this turn" expire (Rules 514.1, 514.2). Normally, no player receives priority during the cleanup step. However, if discarding a card or the ending of an effect triggers an ability, or if a state-based action must be performed (e.g., a creature dies from an effect that ends), the active player receives priority. Once the stack is empty again, a new cleanup step begins (Rules 514.3).

## **Section 5: Spells, Abilities, and the Stack**

This section details the dynamic process of taking actions in *Magic*, which is governed by the interlocking systems of priority and the stack.

### **5.1. Timing and Priority**

* **Priority** is the right for a player to take an action, such as casting a spell or activating an ability. To perform most actions in the game, a player must have priority (Rules 117.1).  
* The active player receives priority at the beginning of most steps and phases, after any turn-based actions (like drawing a card) have been completed and any triggered abilities have been put onto the stack (Rules 117.3a).  
* Whenever a spell or ability resolves, the active player receives priority again (Rules 117.3b).  
* If a player has priority and chooses not to take any action, they **pass** priority to the next player in turn order (in a two-player game, this is their opponent) (Rules 117.3d).

### **5.2. The Stack: The Heart of Interaction**

The stack is the mechanism that makes *Magic* an interactive game.

* The stack is a shared game zone where spells and abilities are placed when they are cast or when they trigger. They remain on the stack until they are countered, resolve, or are otherwise removed (Rules 405.1).  
* The stack follows a **Last-In, First-Out (LIFO)** principle. This means that the last object put onto the stack is always the first one to resolve (Rules 405.5).  
* When all players pass priority in succession, the top object of the stack resolves. If the stack is empty when all players pass, the current step or phase ends (Rules 117.4).

The LIFO nature of the stack enables all interactive gameplay, creating a "conversational" flow where each action can be met with a "response" before it takes effect. For example, Player A casts a spell, which goes on the stack. Before that spell resolves, Player A gets priority, and if they pass, Player B receives priority. Player B can now "respond" by casting their own spell (e.g., an instant like Counterspell). Player B's spell goes on the stack *on top of* Player A's spell. Now, if both players pass priority, Player B's spell resolves first because it was the last one added. This fundamental structure is the engine of counterplay, allowing players to react to threats, protect their own spells, and create complex chains of effects.

### **5.3. The Process of Casting a Spell**

Casting a spell is a detailed, multi-step process that must be followed in a specific order (Rules 601.2).

1. **Propose the Spell:** The player moves the card from the zone it is in (usually the hand) to the stack. It becomes the topmost object on the stack (Rules 601.2a).  
2. **Make Choices:** The player announces all choices for the spell. This includes choosing modes for a modal spell, declaring intent to pay additional costs like Kicker, or announcing the value for {$X$} in a mana cost (Rules 601.2b).  
3. **Choose Targets:** The player announces the spell's target(s). An object or player becomes a target only if the spell or ability uses the word "target" to identify it (Rules 115.1, 601.2c).  
4. **Divide Effects:** If the spell instructs the player to divide an effect (like damage or counters) among targets, the player announces how the division will be made (Rules 601.2d).  
5. **Determine Total Cost:** The game calculates the spell's total cost. This is its mana cost or an alternative cost, plus any additional costs or cost increases, minus any cost reductions (Rules 601.2f).  
6. **Activate Mana Abilities:** If mana is needed to pay the cost, the player may now activate mana abilities (Rules 601.2g).  
7. **Pay Costs:** The player pays the total cost determined in step 5 (Rules 601.2h).  
8. **Spell Becomes Cast:** Once all costs are paid, the spell is officially "cast." At this moment, any abilities that trigger "whenever you cast a spell" will trigger (Rules 601.2i).

### **5.4. Abilities on the Stack**

* **Activated Abilities:** Activating an ability follows a process very similar to casting a spell: announce the activation, make choices, choose targets, determine and pay the cost. Once completed, the ability is on the stack (Rules 602).  
* **Triggered Abilities:** When a trigger event occurs, the ability "triggers." The next time a player would receive priority, the ability is put onto the stack by its controller. Targets for the ability are chosen at this time, not when the event occurred (Rules 603.3).

### **5.5. Resolving Spells and Abilities**

When an object on the stack resolves, its controller follows its instructions.

* **Checking Targets:** The very first step of resolution is for the game to check whether the spell or ability's targets are still legal. A target might become illegal if it has left the battlefield, gained hexproof, or no longer meets the targeting criteria (e.g., a "target creature" is no longer a creature). If *all* of a spell or ability's targets have become illegal, the spell or ability is removed from the stack and does nothing. This is informally known as "fizzling." If at least one target is still legal, the spell or ability resolves and affects only the legal targets (Rules 608.2b).

## **Section 6: Core Game Mechanics and Interactions**

This section covers the automated game processes that maintain the integrity of the game state and the complex ways that different effects can interact with one another.

### **6.1. State-Based Actions (SBAs)**

State-Based Actions are a set of game conditions that are checked automatically whenever a player is about to receive priority. They are the game's self-correction mechanism and do not use the stack (Rules 704.3).

If the game state matches any of the conditions for an SBA, the game performs the required action (e.g., putting a creature in the graveyard). This happens simultaneously for all applicable SBAs. After they are performed, the game checks again. This process repeats until no SBAs need to be performed. Only then does a player receive priority (Rules 117.5).

Key State-Based Actions for standard 1v1 play include:

* A player with 0 or less life loses the game (Rules 704.5a).  
* A player who has attempted to draw a card from an empty library since the last check loses the game (Rules 704.5b).  
* A player with ten or more poison counters loses the game (Rules 704.5c).  
* A creature with toughness 0 or less is put into its owner's graveyard (Rules 704.5f).  
* A creature with damage marked on it greater than or equal to its toughness is put into its owner's graveyard (this is known as lethal damage) (Rules 704.5g).  
* If a player controls two or more legendary permanents with the same name, that player chooses one and puts the rest into their graveyards (the "legend rule") (Rules 704.5j).  
* A token that is in any zone other than the battlefield ceases to exist (Rules 704.5d).  
* An Aura that is not attached to a legal object or player is put into its owner's graveyard (Rules 704.5m).

### **6.2. Continuous Effects and the Layer System**

* **Continuous Effects** are generated by static abilities (e.g., "Creatures you control get \+1/+1") or by resolving spells and abilities (e.g., "Target creature gets \+3/+3 until end of turn"). They modify the game state for a specified or indefinite duration (Rules 611).  
* When multiple continuous effects are active, they can interact in complex ways. To ensure a consistent outcome, the game applies these effects in a specific sequence of **layers**. This system is a deterministic algorithm for building an object's final characteristics (Rules 613.1).

The order of layers is not arbitrary; it is structured logically to build a permanent's final state. First, the game must know *what* the object is before it can be modified, so **Copy effects (Layer 1\)** are applied first. Next, **Control-changing effects (Layer 2\)** are applied, as control dictates who makes decisions for the permanent. What the card *says* (**Text-changing effects, Layer 3**) must be established before what it *is* (**Type-changing effects, Layer 4**). What it *is* must be known before its **Color (Layer 5\)** can be set or what it *does* (**Ability-adding/removing effects, Layer 6**). Finally, after all of that is determined, its **Power and Toughness (Layer 7\)** can be calculated.

A simplified overview of the layers is as follows:

* **Layer 1:** Copy effects.  
* **Layer 2:** Control-changing effects.  
* **Layer 3:** Text-changing effects.  
* **Layer 4:** Type-changing effects (e.g., turning a land into a creature).  
* **Layer 5:** Color-changing effects.  
* **Layer 6:** Ability-adding and ability-removing effects.  
* **Layer 7:** Power- and/or toughness-modifying effects. This crucial layer is further subdivided:  
  * **7a:** Characteristic-defining abilities that set P/T (e.g., a creature with \*/\*).  
  * **7b:** Effects that set P/T to a specific value (e.g., an effect that makes a creature a "1/1").  
  * **7c:** Effects that modify P/T with counters or bonuses (e.g., \+1/+1 counters, or effects like "gets \+2/+2").  
  * **7d:** Effects that switch a creature's power and toughness.

Within each layer or sublayer, effects are generally applied in **timestamp order**—the most recently created effect is applied last (Rules 613.7).

### **6.3. Replacement and Prevention Effects**

These are continuous effects that watch for a specific event and modify it as it happens.

* **Replacement Effects:** If an event would happen, a replacement effect causes a different event to happen instead. The original event never occurs. These effects often use the word "instead" or "skip" (Rules 614.1, 614.6). For example, an effect that reads, "If you would draw a card, instead scry 1, then draw a card," modifies the action of drawing.  
* **Prevention Effects:** This is a subset of replacement effects that specifically apply to damage. They often use the word "prevent" (Rules 615.1). For example, "Prevent the next 3 damage that would be dealt to target creature."  
* **Interaction:** If multiple replacement or prevention effects could apply to the same event, the controller of the affected object or the affected player chooses one of them to apply. After that modified event is created, the game checks again to see if any other replacement effects could apply to the new event, and the process repeats until no more replacement effects are applicable (Rules 616.1).

## **Section 7: Glossary of Essential Keywords**

This section provides definitions for fundamental game actions and the most common "evergreen" keyword abilities that appear in nearly every *Magic* set.

### **7.1. Keyword Actions**

These are common verbs with specific rules meanings.

* **Attach:** To move an Aura, Equipment, or Fortification onto an object or player so it is attached to it (Rules 701.3).  
* **Counter:** To cancel a spell or ability on the stack, removing it and preventing its effects. A countered spell is put into its owner's graveyard (Rules 701.6).  
* **Create:** To put a token onto the battlefield (Rules 701.7).  
* **Destroy:** To move a permanent from the battlefield to its owner's graveyard (Rules 701.8).  
* **Discard:** To move a card from its owner's hand to that player's graveyard (Rules 701.9).  
* **Exile:** To move an object to the exile zone (Rules 701.13).  
* **Fight:** Two creatures each deal damage equal to their power to the other (Rules 701.14).  
* **Sacrifice:** To move a permanent you control from the battlefield to its owner's graveyard (Rules 701.21).  
* **Search:** To look through a hidden zone (usually a library) for a card with specific characteristics and find it (Rules 701.23).  
* **Tap and Untap:** To tap a permanent is to turn it sideways to show it has been used for an ability or to attack. To untap it is to return it to its upright position (Rules 701.26).

### **7.2. Evergreen Keyword Abilities**

These are common abilities whose rules are defined here rather than on every card.

* **Deathtouch:** Any nonzero amount of damage this source deals to a creature is considered to be lethal damage (Rules 702.2).  
* **First Strike:** This creature deals combat damage before creatures without first strike (Rules 702.7).  
* **Double Strike:** This creature deals combat damage during both the first strike combat damage step and the normal combat damage step (Rules 702.4).  
* **Flying:** This creature can't be blocked except by creatures with flying or reach (Rules 702.9).  
* **Haste:** This creature can attack and activate its {$T$} abilities even if it hasn't been under its controller's control since the beginning of their most recent turn (Rules 702.10).  
* **Hexproof:** This permanent or player can't be the target of spells or abilities your opponents control (Rules 702.11).  
* **Indestructible:** Permanents with indestructible can't be destroyed by effects that say "destroy" or by lethal damage (Rules 702.12).  
* **Lifelink:** Damage dealt by a source with lifelink also causes its controller to gain that much life (Rules 702.15).  
* **Trample:** If this creature would assign enough combat damage to its blockers to destroy them, you may have it assign the rest of its damage to the player, planeswalker, or battle it's attacking (Rules 702.19).  
* **Vigilance:** Attacking doesn't cause this creature to tap (Rules 702.20).

## **Conclusion: Summary of Omissions**

This guide is intentionally focused on the core rules of two-player *Magic*. As such, the following major sections and concepts from the full *Comprehensive Rules* have been omitted to maintain clarity and conciseness:

* **Multiplayer Rules (Section 800):** All rules governing games with more than two players, including variants like Free-for-All, Emperor, and Two-Headed Giant.  
* **Casual Variants (Section 900):** All rules for formats like Commander (Brawl), Planechase, Archenemy, and Vanguard.  
* **Non-Standard Card Types:** Rules for card types not found in standard sets, such as Conspiracy, Phenomenon, Plane, Scheme, and Vanguard.  
* **Obscure Mechanics:** Rules for dozens of set-specific keyword abilities and actions not considered "evergreen" (e.g., Banding, Phasing, Suspend, Morph, etc.), as well as rules for niche interactions like subgames (Rules 728\) or controlling another player (Rules 722).  
* **Tournament Procedures:** While based on the competitive rule set, this guide does not cover tournament-specific procedures, which are found in the *Magic: The Gathering Tournament Rules*.

This document is intended as a bridge between a basic understanding of *Magic* and the full depth of its comprehensive rule set. For rulings on specific card interactions or formats not covered here, the complete *Comprehensive Rules* remains the ultimate authority.