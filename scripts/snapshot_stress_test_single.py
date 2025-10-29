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

        # Skip diagnostic messages like [SUPPRESSED], emoji markers, etc.
        if "[SUPPRESSED]" in stripped or "🔄" in stripped or "📸" in stripped or "✅" in stripped:
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
            # Skip metadata/presentation fields that differ between normal and stop/go modes:
            # - choice_id: unique ID for each choice (increments differently based on stop points)
            # - undo_log: snapshot/replay metadata, not actual game state
            # - show_choice_menu: presentation flag (set true in stop/go mode)
            # - output_mode: presentation setting (Stdout vs Both)
            # - output_format: presentation setting (Text vs JSON)
            # - numeric_choices: presentation setting
            # - step_header_printed: transient UI state
            if key in ("choice_id", "undo_log", "show_choice_menu", "output_mode",
                      "output_format", "numeric_choices", "step_header_printed"):
                continue
            result[key] = strip_metadata_fields(value)
        return result
    elif isinstance(obj, list):
        return [strip_metadata_fields(item) for item in obj]
    else:
        return obj

def compare_gamestates(normal_state_file: Path, stopgo_state_file: Path, verbose: bool = False) -> bool:
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

    if normal_json != stopgo_json and verbose:
        # Show differences using unified diff
        import difflib
        diff = difflib.unified_diff(
            normal_json.splitlines(keepends=True),
            stopgo_json.splitlines(keepends=True),
            fromfile='normal_gamestate',
            tofile='stopgo_gamestate',
            lineterm=''
        )
        print_color(YELLOW, "    GameState differences:")
        for i, line in enumerate(diff):
            if i < 50:  # Show first 50 lines of diff
                print(f"      {line.rstrip()}")
            elif i == 50:
                print("      ... (diff truncated)")
                break

    return normal_json == stopgo_json

def compare_game_logs(normal_log: str, stopgo_log: str, verbose: bool = False,
                      save_logs: bool = False, log_dir: Optional[Path] = None,
                      test_name: str = "") -> Tuple[bool, List[str], List[str], Optional[Path], Optional[Path]]:
    """Compare game action logs for exact match.

    Returns: (match_success, normal_actions, stopgo_actions, normal_log_path, stopgo_log_path)
    """
    normal_actions = filter_game_actions(normal_log)
    stopgo_actions = filter_game_actions(stopgo_log)

    # Save logs if requested
    normal_log_path = None
    stopgo_log_path = None
    if save_logs and log_dir:
        log_dir.mkdir(parents=True, exist_ok=True)

        # Sanitize test name for filename
        safe_name = test_name.replace(" ", "_").replace("/", "_")

        normal_log_path = log_dir / f"{safe_name}_normal.log"
        stopgo_log_path = log_dir / f"{safe_name}_stopgo.log"

        # Write filtered actions to files
        with open(normal_log_path, 'w') as f:
            f.write('\n'.join(normal_actions))

        with open(stopgo_log_path, 'w') as f:
            f.write('\n'.join(stopgo_actions))

    if len(normal_actions) == 0 or len(stopgo_actions) == 0:
        if verbose:
            print_color(RED, f"  Empty action logs: normal={len(normal_actions)}, stopgo={len(stopgo_actions)}")
        return False, normal_actions, stopgo_actions, normal_log_path, stopgo_log_path

    # Compare action by action
    match = True
    if len(normal_actions) != len(stopgo_actions):
        match = False
        if verbose:
            print_color(RED, f"  Action count mismatch: normal={len(normal_actions)}, stopgo={len(stopgo_actions)}")

    for i in range(min(len(normal_actions), len(stopgo_actions))):
        if normal_actions[i] != stopgo_actions[i]:
            match = False
            if verbose and i < 20:  # Show first 20 differences
                print_color(RED, f"  Line {i+1} differs:")
                print(f"    Normal:  {normal_actions[i]}")
                print(f"    Stop-go: {stopgo_actions[i]}")

    # If lengths differ, show where they diverge
    if verbose and len(normal_actions) != len(stopgo_actions):
        shorter = min(len(normal_actions), len(stopgo_actions))
        # Show last few actions before divergence
        print_color(CYAN, f"  Last 5 common actions before divergence:")
        for i in range(max(0, shorter - 5), shorter):
            print(f"    [{i+1}] {normal_actions[i]}")

        if len(normal_actions) > len(stopgo_actions):
            print_color(YELLOW, f"  Normal has {len(normal_actions) - shorter} extra actions:")
            for i in range(shorter, min(shorter + 10, len(normal_actions))):
                print(f"    [{i+1}] {normal_actions[i]}")
        else:
            print_color(YELLOW, f"  Stop-go has {len(stopgo_actions) - shorter} extra actions:")
            for i in range(shorter, min(shorter + 10, len(stopgo_actions))):
                print(f"    [{i+1}] {stopgo_actions[i]}")

    return match, normal_actions, stopgo_actions, normal_log_path, stopgo_log_path

def run_test_for_deck(mtg_bin: Path, deck_path: str,
                      p1_controller: str, p2_controller: str, seed: int,
                      num_replays: int = 3, verbose: bool = False,
                      keep_logs: bool = False, log_dir: Optional[Path] = None) -> bool:
    """Run complete test for a specific deck with multiple replay runs."""
    # Create temp files for gamestates
    import tempfile
    normal_state_file = Path(tempfile.mktemp(suffix="_normal.gamestate"))

    # Run normal game and extract choices
    normal_log, p1_choices, p2_choices = run_normal_game(
        mtg_bin, deck_path, deck_path, p1_controller, p2_controller, seed,
        save_gamestate=normal_state_file
    )

    # Extract deck name for test naming
    deck_name = Path(deck_path).stem

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
        test_name = f"{deck_name}_{p1_controller}v{p2_controller}_seed{seed}_replay{replay_num+1}"
        log_success, normal_actions, stopgo_actions, normal_log_path, stopgo_log_path = compare_game_logs(
            normal_log, stopgo_log, verbose=verbose,
            save_logs=keep_logs, log_dir=log_dir, test_name=test_name
        )

        # Compare final gamestates
        gamestate_success = True
        if normal_state_file.exists() and stopgo_state_file.exists():
            gamestate_success = compare_gamestates(normal_state_file, stopgo_state_file, verbose=verbose)

        # Check if this replay succeeded
        replay_success = log_success and gamestate_success
        all_success = all_success and replay_success

        if verbose:
            if replay_success:
                print_color(GREEN, f"  ✓ Replay {replay_num+1}/{num_replays} PASSED")
            else:
                print_color(RED, f"  ✗ Replay {replay_num+1}/{num_replays} FAILED")
                if not log_success:
                    print_color(RED, "    - Log comparison failed")
                if not gamestate_success:
                    print_color(RED, "    - GameState comparison failed")

        # Report log file paths if logs were saved
        if keep_logs and normal_log_path and stopgo_log_path:
            if not verbose and not replay_success:
                # Always show log paths for failures, even without verbose
                print_color(CYAN, f"  Saved logs for replay {replay_num+1}:")
                print(f"    Normal:  {normal_log_path}")
                print(f"    Stop-go: {stopgo_log_path}")
            elif verbose:
                # In verbose mode, show paths for all replays
                print_color(CYAN, f"  Saved logs for replay {replay_num+1}:")
                print(f"    Normal:  {normal_log_path}")
                print(f"    Stop-go: {stopgo_log_path}")

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

    parser.add_argument(
        '--verbose', '-v',
        action='store_true',
        help='Show detailed comparison output including log differences'
    )

    parser.add_argument(
        '--keep-logs',
        action='store_true',
        help='Save filtered game action logs for inspection (default: logs are not saved)'
    )

    parser.add_argument(
        '--log-dir',
        type=str,
        default='test_logs',
        help='Directory to save logs when --keep-logs is used (default: test_logs/)'
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
        args.seed, num_replays=args.replays, verbose=args.verbose,
        keep_logs=args.keep_logs, log_dir=Path(args.log_dir) if args.keep_logs else None
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
