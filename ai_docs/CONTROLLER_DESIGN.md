# Controller Architecture - Two Layers

## Overview

We have TWO distinct interfaces for player decisions, each optimized for different use cases:

### Layer 1: PlayerController (Type-Safe Specific Callbacks)
```rust
trait PlayerController {
    fn choose_land_to_play(&mut self, view: &GameStateView, lands: &[CardId]) -> Option<CardId>;
    fn choose_spell_to_cast(&mut self, view: &GameStateView, spells: &[CardId]) -> Option<(CardId, SmallVec<[CardId; 4]>)>;
    // ... other specific methods
}
```

**Use cases:**
- Human player UI (graphical or text-based)
- Heuristic AI with domain knowledge
- Agents that need strong types and game semantics

**Advantages:**
- Type safety
- Domain-specific logic
- Zero-copy (slices, SmallVec)
- Clear intent

### Layer 2: DecisionMaker (Generic String-Based Choices)
```rust
trait DecisionMaker {
    fn make_choice(&mut self, prompt: &str, options: &[&str]) -> usize;
}
```

**Use cases:**
- Tree search algorithms (minimax, MCTS)
- Recording/replaying games
- Testing determinism
- Language models / neural networks

**Advantages:**
- Generic - works for any game
- Serializable decisions
- Easy to log/debug
- No game-specific types

## GameLoop Integration

The GameLoop is responsible for:
1. Detecting what decision is needed
2. Gathering available options  
3. Calling the appropriate interface:
   - For PlayerController: Call specific method with typed options
   - For DecisionMaker: Convert options to strings, call make_choice(), map index back

## Example: Playing a Land

### Using PlayerController:
```rust
let lands_in_hand: Vec<CardId> = /* gather from game state */;
if let Some(land_id) = controller.choose_land_to_play(&view, &lands_in_hand) {
    game.play_land(player_id, land_id)?;
}
```

### Using DecisionMaker:
```rust
let lands_in_hand: Vec<CardId> = /* gather from game state */;
let land_names: Vec<&str> = lands_in_hand.iter()
    .map(|&id| game.cards.get(id).unwrap().name.as_str())
    .collect();

let choice_idx = decision_maker.make_choice("Choose land to play", &land_names);
let land_id = lands_in_hand[choice_idx];
game.play_land(player_id, land_id)?;
```

## Zero-Copy Principles

- **PlayerController**: Uses &[CardId] slices, returns SmallVec
- **DecisionMaker**: Uses &str slices (borrowing from game state where possible)
- Avoid Vec allocation in hot paths
- SmallVec for small collections (typically < 8 items)

## Implementation Status

- [x] Define traits
- [ ] Update RandomController to implement PlayerController
- [ ] Update ZeroController to implement PlayerController  
- [ ] Update GameLoop to use PlayerController methods
- [ ] Implement DecisionMaker for search algorithms (future)
- [ ] Create adapters between layers (future, if needed)
