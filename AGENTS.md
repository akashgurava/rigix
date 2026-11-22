# Agent Operating Guidelines

## Codebase Scope
- **Greenfield**: Base all implementations strictly on the active specification and the current working directory tree.
- **Search**: In-tree search is conducted using `git grep` or file-read tools.
- **Source Control**: All git lifecycle operations (branching, commits, pushes) are user-managed. Do not commit unless explicitly instructed.

## Git Commits (When Authorized)
- Commit message summaries must be strictly less than 70 characters (`< 70 chars`).

## Engineering Standards
- **TDD**: Build behavior in tight, test-driven slices (`red-green-refactor`).
- **Deep Modules**: Expose simple interfaces that hide rich, robust implementation details.
- **Domain Language**: Settle domain vocabulary in `GLOSSARY.md` and hard architectural choices in `docs/adr/`.

## Completion Criteria
Before declaring a task done:
1. All newly introduced behavior has corresponding automated tests.
2. The project's type checker and test suite pass with zero errors.
3. No unstaged temporary files or scratch artifacts remain in the tree.
