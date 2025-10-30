# Debug Directory

This directory contains debugging artifacts, reproducers, and analysis materials for the MTG Forge Rust project.

## Structure

### `reproducers/`
Contains minimal reproducible test cases for bugs and issues. Each reproducer includes:
- Shell script (`.sh`) to run the reproducer
- Puzzle file (`.pzl`) if applicable
- Documentation explaining the bug and expected behavior

**Usage:**
```bash
cd debug/reproducers
./reproducer_name.sh
```

### `artifacts/` (future)
Contains debugging artifacts such as:
- Game snapshots
- Performance profiles
- Memory dumps
- Log files from specific bug investigations

### `analysis/` (future)
Contains analysis documents and reports:
- Performance analysis
- Bug investigation notes
- Comparative analysis with Java Forge

## Adding New Reproducers

When you discover a bug:

1. Create a minimal reproducer using puzzle files or fixed controllers
2. Write a shell script in `reproducers/` that demonstrates the bug
3. Include comments explaining:
   - What the bug is
   - Expected vs actual behavior
   - MTG rule references if applicable
4. Add any supporting puzzle files to the reproducers directory

## Example

See `reproducers/royal_assassin_combat_bug.sh` for a complete example of a well-documented reproducer.
