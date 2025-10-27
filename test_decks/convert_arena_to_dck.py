#!/usr/bin/env python3
"""
Convert Arena Export format (.txt) to .dck format for MTG Forge Rust.

Arena Export format:
    Deck
    <count> <card name> (SET) code
    ...

    Sideboard
    <count> <card name> (SET) code
    ...

.dck format:
    [metadata]
    Name=<deck name>
    Description=<description>

    [Main]
    <count> <card name>
    ...

    [Sideboard]
    <count> <card name>
    ...
"""

import re
import sys
from pathlib import Path


def convert_arena_to_dck(input_file: Path) -> str:
    """Convert Arena Export format to .dck format."""

    with open(input_file, 'r') as f:
        content = f.read()

    # Determine deck name from filename
    deck_name = input_file.stem.replace('_', ' ').title()

    # Split into sections
    lines = content.strip().split('\n')

    main_deck = []
    sideboard = []
    current_section = None

    for line in lines:
        line = line.strip()
        if not line:
            continue

        # Check for section headers
        if line.lower() == 'deck':
            current_section = 'main'
            continue
        elif line.lower() == 'sideboard':
            current_section = 'sideboard'
            continue

        # Parse card line: "4 Lightning Bolt (GN3) 83" -> "4 Lightning Bolt"
        # Remove set codes in parentheses and anything after
        card_line = re.sub(r'\s*\([^)]+\).*$', '', line)
        card_line = card_line.strip()

        if not card_line:
            continue

        # Add to appropriate section
        if current_section == 'main':
            main_deck.append(card_line)
        elif current_section == 'sideboard':
            sideboard.append(card_line)

    # Build .dck content
    dck_content = f"""[metadata]
Name={deck_name}
Description=Old School 93/94 deck

[Main]
"""

    for card in main_deck:
        dck_content += f"{card}\n"

    dck_content += "\n[Sideboard]\n"

    for card in sideboard:
        dck_content += f"{card}\n"

    return dck_content


def main():
    if len(sys.argv) != 2:
        print("Usage: convert_arena_to_dck.py <input_file.txt>")
        sys.exit(1)

    input_file = Path(sys.argv[1])

    if not input_file.exists():
        print(f"Error: File not found: {input_file}")
        sys.exit(1)

    # Convert
    dck_content = convert_arena_to_dck(input_file)

    # Write to .dck file
    output_file = input_file.with_suffix('.dck')
    with open(output_file, 'w') as f:
        f.write(dck_content)

    print(f"Converted {input_file} -> {output_file}")


if __name__ == '__main__':
    main()
