//! Player representation

use crate::core::{GameEntity, ManaPool, PlayerId, PlayerName};
use serde::{Deserialize, Serialize};

/// Represents a player in the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    /// Unique ID for this player
    pub id: PlayerId,

    /// Player name
    pub name: PlayerName,

    /// Life total
    pub life: i32,

    /// Mana pool
    pub mana_pool: ManaPool,

    /// Has the player lost?
    pub has_lost: bool,

    /// Lands played this turn
    pub lands_played_this_turn: u8,

    /// Maximum lands per turn (usually 1)
    pub max_lands_per_turn: u8,
}

impl Player {
    pub fn new(id: PlayerId, name: impl Into<PlayerName>, starting_life: i32) -> Self {
        Player {
            id,
            name: name.into(),
            life: starting_life,
            mana_pool: ManaPool::new(),
            has_lost: false,
            lands_played_this_turn: 0,
            max_lands_per_turn: 1,
        }
    }

    pub fn gain_life(&mut self, amount: i32) {
        self.life += amount;
    }

    pub fn lose_life(&mut self, amount: i32) {
        self.life -= amount;
        if self.life <= 0 {
            self.has_lost = true;
        }
    }

    pub fn can_play_land(&self) -> bool {
        self.lands_played_this_turn < self.max_lands_per_turn
    }

    pub fn play_land(&mut self) {
        self.lands_played_this_turn += 1;
    }

    pub fn reset_lands_played(&mut self) {
        self.lands_played_this_turn = 0;
    }

    pub fn empty_mana_pool(&mut self) {
        self.mana_pool.clear();
    }
}

impl GameEntity<Player> for Player {
    fn id(&self) -> PlayerId {
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
    fn test_player_creation() {
        let id = PlayerId::new(1);
        let player = Player::new(id, "Alice", 20);

        assert_eq!(player.id, id);
        assert_eq!(player.name.as_str(), "Alice");
        assert_eq!(player.life, 20);
        assert!(!player.has_lost);
    }

    #[test]
    fn test_player_life() {
        let id = PlayerId::new(1);
        let mut player = Player::new(id, "Bob", 20);

        player.lose_life(5);
        assert_eq!(player.life, 15);
        assert!(!player.has_lost);

        player.lose_life(15);
        assert_eq!(player.life, 0);
        assert!(player.has_lost);

        player.gain_life(10);
        assert_eq!(player.life, 10);
        // has_lost stays true once triggered
        assert!(player.has_lost);
    }

    #[test]
    fn test_land_playing() {
        let id = PlayerId::new(1);
        let mut player = Player::new(id, "Charlie", 20);

        assert!(player.can_play_land());
        player.play_land();
        assert!(!player.can_play_land());

        player.reset_lands_played();
        assert!(player.can_play_land());
    }
}
