#!/bin/bash
INPUT=$(cat)

# Locate .git-blocked-patterns file
PATTERNS_FILE=""
if [ -f "./.git-blocked-patterns" ]; then
  PATTERNS_FILE="./.git-blocked-patterns"
elif [ -f "../.git-blocked-patterns" ]; then
  PATTERNS_FILE="../.git-blocked-patterns"
elif [ -n "$CLAUDE_PROJECT_DIR" ] && [ -f "$CLAUDE_PROJECT_DIR/.git-blocked-patterns" ]; then
  PATTERNS_FILE="$CLAUDE_PROJECT_DIR/.git-blocked-patterns"
fi

COMMAND=$(echo "$INPUT" | jq -r '.toolCall.args.CommandLine // empty' 2>/dev/null)
if [ -z "$COMMAND" ]; then
  COMMAND=$(python3 -c "import sys, json; data=json.load(sys.stdin); print(data.get('toolCall',{}).get('args',{}).get('CommandLine',''))" <<< "$INPUT" 2>/dev/null)
fi

if [ -n "$PATTERNS_FILE" ] && [ -f "$PATTERNS_FILE" ]; then
  while IFS= read -r pattern || [ -n "$pattern" ]; do
    pattern=$(echo "$pattern" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')
    [[ -z "$pattern" || "$pattern" =~ ^# ]] && continue
    if echo "$COMMAND" | grep -qE "$pattern"; then
      jq -n --arg pattern "$pattern" \
        '{decision: "deny", reason: ("BLOCKED: Command matches prohibited pattern '\''" + $pattern + "'\'' in .git-blocked-patterns. Work strictly from the current working tree and active specifications.")}'
      exit 0
    fi
  done < "$PATTERNS_FILE"
fi

echo '{"decision": "allow"}'
exit 0
