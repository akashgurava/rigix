#!/bin/bash
INPUT=$(cat)

# Extract command from Claude Code PreToolUse payload
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null)
if [ -z "$COMMAND" ]; then
  COMMAND=$(python3 -c "import sys, json; data=json.load(sys.stdin); print(data.get('tool_input',{}).get('command',''))" <<< "$INPUT" 2>/dev/null)
fi

BLOCKED_PATTERNS=(
  "git[[:space:]]+log"
  "git[[:space:]]+show"
  "git[[:space:]]+blame"
  "git[[:space:]]+reflog"
  "git[[:space:]]+diff"
  "git[[:space:]]+rev-list"
  "git[[:space:]]+whatchanged"
  "git[[:space:]]+push"
  "git[[:space:]]+reset[[:space:]]+--hard"
  "git[[:space:]]+clean[[:space:]]+-f"
  "git[[:space:]]+branch[[:space:]]+-D"
)

for pattern in "${BLOCKED_PATTERNS[@]}"; do
  if echo "$COMMAND" | grep -qE "$pattern"; then
    echo "BLOCKED: '$COMMAND' matches prohibited git pattern '$pattern'. History inspection and dangerous git commands are disabled in this workspace. Work strictly from the current working tree and active specifications." >&2
    exit 2
  fi
done

exit 0
