#!/bin/bash
INPUT=$(cat)

# Locate .git-blocked-patterns file
PATTERNS_FILE=""
if [ -n "$CLAUDE_PROJECT_DIR" ] && [ -f "$CLAUDE_PROJECT_DIR/.git-blocked-patterns" ]; then
  PATTERNS_FILE="$CLAUDE_PROJECT_DIR/.git-blocked-patterns"
elif [ -f "./.git-blocked-patterns" ]; then
  PATTERNS_FILE="./.git-blocked-patterns"
elif [ -f "../.git-blocked-patterns" ]; then
  PATTERNS_FILE="../.git-blocked-patterns"
fi

COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null)
if [ -z "$COMMAND" ]; then
  COMMAND=$(python3 -c "import sys, json; data=json.load(sys.stdin); print(data.get('tool_input',{}).get('command',''))" <<< "$INPUT" 2>/dev/null)
fi

if [ -n "$PATTERNS_FILE" ] && [ -f "$PATTERNS_FILE" ]; then
  while IFS= read -r pattern || [ -n "$pattern" ]; do
    pattern=$(echo "$pattern" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')
    [[ -z "$pattern" || "$pattern" =~ ^# ]] && continue
    if echo "$COMMAND" | grep -qE "$pattern"; then
      echo "BLOCKED: '$COMMAND' matches prohibited pattern '$pattern' in .git-blocked-patterns. Work strictly from the current working tree and active specifications." >&2
      exit 2
    fi
  done < "$PATTERNS_FILE"
fi

exit 0
