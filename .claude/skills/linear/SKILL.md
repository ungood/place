---
name: linear
description: Conventions for using the Linear MCP server — issue body template, state transitions, label/milestone use, and query patterns. Invoke when creating, updating, querying, or commenting on Linear issues. Project-specific identifiers (team, project, milestones, label vocabulary) are not in this skill — they live in the project's CLAUDE.md.
---

# linear

How to work with Linear via the `mcp__linear-server__*` tools. This skill is project-agnostic: every concrete identifier (team, project, milestones, label vocabulary) lives in the project's `CLAUDE.md`. Read that first.

## Issue body template

Every issue uses these markdown sections. A project may extend the template in its `CLAUDE.md`.

- **Story** — a concrete user (or operator) doing a concrete thing. One short paragraph.
- **Acceptance** — bullet list, each bullet directly testable in Gherkin.
- **Spec** — path(s) to `.feature` file(s), or `TBD` until the spec is authored.
- **Notes** — constraints, references, open questions.

## State, not labels, for phase

Use Linear's workflow state to convey *where in the loop the issue is*:

- **Backlog** — not yet groomed.
- **In Progress** — Grooming has begun, or any later phase up to PR open.
- **In Review** — PR is open.
- **Done** — PR merged.
- **Cancelled** — explicitly aborted; requires human direction.

Labels are reserved for issue *type* (e.g., `Feature`, `Epic`, `Bug`, `Refactor`, `Chore`). Each project declares its label vocabulary in `CLAUDE.md`.

## Cross-linking with code

- When a `.feature` file is created or moved, update the issue's **Spec** field in the same change. Use `save_issue` with the issue's `id` and only the changed fields.
- Mention the issue key (e.g., `ABC-123`) in commit messages and PR titles/bodies. **Never** in source comments or `.feature` files — those rot when the issue is renamed or the project moves.

## MCP tool usage

- `list_issues` — find work. Filter by `project`, `milestone`, `state`, `assignee: "me"`. Set `includeArchived: false` unless you really want archived items.
- `save_issue` — creates if no `id`, updates otherwise. `team` and `title` are required when creating; `labels` and `milestone` accept names *or* IDs.
- `save_comment` — for status notes the human/agents should see, including phase-handoff signals between PM and PE agents.
- `list_milestones` / `save_milestone` — milestones are project-scoped; pass the project name or ID.
- `list_issue_labels` / `create_issue_label` — workspace- or team-scoped. Diff before creating to avoid duplicates and casing drift.

### MCP gotcha

Pass real newlines in `description` and comment bodies. Never literal `\n` — the linear-server MCP renders them verbatim.

## Common queries

- **My next issue**: `list_issues` with `assignee: "me"` and `state: "in_progress"`; if none, `assignee: "me"` and `state: "backlog"`; if none, the milestone's top-priority backlog issue.
- **Has this been done before?**: `list_issues` with `query: "<title-fragment>"` before creating a new issue.
- **What's blocking phase X of milestone Y?**: `list_issues` with the milestone, filtered to non-Done states.
