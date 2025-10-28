#!/usr/bin/env python3
"""
Single-deck snapshot/resume stress test.

Runs a snapshot/resume stress test for a single deck with a single controller matchup.
Designed to be called by run_stress_tests.sh for parallel execution.

Usage:
    snapshot_stress_test_single.py <deck_path> <p1_controller> <p2_controller> [--seed SEED] [--replays N]

Examples:
    snapshot_stress_test_single.py decks/royal_assassin.dck heuristic heuristic
    snapshot_stress_test_single.py decks/white_aggro_4ed.dck random heuristic --seed 42 --replays 3
"""

import argparse
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
    if "chose no attackers" in line:
        return 0  # No attackers is choice 0

    # Match "chose N" at the beginning of choice descriptions
    match = re.search(r'chose (\d+)', line)
    if match:
        return int(match.group(1))

    return None

def extract_choices_from_log(log: str) -> Tuple[List[int], List[int]]:
    """Extract P1 and P2 choice sequences from verbose game log."""
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
    """Filter log to only include meaningful game actions for comparison."""
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

        # Skip "Turn number:" messages (from snapshot resume)
        if "Turn number:" in stripped:
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
                    p1_controller: str, p2_controller: str, seed: int,
                    save_gamestate: Optional[Path] = None) -> Tuple[str, List[int], List[int]]:
    """Run a normal game and extract choices."""
    cmd = [
        str(mtg_bin), "tui",
        deck1, deck2,
        f"--p1={p1_controller}",
        f"--p2={p2_controller}",
        f"--seed={seed}",
        "--verbosity=3"
    ]

    if save_gamestate:
        cmd.append(f"--save-final-gamestate={save_gamestate}")

    result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)

    if result.returncode != 0:
        print_color(RED, f"Normal game failed with code {result.returncode}")
        print("STDOUT:", result.stdout[:2000])
        print("STDERR:", result.stderr[:2000])
        sys.exit(1)

    # Extract choices
    p1_choices, p2_choices = extract_choices_from_log(result.stdout)

    return result.stdout, p1_choices, p2_choices

def run_stop_and_go_game(mtg_bin: Path, deck1: str, deck2: str,
                         p1_controller: str, p2_controller: str,
                         p1_choices: List[int], p2_choices: List[int],
                         seed: int, num_stops: int = 3,
                         save_gamestate: Optional[Path] = None) -> str:
    """Run a stop-and-go game with randomized stop points."""
    import tempfile
    accumulated_log = ""
    # Use unique temp file to avoid conflicts when running in parallel
    snapshot_file = Path(tempfile.mktemp(suffix="_snapshot.json"))

    # Calculate stop points
    all_player_choices = []
    all_player_choices.extend(p1_choices)
    all_player_choices.extend(p2_choices)

    total_choices = len(all_player_choices)
    if total_choices == 0:
        # Fall back to running game normally
        cmd = [
            str(mtg_bin), "tui",
            deck1, deck2,
            f"--p1={p1_controller}",
            f"--p2={p2_controller}",
            f"--seed={seed}",
            "--verbosity=3"
        ]
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
        return result.stdout

    # Generate random stop points
    actual_num_stops = min(num_stops, max(1, total_choices - 1))

    choices_per_stop = max(1, total_choices // (actual_num_stops + 1))
    stop_points = []
    cumulative = 0
    for i in range(actual_num_stops):
        remaining = total_choices - cumulative - (actual_num_stops - i)
        if remaining <= 0:
            break
        advance = random.randint(1, min(choices_per_stop * 2, remaining))
        if advance > 0:
            cumulative += advance
            stop_points.append(advance)

    if not stop_points:
        stop_points = [1]  # At least one stop

    # Run segments
    for i, stop_after in enumerate(stop_points + [0]):  # 0 = run to completion
        if i == 0:
            # First segment: start from beginning
            cmd = [
                str(mtg_bin), "tui",
                deck1, deck2,
                f"--p1={p1_controller}",
                f"--p2={p2_controller}",
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
            if not snapshot_file.exists():
                print_color(RED, f"✗ Snapshot file missing at segment {i+1}")
                return ""

            cmd = [
                str(mtg_bin), "tui",
                f"--start-from={snapshot_file}",
                f"--p1={p1_controller}",
                f"--p2={p2_controller}",
                "--verbosity=3"
            ]

            if stop_after > 0:
                cmd.extend([
                    f"--stop-every=both:choice:{stop_after}",
                    f"--snapshot-output={snapshot_file}",
                ])
            elif save_gamestate:
                # Final segment - save gamestate
                cmd.append(f"--save-final-gamestate={save_gamestate}")

        result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)

        if result.returncode != 0:
            print_color(RED, f"✗ Segment {i+1} failed with code {result.returncode}")
            print("STDOUT:", result.stdout[:2000])
            print("STDERR:", result.stderr[:2000])
            return ""

        accumulated_log += result.stdout

        # Check if game ended
        if "Game Over" in result.stdout or "wins!" in result.stdout:
            break

    # Cleanup
    if snapshot_file.exists():
        snapshot_file.unlink()

    return accumulated_log

def strip_metadata_fields(obj):
    """Recursively strip metadata fields from gamestate."""
    if isinstance(obj, dict):
        result = {}
        for key, value in obj.items():
            # Skip choice_id fields in ChoicePoint entries
            if key == "choice_id":
                continue
            result[key] = strip_metadata_fields(value)
        return result
    elif isinstance(obj, list):
        return [strip_metadata_fields(item) for item in obj]
    else:
        return obj

def compare_gamestates(normal_state_file: Path, stopgo_state_file: Path) -> bool:
    """Compare final GameState snapshots for differences."""
    try:
        with open(normal_state_file, 'r') as f:
            normal_state = json.load(f)
        with open(stopgo_state_file, 'r') as f:
            stopgo_state = json.load(f)
    except Exception as e:
        print_color(RED, f"✗ Failed to load gamestate files: {e}")
        return False

    # Extract just the game_state portion
    normal_gs = normal_state.get("game_state", {})
    stopgo_gs = stopgo_state.get("game_state", {})

    # Strip metadata fields
    normal_gs = strip_metadata_fields(normal_gs)
    stopgo_gs = strip_metadata_fields(stopgo_gs)

    # Serialize both to canonical JSON for comparison
    normal_json = json.dumps(normal_gs, sort_keys=True, indent=2)
    stopgo_json = json.dumps(stopgo_gs, sort_keys=True, indent=2)

    return normal_json == stopgo_json

def compare_game_logs(normal_log: str, stopgo_log: str) -> bool:
    """Compare game action logs for exact match."""
    normal_actions = filter_game_actions(normal_log)
    stopgo_actions = filter_game_actions(stopgo_log)

    if len(normal_actions) == 0 or len(stopgo_actions) == 0:
        return False

    # Compare action by action
    if len(normal_actions) != len(stopgo_actions):
        return False

    for i in range(len(normal_actions)):
        if normal_actions[i] != stopgo_actions[i]:
            return False

    return True

def run_test_for_deck(mtg_bin: Path, deck_path: str,
                      p1_controller: str, p2_controller: str, seed: int,
                      num_replays: int = 3) -> bool:
    """Run complete test for a specific deck with multiple replay runs."""
    # Create temp files for gamestates
    import tempfile
    normal_state_file = Path(tempfile.mktemp(suffix="_normal.gamestate"))

    # Run normal game and extract choices
    normal_log, p1_choices, p2_choices = run_normal_game(
        mtg_bin, deck_path, deck_path, p1_controller, p2_controller, seed,
        save_gamestate=normal_state_file
    )

    # Run multiple stop-and-go games with different random stop points
    all_success = True

    for replay_num in range(num_replays):
        # Use different random seed for stop point generation each replay
        random.seed(seed + replay_num + 1000)

        stopgo_state_file = Path(tempfile.mktemp(suffix=f"_stopgo_{replay_num}.gamestate"))

        # Run stop-and-go game with randomized stop points (5 stops)
        stopgo_log = run_stop_and_go_game(
            mtg_bin, deck_path, deck_path,
            p1_controller, p2_controller,
            p1_choices, p2_choices, seed, num_stops=5,
            save_gamestate=stopgo_state_file
        )

        if not stopgo_log:
            all_success = False
            if stopgo_state_file.exists():
                stopgo_state_file.unlink()
            continue

        # Compare logs for exact match
        log_success = compare_game_logs(normal_log, stopgo_log)

        # Compare final gamestates
        gamestate_success = True
        if normal_state_file.exists() and stopgo_state_file.exists():
            gamestate_success = compare_gamestates(normal_state_file, stopgo_state_file)

        # Check if this replay succeeded
        replay_success = log_success and gamestate_success
        all_success = all_success and replay_success

        # Cleanup this replay's gamestate file
        if stopgo_state_file.exists():
            stopgo_state_file.unlink()

    # Cleanup normal gamestate file
    if normal_state_file.exists():
        normal_state_file.unlink()

    return all_success

def parse_args():
    """Parse command line arguments"""
    parser = argparse.ArgumentParser(
        description='Single-deck snapshot/resume stress test',
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )

    parser.add_argument(
        'deck_path',
        type=str,
        help='Path to the deck file (.dck)'
    )

    parser.add_argument(
        'p1_controller',
        type=str,
        choices=['random', 'heuristic'],
        help='Controller type for player 1'
    )

    parser.add_argument(
        'p2_controller',
        type=str,
        choices=['random', 'heuristic'],
        help='Controller type for player 2'
    )

    parser.add_argument(
        '--seed',
        type=int,
        default=42,
        help='Random seed for the game (default: 42)'
    )

    parser.add_argument(
        '--replays',
        type=int,
        default=3,
        help='Number of stop-and-go replay runs (default: 3)'
    )

    parser.add_argument(
        '--quiet',
        action='store_true',
        help='Suppress output except for errors'
    )

    return parser.parse_args()


def main():
    args = parse_args()

    # Check for cardsfolder
    if not Path("cardsfolder").exists():
        if not args.quiet:
            print_color(YELLOW, "Warning: cardsfolder not found, skipping test")
        sys.exit(0)

    # Check if deck exists
    if not Path(args.deck_path).exists():
        print_color(RED, f"Error: Deck file not found: {args.deck_path}")
        sys.exit(1)

    # Find binary
    mtg_bin = find_mtg_binary()

    # Extract deck name for display
    deck_name = Path(args.deck_path).stem

    if not args.quiet:
        print(f"{CYAN}Testing: {deck_name} ({args.p1_controller} vs {args.p2_controller}){NC}")

    # Run test
    passed = run_test_for_deck(
        mtg_bin, args.deck_path, args.p1_controller, args.p2_controller,
        args.seed, num_replays=args.replays
    )

    if passed:
        if not args.quiet:
            print_color(GREEN, f"✓ PASSED: {deck_name} ({args.p1_controller} vs {args.p2_controller})")
        sys.exit(0)
    else:
        print_color(RED, f"✗ FAILED: {deck_name} ({args.p1_controller} vs {args.p2_controller})")
        sys.exit(1)

if __name__ == "__main__":
    main()
