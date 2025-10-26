#!/usr/bin/env python3
"""
Show git history of changes to beads issues in a readable format.

Instead of showing raw JSON diffs, this extracts title+description from each
issue across commits and shows clean, colorized diffs of the actual content.
"""

import argparse
import json
import subprocess
import sys
import tempfile
from collections import defaultdict
from pathlib import Path
from typing import Dict, List, Optional, Tuple


def get_commit_list(num_commits: int) -> List[tuple[str, str]]:
    """Get list of (hash, subject) for the last N commits that touched issues.jsonl."""
    cmd = [
        "git",
        "log",
        f"-{num_commits}",
        "--format=%H %s",
        "--",
        ".beads/issues.jsonl",
    ]
    result = subprocess.run(cmd, capture_output=True, text=True, check=True)

    commits = []
    for line in result.stdout.strip().split("\n"):
        if not line:
            continue
        parts = line.split(" ", 1)
        commit_hash = parts[0]
        subject = parts[1] if len(parts) > 1 else ""
        commits.append((commit_hash, subject))

    return commits


def get_issues_at_commit(commit_hash: str) -> Dict[str, Dict]:
    """Extract issues.jsonl content at a specific commit."""
    cmd = ["git", "show", f"{commit_hash}:.beads/issues.jsonl"]
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
    except subprocess.CalledProcessError:
        # File might not exist at this commit
        return {}

    issues = {}
    for line in result.stdout.strip().split("\n"):
        if not line:
            continue
        try:
            issue = json.loads(line)
            issue_id = issue.get("id", "unknown")
            issues[issue_id] = issue
        except json.JSONDecodeError:
            continue

    return issues


def issue_to_text(issue: Dict) -> str:
    """Convert issue to simple title + description format."""
    title = issue.get("title", "")
    description = issue.get("description", "")

    return f"{title}\n{'-' * 80}\n{description}"


def get_colored_diff(old_text: str, new_text: str) -> Optional[str]:
    """Generate colorized word diff using git diff."""
    with tempfile.TemporaryDirectory() as tmpdir:
        tmppath = Path(tmpdir)
        old_file = tmppath / "old.txt"
        new_file = tmppath / "new.txt"

        old_file.write_text(old_text)
        new_file.write_text(new_text)

        cmd = [
            "git",
            "diff",
            "--no-index",
            "--color=always",
            "--color-words",
            "-w",
            str(old_file),
            str(new_file),
        ]

        result = subprocess.run(cmd, capture_output=True, text=True)

        # git diff --no-index returns exit code 1 when files differ, which is expected
        if result.returncode not in (0, 1):
            return None

        # Remove the temp file paths from the diff header
        lines = result.stdout.split("\n")
        output_lines = []
        for line in lines:
            # Skip diff metadata lines with temp paths
            # Note: lines may start with ANSI color codes like \033[1m
            stripped = line.lstrip('\033[0123456789;m')
            if stripped.startswith("diff --git"):
                continue
            elif stripped.startswith("index "):
                continue
            elif stripped.startswith("--- ") and "/tmp" in line:
                continue
            elif stripped.startswith("+++ ") and "/tmp" in line:
                continue
            else:
                output_lines.append(line)

        diff_output = "\n".join(output_lines).strip()
        return diff_output if diff_output else None


def parse_range(range_str: str) -> Tuple[int, int]:
    """Parse range string like '1-10' into (start, end) tuple."""
    try:
        parts = range_str.split('-')
        if len(parts) != 2:
            raise ValueError("Range must be in format 'start-end'")
        start = int(parts[0])
        end = int(parts[1])
        if start > end:
            raise ValueError("Range start must be <= end")
        return (start, end)
    except ValueError as e:
        print(f"Error parsing range '{range_str}': {e}", file=sys.stderr)
        sys.exit(1)


def get_issue_number(issue_id: str) -> Optional[int]:
    """Extract numeric part from issue ID like 'mtg-5' -> 5."""
    parts = issue_id.split('-')
    if len(parts) >= 2:
        try:
            return int(parts[-1])
        except ValueError:
            return None
    return None


def main():
    parser = argparse.ArgumentParser(
        description="Show readable git history of beads issues"
    )
    parser.add_argument(
        "-n",
        "--num-commits",
        type=int,
        default=10,
        help="Number of commits to look back (default: 10)",
    )
    parser.add_argument(
        "--no-color",
        action="store_true",
        help="Disable colored output",
    )
    parser.add_argument(
        "--range",
        type=str,
        help="Filter to show only issues in numeric range (e.g., --range=1-10)",
    )
    parser.add_argument(
        "--only-changed",
        action="store_true",
        help="Only show issues that were updated (not just created) during the commit window",
    )
    parser.add_argument(
        "--hide-creation",
        action="store_true",
        help="Hide the initial 'Created in' state, only show changes",
    )

    # Mutually exclusive status filters
    status_group = parser.add_mutually_exclusive_group()
    status_group.add_argument(
        "--open",
        action="store_true",
        help="Only show issues that are open in the most recent commit (HEAD)",
    )
    status_group.add_argument(
        "--closed",
        action="store_true",
        help="Only show issues that are closed in the most recent commit (HEAD)",
    )

    args = parser.parse_args()

    # Get commit history
    try:
        commits = get_commit_list(args.num_commits)
    except subprocess.CalledProcessError as e:
        print(f"Error getting commit history: {e}", file=sys.stderr)
        return 1

    if not commits:
        print("No commits found that modified .beads/issues.jsonl")
        return 0

    print(f"Analyzing {len(commits)} commits...\n")

    # Build history for each issue
    issue_history = defaultdict(list)  # issue_id -> [(commit_hash, commit_subject, issue_data, depth)]

    for idx, (commit_hash, subject) in enumerate(commits):
        issues = get_issues_at_commit(commit_hash)
        for issue_id, issue_data in issues.items():
            issue_history[issue_id].append((commit_hash, subject, issue_data, idx))

    # Sort issues by ID
    sorted_issues = sorted(issue_history.keys())

    # Apply range filter if specified
    if args.range:
        range_start, range_end = parse_range(args.range)
        filtered_issues = []
        for issue_id in sorted_issues:
            issue_num = get_issue_number(issue_id)
            if issue_num is not None and range_start <= issue_num <= range_end:
                filtered_issues.append(issue_id)
        sorted_issues = filtered_issues
        print(f"Filtered to issues {range_start}-{range_end}: {len(sorted_issues)} issues found\n")

    # Apply only-changed filter if specified
    if args.only_changed:
        filtered_issues = []
        for issue_id in sorted_issues:
            history = issue_history[issue_id]
            # Check if any consecutive versions have different content
            has_changes = False
            for i in range(len(history) - 1):
                current_text = issue_to_text(history[i][2])
                next_text = issue_to_text(history[i + 1][2])
                if current_text != next_text:
                    has_changes = True
                    break
            if has_changes:
                filtered_issues.append(issue_id)
        sorted_issues = filtered_issues
        print(f"Filtered to only changed issues: {len(sorted_issues)} issues found\n")

    # Apply status filter if specified
    if args.open or args.closed:
        filtered_issues = []
        for issue_id in sorted_issues:
            history = issue_history[issue_id]
            # Check status in most recent commit (history[0] is newest)
            if history:
                most_recent_issue = history[0][2]  # (commit_hash, subject, issue_data, depth)
                status = most_recent_issue.get("status", "unknown")

                if args.open and status == "open":
                    filtered_issues.append(issue_id)
                elif args.closed and status == "closed":
                    filtered_issues.append(issue_id)

        sorted_issues = filtered_issues
        status_filter = "open" if args.open else "closed"
        print(f"Filtered to {status_filter} issues: {len(sorted_issues)} issues found\n")

    # Show diffs for each issue
    for issue_id in sorted_issues:
        history = issue_history[issue_id]

        print("=" * 80)
        print(f"Issue {issue_id}")
        print("=" * 80)
        print()

        # Show changes in reverse chronological order (newest to oldest, like git log)
        # history is already in reverse chronological order from commit collection
        for i, (commit_hash, subject, issue_data, depth) in enumerate(history):
            current_text = issue_to_text(issue_data)

            # Format depth as HEAD or HEAD~N
            depth_str = "HEAD" if depth == 0 else f"HEAD~{depth}"
            short_hash = commit_hash[:7]

            # Last item in history is the creation
            if i == len(history) - 1:
                if not args.hide_creation:
                    print(f"Created in {short_hash}/{depth_str}: {subject}")
                    print("-" * 80)
                    print(current_text)
                    print()
            else:
                # Show diff from next (older) version to current version
                next_text = issue_to_text(history[i + 1][2])  # history[i+1][2] is next issue_data

                if current_text != next_text:
                    print(f"Changed in {short_hash}/{depth_str}: {subject}")
                    print("-" * 80)

                    diff = get_colored_diff(next_text, current_text)
                    if diff:
                        print(diff)
                    else:
                        print("(no diff generated)")
                    print()

        print()

    return 0


if __name__ == "__main__":
    sys.exit(main())
