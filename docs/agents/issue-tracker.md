# Issue Tracker: Kaneo MCP Protocol

Rigix uses **Kaneo** via Model Context Protocol (MCP) as its issue tracking system. Issues and decision tickets are tracked as Kaneo tasks.

## Protocol & Discovery

- **Identity & Workspace**: Discover identity and active workspace dynamically via MCP (`whoami`, `list_workspaces`, `list_projects`). Confine git commits strictly to repository assets.
- **Frontier Query**: Find actionable work by querying tasks where status is `to-do` and all upstream blocking task relations are satisfied (`done`).
- **Claiming Tasks**: Claim any task before starting work by moving its status to `in-progress` via `update_task_status`.
- **Status Updates**: On completion, update the task description with final rationale/deliverables and move status to `done`.
