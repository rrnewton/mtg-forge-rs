#!/bin/bash

# These submods should stay pinned at the latest on their respective branches.

# forge-java -> master branch

# ./.claude_template -> mtg-rs branch

# This script switches to each subdirectory, and if the submodule is
# clean, switches to the desired branch and pulls the latest changes.
# It then switches back to the root directory and stages both the
# submodule changes.

set -e  # Exit on error
set -o pipefail  # Propagate pipeline errors

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration: submodule path -> branch mapping
declare -A SUBMODULES=(
    ["forge-java"]="master"
    [".claude_template"]="mtg-rs"
)

# Track if any updates occurred
UPDATES_MADE=false

# Update a single submodule
update_submodule() {
    local submod_path="$1"
    local target_branch="$2"

    echo ""
    echo -e "${CYAN}=== Updating submodule: ${submod_path} ===${NC}"

    # Check if submodule directory exists
    if [ ! -d "$submod_path" ]; then
        echo -e "${RED}Error: Submodule directory '$submod_path' does not exist${NC}"
        return 1
    fi

    # Enter submodule directory
    cd "$submod_path"

    # Check if directory is a git repository (handle both .git directory and file for submodules)
    if [ ! -e .git ]; then
        echo -e "${RED}Error: '$submod_path' is not a git repository${NC}"
        cd - > /dev/null
        return 1
    fi

    # Refresh the index to avoid false positives from stale stat information
    git update-index --refresh -q 2>/dev/null || true

    # Check if working directory is clean
    if ! git diff-index --quiet HEAD -- 2>/dev/null; then
        echo -e "${YELLOW}Warning: Submodule has uncommitted changes. Skipping.${NC}"
        cd - > /dev/null
        return 0
    fi

    # Get current branch
    current_branch=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "HEAD")

    # Switch to target branch if not already on it
    if [ "$current_branch" != "$target_branch" ]; then
        echo -e "${CYAN}Switching from '$current_branch' to '$target_branch'...${NC}"
        if ! git checkout "$target_branch" 2>/dev/null; then
            echo -e "${RED}Error: Failed to checkout branch '$target_branch'${NC}"
            cd - > /dev/null
            return 1
        fi
    fi

    # Store current commit hash
    old_commit=$(git rev-parse HEAD)

    # Pull latest changes
    echo -e "${CYAN}Pulling latest changes from remote...${NC}"
    if ! git pull --ff-only origin "$target_branch" 2>/dev/null; then
        echo -e "${YELLOW}Warning: Failed to pull (possibly no remote or network issue)${NC}"
        cd - > /dev/null
        return 0
    fi

    # Get new commit hash
    new_commit=$(git rev-parse HEAD)

    # Check if update occurred
    if [ "$old_commit" != "$new_commit" ]; then
        echo -e "${GREEN}✓ Updated from ${old_commit:0:8} to ${new_commit:0:8}${NC}"
        UPDATES_MADE=true
    else
        echo -e "${GREEN}✓ Already up to date at ${old_commit:0:8}${NC}"
    fi

    # Return to root directory
    cd - > /dev/null
}

# Main script
echo "==================================="
echo "Updating Submodules"
echo "==================================="

# Store root directory
ROOT_DIR=$(pwd)

# Update each submodule
for submod_path in "${!SUBMODULES[@]}"; do
    target_branch="${SUBMODULES[$submod_path]}"
    update_submodule "$submod_path" "$target_branch" || {
        echo -e "${RED}Failed to update $submod_path${NC}"
        exit 1
    }
    # Make sure we're back in root
    cd "$ROOT_DIR"
done

echo ""
echo "==================================="

# Stage submodule changes if updates were made
if [ "$UPDATES_MADE" = true ]; then
    echo -e "${CYAN}Staging submodule updates...${NC}"
    for submod_path in "${!SUBMODULES[@]}"; do
        git add "$submod_path"
    done
    echo -e "${GREEN}✓ Submodule changes staged${NC}"
    echo ""
    echo -e "${YELLOW}Note: Run 'git status' to see staged changes${NC}"
    echo -e "${YELLOW}      Run 'git commit' to commit the updates${NC}"
else
    echo -e "${GREEN}✓ All submodules already up to date${NC}"
fi

echo "==================================="
echo ""
