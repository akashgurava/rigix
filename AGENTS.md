# Agent Operating Guidelines

## Agent skills

### Issue tracker
Kaneo MCP protocol for issue and frontier task management. See `docs/agents/issue-tracker.md`.

### Triage labels
Standard 5-role triage vocabulary mapped to Kaneo status and tags. See `docs/agents/triage-labels.md`.

### Domain docs
Single-context repository layout (`GLOSSARY.md` and `docs/adr/`). See `docs/agents/domain.md`.

## Codebase Scope
- **Greenfield**: Base all implementations strictly on the active specification and the current working directory tree.
- **Task Authority**: Feature requirements, acceptance criteria, and task history reside in the Kaneo issue tracker.

## Source Control Boundary
- **Git Command Policy**: All `git` commands are forbidden by default.
  - Allowed autonomously: `git grep` (code search only).
  - Allowed on explicit user instruction: `git add` and `git commit`.
  - All other git operations (including `status`, `diff`, `log`, `branch`, `push`) remain strictly off-limits.

## Working Tree & Commits (When Authorized)
- **Working Tree**: Keep all edits directly in the working tree.
- **Atomic Commits**: When explicitly requested to commit, stage and commit target files in a single step:
  `git add <files> && git commit -m "<summary>"` (< 70 chars).

## Engineering Standards
- **TDD**: Build behavior in tight, test-driven slices (`red-green-refactor`).
- **Deep Modules**: Expose simple interfaces that hide rich, robust implementation details.
- **Domain Language**: Settle domain vocabulary in `GLOSSARY.md` and hard architectural choices in `docs/adr/`.


## Staged Implementation Protocol (`/implement`)
When a child agent is invoked on a ticket (e.g. `RGX-<id>`):
1. **Scope**: Confine work strictly to the assigned ticket.
2. **Branching**: Create and switch to a dedicated branch: `feat/RGX-<id>-<slug>`.
3. **Claim & Initialize**:
   - Move ticket status to `in-progress` in Kaneo.
   - Post comment via `create_task_comment`: `"Started work on branch feat/RGX-<id>."`

### Stage 1: Red (Test Authoring & Approval Gate)
- Write automated tests verifying behavior against public interfaces/seams.
- Commit tests: `test(RGX-<id>): add tests for <feature>` (summary < 70 chars).
- Post comment to Kaneo ticket detailing:
  - List of all tests added, with a **one-line summary of each test**.
  - Current failure status (red).
- **APPROVAL GATE**: Stop and ask the user for explicit approval/go-ahead in chat before writing any implementation code.

### Stage 2: Green (Implementation & Completion Gate)
- Upon receiving user approval, implement the minimal code required to pass the tests.
- Run typechecking and tests; ensure zero regressions and 100% pass rate.
- Commit implementation: `feat(RGX-<id>): implement <feature>` (summary < 70 chars).
- Post comment to Kaneo ticket detailing what was implemented and test run results.
- Move ticket status to `done`.

## Completion Criteria
Before declaring a task done:
1. All newly introduced behavior has corresponding automated tests.
2. The project's type checker and test suite pass with zero errors.
3. The git working tree is completely clean with all deliverables committed.
