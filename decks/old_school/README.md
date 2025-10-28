# Old School 93/94 Format Decks

These decks are representative of archetypes from the Old School Magic: The Gathering format (93/94 era), manually downloaded from mtgdecks.

## Status

**Deck Loading**: ✅ All decks load successfully
- Card name normalization properly handles apostrophes and special characters
- All 6 deck files parse correctly and load their cards from cardsfolder

**Gameplay**: ⚠️ Currently blocked by unimplemented complex mana sources
- These decks contain dual lands (e.g., Tundra, Underground Sea) and multicolor lands (e.g., City of Brass)
- The mana engine needs enhancement to handle lands that can produce multiple colors
- Simpler decks in `/decks/` work fine with both random and heuristic controllers

## Deck List

1. `01_rogue_rogerbrand.dck` - Rogue multicolor deck by Roger Brand
2. `02_thedeck_peterschnidrig.dck` - "The Deck" control by Peter Schnidrig
3. `03_robots_jesseisbak.dck` - Artifact-based "Robots" by Jesse Isbak
4. `05_mono_black_rogerbrand.dck` - Mono-black control by Roger Brand
5. `06_jeskai_aggro_joseantonioprieto.dck` - Jeskai aggro by Jose Antonio Prieto
6. `06_troll_disk_daniellebrunazzo.dck` - Troll Disk combo by Danielle Brunazzo

## Usage Example

```bash
# This will load successfully but fail at runtime when complex mana is encountered:
cargo run --bin mtg --release -- tui \
  decks/old_school/01_rogue_rogerbrand.dck \
  decks/old_school/02_thedeck_peterschnidrig.dck \
  --p1=random --p2=random --seed=42
```

## See Also

- `/docs/DCK_FORMAT.md` - Complete .dck file format specification
- `/decks/` - Simpler decks that work with current mana engine
