# Issue Tracker: Kaneo MCP Protocol

Rigix uses **Kaneo** via Model Context Protocol (MCP) as its issue tracking system. Issues and decision tickets are tracked as Kaneo tasks.

## Protocol & Discovery

- **Identity & Workspace**: Discover current user identity and active workspace dynamically via MCP (`whoami`, `list_workspaces`, `list_projects`). Use dynamic authentication for all operations.
- **Frontier Query**: Find actionable work by querying tasks where status is `to-do` and all upstream blocking task relations are satisfied (`done`).
- **Claiming Tasks**: Before starting any task (grilling, research, implementation, or review):
  1. Discover user ID via `whoami`.
  2. Assign the task to self via `update_task_assignee`.
  3. Set status to `in-progress` via `update_task_status`.
- **Status Updates**: On completion, update task description with final rationale/deliverables and move status to `done`.
