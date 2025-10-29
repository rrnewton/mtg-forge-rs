#!/usr/bin/env python3
"""
Compare two game action log files.

Usage:
    diff_logs.py <normal_log> <stopgo_log> [--verbose]

The script compares filtered game actions from both logs and reports differences.
"""

import argparse
import sys
from pathlib import Path

# ANSI color codes
RED = '\033[0;31m'
GREEN = '\033[0;32m'
YELLOW = '\033[1;33m'
CYAN = '\033[0;36m'
NC = '\033[0m'  # No Color

def print_color(color: str, message: str):
    """Print colored message"""
    print(f"{color}{message}{NC}")

def read_action_log(log_path: Path) -> list:
    """Read filtered action log from file"""
    with open(log_path, 'r') as f:
        return [line.strip() for line in f if line.strip()]

def compare_logs(normal_actions: list, stopgo_actions: list, verbose: bool = False) -> bool:
    """Compare two action logs and report differences"""
    if len(normal_actions) == 0 or len(stopgo_actions) == 0:
        print_color(RED, f"Empty action logs: normal={len(normal_actions)}, stopgo={len(stopgo_actions)}")
        return False

    # Compare action by action
    match = True
    if len(normal_actions) != len(stopgo_actions):
        match = False
        print_color(RED, f"Action count mismatch: normal={len(normal_actions)}, stopgo={len(stopgo_actions)}")

    for i in range(min(len(normal_actions), len(stopgo_actions))):
        if normal_actions[i] != stopgo_actions[i]:
            match = False
            if verbose and i < 20:  # Show first 20 differences
                print_color(RED, f"Line {i+1} differs:")
                print(f"  Normal:  {normal_actions[i]}")
                print(f"  Stop-go: {stopgo_actions[i]}")

    # If lengths differ, show where they diverge
    if len(normal_actions) != len(stopgo_actions):
        shorter = min(len(normal_actions), len(stopgo_actions))
        # Show last few actions before divergence
        print_color(CYAN, f"Last 5 common actions before divergence:")
        for i in range(max(0, shorter - 5), shorter):
            print(f"  [{i+1}] {normal_actions[i]}")

        if len(normal_actions) > len(stopgo_actions):
            print_color(YELLOW, f"Normal has {len(normal_actions) - shorter} extra actions:")
            for i in range(shorter, min(shorter + 10, len(normal_actions))):
                print(f"  [{i+1}] {normal_actions[i]}")
        else:
            print_color(YELLOW, f"Stop-go has {len(stopgo_actions) - shorter} extra actions:")
            for i in range(shorter, min(shorter + 10, len(stopgo_actions))):
                print(f"  [{i+1}] {stopgo_actions[i]}")

    return match

def main():
    parser = argparse.ArgumentParser(
        description='Compare two game action log files',
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )

    parser.add_argument('normal_log', type=str, help='Path to normal game log')
    parser.add_argument('stopgo_log', type=str, help='Path to stop-go game log')
    parser.add_argument('--verbose', '-v', action='store_true', help='Show detailed differences')

    args = parser.parse_args()

    normal_path = Path(args.normal_log)
    stopgo_path = Path(args.stopgo_log)

    if not normal_path.exists():
        print_color(RED, f"Error: Normal log not found: {normal_path}")
        sys.exit(1)

    if not stopgo_path.exists():
        print_color(RED, f"Error: Stop-go log not found: {stopgo_path}")
        sys.exit(1)

    # Read logs
    normal_actions = read_action_log(normal_path)
    stopgo_actions = read_action_log(stopgo_path)

    # Compare
    match = compare_logs(normal_actions, stopgo_actions, verbose=args.verbose)

    if match:
        print_color(GREEN, "✓ Logs match")
        sys.exit(0)
    else:
        print_color(RED, "✗ Logs differ")
        sys.exit(1)

if __name__ == "__main__":
    main()
