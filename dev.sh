#!/usr/bin/env bash
set -euo pipefail

# Color support
if [ -t 1 ]; then
  GREEN='\033[0;32m'
  RED='\033[0;31m'
  BOLD='\033[1m'
  NC='\033[0m'
else
  GREEN=''
  RED=''
  BOLD=''
  NC=''
fi

CHECK_MARK="${GREEN}✓${NC}"
CROSS_MARK="${RED}✗${NC}"

usage() {
  cat <<EOF
Rigix Development CLI

Usage:
  ./dev.sh <command> [options]

Commands:
  doctor    Inspect local machine prerequisites (Rust, Cargo, Node, pnpm, Git)
  help      Display this help message

Options:
  -h, --help    Show help
EOF
}

doctor() {
  printf "%bChecking local machine prerequisites...%b\n\n" "${BOLD}" "${NC}"
  local errors=0

  # Check Git
  if command -v git >/dev/null 2>&1; then
    local git_ver
    git_ver=$(git --version 2>&1 || true)
    printf "  %b Git: %s\n" "${CHECK_MARK}" "${git_ver}"
  else
    printf "  %b Git: missing (git command not found)\n" "${CROSS_MARK}"
    errors=$((errors + 1))
  fi

  # Check Rust & Cargo
  if command -v rustc >/dev/null 2>&1; then
    local rust_ver
    rust_ver=$(rustc --version 2>&1 || true)
    printf "  %b Rust: %s\n" "${CHECK_MARK}" "${rust_ver}"
  else
    printf "  %b Rust: missing (rustc command not found)\n" "${CROSS_MARK}"
    errors=$((errors + 1))
  fi

  if command -v cargo >/dev/null 2>&1; then
    local cargo_ver
    cargo_ver=$(cargo --version 2>&1 || true)
    printf "  %b Cargo: %s\n" "${CHECK_MARK}" "${cargo_ver}"
  else
    printf "  %b Cargo: missing (cargo command not found)\n" "${CROSS_MARK}"
    errors=$((errors + 1))
  fi

  # Check Node & pnpm
  if command -v node >/dev/null 2>&1; then
    local node_ver
    node_ver=$(node --version 2>&1 || true)
    printf "  %b Node: %s\n" "${CHECK_MARK}" "${node_ver}"
  else
    printf "  %b Node: missing (node command not found)\n" "${CROSS_MARK}"
    errors=$((errors + 1))
  fi

  if command -v pnpm >/dev/null 2>&1; then
    local pnpm_ver
    pnpm_ver=$(pnpm --version 2>&1 || true)
    printf "  %b pnpm: %s\n" "${CHECK_MARK}" "${pnpm_ver}"
  else
    printf "  %b pnpm: missing (pnpm command not found)\n" "${CROSS_MARK}"
    errors=$((errors + 1))
  fi

  printf "\n"
  if [ $errors -eq 0 ]; then
    printf "%bAll prerequisites satisfied.%b\n" "${GREEN}${BOLD}" "${NC}"
    return 0
  else
    printf "%b%d prerequisite(s) missing. Please install required dependencies.%b\n" "${RED}${BOLD}" "$errors" "${NC}"
    return 1
  fi
}

main() {
  local cmd="${1:-}"

  case "$cmd" in
    doctor)
      doctor
      ;;
    help|--help|-h)
      usage
      exit 0
      ;;
    "")
      usage
      exit 1
      ;;
    *)
      echo "Error: Unknown command '$cmd'. Run './dev.sh help' for usage." >&2
      exit 1
      ;;
  esac
}

main "$@"
