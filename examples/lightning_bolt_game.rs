//! Lightning Bolt MVP Demo
//!
//! Demonstrates a simple MTG game with just Mountains and Lightning Bolts.
//! This is the minimal viable product for the game engine.

use mtg_forge_rs::core::{Card, CardType, Color, ManaCost};
use mtg_forge_rs::game::GameState;

fn main() {
    println!("=== MTG Forge - Lightning Bolt MVP ===\n");

    // Create a two-player game
    let mut game = GameState::new_two_player(
        "Alice".to_string(),
        "Bob".to_string(),
        20
    );

    let players: Vec<_> = game.players.iter().map(|(id, _)| *id).collect();
    let alice = players[0];
    let bob = players[1];

    println!("Players:");
    println!("  Alice (P1): 20 life");
    println!("  Bob (P2): 20 life\n");

    // Setup: Give Alice some cards
    println!("Setup: Giving Alice 3 Mountains and 2 Lightning Bolts");

    // Create 3 Mountains for Alice
    for i in 0..3 {
        let card_id = game.next_entity_id();
        let mut card = Card::new(card_id, format!("Mountain {}", i+1), alice);
        card.types.push(CardType::Land);
        card.colors.push(Color::Red);
        game.cards.insert(card_id, card);

        // Add to hand
        if let Some(zones) = game.get_player_zones_mut(alice) {
            zones.hand.add(card_id);
        }
    }

    // Create 2 Lightning Bolts for Alice
    for i in 0..2 {
        let card_id = game.next_entity_id();
        let mut card = Card::new(card_id, format!("Lightning Bolt {}", i+1), alice);
        card.types.push(CardType::Instant);
        card.mana_cost = ManaCost::from_string("R");
        card.colors.push(Color::Red);
        card.text = "Lightning Bolt deals 3 damage to any target.".to_string();
        game.cards.insert(card_id, card);

        // Add to hand
        if let Some(zones) = game.get_player_zones_mut(alice) {
            zones.hand.add(card_id);
        }
    }

    // Display Alice's hand
    println!("\nAlice's hand:");
    if let Some(zones) = game.get_player_zones(alice) {
        for card_id in &zones.hand.cards {
            let card = game.cards.get(*card_id).unwrap();
            println!("  - {} ({})", card.name, if card.is_land() { "Land" } else { &card.mana_cost.to_string() });
        }
    }

    // Turn 1: Alice plays a Mountain
    println!("\n--- Turn 1: Alice ---");
    println!("Alice plays Mountain 1");

    let mountain_id = game.get_player_zones(alice).unwrap().hand.cards[0];
    game.play_land(alice, mountain_id).expect("Failed to play land");

    println!("  Battlefield: Mountain 1");

    // Turn 2: Alice plays another Mountain
    println!("\n--- Turn 2: Alice ---");
    println!("Alice draws a card (skipped in demo)");

    // Reset land counter for new turn
    game.players.get_mut(alice).unwrap().reset_lands_played();

    println!("Alice plays Mountain 2");

    let mountain2_id = game.get_player_zones(alice).unwrap().hand.cards[0];
    game.play_land(alice, mountain2_id).expect("Failed to play land");

    println!("  Battlefield: Mountain 1, Mountain 2");

    // Turn 3: Alice casts Lightning Bolt!
    println!("\n--- Turn 3: Alice ---");
    println!("Alice draws a card (skipped in demo)");

    // Reset land counter for new turn
    game.players.get_mut(alice).unwrap().reset_lands_played();

    println!("Alice plays Mountain 3");

    let mountain3_id = game.get_player_zones(alice).unwrap().hand.cards[0];
    game.play_land(alice, mountain3_id).expect("Failed to play land");

    println!("  Battlefield: Mountain 1, Mountain 2, Mountain 3");

    println!("\nAlice taps Mountain 1 for (R)");
    let mountains: Vec<_> = game.battlefield.cards.clone();
    game.tap_for_mana(alice, mountains[0]).expect("Failed to tap for mana");

    let alice_player = game.players.get(alice).unwrap();
    println!("  Mana pool: {}", alice_player.mana_pool.red);

    println!("\nAlice casts Lightning Bolt targeting Bob!");

    // Get Lightning Bolt from hand
    let bolt_id = game.get_player_zones(alice).unwrap().hand.cards
        .iter()
        .find(|&id| {
            let card = game.cards.get(*id).unwrap();
            card.name.contains("Lightning Bolt")
        })
        .copied()
        .expect("No Lightning Bolt in hand");

    // Cast the spell (simplified - no actual mana payment yet)
    // For now, we'll just move it to stack manually
    game.get_player_zones_mut(alice).unwrap().hand.remove(bolt_id);
    game.stack.add(bolt_id);

    println!("  Stack: Lightning Bolt (targeting Bob)");

    println!("\nLightning Bolt resolves:");
    game.deal_damage(bob, 3).expect("Failed to deal damage");

    // Move bolt to graveyard
    game.stack.remove(bolt_id);
    game.get_player_zones_mut(alice).unwrap().graveyard.add(bolt_id);

    let bob_player = game.players.get(bob).unwrap();
    println!("  Bob takes 3 damage!");
    println!("  Bob's life: {}", bob_player.life);

    // Check game state
    println!("\n=== Game State ===");
    for (_id, player) in game.players.iter() {
        let status = if player.has_lost { " (LOST)" } else { "" };
        println!("{}: {} life{}", player.name, player.life, status);
    }

    if game.is_game_over() {
        if let Some(winner_id) = game.get_winner() {
            let winner = game.players.get(winner_id).unwrap();
            println!("\n{} wins!", winner.name);
        }
    } else {
        println!("\nGame continues...");
    }

    println!("\n=== MVP Demo Complete ===");
    println!("This demonstrates:");
    println!("  ✓ Playing lands");
    println!("  ✓ Tapping for mana");
    println!("  ✓ Casting spells");
    println!("  ✓ Dealing damage");
    println!("  ✓ Tracking life totals");
    println!("  ✓ Game state management");
}
