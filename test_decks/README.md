# Test Decks

This directory contains minimal deck files (.dck) used for end-to-end testing of the MTG Forge Rust engine.

## Available Test Decks

### simple_bolt.dck
- **Purpose**: Basic e2e testing for TUI and game loop
- **Contents**: 20 Mountains, 40 Lightning Bolts (60 cards total)
- **Use Cases**:
  - Verifying game completion with zero controllers
  - Testing deterministic gameplay with seeds
  - Basic integration testing

## Usage

Test decks are referenced in integration tests located in `/tests/`:

```rust
let deck_path = PathBuf::from("test_decks/simple_bolt.dck");
let deck = DeckLoader::load_from_file(&deck_path)?;
```

## Adding New Test Decks

When adding new test decks:
1. Use simple, well-known cards from early sets (Limited/Alpha/Beta/4th Edition)
2. Keep decks minimal (60 cards) unless testing specific scenarios
3. Document the deck's purpose in this README
4. Add corresponding integration tests in `/tests/`

## Deck Format

Test decks follow the standard Forge .dck format:

```
[metadata]
Name=Deck Name
Description=Optional description

[Main]
<count> <card name>
...

[Sideboard]
<count> <card name>
...
```

Example:
```
[Main]
20 Mountain
40 Lightning Bolt
```
