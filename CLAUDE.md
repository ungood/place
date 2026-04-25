# place

`place` is a Rust CLI for managing agent-friendly, multi-repo "workplaces." It generates and maintains a workplace repo containing shared mise config, a VS Code workspace, and a Claude config directory — so Claude Code on the web (https://code.claude.com) can be productive across related repos with a single bootstrap step.

## Working in this repo

Three skills drive how work happens here. Read each when its trigger applies; **always** invoke `workflow` when starting a Linear issue.

- `linear` — conventions for the Linear MCP (issue templates, state transitions, query patterns).
- `cucumber` — conventions for `.feature` files and cucumber-rs step definitions.
- `workflow` — the PM/PE spec-driven loop: Grooming → Spec ∥ Plan → Human → Implement.

## Linear binding

- Team: **One True Name** (`4298c63b-c726-415a-85d0-383e875f818e`, key `ONE`)
- Project: **Software Factory** (`fa52409f-b878-4f89-b34e-54c8907dc5e5`)
- Active milestones: `Bootstrap`, `place: MVP`
- Label vocabulary used here: `Feature`, `Epic`, `Bug`, `Refactor`, `Chore` (capitalized to match the existing workspace labels)

## Repo paths

- `features/` — Gherkin `.feature` files (one per top-level capability; kebab-case filenames).
- `tests/cucumber.rs` — single cucumber-rs runner binary.
- `tests/steps/` — step definitions, grouped by area.
- `.claude/plans/<issue-key>.md` — Phase-2 technical plans for non-trivial work.
- `.claude/skills/` — the three project-scoped skills above.

## Conventions specific to this repo

- Issue keys (`ONE-123`) appear in commit messages and PR descriptions only — never in source comments or `.feature` files.
- `@wip` is the only enforced Gherkin tag; CI excludes it. Other tags will be declared here as we adopt them.
- The Cargo project, CI, and cucumber runner are not yet scaffolded — they will be created the first time `place: MVP` issue 1 (`place init`) is worked.
