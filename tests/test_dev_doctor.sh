#!/usr/bin/env bash
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

PASSED=0
FAILED=0

pass() {
  echo "  ✓ $1"
  PASSED=$((PASSED + 1))
}

fail() {
  echo "  ✗ $1: $2"
  FAILED=$((FAILED + 1))
}

echo "Running RGX-15 Root Toolchain & ./dev.sh doctor test suite..."

# 1. Developer Shell Entrypoint (./dev.sh)
test_1_1_dev_sh_help_shows_doctor() {
  local id="1.1" name="test_dev_sh_help_shows_doctor"
  if [ ! -x "$REPO_ROOT/dev.sh" ]; then
    fail "$id $name" "./dev.sh executable missing"
    return
  fi
  local output
  output=$("$REPO_ROOT/dev.sh" --help 2>&1 || true)
  if echo "$output" | grep -q "doctor"; then
    pass "$id $name"
  else
    fail "$id $name" "Usage output does not mention 'doctor' subcommand"
  fi
}

test_1_2_dev_sh_unknown_subcommand_fails() {
  local id="1.2" name="test_dev_sh_unknown_subcommand_fails"
  if [ ! -x "$REPO_ROOT/dev.sh" ]; then
    fail "$id $name" "./dev.sh executable missing"
    return
  fi
  local exit_code=0
  "$REPO_ROOT/dev.sh" invalid_subcmd >/dev/null 2>&1 || exit_code=$?
  if [ $exit_code -eq 0 ]; then
    fail "$id $name" "Expected non-zero exit code for unknown subcommand"
    return
  fi
  pass "$id $name"
}

# 2. Diagnostics Subcommand (./dev.sh doctor)
test_2_1_dev_sh_doctor_checks_prerequisites() {
  local id="2.1" name="test_dev_sh_doctor_checks_prerequisites"
  if [ ! -x "$REPO_ROOT/dev.sh" ]; then
    fail "$id $name" "./dev.sh executable missing"
    return
  fi
  local output exit_code=0
  output=$("$REPO_ROOT/dev.sh" doctor 2>&1) || exit_code=$?
  if [ $exit_code -ne 0 ]; then
    fail "$id $name" "Expected exit code 0, got $exit_code. Output: $output"
    return
  fi
  for tool in "rust" "cargo" "node" "pnpm" "git"; do
    if ! echo "$output" | grep -qi "$tool"; then
      fail "$id $name" "Doctor output does not mention '$tool'"
      return
    fi
  done
  if ! echo "$output" | grep -qE "✓|✔|\[ok\]|\[OK\]"; then
    fail "$id $name" "Doctor output lacks status symbols (checkmarks)"
    return
  fi
  pass "$id $name"
}

test_2_2_dev_sh_doctor_missing_tool_fails() {
  local id="2.2" name="test_dev_sh_doctor_missing_tool_fails"
  if [ ! -x "$REPO_ROOT/dev.sh" ]; then
    fail "$id $name" "./dev.sh executable missing"
    return
  fi
  local fake_bin
  fake_bin=$(mktemp -d)
  local output exit_code=0
  output=$(PATH="$fake_bin:/bin:/usr/bin" "$REPO_ROOT/dev.sh" doctor 2>&1) || exit_code=$?
  rm -rf "$fake_bin"
  if [ $exit_code -eq 0 ]; then
    fail "$id $name" "Doctor should have failed when tools are missing, but exited with 0"
    return
  fi
  if ! echo "$output" | grep -qE "✗|✘|\[fail\]|\[FAIL\]|missing|not found"; then
    fail "$id $name" "Doctor output lacks failure indicator when tools are missing"
    return
  fi
  pass "$id $name"
}

test_1_1_dev_sh_help_shows_doctor
test_1_2_dev_sh_unknown_subcommand_fails
test_2_1_dev_sh_doctor_checks_prerequisites
test_2_2_dev_sh_doctor_missing_tool_fails

echo "Test Results: $PASSED passed, $FAILED failed"
if [ $FAILED -gt 0 ]; then
  exit 1
fi
exit 0
