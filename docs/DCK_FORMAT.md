# Forge .dck Deck File Format Specification

## Overview

The `.dck` file format is a plain text format used by MTG Forge for storing deck lists. It uses an INI-like structure with sections and key-value pairs.

## File Structure

A `.dck` file consists of three main sections:

```
[metadata]
Name=Deck Name
Description=Deck description

[Main]
4 Card Name
3 Card Name|SET
1 Card Name|SET|2

[Sideboard]
3 Card Name
2 Card Name|SET
```

## Sections

### [metadata] Section

Contains deck metadata as key-value pairs.

**Supported Keys:**
- `Name` - The name of the deck (required)
- `Description` - A brief description of the deck (optional)

Example:
```
[metadata]
Name=My Awesome Deck
Description=An aggressive red-green deck
```

### [Main] Section

Contains the main deck card list. Each line specifies a quantity and card name.

**Format:**
```
<quantity> <card name>[|<set code>][|<art index>]
```

Where:
- `<quantity>` - Number (1-255) indicating how many copies of the card
- `<card name>` - The full card name as it appears in MTG
- `<set code>` - (Optional) Three-letter set code (e.g., LEA, M20, IKO)
- `<art index>` - (Optional) Art variant index (1-based) for cards with multiple arts

**Examples:**
```
4 Lightning Bolt
3 Lightning Bolt|M10
1 Lightning Bolt|M10|2
20 Mountain
4 Shivan Dragon
```

### [Sideboard] Section

Contains the sideboard card list. Uses the same format as the [Main] section.

**Examples:**
```
3 Red Elemental Blast
2 Pyroblast|ICE
1 Boil
```

## Card Name Normalization

When loading cards from the cardsfolder, card names are normalized using these rules:

1. Convert to lowercase
2. Replace spaces with underscores (`_`)
3. Remove apostrophes (`'`)
4. Remove commas (`,`)
5. Replace hyphens with underscores (`-` → `_`)
6. Remove colons (`:`)
7. Remove exclamation marks (`!`)
8. Remove question marks (`?`)

**Examples:**
- `"All Hallow's Eve"` → `all_hallows_eve.txt`
- `"Nevinyrral's Disk"` → `nevinyrrals_disk.txt`
- `"Mishra's Factory"` → `mishras_factory.txt`
- `"Jace, the Mind Sculptor"` → `jace_the_mind_sculptor.txt`
- `"Who/What/When/Where/Why"` → `who_what_when_where_why.txt`

## Comments and Empty Lines

- Empty lines are ignored
- Lines starting with `#` are treated as comments (although not commonly used)
- Section headers (lines starting with `[`) are parsed to determine the current section

## File Encoding

Files should be encoded as UTF-8 plain text.

## Validation Rules

1. At least one card must be present in the [Main] section
2. The [metadata] section should appear first (though parsers may be lenient)
3. Quantity must be a positive integer (typically 1-255)
4. Card names are case-insensitive when matching to card files

## Current Implementation Status

### Supported
- ✅ Basic [metadata], [Main], and [Sideboard] sections
- ✅ Card quantity parsing
- ✅ Card name normalization with special character handling
- ✅ Set codes (parsed but currently ignored during loading)
- ✅ Comments and empty lines

### Not Yet Supported
- ❌ Art index selection (parsed but not used)
- ❌ Additional metadata fields beyond Name/Description
- ❌ Expansion-specific card variants

## Example Complete Deck File

```
[metadata]
Name=Lightning Bolt Burn
Description=Classic red burn deck using only Lightning Bolt and Mountains

[Main]
40 Lightning Bolt
20 Mountain

[Sideboard]
15 Shock
```

## Example with Set Codes and Art Indices

```
[metadata]
Name=Power 9 Showcase
Description=Deck featuring the Power 9 cards from various sets

[Main]
1 Black Lotus|LEA
1 Ancestral Recall|LEA
1 Time Walk|LEA
1 Timetwister|LEA
1 Mox Pearl|LEA
1 Mox Sapphire|LEA
1 Mox Jet|LEA
1 Mox Ruby|LEA
1 Mox Emerald|LEA
51 Island

[Sideboard]
15 Force of Will|ALL
```

## Compatibility Notes

- This format is compatible with Java Forge deck files (tested with Old School 93/94 decks)
- Set codes and art indices can be included but are currently optional in our implementation
- The parser is lenient with whitespace and empty lines
- Unknown metadata keys are silently ignored

## See Also

- `/workspace/src/loader/deck.rs` - Rust parser implementation
- `/workspace/test_decks/` - Example deck files
- `/workspace/test_decks/old_school/` - Complex historical deck examples
