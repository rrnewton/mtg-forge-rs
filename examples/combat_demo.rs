//! Combat Demonstration Example
//!
//! Demonstrates the complete combat system including:
//! - Declaring attackers
//! - Declaring blockers
//! - Combat damage assignment
//! - Creature death from combat damage
//!
//! Uses classic cards from Limited/Alpha/Beta/4th Edition

use mtg_forge_rs::core::{Card, CardType, EntityId, Player};
use mtg_forge_rs::game::GameState;

fn main() {
    println!("=== MTG Forge - Combat System Demo ===\n");
    println!("Demonstrating:");
    println!("  - Creature combat with attacking and blocking");
    println!("  - Combat damage assignment");
    println!("  - Creature death from lethal damage");
    println!("  - Using classic 4ED cards\n");

    // Create a two-player game
    let mut game = GameState::new_two_player("Alice".to_string(), "Bob".to_string(), 20);

    let players: Vec<_> = game.players.iter().map(|(id, _)| *id).collect();
    let alice = players[0];
    let bob = players[1];

    println!("=== Initial Setup ===");
    println!("Alice: 20 life");
    println!("Bob: 20 life\n");

    // Create creatures for Alice (the attacker)
    println!("=== Creating Creatures ===");

    // Alice gets a Grizzly Bears (2/2) and a Gray Ogre (2/2)
    let bears_id = game.next_card_id();
    let mut bears = Card::new(bears_id, "Grizzly Bears".to_string(), alice);
    bears.types.push(CardType::Creature);
    bears.power = Some(2);
    bears.toughness = Some(2);
    bears.controller = alice;
    game.cards.insert(bears_id, bears);
    game.battlefield.add(bears_id);
    println!("  Alice: Grizzly Bears (2/2) [4ED]");

    let ogre_id = game.next_card_id();
    let mut ogre = Card::new(ogre_id, "Gray Ogre".to_string(), alice);
    ogre.types.push(CardType::Creature);
    ogre.power = Some(2);
    ogre.toughness = Some(2);
    ogre.controller = alice;
    game.cards.insert(ogre_id, ogre);
    game.battlefield.add(ogre_id);
    println!("  Alice: Gray Ogre (2/2) [4ED]");

    // Bob gets a Wall of Stone (0/8) and a Hill Giant (3/3)
    let wall_id = game.next_card_id();
    let mut wall = Card::new(wall_id, "Wall of Stone".to_string(), bob);
    wall.types.push(CardType::Creature);
    wall.power = Some(0);
    wall.toughness = Some(8);
    wall.controller = bob;
    game.cards.insert(wall_id, wall);
    game.battlefield.add(wall_id);
    println!("  Bob: Wall of Stone (0/8) [4ED]");

    let giant_id = game.next_card_id();
    let mut giant = Card::new(giant_id, "Hill Giant".to_string(), bob);
    giant.types.push(CardType::Creature);
    giant.power = Some(3);
    giant.toughness = Some(3);
    giant.controller = bob;
    game.cards.insert(giant_id, giant);
    game.battlefield.add(giant_id);
    println!("  Bob: Hill Giant (3/3) [4ED]\n");

    print_battlefield(&game, alice, bob);

    // === Combat Phase ===
    println!("\n=== Beginning Combat Phase ===");
    println!("Alice is the active player and will attack\n");

    // Alice declares both creatures as attackers
    println!("--- Declare Attackers Step ---");
    println!("Alice declares attackers:");

    // Declare Grizzly Bears as attacker
    match game.declare_attacker(alice, bears_id) {
        Ok(_) => {
            println!("  ✓ Grizzly Bears attacks (and taps)");
        }
        Err(e) => {
            println!("  ✗ Failed to declare Grizzly Bears as attacker: {e:?}");
        }
    }

    // Declare Gray Ogre as attacker
    match game.declare_attacker(alice, ogre_id) {
        Ok(_) => {
            println!("  ✓ Gray Ogre attacks (and taps)");
        }
        Err(e) => {
            println!("  ✗ Failed to declare Gray Ogre as attacker: {e:?}");
        }
    }

    println!("\nAttacking creatures: 2");
    println!("  - Grizzly Bears (2/2)");
    println!("  - Gray Ogre (2/2)\n");

    // Bob declares blockers
    println!("--- Declare Blockers Step ---");
    println!("Bob declares blockers:");

    // Wall of Stone blocks Grizzly Bears
    match game.declare_blocker(bob, wall_id, vec![bears_id]) {
        Ok(_) => {
            println!("  ✓ Wall of Stone blocks Grizzly Bears");
        }
        Err(e) => {
            println!("  ✗ Failed to block: {e:?}");
        }
    }

    // Hill Giant blocks Gray Ogre
    match game.declare_blocker(bob, giant_id, vec![ogre_id]) {
        Ok(_) => {
            println!("  ✓ Hill Giant blocks Gray Ogre");
        }
        Err(e) => {
            println!("  ✗ Failed to block: {e:?}");
        }
    }

    println!("\nBlocking assignments:");
    println!("  - Grizzly Bears (2/2) is blocked by Wall of Stone (0/8)");
    println!("  - Gray Ogre (2/2) is blocked by Hill Giant (3/3)\n");

    // Combat Damage Step
    println!("--- Combat Damage Step ---");
    println!("Assigning combat damage...\n");

    match game.assign_combat_damage() {
        Ok(_) => {
            println!("Combat damage assigned successfully!");
        }
        Err(e) => {
            println!("Error assigning combat damage: {e:?}");
            return;
        }
    }

    println!("\nCombat damage dealt:");
    println!("  Combat 1: Grizzly Bears (2/2) vs Wall of Stone (0/8)");
    println!("    - Bears deals 2 damage to Wall");
    println!("    - Wall deals 0 damage to Bears");
    println!("    - Wall survives (2 damage < 8 toughness)");
    println!("    - Bears survives (0 damage)");
    println!();
    println!("  Combat 2: Gray Ogre (2/2) vs Hill Giant (3/3)");
    println!("    - Ogre deals 2 damage to Giant");
    println!("    - Giant deals 3 damage to Ogre");
    println!("    - Giant survives (2 damage < 3 toughness)");
    println!("    - Ogre dies (3 damage >= 2 toughness) 💀");

    println!("\n=== After Combat ===");
    print_battlefield(&game, alice, bob);

    // Check life totals
    let alice_life = game.players.get(alice).unwrap().life;
    let bob_life = game.players.get(bob).unwrap().life;

    println!("\n=== Final State ===");
    println!("Alice: {alice_life} life (no damage taken - all attackers blocked)");
    println!("Bob: {bob_life} life (no damage taken - all attackers blocked)");

    // Check graveyards
    let alice_graveyard = game
        .get_player_zones(alice)
        .map(|z| z.graveyard.cards.len())
        .unwrap_or(0);
    let bob_graveyard = game
        .get_player_zones(bob)
        .map(|z| z.graveyard.cards.len())
        .unwrap_or(0);

    println!("\nGraveyards:");
    println!("  Alice: {alice_graveyard} cards");
    println!("  Bob: {bob_graveyard} cards");

    // Verify expected outcome
    println!("\n=== Verification ===");
    let mut success = true;

    // Gray Ogre should be dead
    if let Some(zones) = game.get_player_zones(alice) {
        if zones.graveyard.contains(ogre_id) {
            println!("✓ Gray Ogre correctly died and went to graveyard");
        } else {
            println!("✗ Gray Ogre should be in graveyard but isn't");
            success = false;
        }
    }

    // Grizzly Bears should still be alive
    if game.battlefield.contains(bears_id) {
        println!("✓ Grizzly Bears correctly survived");
    } else {
        println!("✗ Grizzly Bears should still be on battlefield");
        success = false;
    }

    // Wall should still be alive
    if game.battlefield.contains(wall_id) {
        println!("✓ Wall of Stone correctly survived");
    } else {
        println!("✗ Wall of Stone should still be on battlefield");
        success = false;
    }

    // Hill Giant should still be alive
    if game.battlefield.contains(giant_id) {
        println!("✓ Hill Giant correctly survived");
    } else {
        println!("✗ Hill Giant should still be on battlefield");
        success = false;
    }

    // No player damage
    if alice_life == 20 && bob_life == 20 {
        println!("✓ Life totals unchanged (all attackers were blocked)");
    } else {
        println!("✗ Life totals should be unchanged");
        success = false;
    }

    println!("\n=== Combat Demo Complete ===");
    if success {
        println!("✅ All combat mechanics working correctly!");
        println!("\nKey features demonstrated:");
        println!("  ✓ Declaring multiple attackers");
        println!("  ✓ Declaring blockers for each attacker");
        println!("  ✓ Combat damage calculation");
        println!("  ✓ Creature death from lethal damage");
        println!("  ✓ Creatures surviving non-lethal damage");
        println!("  ✓ Blocked attackers don't damage defending player");
    } else {
        println!("❌ Some combat mechanics failed verification");
        std::process::exit(1);
    }
}

/// Print the current battlefield state
fn print_battlefield(game: &GameState, alice: EntityId<Player>, bob: EntityId<Player>) {
    println!("Battlefield:");

    // Alice's creatures
    print!("  Alice: ");
    let alice_creatures: Vec<_> = game
        .battlefield
        .cards
        .iter()
        .filter_map(|&id| {
            game.cards.get(id).ok().and_then(|card| {
                if card.controller == alice && card.is_creature() {
                    Some((
                        card.name.as_str(),
                        card.power.unwrap_or(0),
                        card.toughness.unwrap_or(0),
                        card.tapped,
                    ))
                } else {
                    None
                }
            })
        })
        .collect();

    if alice_creatures.is_empty() {
        println!("(no creatures)");
    } else {
        for (i, (name, pow, tou, tapped)) in alice_creatures.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            let tap_str = if *tapped { " (tapped)" } else { "" };
            print!("{name} ({pow}/{tou}){tap_str}");
        }
        println!();
    }

    // Bob's creatures
    print!("  Bob: ");
    let bob_creatures: Vec<_> = game
        .battlefield
        .cards
        .iter()
        .filter_map(|&id| {
            game.cards.get(id).ok().and_then(|card| {
                if card.controller == bob && card.is_creature() {
                    Some((
                        card.name.as_str(),
                        card.power.unwrap_or(0),
                        card.toughness.unwrap_or(0),
                        card.tapped,
                    ))
                } else {
                    None
                }
            })
        })
        .collect();

    if bob_creatures.is_empty() {
        println!("(no creatures)");
    } else {
        for (i, (name, pow, tou, tapped)) in bob_creatures.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            let tap_str = if *tapped { " (tapped)" } else { "" };
            print!("{name} ({pow}/{tou}){tap_str}");
        }
        println!();
    }
}
