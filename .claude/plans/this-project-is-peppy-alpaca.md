# Bootstrap: spec-driven workflow for `place`

## Context

`place` will be a Rust CLI (named after, and built around, mise) for managing agent-friendly, multi-repo "workplaces." Its first job is to make Claude Code on the web (https://code.claude.com) productive on related repos by generating a workplace repo containing shared mise config, a VS Code workspace, and a Claude config directory.

Before writing any tool code, we want a **spec-driven workflow** in place: Linear holds the user stories, repo `.feature` files (cucumber-rs) hold the executable Gherkin, and Claude Code skills tell future agent sessions how to operate the loop. This plan only bootstraps that workflow — Cargo/cucumber-rs scaffolding is deferred to the first MVP issue, so the scaffold itself is exercised by the workflow it enables.

### Decisions captured from the conversation

- **Linear scope**: existing project **Software Factory** (team `One True Name` / `ONE`); `place` lives there as a milestone, not a separate project.
- **Issue ↔ spec granularity**: 1 Linear issue ≈ 1 `.feature` file *initially*. The workflow must allow issues to span multiple features later (epics, refactors, bug fixes).
- **Spec source of truth**: Gherkin lives in repo `.feature` files. Linear issues hold prose + a link to the feature file path.
- **Skills are project-agnostic**: each skill describes *how to use a tool/process*, never repo-specific identifiers. Project-specific config (team IDs, milestone names, label vocab, repo paths) lives in `CLAUDE.md`.
- **No issue-tracker references in code**: Linear keys/URLs go in commit messages and PR descriptions, never in source comments or `.feature` files.
- **Workflow shape mimics PM/PE collaboration at a real product team**:
  - Two agent roles: a **PM agent** (owns the user-facing spec) and a **PE agent** (owns the technical plan).
  - Phases run as: **Grooming → (Spec ∥ Plan) → Human review → Implement**.
  - Spec and Plan are **parallel** after Grooming. The PM writes the spec; the PE reviews it. The PE writes the plan; the PM reviews it. Reviewers decide whether each artifact is *ready for human review* — they do not approve on the human's behalf.
  - Once both cross-reviews are clean, the human reviews **spec and plan together** before Implement starts.
- **Bootstrap deliverables**: (a) Linear seed (labels, two milestones, initial issues) and (b) three project-agnostic Claude Code skills — `linear`, `cucumber`, `workflow` — plus a project-specific `CLAUDE.md`.

## Deliverables

### 1. Repo files

```
/CLAUDE.md                              # project-specific config: Linear IDs, milestone names,
                                        # label vocab, repo paths; pointers to the three skills
/.claude/skills/linear/SKILL.md         # generic Linear-via-MCP conventions
/.claude/skills/cucumber/SKILL.md       # generic cucumber-rs / Gherkin conventions
/.claude/skills/workflow/SKILL.md       # generic spec-driven loop with review gates
```

#### `linear` skill — project-agnostic

Tool-and-process conventions for using the `linear-server` MCP, applicable to any project. **No** team/project/milestone IDs.

- **Issue body template** (markdown): `Story` (user-facing prose), `Acceptance` (testable bullets), `Spec` (path to `.feature` file or `TBD`), `Notes`. Each project's `CLAUDE.md` may extend the template.
- **Phase via Linear workflow state, not labels**: Backlog → In Progress (on Grooming start) → In Review (on PR open) → Done (on merge). Labels are reserved for issue *type* (project chooses its own vocab in `CLAUDE.md`).
- **Cross-linking**: when a `.feature` file is created or moved, update the issue's `Spec` field in the same change. Mention the issue in commits/PRs (`ONE-123`), never in source comments.
- **MCP tool pointers**: when to use `list_issues`, `save_issue`, `save_comment`, `list_milestones`, `save_milestone`, `list_issue_labels`, `create_issue_label`. Always pass real newlines, never `\n` literals (per the Linear MCP server's instructions).
- **Querying patterns**: how to find "next issue in milestone X" / "open issues assigned to me" without project-specific IDs.

#### `cucumber` skill — project-agnostic

Conventions for writing Gherkin and step definitions in cucumber-rs Rust projects.

- **File layout**:
  - `features/<area>.feature` — one feature file per top-level capability; filename is kebab-case.
  - `tests/cucumber.rs` — single cucumber-rs runner binary.
  - `tests/steps/` — step definitions, grouped by area.
- **Gherkin style**: `Feature:` line is the user-facing capability name; scenarios use concrete example values rather than placeholders; one declarative `Given/When/Then` chain per scenario.
- **Tags**: `@wip` for committed-but-skipped scenarios (excluded by default in CI); other tags are project-specific (declared in `CLAUDE.md`).
- **Run command**: `cargo test --test cucumber`.
- **No issue-tracker references** in `.feature` files. Traceability lives in Linear's `Spec` field and commit/PR text.
- **Multi-feature issues**: when one issue spans several `.feature` files, list all paths in the issue's `Spec` section; the workflow loop runs per file.

#### `workflow` skill — project-agnostic

The spec-driven loop, written so any project combining Linear + cucumber-rs can adopt it. Models a real product team: a **PM agent** owns user-facing specs, a **PE agent** owns technical plans, and they cross-review each other's work before a human reviews both together.

##### Roles

- **PM agent** (Product Manager) — author of the **spec** (Gherkin `.feature` files) and reviewer of the **plan**. Cares about: user-facing behavior, edge cases, acceptance criteria, naming, scope.
- **PE agent** (Product Engineer) — author of the **plan** (technical approach) and reviewer of the **spec**. Cares about: feasibility, files-to-touch, risks, dependencies, test coverage of acceptance.
- **Code reviewer agent** — used in Implement only; can be a generic `code-reviewer` agent. Distinct from PM/PE.
- **Human** — final gate before Implement and final gate at PR merge.

Each agent runs as a separate Claude session via the `Agent` tool so reviews are independent of the worker's context.

##### Phase 1 — Grooming

- **Driver**: PM agent.
- **Artifact**: a refined Linear issue (Story, Acceptance, scope, type label, milestone).
- **PM produces**: a concrete user story, testable acceptance bullets, an explicit scope decision (single feature vs. multi-feature epic), label, milestone.
- **PE reviews**: feasibility-flag pass — calls out anything that looks expensive, unclear, or dependent on missing capability. Not approval; just surfacing surprises before the human is pulled in.
- **Exit gate**: human accepts the groomed issue and moves it to In Progress.

##### Phase 2 — Spec ∥ Plan (parallel)

After Grooming, two branches run **concurrently**, each authored by one role and reviewed by the other.

- **Spec branch (PM authors, PE reviews)**
  - **Artifact**: one or more `.feature` files on a branch named `<issue-key>-<slug>`; issue's `Spec` field updated to the path(s); `cargo test --test cucumber` is **red** (or scenarios unimplemented).
  - **PM produces**: Gherkin scenarios with concrete example values, covering every Acceptance bullet; scenario layout per the `cucumber` skill.
  - **PE reviews**: every Acceptance bullet has at least one scenario; scenarios are testable from where the code can observe them; tests actually fail (no false-green); no infeasible assumptions.

- **Plan branch (PE authors, PM reviews)**
  - **Artifact**: a short technical plan in the issue (or `.claude/plans/<issue-key>.md` for larger work) — files to touch, approach, risks, dependency map.
  - **PE produces**: file-level plan, called-out risks, alternatives considered, mapping from Acceptance bullets → planned scenarios → planned code changes.
  - **PM reviews**: the plan covers every Acceptance bullet; the user-visible behavior matches the spec; nothing is silently descoped; risks the user should know about are surfaced.

- **Coordination**: the two branches share the same Linear issue and feature branch. If one branch surfaces a finding that invalidates the other (e.g., the plan reveals a constraint that changes the spec), that branch comments on the issue and the affected branch reworks. Both branches must reach a clean cross-review before Phase 2 ends.

- **Exit gate**: **human reviews spec and plan together**. The human accepts the pair, sends one or both back for rework, or aborts.

##### Phase 3 — Implement

- **Driver**: PE agent.
- **Artifact**: code that turns the new scenarios green; PR opened; issue moved to In Review.
- **PE produces**: minimum code to satisfy the spec; incremental commits; green `cargo test --test cucumber`; PR referencing the issue key in title and body.
- **Code reviewer agent reviews**: correctness vs. the spec, code quality, no regressions in other features, no issue-tracker references in source.
- **Exit gate**: human merges the PR; issue moves to Done. Any `@wip`-tagged scenarios become a follow-up issue.

##### Meta-rules

- **Don't write code without a `.feature` to justify it.** Phase 3 is gated on Phase 2.
- **Bug fixes** start with a regression scenario in the Spec branch (red), then Implement.
- **Epics / cross-feature refactors**: the issue tracks a checklist of feature paths; the Spec branch lists them all; the Plan branch sequences them; Implement runs per file; the issue stays open until all paths are green.
- **Phase collapse is explicit, not implicit**: trivial chores (typo fix, dep bump) may collapse the PM/PE split, but the agent must say so and the human must accept.

### 2. `CLAUDE.md` — project-specific config

Holds everything the skills deliberately omit. Concretely:

- One-paragraph description of `place` and the Claude-on-the-web context.
- **Linear binding**:
  - Team: `One True Name` (`4298c63b-c726-415a-85d0-383e875f818e`)
  - Project: `Software Factory` (`fa52409f-b878-4f89-b34e-54c8907dc5e5`)
  - Active milestones: `Bootstrap`, `place: MVP`
  - Label vocabulary used here: `feature`, `epic`, `bug`, `refactor`, `chore`
- **Repo paths**: features live at `features/`; cucumber runner at `tests/cucumber.rs`; step defs at `tests/steps/`.
- **Skill index**: short pointer to each of the three skills and when to invoke them.
- **Phase-2 plan location**: `.claude/plans/<issue-key>.md` for non-trivial plans.

### 3. Linear seed (created live via MCP)

Inside the **Software Factory** project on team **One True Name**:

- **Labels** (create if missing): `feature`, `epic`, `bug`, `refactor`, `chore`.
- **Milestones** (two):
  1. `Bootstrap` — the work *of producing this workflow itself*. The bootstrap is the 0th iteration: tracked in Linear for traceability, but produced manually since the skills don't yet exist to drive it. Issues:
     - `Write the linear skill` (`feature` label)
     - `Write the cucumber skill` (`feature` label)
     - `Write the workflow skill` (`feature` label)
     - `Write CLAUDE.md` (`chore` label)
     - `Seed place: MVP issues` (`chore` label)
  2. `place: MVP` — the first real run of the loop. Issues (each `feature`-labeled, body using the `linear`-skill template; first time worked, drives Cargo/cucumber-rs scaffold creation as part of issue 1):
     1. `place init` — generate a workplace repo with `mise.toml`, `claude/` dir, `.code-workspace`.
     2. `place add <repo>...` — clone & register one or more repos into the current workplace.
     3. `mise integration` — `mise.toml` conventions, `CLAUDE_CONFIG_DIR` pointing at the workplace `claude/` dir, install of `place` itself via mise.
     4. `vscode workspace generation` — generate/update `.code-workspace` as repos are added/removed.
     5. `claude web bootstrap` — install `place` and run it from a Claude-web `startsession` hook so a fresh sandbox lands in a configured workplace.
     6. `subagent delegation per repo` — invoking subagents inside an added repo picks up that repo's own `.claude/` config.

## Critical files to create

- `/Users/ungood/workspaces/personal/place/CLAUDE.md`
- `/Users/ungood/workspaces/personal/place/.claude/skills/linear/SKILL.md`
- `/Users/ungood/workspaces/personal/place/.claude/skills/cucumber/SKILL.md`
- `/Users/ungood/workspaces/personal/place/.claude/skills/workflow/SKILL.md`

## Linear actions to perform

(All via `mcp__linear-server__*` against project `fa52409f-b878-4f89-b34e-54c8907dc5e5`.)

1. `list_issue_labels` → diff against `{feature, epic, bug, refactor, chore}` → `create_issue_label` for any missing.
2. `list_milestones` → if `Bootstrap` and/or `place: MVP` not present, `save_milestone` to create them.
3. `save_issue` ×5 for the Bootstrap milestone.
4. `save_issue` ×6 for the `place: MVP` milestone.

All bodies use real newlines (per the linear-server MCP instructions).

## Verification

End-to-end check that the bootstrap actually works:

1. **Skills load**: open a fresh Claude Code session in this repo; confirm `linear`, `cucumber`, and `workflow` appear in the available-skills list.
2. **Skills are project-agnostic**: grep the three SKILL.md files for `place`, `Software Factory`, `One True Name`, `ONE-`, `4298c63b`, `fa52409f` — expect zero matches. Repo-specific values appear only in `CLAUDE.md`.
3. **Linear state**: `list_issues` filtered to milestone `Bootstrap` returns 5 issues; filtered to `place: MVP` returns 6 issues with `feature` label.
4. **Dogfood the loop on `place: MVP` / issue 1**: invoke the `workflow` skill on `place init`. The skill should walk us through Phase 1 (Grooming, PM-driven with PE feasibility check, then human approval), then Phase 2 in parallel (PM authors the spec while PE authors the plan; each cross-reviews the other; the spec branch also drives Cargo/cucumber-rs scaffold creation since issue 1 is the first to need it) until both reviews are clean and the human has approved the spec/plan pair. Stop there; Phase 3 (Implement) is out of scope for *this* plan. Any phase-step ambiguous to a fresh agent is a defect in the corresponding skill — fix the skill.
