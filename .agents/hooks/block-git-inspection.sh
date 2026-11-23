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

# Block standalone git add unless chained with git commit
if echo "$COMMAND" | grep -qE 'git[[:space:]]+add' && ! echo "$COMMAND" | grep -qE 'git[[:space:]]+commit'; then
  jq -n '{decision: "deny", reason: "BLOCKED: Standalone '\''git add'\'' is prohibited. Keep edits in the working tree. Run '\''git add'\'' only when chaining an atomic commit (e.g. git add <files> && git commit -m \"<summary>\")."}'
  exit 0
fi

# Restrict git diff: only uncommitted changes vs last commit (HEAD)
if echo "$COMMAND" | grep -qE 'git[[:space:]]+diff'; then
  DIFF_ERR=$(python3 -c "
import sys, re, os, subprocess, shlex

cmd_line = sys.argv[1]
allowed_single_refs = {'head', '@'}

parts = re.split(r'&&|\|\||;|\n|\|', cmd_line)
for part in parts:
    part = part.strip()
    if not re.search(r'\bgit\s+diff\b', part):
        continue
    try:
        tokens = shlex.split(part)
    except Exception:
        tokens = part.split()
        
    i = 0
    while i < len(tokens) - 1:
        if tokens[i] == 'git' and tokens[i+1] == 'diff':
            diff_args = tokens[i+2:]
            break
        i += 1
    else:
        continue
        
    commit_refs = []
    after_double_dash = False
    for tok in diff_args:
        if tok == '--':
            after_double_dash = True
            continue
        if after_double_dash or tok.startswith('-'):
            continue
        if tok.lower() in allowed_single_refs:
            commit_refs.append(tok)
            continue
        if os.path.exists(tok):
            continue
        res = subprocess.run(['git', 'ls-files', tok], stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)
        if res.stdout.strip():
            continue
        print(f'BLOCKED: git diff target \"{tok}\" is prohibited. Only uncommitted changes vs last commit (HEAD) may be diffed.')
        sys.exit(0)
        
    if len(commit_refs) > 1:
        print('BLOCKED: Comparing commit to commit is prohibited. Only uncommitted changes vs last commit (HEAD) may be diffed.')
        sys.exit(0)
" "$COMMAND" 2>/dev/null)

  if [ -n "$DIFF_ERR" ]; then
    jq -n --arg err "$DIFF_ERR" '{decision: "deny", reason: $err}'
    exit 0
  fi
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
