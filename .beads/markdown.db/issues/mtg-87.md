---
title: Fix player name consistency throughout TUI
status: open
priority: 2
issue_type: bug
created_at: "2025-10-26T23:49:49Z"
updated_at: "2025-10-27T05:58:46-04:00"
---

# Description

Player names are inconsistent during gameplay. Seeing:
- "Turn 22 - Player 2's turn"  
- "Player 1 discards Royal Assassin"
- "=== Your Turn (Player 0) ==="

All in the same game. This is confusing.

**Requirements:**
1. Use one consistent name per player throughout entire game
2. Add --p1-name and --p2-name CLI options  
3. If names are "Alice" and "Bob", never show any other names
4. Fix all display code to use player.name field consistently
5. Search for "Player 0", "Player 1", "Player {:?}" patterns

**Files likely involved:**
- src/main.rs (CLI options, initialization)
- src/game/game_loop.rs (turn announcements, logging)
- src/game/interactive_controller.rs (TUI display)
- src/game/logger.rs (game event logging)
