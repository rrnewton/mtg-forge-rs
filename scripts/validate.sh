#!/bin/bash
#
# Validation script with caching and atomic log writes
#
# This script runs comprehensive pre-commit validation including:
# - Code formatting checks
# - Linting with clippy
# - Unit tests
# - Examples
#
# Results are cached based on commit hash to avoid redundant validation.
# Only clean (committed) working copies are cached.

set -e  # Exit on error
set -o pipefail  # Propagate pipeline errors

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
LOG_DIR="validate_logs"
LATEST_SYMLINK="$LOG_DIR/validate_latest.log"

# Create log directory if it doesn't exist
mkdir -p "$LOG_DIR"

# Get current commit hash and working copy status
COMMIT_HASH=$(git rev-parse HEAD 2>/dev/null || echo "unknown")
if git diff-index --quiet HEAD -- 2>/dev/null; then
    CLEAN_STATUS="clean"
    LOG_FILE="$LOG_DIR/validate_${COMMIT_HASH}.log"
else
    CLEAN_STATUS="dirty"
    LOG_FILE="$LOG_DIR/validate_${COMMIT_HASH}_dirty.log"
fi

WIP_FILE="${LOG_FILE}.wip"

# Check if we have a cached successful validation
if [ "$CLEAN_STATUS" = "clean" ] && [ -f "$LOG_FILE" ]; then
    echo ""
    echo "==================================="
    echo -e "${GREEN}✓ Validation cache hit for commit ${COMMIT_HASH}${NC}"
    echo -e "${GREEN}✓ Validation already passed!${NC}"
    echo "==================================="
    echo ""
    echo "Log file: $LOG_FILE"
    echo ""
    exit 0
fi

# Run validation (not cached or dirty working copy)
echo "==================================="
echo "Running validation"
echo "Commit: ${COMMIT_HASH} (${CLEAN_STATUS})"
echo "Log file: ${LOG_FILE}"
echo "==================================="
echo ""

# Run validation via make validate-impl
# The actual validation logic stays in the Makefile
run_validation() {
    make validate-impl
}

# Run validation and capture output to WIP file
# The output goes both to the file and to stdout (via tee)
if run_validation 2>&1 | tee "$WIP_FILE"; then
    # Validation succeeded - atomically move WIP to final log file
    mv "$WIP_FILE" "$LOG_FILE"

    # Update latest symlink (only for successful validations)
    rm -f "$LATEST_SYMLINK"
    ln -s "$(basename "$LOG_FILE")" "$LATEST_SYMLINK"

    echo ""
    echo "==================================="
    echo -e "${GREEN}✓ All validation checks passed!${NC}"
    echo "==================================="
    echo ""
    echo "Results cached to: $LOG_FILE"
    echo "Latest: $LATEST_SYMLINK -> $(basename "$LOG_FILE")"
    echo ""
    exit 0
else
    # Validation failed - remove WIP file (don't cache failures)
    rm -f "$WIP_FILE"

    echo ""
    echo "==================================="
    echo -e "${RED}✗ Validation failed!${NC}"
    echo "==================================="
    echo ""
    exit 1
fi
