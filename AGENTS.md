# Agent Operating Guidelines

## Codebase Scope
- **Greenfield**: Base all implementations strictly on the active specification and the current working directory tree.
- **Search**: In-tree search is conducted using `git grep` or file-read tools.
- **Source Control**: All git lifecycle operations (branching, commits, pushes) are user-managed unless following the authorized Staged Implementation Protocol below.

## Git Commits (When Authorized)
- Commit message summaries must be strictly less than 70 characters (`< 70 chars`).

## Engineering Standards
- **TDD**: Build behavior in tight, test-driven slices (`red-green-refactor`).
- **Deep Modules**: Expose simple interfaces that hide rich, robust implementation details.
- **Domain Language**: Settle domain vocabulary in `GLOSSARY.md` and hard architectural choices in `docs/adr/`.

## Task Tracking & Kaneo Protocol
- **Privacy & Dynamic Discovery**: Never hardcode personal identifiers, emails, names, or tokens in git. Discover identity and workspace dynamically via MCP (`whoami`, `list_workspaces`, `list_projects`).
- **Frontier Query**: Find tickets where status is `to-do` and all upstream blocking dependencies are satisfied (`done`).
- **Mandatory Task Claiming & Assignment**: Whenever picking up ANY task (grilling, research, prototype, task, or implementation), the worker MUST immediately claim it before doing any work:
  1. Discover current user ID dynamically via `whoami`.
  2. Assign the ticket to self via `update_task_assignee` (`userId: <id>`).
  3. Move ticket status to `in-progress` via `update_task_status`.
  *An unassigned open task is strictly considered unclaimed; assigning it prevents parallel workers from colliding on the same ticket.*

## Grilling Protocol (`wayfinder:grilling`)
When resolving a decision ticket via grilling:

### 1. In-Chat Grilling Standards
- **Real Engineering Dilemmas Only**: Pose questions solely on non-obvious architectural choices where both options have real trade-offs. Never grill the user on common sense, baseline security hygiene (e.g. Keychain vs plain text), or table-stakes standards.
- **Natural Engineering Dialogue**: Speak in clear architectural prose without robotic meta-labels. Do NOT output literal `*1-line summary*:` or `*Options*: Option A / Option B` labels in chat.
- **Proven Question Layout**:
  - `❓ **Q<number>** - **<Topic Title>**: <Context, rationale, and trade-off comparison in natural prose>`
  - Direct inquiry: `Should Rigix standardize on <X> because of <Y>?`
  - Explicit recommendation: `➡️ **Yes**, adopt <X> / standardize on <X>.`

### 2. Mandatory Ticket Comment Logging
- **Log Concluded Rounds Only**: Post a comment to the Kaneo ticket using the schema below **only** when a round of proposals concludes (when the user decides, pushes back, or approves). **Never** log comments for intermediate conversational clarifications (e.g. "where does X live?", "can you explain Y?").
- **Concise Proposal Bullets**: Each proposal must be a clean, single-sentence statement summarizing the proposed architecture/standard without stuffing `Options: Option A (...), Option B (...)` into the comment.
- **No Raw Quote Dumps**: `Response` is a section header, not a quote block; synthesize outcomes under its three subheaders (`Conclusions`, `Pushbacks`, `Explanation Requests`).
- **No Standalone Summary Comments**: Do not post an extra "Resolution Summary" comment when finished; the ticket description and the final round comment capture the settled architecture.

### Ticket Round Comment Schema
```markdown
### Grilling Round: <X> - <Topic/Focus>

#### Proposals
1. **<Proposal 1 Title>**: <One clear sentence summarizing the proposed architecture/standard>.
2. **<Proposal 2 Title>**: <One clear sentence summarizing the proposed architecture/standard>.

#### Response

##### Conclusions
- <Locked decisions and settled architectural choices>

##### Pushbacks
- <Premises or assumptions the user corrected or rejected, and why>

##### Explanation Requests
- <What the user asked to investigate, compare, or clarify for the next round>
```

## Staged Implementation Protocol (`/implement`)
When a child agent is invoked on a ticket (e.g. `RGX-10`):
1. **Scope**: Exactly one ticket per child thread. Never expand scope to sibling tickets.
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
3. No unstaged temporary files or scratch artifacts remain in the tree.
