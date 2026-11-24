# Agent Operating Guidelines

## Agent skills

### Issue tracker
Kaneo MCP protocol for issue and frontier task management. See `docs/agents/issue-tracker.md`.

### Triage labels
Standard 5-role triage vocabulary mapped to Kaneo status and tags. See `docs/agents/triage-labels.md`.

### Domain docs
Single-context repository layout (`GLOSSARY.md` and `docs/adr/`). See `docs/agents/domain.md`.

### Presentation Shell Targets
Target-specific architecture, persistence, and state guidelines: see `docs/agents/targets/`.

## Codebase Scope
- **Greenfield**: Base all implementations strictly on the active specification and the current working directory tree.
- **Task Authority**: Feature requirements, acceptance criteria, and task history reside in the Kaneo issue tracker.
- **Task Description Immutability**: Agents are STRICTLY FORBIDDEN from modifying or editing task descriptions in Kaneo at any time. Acceptance criteria checkboxes are reserved for human review. If an agent discovers an issue with the description, encounters missing prerequisites (e.g. SDK/toolchain installation), or is unable to satisfy an acceptance criterion, it MUST NOT edit the task; it must move the task status to `blocked-on-user` ("Waiting for User"), post a detailed explanatory comment on the task, and pause.

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
   - Query self-identity via `whoami` to obtain user ID.
   - Assign the task to self via `update_task_assignee`.
   - Move ticket status to `in-progress` in Kaneo via `update_task_status`.
   - Post comment via `create_task_comment`: `"Started work on branch feat/RGX-<id>."`
4. **Target Execution Policy**: For any UI or platform target, verification requires both a successful test build and a live test run on the target's runtime/emulator (e.g. Android app running on Android emulator, Apple app on iOS simulator).
5. **Eager Milestone Commenting**: Agents MUST post detailed comments on the Kaneo task at every milestone:
   - On branch start.
   - On Stage 1 Red (test suite summary + failure status, before transitioning to `blocked-on-user`).
   - On any blocker, architectural pivot, or unexpected discovery.
   - On Stage 2 Green (implementation summary + build/run verification logs).
   - On submitting for review (`in-review`).

### Stage 1: Red (Test Authoring & Approval Gate)
- Write automated tests verifying behavior against public interfaces/seams.
- Commit tests: `test(RGX-<id>): add tests for <feature>` (summary < 70 chars).
- Post comment to Kaneo ticket detailing:
  - List of all tests added, organized with **layered numbering** (e.g. `1. Feature/Module`, `1.1. Specific test case`) and a **one-line summary of each test** so the user can easily reference individual tests.
  - Current failure status (red).
- **APPROVAL GATE**: Move task status to `blocked-on-user` ("Waiting for User") in Kaneo, stop, and ask the user for explicit approval/go-ahead in chat before writing any implementation code.

### Stage 2: Green (Implementation & Completion Gate)
- Upon receiving user approval, move task status back to `in-progress` in Kaneo and implement the minimal code required to pass the tests.
- Run typechecking, build tests, and live target runs (e.g. emulator/runtime launch); ensure zero regressions and 100% pass rate.
- Commit implementation: `feat(RGX-<id>): implement <feature>` (summary < 70 chars).
- Post comment to Kaneo ticket detailing what was implemented and test run results.
- **REVIEW GATE**: Move ticket status to `in-review` in Kaneo.
  - **CRITICAL**: Agents are STRICTLY FORBIDDEN from moving any task to `done`.
  - Only the human reviewer may move a ticket to `done` after inspecting deliverables.
- Hand back to the user in chat with summary of changes, commit SHA, and test/run verification evidence.

## Completion Criteria
Before declaring a task ready for review:
1. All newly introduced behavior has corresponding automated tests.
2. The project's type checker and test suite pass with zero errors.
3. Target build and live runtime execution on emulator/simulator succeed.
4. The git working tree is completely clean with all deliverables committed.
5. The task status in Kaneo is moved to `in-review` (never `done`).
