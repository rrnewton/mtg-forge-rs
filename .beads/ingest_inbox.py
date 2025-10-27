#!/usr/bin/env python3
"""
Ingest human-written issues from .beads/inbox/*.md

Parses markdown files with setext-style headers (underlined with = or -)
and creates beads issues for each section with priority 0 and label "human".
"""

import json
import os
import re
import subprocess
import sys
from datetime import datetime
from pathlib import Path
from typing import List, Tuple, Optional


# Colors for output
class Colors:
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    CYAN = '\033[0;36m'
    RED = '\033[0;31m'
    NC = '\033[0m'  # No Color


def check_bd_supports_no_db() -> bool:
    """Check if bd command supports --no-db option."""
    try:
        result = subprocess.run(
            ['bd', '-h'],
            capture_output=True,
            text=True,
            timeout=5
        )
        return '--no-db' in result.stdout
    except Exception:
        return False


def check_bd_database_exists() -> bool:
    """Check if beads database exists and is accessible."""
    try:
        result = subprocess.run(
            ['bd', 'list', '--limit', '1'],
            capture_output=True,
            text=True,
            timeout=5
        )
        # If we get "no beads database found" error, db doesn't exist
        if 'no beads database found' in result.stderr.lower() or \
           'no beads database found' in result.stdout.lower():
            return False
        return result.returncode == 0
    except Exception:
        return False


def init_beads_database(project_root: Path) -> bool:
    """Initialize beads database with mtg prefix."""
    try:
        print(f"{Colors.YELLOW}No beads database found. Initializing...{Colors.NC}")
        result = subprocess.run(
            ['bd', 'init', '-p', 'mtg'],
            capture_output=True,
            text=True,
            cwd=project_root,
            timeout=10
        )

        if result.returncode != 0:
            print(f"{Colors.RED}Failed to initialize beads database:{Colors.NC}")
            print(result.stderr)
            return False

        print(f"{Colors.GREEN}✓ Beads database initialized{Colors.NC}\n")
        return True
    except Exception as e:
        print(f"{Colors.RED}Error initializing beads database: {e}{Colors.NC}")
        return False


def parse_setext_markdown(content: str) -> List[Tuple[str, str]]:
    """
    Parse setext-style markdown headers and return list of (title, body) tuples.

    Setext headers use underlines:
      Title
      =====

      Body text...
    """
    sections = []
    lines = content.split('\n')

    i = 0
    while i < len(lines):
        # Look for setext header (line followed by === or ---)
        if i + 1 < len(lines):
            current_line = lines[i].strip()
            next_line = lines[i + 1].strip()

            # Check if next line is all = or all -
            if current_line and (
                (next_line and all(c == '=' for c in next_line)) or
                (next_line and all(c == '-' for c in next_line))
            ):
                # Found a header
                title = current_line
                i += 2  # Skip title and underline

                # Collect body until next header or end
                body_lines = []
                while i < len(lines):
                    # Check if we hit another header
                    if i + 1 < len(lines):
                        peek_current = lines[i].strip()
                        peek_next = lines[i + 1].strip()
                        if peek_current and (
                            (peek_next and all(c == '=' for c in peek_next)) or
                            (peek_next and all(c == '-' for c in peek_next))
                        ):
                            # Next header found, stop collecting body
                            break

                    body_lines.append(lines[i])
                    i += 1

                # Join body and strip leading/trailing whitespace
                body = '\n'.join(body_lines).strip()
                sections.append((title, body))
                continue

        i += 1

    return sections


def create_bd_issue(title: str, body: str, use_no_db: bool) -> Tuple[bool, Optional[str], str]:
    """
    Create a bd issue with the given title and body.

    Returns: (success, issue_id, message)
    """
    try:
        cmd = ['bd', 'create', title, '-p', '0', '-l', 'human', '-d', body, '--json']
        if use_no_db:
            # Insert --no-db before 'create' subcommand
            cmd = ['bd', '--no-db'] + cmd[1:]

        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=30
        )

        if result.returncode != 0:
            error_msg = result.stderr.strip() or result.stdout.strip() or "Unknown error"
            return False, None, error_msg

        # Parse JSON response to get issue ID
        try:
            response = json.loads(result.stdout)
            issue_id = response.get('id', 'unknown')
            return True, issue_id, ""
        except json.JSONDecodeError:
            # If --json isn't supported or fails, try to extract ID from output
            output = result.stdout.strip()
            # Look for patterns like "mtg-123" or "Created: mtg-123"
            match = re.search(r'\b(mtg-\d+)\b', output)
            if match:
                return True, match.group(1), ""
            return True, None, "Issue created but ID not found in output"

    except subprocess.TimeoutExpired:
        return False, None, "Command timed out"
    except Exception as e:
        return False, None, f"Exception: {str(e)}"


def process_file(file_path: Path, done_dir: Path, use_no_db: bool) -> Tuple[int, List[str]]:
    """
    Process a single markdown file from inbox.

    Returns: (number_of_issues_created, list_of_issue_ids)
    """
    print(f"{Colors.CYAN}Processing: {file_path.name}{Colors.NC}")

    # Read file content
    try:
        content = file_path.read_text(encoding='utf-8')
    except Exception as e:
        print(f"  {Colors.RED}✗ Failed to read file: {e}{Colors.NC}")
        return 0, []

    # Parse sections
    sections = parse_setext_markdown(content)

    if not sections:
        print(f"  {Colors.YELLOW}⚠ No sections found with setext-style headers{Colors.NC}")
        return 0, []

    # Create issues
    issues_created = 0
    issue_ids = []

    for title, body in sections:
        print(f"  {Colors.GREEN}→{Colors.NC} Creating issue: {title}")

        success, issue_id, error_msg = create_bd_issue(title, body, use_no_db)

        if success:
            if issue_id:
                print(f"    {Colors.GREEN}✓ Created: {issue_id}{Colors.NC}")
                issue_ids.append(issue_id)
            else:
                print(f"    {Colors.GREEN}✓ Created (ID not available){Colors.NC}")
            issues_created += 1
        else:
            print(f"    {Colors.RED}✗ Failed to create issue{Colors.NC}")
            print(f"    {Colors.RED}Error: {error_msg}{Colors.NC}")

    if issues_created > 0:
        print(f"  {Colors.GREEN}✓ Created {issues_created} issue(s) from {file_path.name}{Colors.NC}")
    else:
        print(f"  {Colors.RED}✗ No issues created from {file_path.name}{Colors.NC}")
        print(f"  {Colors.YELLOW}⚠ File left in inbox for review{Colors.NC}")
        print()
        return 0, []

    print()

    # Move processed file to done directory (only if issues were created)
    # Check if filename already has a timestamp prefix to avoid redundancy
    timestamp_pattern = re.compile(r'^\d{8}_\d{6}_')
    if timestamp_pattern.match(file_path.name):
        # Already has timestamp, don't add another
        done_file = done_dir / file_path.name
    else:
        # Add timestamp prefix
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        done_file = done_dir / f"{timestamp}_{file_path.name}"

    try:
        file_path.rename(done_file)
        print(f"  {Colors.CYAN}→{Colors.NC} Moved to: .beads/done/{done_file.name}")

        # Add to git
        subprocess.run(
            ['git', 'add', str(done_file)],
            capture_output=True,
            timeout=10
        )
        print(f"  {Colors.CYAN}→{Colors.NC} Added to git")
        print()
    except Exception as e:
        print(f"  {Colors.YELLOW}⚠ Failed to move/add file: {e}{Colors.NC}")
        print()

    return issues_created, issue_ids


def main():
    """Main entry point."""
    script_dir = Path(__file__).parent
    inbox_dir = script_dir / 'inbox'
    done_dir = script_dir / 'done'
    project_root = script_dir.parent

    # Ensure directories exist
    inbox_dir.mkdir(exist_ok=True)
    done_dir.mkdir(exist_ok=True)

    # Find markdown files in inbox
    md_files = list(inbox_dir.glob('*.md'))

    if not md_files:
        print()
        print(f"{Colors.YELLOW}No files found in .beads/inbox/{Colors.NC}")
        print("Place markdown files with section headers in .beads/inbox/ to ingest them.")
        print()
        print("Example format:")
        print("  First Issue Title")
        print("  =================")
        print("  ")
        print("  Description of the first issue here.")
        print("  ")
        print("  Second Issue Title")
        print("  ------------------")
        print("  ")
        print("  Description of the second issue.")
        print()
        return 0

    # Check for --no-db support
    use_no_db = check_bd_supports_no_db()

    # Check if database exists (if not using --no-db)
    if not use_no_db and not check_bd_database_exists():
        if not init_beads_database(project_root):
            print(f"{Colors.RED}Failed to initialize beads database. Exiting.{Colors.NC}")
            return 1

    print()
    print("===================================")
    print(f"{Colors.CYAN}Processing {len(md_files)} file(s) from inbox{Colors.NC}")
    print("===================================")
    print()

    total_issues_created = 0
    all_issue_ids = []

    # Process each file
    for md_file in sorted(md_files):
        count, ids = process_file(md_file, done_dir, use_no_db)
        total_issues_created += count
        all_issue_ids.extend(ids)

    print("===================================")
    print(f"{Colors.GREEN}✓ Ingestion complete!{Colors.NC}")
    print("===================================")
    print()
    print(f"Total issues created: {total_issues_created}")

    if all_issue_ids:
        print(f"Issue IDs: {', '.join(all_issue_ids)}")

    print()

    if total_issues_created > 0:
        print("View created issues:")
        print("  bd list --label human")
        print()

    return 0


if __name__ == '__main__':
    sys.exit(main())
