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
- **Source Control**: All git lifecycle operations (branching, commits, pushes) are user-managed unless following the authorized Staged Implementation Protocol below.

## Working Tree & Commits (When Authorized)
- **Working Tree**: Keep all edits directly in the working tree. Inspect status with `git status -s`.
- **Atomic Commits**: When explicitly requested to commit, stage and commit target files in a single step:
  `git add <files> && git commit -m "<summary>"` (< 70 chars).

## Engineering Standards
- **TDD**: Build behavior in tight, test-driven slices (`red-green-refactor`).
- **Deep Modules**: Expose simple interfaces that hide rich, robust implementation details.
- **Domain Language**: Settle domain vocabulary in `GLOSSARY.md` and hard architectural choices in `docs/adr/`.

## Grilling Protocol (`wayfinder:grilling`)
When resolving a decision ticket via grilling:

### 1. In-Chat Grilling Standards
- **Real Engineering Dilemmas Only**: Pose questions solely on non-obvious architectural choices where both options have real trade-offs. Skip common sense, baseline security hygiene (e.g. Keychain vs plain text), or table-stakes standards.
- **Natural Engineering Dialogue**: Speak in clear architectural prose, framing each inquiry around the core decision dilemma and its direct trade-offs.
- **Proven Question Layout**:
  - `❓ **Q<number>** - **<Topic Title>**: <Context, rationale, and trade-off comparison in natural prose>`
  - Direct inquiry: `Should Rigix standardize on <X> because of <Y>?`
  - Explicit recommendation: `➡️ **Yes**, adopt <X> / standardize on <X>.`

### 2. Mandatory Ticket Comment Logging
- **Thread Attribution**: The comment MUST begin with the active thread ID or link at the very top (e.g. `**Thread**: <threadId>`).
- **Log Concluded Rounds Only**: Post a comment to the Kaneo ticket using the schema below exclusively when a round of proposals concludes (upon user decision, pushback, or approval).
- **Concise Proposal Bullets**: Each proposal must be a single declarative sentence stating the proposed architecture or standard.
- **Structured Response Synthesis**: `Response` is a section header; synthesize outcomes under its three subheaders (`Conclusions`, `Pushbacks`, `Explanation Requests`).
- **Single Source of Truth**: The ticket description and the final round comment serve as the sole record of the settled architecture.

### Ticket Round Comment Schema
```markdown
**Thread**: <threadId>

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

### 3. Concluding a Grilling Session & Commit Protocol
When all questions on a grilling ticket are settled and approved by the user:
1. **Produce Deliverables**: Ensure the technical specification (`docs/specs/<topic>.md`), Architecture Decision Record (`docs/adr/000X-<topic>.md`), and any domain terms in `GLOSSARY.md` are authored.
2. **Atomic Git Commit on `main`**:
   - Foundational architecture/spec tickets commit directly to `main` (no feature branch needed for pure docs).
   - Commit message summary line MUST be strictly less than 70 characters:
     `docs(RGX-<id>): specify <topic> & record ADR (ADR-000X)`
3. **Update Kaneo Ticket & Map**:
   - Update the ticket description with the Decision Tree, Final Architecture Rationale, and Artifact References.
   - Update the Kaneo Map task (`Rigix Map`) by appending the closed ticket link and 1-line gist under `## Decisions so far`.
   - Transition ticket status to `done` via `update_task_status`.
   - Verify working tree is 100% clean.

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
