use mtg_forge_rs::game::GameState;

fn main() {
    println!("MTG Forge Rust - Starting...");

    // Create a simple two-player game
    let game = GameState::new_two_player("Player 1".to_string(), "Player 2".to_string(), 20);

    println!("Game created with {} players", game.players.len());
    println!(
        "Turn {}, Step: {:?}",
        game.turn.turn_number, game.turn.current_step
    );
    println!("Active player: {}", game.turn.active_player);

    println!("\nFoundation is working! Next steps:");
    println!("- Add card/deck loading");
    println!("- Implement basic gameplay actions");
    println!("- Build TUI interface");
    println!("- Create Lightning Bolt MVP");
}
