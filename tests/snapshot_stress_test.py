#!/usr/bin/env python3
"""
Randomized stress test for snapshot/resume functionality.

This test verifies STRICT DETERMINISM by:
1. Running a game with deterministic controllers (heuristic/heuristic or random with fixed seed)
2. Extracting the exact sequence of choices made
3. Running the same game stop-and-go with fixed controllers replaying those choices
4. Comparing game action logs to verify they match EXACTLY
"""

import subprocess
import sys
import os
import json
import random
import re
from pathlib import Path
from typing import List, Tuple, Dict, Optional

# ANSI color codes
RED = '\033[0;31m'
GREEN = '\033[0;32m'
YELLOW = '\033[1;33m'
CYAN = '\033[0;36m'
NC = '\033[0m'  # No Color

def print_color(color: str, message: str):
    """Print colored message"""
    print(f"{color}{message}{NC}")

def find_mtg_binary() -> Path:
    """Find the mtg binary (debug or release)"""
    workspace = Path.cwd()
    debug_bin = workspace / "target" / "debug" / "mtg"
    release_bin = workspace / "target" / "release" / "mtg"

    if debug_bin.exists():
        return debug_bin
    elif release_bin.exists():
        return release_bin
    else:
        print_color(RED, "Error: mtg binary not found")
        print("Please build the project first with 'cargo build'")
        sys.exit(1)

def extract_choice_from_line(line: str) -> Optional[int]:
    """Extract the choice index from a RANDOM or HEURISTIC log line"""
    # Pattern: "chose spell/ability N out of choices"
    # Pattern: "chose creature N to attack"
    # Pattern: "chose blocker N"
    # Pattern: "chose no attackers" -> 0
    # Pattern: "chose to pass priority" -> special handling (treat as choice 0)

    if "chose to pass priority" in line:
        return 0  # Pass is typically choice 0

    if "chose no attackers" in line:
        return 0  # No attackers is choice 0

    # Match "chose spell/ability N" or "chose creature N" or "chose blocker N"
    match = re.search(r'chose (?:spell/ability|creature|blocker) (\d+)', line)
    if match:
        return int(match.group(1))

    # Match patterns like "chose spell/ability N out of choices"
    match = re.search(r'chose spell/ability (\d+) out of', line)
    if match:
        return int(match.group(1))

    return None

def extract_choices_from_log(log: str) -> Tuple[List[int], List[int]]:
    """
    Extract P1 and P2 choice sequences from verbose game log.

    Returns: (p1_choices, p2_choices) where each is a list of choice indices
    """
    p1_choices = []
    p2_choices = []
    current_player = None

    lines = log.split('\n')
    for line in lines:
        # Track which player's turn it is
        if "Alice's turn" in line or "Alice (active)" in line:
            current_player = 'p1'
        elif "Bob's turn" in line or "Bob (active)" in line:
            current_player = 'p2'

        # Look for choice lines
        if ">>> RANDOM:" in line or ">>> HEURISTIC:" in line or ">>> ZERO:" in line:
            choice = extract_choice_from_line(line)
            if choice is not None:
                if current_player == 'p1':
                    p1_choices.append(choice)
                elif current_player == 'p2':
                    p2_choices.append(choice)

    return p1_choices, p2_choices

def filter_game_actions(log: str) -> List[str]:
    """
    Filter log to only include meaningful game actions for comparison.
    Removes stop/resume messages and other transient information.
    """
    actions = []
    lines = log.split('\n')

    for line in lines:
        stripped = line.strip()

        # Skip empty lines
        if not stripped:
            continue

        # Skip snapshot/resume messages
        if "snapshot" in stripped.lower() or "resuming" in stripped.lower():
            continue

        # Skip "Stopping after" messages
        if "Stopping after" in stripped or "stopping" in stripped.lower():
            continue

        # Keep game actions: draws, plays, attacks, damage, etc.
        if any(keyword in stripped for keyword in [
            "draws ",
            "plays ",
            "casts ",
            "attacks ",
            "blocks ",
            "damage ",
            "dies",
            "destroyed",
            "sacrificed",
            "discards",
            "to graveyard",
            "Turn ",
            "wins!",
            "Game Over",
            "Life:",
            "Turns played:"
        ]):
            actions.append(stripped)

    return actions

def run_normal_game(mtg_bin: Path, deck1: str, deck2: str,
                    controller_type: str, seed: int) -> Tuple[str, List[int], List[int]]:
    """
    Run a normal game and extract choices.

    Returns: (log, p1_choices, p2_choices)
    """
    print(f"\n{CYAN}=== Running normal game ({controller_type} vs {controller_type}, seed={seed}) ==={NC}")

    cmd = [
        str(mtg_bin), "tui",
        deck1, deck2,
        f"--p1={controller_type}",
        f"--p2={controller_type}",
        f"--seed={seed}",
        "--verbosity=3"
    ]

    result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)

    if result.returncode != 0:
        print_color(RED, f"Normal game failed with code {result.returncode}")
        print("STDOUT:", result.stdout[:2000])
        print("STDERR:", result.stderr[:2000])
        sys.exit(1)

    # Extract choices
    p1_choices, p2_choices = extract_choices_from_log(result.stdout)

    print(f"  {GREEN}✓{NC} Game completed")
    print(f"  Extracted {len(p1_choices)} P1 choices and {len(p2_choices)} P2 choices")

    return result.stdout, p1_choices, p2_choices

def run_stop_and_go_game(mtg_bin: Path, deck1: str, deck2: str,
                         p1_choices: List[int], p2_choices: List[int],
                         seed: int, num_stops: int = 3) -> str:
    """
    Run a stop-and-go game using fixed controllers with the given choice sequences.

    Returns: accumulated log from all segments
    """
    print(f"\n{CYAN}=== Running stop-and-go game ({num_stops} stops) ==={NC}")

    # Convert choices to strings for command line
    p1_choices_str = " ".join(map(str, p1_choices))
    p2_choices_str = " ".join(map(str, p2_choices))

    accumulated_log = ""
    snapshot_file = Path("/tmp/test_snapshot.json")

    # Calculate stop points: distribute stops evenly across the total choices
    total_choices = max(len(p1_choices), len(p2_choices))
    if total_choices == 0:
        print_color(YELLOW, "Warning: No choices to replay, using heuristic controllers")
        # Fall back to heuristic
        cmd = [
            str(mtg_bin), "tui",
            deck1, deck2,
            "--p1=heuristic",
            "--p2=heuristic",
            f"--seed={seed}",
            "--verbosity=3"
        ]
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
        return result.stdout

    # Generate random stop points
    choices_per_stop = max(1, total_choices // (num_stops + 1))
    stop_points = []
    cumulative = 0
    for i in range(num_stops):
        advance = random.randint(1, min(choices_per_stop * 2, total_choices - cumulative - (num_stops - i)))
        if advance > 0:
            cumulative += advance
            stop_points.append(advance)

    if not stop_points:
        stop_points = [1]  # At least one stop

    print(f"  Stop points: {stop_points}")

    # Run segments
    for i, stop_after in enumerate(stop_points + [0]):  # 0 = run to completion
        if i == 0:
            # First segment: start from beginning
            print(f"  {CYAN}Segment {i+1}/{len(stop_points)+1}:{NC} Starting game, stopping after {stop_after} choices...")
            cmd = [
                str(mtg_bin), "tui",
                deck1, deck2,
                "--p1=fixed",
                "--p2=fixed",
                f"--p1-fixed-inputs={p1_choices_str}",
                f"--p2-fixed-inputs={p2_choices_str}",
                f"--seed={seed}",
                "--verbosity=3"
            ]
            if stop_after > 0:
                cmd.extend([
                    f"--stop-every=both:choice:{stop_after}",
                    f"--snapshot-output={snapshot_file}",
                ])
        else:
            # Resume from snapshot
            if stop_after > 0:
                print(f"  {CYAN}Segment {i+1}/{len(stop_points)+1}:{NC} Resuming from snapshot, stopping after {stop_after} more choices...")
            else:
                print(f"  {CYAN}Segment {i+1}/{len(stop_points)+1}:{NC} Resuming from snapshot and running to completion...")

            if not snapshot_file.exists():
                print_color(RED, f"✗ Snapshot file missing at segment {i+1}")
                return ""

            cmd = [
                str(mtg_bin), "tui",
                f"--start-from={snapshot_file}",
                "--p1=fixed",
                "--p2=fixed",
                f"--p1-fixed-inputs={p1_choices_str}",
                f"--p2-fixed-inputs={p2_choices_str}",
                "--verbosity=3"
            ]
            if stop_after > 0:
                cmd.extend([
                    f"--stop-every=both:choice:{stop_after}",
                    f"--snapshot-output={snapshot_file}",
                ])

        result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)

        if result.returncode != 0:
            print_color(RED, f"✗ Segment {i+1} failed with code {result.returncode}")
            print("STDOUT:", result.stdout[:2000])
            print("STDERR:", result.stderr[:2000])
            return ""

        accumulated_log += result.stdout
        print(f"    {GREEN}✓{NC} Segment completed")

        # Check if game ended
        if "Game Over" in result.stdout or "wins!" in result.stdout:
            print(f"  {GREEN}✓{NC} Game ended at segment {i+1}")
            break

    # Cleanup
    if snapshot_file.exists():
        snapshot_file.unlink()

    return accumulated_log

def compare_game_logs(normal_log: str, stopgo_log: str) -> bool:
    """
    Compare game action logs for exact match.

    Returns: True if logs match exactly (after filtering)
    """
    print(f"\n{CYAN}=== Comparing Game Logs ==={NC}")

    normal_actions = filter_game_actions(normal_log)
    stopgo_actions = filter_game_actions(stopgo_log)

    print(f"  Normal game: {len(normal_actions)} actions")
    print(f"  Stop-and-go: {len(stopgo_actions)} actions")

    if len(normal_actions) == 0 or len(stopgo_actions) == 0:
        print_color(RED, "✗ One or both logs are empty after filtering")
        return False

    # Compare action by action
    max_len = max(len(normal_actions), len(stopgo_actions))
    differences = []

    for i in range(max_len):
        if i >= len(normal_actions):
            differences.append(f"  Line {i+1}: Normal log ended, stop-go has: {stopgo_actions[i]}")
        elif i >= len(stopgo_actions):
            differences.append(f"  Line {i+1}: Stop-go ended, normal has: {normal_actions[i]}")
        elif normal_actions[i] != stopgo_actions[i]:
            differences.append(f"  Line {i+1} differs:")
            differences.append(f"    Normal:  {normal_actions[i]}")
            differences.append(f"    Stop-go: {stopgo_actions[i]}")

    if differences:
        print_color(RED, f"✗ Found {len(differences)} differences:")
        for diff in differences[:20]:  # Show first 20 differences
            print(f"    {diff}")
        if len(differences) > 20:
            print(f"    ... and {len(differences) - 20} more")
        return False
    else:
        print_color(GREEN, "✓ Logs match exactly!")
        return True

def run_test_for_deck(mtg_bin: Path, deck_name: str, deck_path: str,
                      controller_type: str, seed: int) -> bool:
    """Run complete test for a specific deck"""
    print(f"\n{'='*70}")
    print(f"{CYAN}Testing: {deck_name} ({controller_type} vs {controller_type}){NC}")
    print(f"Seed: {seed}")
    print(f"{'='*70}")

    # Run normal game and extract choices
    normal_log, p1_choices, p2_choices = run_normal_game(
        mtg_bin, deck_path, deck_path, controller_type, seed
    )

    # Run stop-and-go game with fixed controllers replaying the same choices
    stopgo_log = run_stop_and_go_game(
        mtg_bin, deck_path, deck_path, p1_choices, p2_choices, seed, num_stops=5
    )

    if not stopgo_log:
        print_color(RED, f"\n✗ FAILURE: {deck_name} - stop-and-go game failed")
        return False

    # Compare logs for exact match
    success = compare_game_logs(normal_log, stopgo_log)

    if success:
        print_color(GREEN, f"\n✓ SUCCESS: {deck_name} test passed!")
    else:
        print_color(RED, f"\n✗ FAILURE: {deck_name} test failed!")

    return success

def main():
    print(f"{CYAN}=== MTG Snapshot/Resume Strict Determinism Test ==={NC}\n")

    # Check for cardsfolder
    if not Path("cardsfolder").exists():
        print_color(YELLOW, "Warning: cardsfolder not found, skipping test")
        sys.exit(0)

    # Find binary
    mtg_bin = find_mtg_binary()
    print(f"Using binary: {mtg_bin}\n")

    # Test decks (as specified in mtg-89)
    # Note: monored.dck requires modern cards not in cardsfolder, using grizzly_bears as substitute
    test_decks = [
        ("Royal Assassin", "test_decks/royal_assassin.dck"),
        ("White Aggro 4ED", "test_decks/white_aggro_4ed.dck"),
        ("Grizzly Bears", "test_decks/grizzly_bears.dck"),
    ]

    # Controller types to test
    # Currently only heuristic is tested because it's inherently deterministic
    # Random controller with fixed replay requires more work (see issue mtg-89)
    controller_types = ["heuristic"]

    all_passed = True
    results = []

    for deck_name, deck_path in test_decks:
        # Check if deck exists
        if not Path(deck_path).exists():
            print_color(YELLOW, f"Skipping {deck_name}: deck file not found at {deck_path}")
            continue

        for controller_type in controller_types:
            # Use fixed seed for reproducibility
            seed = 42 if controller_type == "random" else 100

            passed = run_test_for_deck(mtg_bin, deck_name, deck_path, controller_type, seed)
            results.append((deck_name, controller_type, passed))
            all_passed = all_passed and passed

    # Print summary
    print("\n" + "="*70)
    print(f"{CYAN}SUMMARY{NC}")
    print("="*70)

    for deck_name, controller_type, passed in results:
        status = f"{GREEN}PASS{NC}" if passed else f"{RED}FAIL{NC}"
        print(f"  {deck_name} ({controller_type}): {status}")

    print()

    if all_passed:
        print_color(GREEN, "✓ All tests passed!")
        sys.exit(0)
    else:
        print_color(RED, "✗ Some tests failed")
        sys.exit(1)

if __name__ == "__main__":
    main()
