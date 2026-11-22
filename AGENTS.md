# Agent Operating Guidelines

## Codebase Scope (Greenfield)
- This repository is a clean-slate greenfield project.
- Treat the files present in the current working tree and the active task/specification as the sole primary sources of truth.
- Do not search for or inspect past commits, revisions, branches, or git logs. Design and implement solely from the fresh project specifications.

## Git Command Policy
- All git operations (commit, push, checkout, branch switching) are reserved for the human developer.
- Agents must NOT execute any git commands, with the sole exception of `git grep` for in-tree text searches.
- Under no circumstances should any other git command be executed without explicit instruction from the user in the prompt.

## Git Commit Rules
- If git commits are explicitly authorized by the user, the commit message summary line MUST be strictly less than 70 characters (`< 70 chars`).

## Engineering Standards
- Build functionality in small, test-driven slices (`red-green-refactor`).
- Keep modules deep: rich behavior hidden behind simple, intuitive interfaces.
- Settle domain vocabulary in `GLOSSARY.md` and architectural decisions in `docs/adr/`.
