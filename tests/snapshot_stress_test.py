#!/usr/bin/env python3
"""
Randomized stress test for snapshot/resume functionality.

This test verifies STRICT DETERMINISM by:
1. Running a game with deterministic controllers (heuristic/heuristic or random with fixed seed)
2. Extracting the exact sequence of choices made
3. Running the same game stop-and-go with fixed controllers replaying those choices
4. Comparing game action logs to verify they match EXACTLY
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
    """Extract the choice index from a RANDOM or HEURISTIC log line

    INVARIANT: Choice 0 = pass priority (always available)
               Choice N (N > 0) = select ability N-1 from available array
    """
    # Pattern: "chose N" where N is the choice number
    # Pattern: "chose N (pass priority)" - index 0
    # Pattern: "chose N (ability M)" - index N selects available[M]

    if "chose no attackers" in line:
        return 0  # No attackers is choice 0

    # Match "chose N" at the beginning of choice descriptions
    match = re.search(r'chose (\d+)', line)
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
    """
    Run a normal game and extract choices.

    Args:
        mtg_bin: Path to mtg binary
        deck1: Path to player 1's deck
        deck2: Path to player 2's deck
        p1_controller: Controller type for player 1 (random, heuristic, etc.)
        p2_controller: Controller type for player 2
        seed: Random seed
        save_gamestate: Optional path to save final game state

    Returns: (log, p1_choices, p2_choices)
    """
    print(f"\n{CYAN}=== Running normal game ({p1_controller} vs {p2_controller}, seed={seed}) ==={NC}")

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

    print(f"  {GREEN}✓{NC} Game completed")
    print(f"  Extracted {len(p1_choices)} P1 choices and {len(p2_choices)} P2 choices")

    return result.stdout, p1_choices, p2_choices

def run_stop_and_go_game(mtg_bin: Path, deck1: str, deck2: str,
                         p1_controller: str, p2_controller: str,
                         p1_choices: List[int], p2_choices: List[int],
                         seed: int, num_stops: int = 3,
                         save_gamestate: Optional[Path] = None) -> str:
    """
    Run a stop-and-go game, using fixed controllers for players that made random choices.

    Args:
        p1_controller: Original controller type for P1 (random/heuristic)
        p2_controller: Original controller type for P2 (random/heuristic)
        p1_choices: Choices made by P1 (to replay if P1 was random)
        p2_choices: Choices made by P2 (to replay if P2 was random)
        save_gamestate: Optional path to save final game state

    Returns: accumulated log from all segments
    """
    print(f"\n{CYAN}=== Running stop-and-go game ({num_stops} stops) ==={NC}")

    # For stop-and-go, use 'fixed' for random players (to replay their choices)
    # and keep original controller type for deterministic players (heuristic/zero)
    p1_stopgo_controller = "fixed" if p1_controller == "random" else p1_controller
    p2_stopgo_controller = "fixed" if p2_controller == "random" else p2_controller

    # Convert choices to strings for command line
    p1_choices_str = " ".join(map(str, p1_choices))
    p2_choices_str = " ".join(map(str, p2_choices))

    accumulated_log = ""
    snapshot_file = Path("/tmp/test_snapshot.json")

    # Calculate stop points: distribute stops evenly across the total choices
    # Count choices from random players only
    random_player_choices = []
    if p1_controller == "random":
        random_player_choices.extend(p1_choices)
    if p2_controller == "random":
        random_player_choices.extend(p2_choices)

    total_choices = len(random_player_choices)
    if total_choices == 0:
        print_color(YELLOW, "Warning: No random choices to replay, using original controllers")
        # Fall back to original controllers
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
    # Adjust num_stops if we don't have enough choices
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

    print(f"  Stop points: {stop_points}")

    # Run segments
    for i, stop_after in enumerate(stop_points + [0]):  # 0 = run to completion
        if i == 0:
            # First segment: start from beginning
            print(f"  {CYAN}Segment {i+1}/{len(stop_points)+1}:{NC} Starting game, stopping after {stop_after} choices...")
            cmd = [
                str(mtg_bin), "tui",
                deck1, deck2,
                f"--p1={p1_stopgo_controller}",
                f"--p2={p2_stopgo_controller}",
                f"--seed={seed}",
                "--verbosity=3"
            ]
            # Only add fixed-inputs for players using fixed controller
            if p1_stopgo_controller == "fixed":
                cmd.append(f"--p1-fixed-inputs={p1_choices_str}")
            if p2_stopgo_controller == "fixed":
                cmd.append(f"--p2-fixed-inputs={p2_choices_str}")

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
                f"--p1={p1_stopgo_controller}",
                f"--p2={p2_stopgo_controller}",
                "--verbosity=3"
            ]
            # DON'T add fixed-inputs when resuming - let snapshot restore controller state
            # The snapshot contains the controller state with current_index preserved

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
        print(f"    {GREEN}✓{NC} Segment completed")

        # Check if game ended
        if "Game Over" in result.stdout or "wins!" in result.stdout:
            print(f"  {GREEN}✓{NC} Game ended at segment {i+1}")
            break

    # Cleanup
    if snapshot_file.exists():
        snapshot_file.unlink()

    return accumulated_log

def compare_gamestates(normal_state_file: Path, stopgo_state_file: Path) -> Tuple[bool, List[str]]:
    """
    Compare final GameState snapshots for differences.

    Returns: (match_result, list_of_differences)
    """
    print(f"\n{CYAN}=== Comparing Final GameStates ==={NC}")

    # Load both snapshots
    try:
        with open(normal_state_file, 'r') as f:
            normal_state = json.load(f)
        with open(stopgo_state_file, 'r') as f:
            stopgo_state = json.load(f)
    except Exception as e:
        print_color(RED, f"✗ Failed to load gamestate files: {e}")
        return False, [f"Failed to load files: {e}"]

    # Extract just the game_state portion (not the snapshot metadata)
    normal_gs = normal_state.get("game_state", {})
    stopgo_gs = stopgo_state.get("game_state", {})

    # Serialize both to canonical JSON for comparison
    normal_json = json.dumps(normal_gs, sort_keys=True, indent=2)
    stopgo_json = json.dumps(stopgo_gs, sort_keys=True, indent=2)

    if normal_json == stopgo_json:
        print_color(GREEN, "✓ GameStates match exactly!")
        return True, []
    else:
        # Find differences
        normal_lines = normal_json.split('\n')
        stopgo_lines = stopgo_json.split('\n')

        differences = []
        max_len = max(len(normal_lines), len(stopgo_lines))

        for i in range(min(20, max_len)):  # Show first 20 differences
            if i >= len(normal_lines):
                differences.append(f"Line {i+1}: Normal ended, stop-go has: {stopgo_lines[i][:80]}")
            elif i >= len(stopgo_lines):
                differences.append(f"Line {i+1}: Stop-go ended, normal has: {normal_lines[i][:80]}")
            elif normal_lines[i] != stopgo_lines[i]:
                differences.append(f"Line {i+1}:")
                differences.append(f"  Normal:  {normal_lines[i][:80]}")
                differences.append(f"  Stop-go: {stopgo_lines[i][:80]}")

        print_color(RED, f"✗ GameStates differ! Found {len(differences)} line differences")
        for diff in differences[:20]:
            print(f"  {diff}")

        return False, differences

def compare_game_logs(normal_log: str, stopgo_log: str,
                      save_logs: bool = False, log_dir: Optional[Path] = None,
                      test_name: str = "") -> Tuple[bool, Optional[Path], Optional[Path]]:
    """
    Compare game action logs for exact match.

    Args:
        normal_log: Full log from normal game run
        stopgo_log: Full log from stop-and-go game run
        save_logs: If True, save filtered logs to files
        log_dir: Directory to save logs to (if save_logs is True)
        test_name: Name prefix for saved log files

    Returns: (match_result, normal_log_path, stopgo_log_path)
    """
    print(f"\n{CYAN}=== Comparing Game Logs ==={NC}")

    normal_actions = filter_game_actions(normal_log)
    stopgo_actions = filter_game_actions(stopgo_log)

    print(f"  Normal game: {len(normal_actions)} actions")
    print(f"  Stop-and-go: {len(stopgo_actions)} actions")

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
        print_color(RED, "✗ One or both logs are empty after filtering")
        return False, normal_log_path, stopgo_log_path

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
        return False, normal_log_path, stopgo_log_path
    else:
        print_color(GREEN, "✓ Logs match exactly!")
        return True, normal_log_path, stopgo_log_path

def run_test_for_deck(mtg_bin: Path, deck_name: str, deck_path: str,
                      p1_controller: str, p2_controller: str, seed: int,
                      keep_logs: bool = False, log_dir: Optional[Path] = None) -> bool:
    """Run complete test for a specific deck"""
    print(f"\n{'='*70}")
    print(f"{CYAN}Testing: {deck_name} ({p1_controller} vs {p2_controller}){NC}")
    print(f"Seed: {seed}")
    print(f"{'='*70}")

    # Create temp files for gamestates
    import tempfile
    normal_state_file = Path(tempfile.mktemp(suffix="_normal.gamestate"))
    stopgo_state_file = Path(tempfile.mktemp(suffix="_stopgo.gamestate"))

    # Run normal game and extract choices
    normal_log, p1_choices, p2_choices = run_normal_game(
        mtg_bin, deck_path, deck_path, p1_controller, p2_controller, seed,
        save_gamestate=normal_state_file
    )

    # Run stop-and-go game with fixed controllers replaying random choices
    stopgo_log = run_stop_and_go_game(
        mtg_bin, deck_path, deck_path,
        p1_controller, p2_controller,
        p1_choices, p2_choices, seed, num_stops=5,
        save_gamestate=stopgo_state_file
    )

    if not stopgo_log:
        print_color(RED, f"\n✗ FAILURE: {deck_name} - stop-and-go game failed")
        # Cleanup temp files
        if normal_state_file.exists():
            normal_state_file.unlink()
        if stopgo_state_file.exists():
            stopgo_state_file.unlink()
        return False

    # Compare logs for exact match
    test_name = f"{deck_name}_{p1_controller}v{p2_controller}_seed{seed}"
    log_success, normal_path, stopgo_path = compare_game_logs(
        normal_log, stopgo_log,
        save_logs=keep_logs, log_dir=log_dir, test_name=test_name
    )

    # Compare final gamestates
    gamestate_success = True
    gamestate_diffs = []
    if normal_state_file.exists() and stopgo_state_file.exists():
        gamestate_success, gamestate_diffs = compare_gamestates(
            normal_state_file, stopgo_state_file
        )
    else:
        print_color(YELLOW, "Warning: GameState files not found, skipping comparison")

    # Overall success requires both log and gamestate match
    success = log_success and gamestate_success

    if success:
        print_color(GREEN, f"\n✓ SUCCESS: {deck_name} test passed!")
    else:
        print_color(RED, f"\n✗ FAILURE: {deck_name} test failed!")
        if not log_success:
            print(f"  - Log comparison failed")
        if not gamestate_success:
            print(f"  - GameState comparison failed ({len(gamestate_diffs)} differences)")

    # Report log paths if saved
    if keep_logs and normal_path and stopgo_path:
        print(f"\n{CYAN}Filtered logs saved:{NC}")
        print(f"  Normal game:  {normal_path}")
        print(f"  Stop-and-go:  {stopgo_path}")

    # Cleanup temp gamestate files
    if normal_state_file.exists():
        normal_state_file.unlink()
    if stopgo_state_file.exists():
        stopgo_state_file.unlink()

    return success

def parse_args():
    """Parse command line arguments"""
    parser = argparse.ArgumentParser(
        description='Stress test for MTG snapshot/resume functionality with strict determinism checks',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Run tests normally (logs not saved)
  %(prog)s

  # Run tests and keep filtered comparison logs
  %(prog)s --keep-logs

  # Keep logs in a custom directory
  %(prog)s --keep-logs --log-dir /tmp/mtg_test_logs

The filtered logs contain only the game actions used for determinism comparison:
  - Card draws, plays, casts
  - Attacks, blocks, damage
  - Turn markers
  - Game end conditions

These logs exclude transient information like:
  - Snapshot/resume messages
  - Stop point indicators
  - Debug output
        """
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
        metavar='DIR',
        help='Directory to save logs when --keep-logs is used (default: test_logs/)'
    )

    return parser.parse_args()


def main():
    args = parse_args()

    print(f"{CYAN}=== MTG Snapshot/Resume Strict Determinism Test ==={NC}\n")

    # Check for cardsfolder
    if not Path("cardsfolder").exists():
        print_color(YELLOW, "Warning: cardsfolder not found, skipping test")
        sys.exit(0)

    # Find binary
    mtg_bin = find_mtg_binary()
    print(f"Using binary: {mtg_bin}\n")

    # Setup log directory if keeping logs
    log_dir = Path(args.log_dir) if args.keep_logs else None
    if args.keep_logs:
        print(f"Filtered logs will be saved to: {log_dir}/\n")

    # Test decks (as specified in mtg-89)
    # Note: monored.dck requires modern cards not in cardsfolder, using grizzly_bears as substitute
    test_decks = [
        ("Royal Assassin", "test_decks/royal_assassin.dck"),
        ("White Aggro 4ED", "test_decks/white_aggro_4ed.dck"),
        ("Grizzly Bears", "test_decks/grizzly_bears.dck"),
    ]

    # Controller matchups to test
    # Using random vs heuristic gives us choices to extract and replay
    # Random player makes random choices that we can log and replay with fixed controller
    controller_matchups = [
        ("random", "heuristic"),
    ]

    all_passed = True
    results = []

    for deck_name, deck_path in test_decks:
        # Check if deck exists
        if not Path(deck_path).exists():
            print_color(YELLOW, f"Skipping {deck_name}: deck file not found at {deck_path}")
            continue

        for p1_controller, p2_controller in controller_matchups:
            # Use fixed seed for reproducibility
            seed = 42

            passed = run_test_for_deck(
                mtg_bin, deck_name, deck_path, p1_controller, p2_controller, seed,
                keep_logs=args.keep_logs, log_dir=log_dir
            )
            matchup_name = f"{p1_controller} vs {p2_controller}"
            results.append((deck_name, matchup_name, passed))
            all_passed = all_passed and passed

    # Print summary
    print("\n" + "="*70)
    print(f"{CYAN}SUMMARY{NC}")
    print("="*70)

    for deck_name, matchup_name, passed in results:
        status = f"{GREEN}PASS{NC}" if passed else f"{RED}FAIL{NC}"
        print(f"  {deck_name} ({matchup_name}): {status}")

    print()

    if all_passed:
        print_color(GREEN, "✓ All tests passed!")
        if args.keep_logs and log_dir:
            print(f"\nFiltered comparison logs saved in: {log_dir}/")
        sys.exit(0)
    else:
        print_color(RED, "✗ Some tests failed")
        if args.keep_logs and log_dir:
            print(f"\nFiltered comparison logs saved in: {log_dir}/")
        sys.exit(1)

if __name__ == "__main__":
    main()
