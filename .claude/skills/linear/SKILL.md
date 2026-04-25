---
name: linear
description: Conventions for using the Linear MCP server — state transitions, label and milestone use, and query patterns. Invoke when creating, updating, querying, or commenting on Linear issues.
---

# linear

How to work with Linear via the `mcp__linear-server__*` tools. This skill is project-agnostic: every concrete identifier (team, project, milestones, label vocabulary) lives in the project's `CLAUDE.md`. Read that first.

## Issue structure

Issue body structure comes from Linear's native issue templates (configured in Linear's workspace settings). This skill does not duplicate template content — start a new issue from the template Linear provides and fill in the prompted sections.

## State, not labels, for phase

Use Linear's workflow state to convey *where in the loop the issue is*:

- **Backlog** — not yet groomed.
- **Planned** — groomed and human-approved; Spec / Plan / human review happening here.
- **In Progress** — implementation underway, up to PR open.
- **In Review** — PR is open.
- **Done** — PR merged.
- **Cancelled** — explicitly aborted; requires human direction.

Labels are reserved for issue *type* (e.g., `Feature`, `Bug`, `Chore`). Label vocabulary is configured in Linear itself — query `list_issue_labels` to discover what's current.

## Cross-linking with code

Mention the issue key (e.g., `ABC-123`) in commit messages and PR titles/bodies. **Never** in source comments — they rot when the issue is renamed or the project moves.

## MCP tool usage

- `list_issues` — find work. Filter by `project`, `milestone`, `state`, `assignee: "me"`. Set `includeArchived: false` unless you really want archived items.
- `save_issue` — creates if no `id`, updates otherwise. `team` and `title` are required when creating; `labels` and `milestone` accept names *or* IDs.
- `save_comment` — for status notes the human/agents should see, including phase-handoff signals between PM, PE, and SWE agents.
- `list_milestones` / `save_milestone` — milestones are project-scoped; pass the project name or ID.
- `list_issue_labels` / `create_issue_label` — workspace- or team-scoped. Diff before creating to avoid duplicates and casing drift.

### MCP gotcha

Pass real newlines in `description` and comment bodies. Never literal `\n` — the linear-server MCP renders them verbatim.

## Common queries

- **My next issue**: `list_issues` with `assignee: "me"` and `state: "in_progress"`; if none, `assignee: "me"` and `state: "planned"`; if none, `state: "backlog"`; if none, the milestone's top-priority backlog issue.
- **Has this been done before?**: `list_issues` with `query: "<title-fragment>"` before creating a new issue.
- **What's blocking phase X of milestone Y?**: `list_issues` with the milestone, filtered to non-Done states.
