#!/bin/bash
#
# Ingest human-written issues from .beads/inbox/*.md
#
# Parses markdown files with setext-style headers (underlined with = or -)
# and creates beads issues for each section with priority 0 and label "human".

set -e  # Exit on error
set -o pipefail
set -u  # Exit on undefined variable

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INBOX_DIR="$SCRIPT_DIR/inbox"
DONE_DIR="$SCRIPT_DIR/done"

# Ensure directories exist
mkdir -p "$INBOX_DIR" "$DONE_DIR"

# Count files to process
FILE_COUNT=$(find "$INBOX_DIR" -name "*.md" -type f 2>/dev/null | wc -l)

if [ "$FILE_COUNT" -eq 0 ]; then
    echo ""
    echo -e "${YELLOW}No files found in .beads/inbox/${NC}"
    echo "Place markdown files with section headers in .beads/inbox/ to ingest them."
    echo ""
    echo "Example format:"
    echo "  First Issue Title"
    echo "  ================="
    echo "  "
    echo "  Description of the first issue here."
    echo "  "
    echo "  Second Issue Title"
    echo "  ------------------"
    echo "  "
    echo "  Description of the second issue."
    echo ""
    exit 0
fi

echo ""
echo "==================================="
echo -e "${CYAN}Processing $FILE_COUNT file(s) from inbox${NC}"
echo "==================================="
echo ""

TOTAL_ISSUES_CREATED=0

# Process each .md file in inbox
for FILE in "$INBOX_DIR"/*.md; do
    [ -e "$FILE" ] || continue  # Skip if no files match

    FILENAME=$(basename "$FILE")
    echo -e "${CYAN}Processing: $FILENAME${NC}"

    # Parse the file and extract sections
    # We'll use awk to split by setext-style headers (= or - underlines)

    ISSUES_IN_FILE=0

    # Create a temporary file to store parsed sections
    TEMP_FILE=$(mktemp)

    # AWK script to parse setext-style markdown headers
    # Uses buffering to prevent next section title from leaking into current body
    awk '
    BEGIN {
        in_section = 0
        buffered_line = ""
        has_buffered = 0
    }

    # Header underline (=== or ---)
    /^=+$/ || /^-+$/ {
        # Close previous section if exists
        if (in_section) {
            # Do not print the buffered line - it is the next title
            print "SECTION_END"
        }

        # Start new section
        if (prev_line != "") {
            print "SECTION_START"
            print prev_line  # This is the title
            print "BODY_START"
            in_section = 1
            has_buffered = 0
            buffered_line = ""
        }
        next
    }

    # Regular line
    {
        if (in_section) {
            # Print the previously buffered line (if any)
            if (has_buffered) {
                print buffered_line
            }
            # Buffer current line
            buffered_line = $0
            has_buffered = 1
        }
        prev_line = $0
    }

    END {
        # Print final buffered line and close section
        if (in_section) {
            if (has_buffered) {
                print buffered_line
            }
            print "SECTION_END"
        }
    }
    ' "$FILE" > "$TEMP_FILE"

    # Read the parsed sections and create issues
    CURRENT_TITLE=""
    CURRENT_BODY=""
    IN_BODY=0

    while IFS= read -r line; do
        if [[ "$line" == "SECTION_START" ]]; then
            CURRENT_TITLE=""
            CURRENT_BODY=""
            IN_BODY=0
        elif [[ "$line" == "BODY_START" ]]; then
            IN_BODY=1
        elif [[ "$line" == "SECTION_END" ]]; then
            # Create the issue
            if [[ -n "$CURRENT_TITLE" ]]; then
                echo -e "  ${GREEN}→${NC} Creating issue: $CURRENT_TITLE"

                # Call bd create with human label and priority 0
                # Use heredoc for body to handle multiline content
                RESULT=$(bd create "$CURRENT_TITLE" \
                    -p 0 \
                    -l human \
                    -d "$CURRENT_BODY" \
                    --json 2>&1)

                if [ $? -eq 0 ]; then
                    ISSUE_ID=$(echo "$RESULT" | jq -r '.id' 2>/dev/null || echo "unknown")
                    echo -e "    ${GREEN}✓${NC} Created: $ISSUE_ID"
                    ((ISSUES_IN_FILE++))
                    ((TOTAL_ISSUES_CREATED++))
                else
                    echo -e "    ${RED}✗${NC} Failed to create issue"
                    echo "$RESULT"
                fi
            fi
        elif [[ "$line" =~ ^TOTAL_SECTIONS: ]]; then
            # Skip metadata line
            :
        elif [ $IN_BODY -eq 1 ]; then
            # Accumulate body content
            if [ -z "$CURRENT_BODY" ]; then
                CURRENT_BODY="$line"
            else
                CURRENT_BODY="$CURRENT_BODY"$'\n'"$line"
            fi
        else
            # This is the title line
            CURRENT_TITLE="$line"
        fi
    done < "$TEMP_FILE"

    rm -f "$TEMP_FILE"

    echo -e "  ${GREEN}✓${NC} Created $ISSUES_IN_FILE issue(s) from $FILENAME"
    echo ""

    # Move processed file to done directory
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)
    DONE_FILE="$DONE_DIR/${TIMESTAMP}_${FILENAME}"
    mv "$FILE" "$DONE_FILE"

    echo -e "  ${CYAN}→${NC} Moved to: .beads/done/$(basename "$DONE_FILE")"

    # Add to git
    git add "$DONE_FILE"
    echo -e "  ${CYAN}→${NC} Added to git"
    echo ""
done

echo "==================================="
echo -e "${GREEN}✓ Ingestion complete!${NC}"
echo "==================================="
echo ""
echo "Total issues created: $TOTAL_ISSUES_CREATED"
echo ""

if [ "$TOTAL_ISSUES_CREATED" -gt 0 ]; then
    echo "View created issues:"
    echo "  bd list --label human"
    echo ""
fi
