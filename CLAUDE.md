# place

`place` is a Rust CLI for managing agent-friendly, multi-repo "workplaces." It generates and maintains a workplace repo containing shared mise config, a VS Code workspace, and a Claude config directory — so Claude Code on the web (https://code.claude.com) can be productive across related repos with a single bootstrap step.

## Documentation philosophy

CLAUDE.md is for **agent-only** context: paths, identifiers, conventions an agent needs to act on this repo. Human-facing documentation lives in its natural home — Linear for issue conventions, `plans/` for design decisions, README/CONTRIBUTING for contributor docs — and CLAUDE.md *references* those places rather than duplicating their content.

## Working in this repo

Three skills drive how work happens here. Read each when its trigger applies; **always** invoke `workflow` when starting a Linear issue.

- `linear` — conventions for the Linear MCP (state transitions, query patterns, label/milestone use).
- `cucumber` — conventions for `.feature` files and cucumber-rs step definitions.
- `workflow` — the PM/PE/SWE spec-driven loop: Grooming → Spec → Plan → Implement.

## Linear binding

- Team: **One True Name** (`4298c63b-c726-415a-85d0-383e875f818e`, key `ONE`)
- Project: **Software Factory** (`fa52409f-b878-4f89-b34e-54c8907dc5e5`)
- Active milestones: `Bootstrap`, `place: MVP`
- Labels: configured in Linear; query with `list_issue_labels`.

## Repo paths

- `features/` — Gherkin `.feature` files (one per top-level capability; kebab-case filenames).
- `tests/cucumber.rs` — single cucumber-rs runner binary.
- `tests/steps/` — step definitions, grouped by area.
- `plans/<issue-key>.md` — Phase-3 technical plans; durable records of decisions.
- `.claude/skills/` — the three project-scoped skills above.

## Conventions specific to this repo

- Issue keys (`ONE-123`) appear in commit messages and PR descriptions only — never in source comments or `.feature` files.
- `@wip` is the only enforced Gherkin tag; CI excludes it. Other tags will be declared here as we adopt them.
- The Cargo project, CI, and cucumber runner are not yet scaffolded — they will be created the first time `place: MVP` issue 1 (`place init`) is worked.
