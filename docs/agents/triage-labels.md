# Triage Labels

The skills speak in terms of five canonical triage roles. In Rigix, these map directly to Kaneo task statuses and label tags.

| Canonical Role | Kaneo Status / Label | Meaning |
| :--- | :--- | :--- |
| `needs-triage` | `to-do` (`needs-triage`) | Maintainer needs to evaluate this issue |
| `needs-info` | `in-progress` (`needs-info`) | Waiting on reporter or user for clarification |
| `ready-for-agent` | `to-do` (`ready-for-agent`) | Fully specified, ready for an autonomous agent |
| `ready-for-human` | `to-do` (`ready-for-human`) | Requires human implementation or physical review |
| `wontfix` | `canceled` (`wontfix`) | Rejected or superseded decision / task |

When a skill mentions a role (e.g. "apply the AFK-ready triage label"), apply the corresponding label tag and set status accordingly.
