# Bootstrap: spec-driven workflow for `place`

## Context

`place` will be a Rust CLI (named after, and built around, mise) for managing agent-friendly, multi-repo "workplaces." Its first job is to make Claude Code on the web (https://code.claude.com) productive on related repos by generating a workplace repo containing shared mise config, a VS Code workspace, and a Claude config directory.

Before writing any tool code, we want a **spec-driven workflow** in place: Linear holds the user stories, repo `.feature` files (cucumber-rs) hold the executable Gherkin, and Claude Code skills tell future agent sessions how to operate the loop. This plan only bootstraps that workflow — Cargo/cucumber-rs scaffolding is deferred to the first MVP issue, so the scaffold itself is exercised by the workflow it enables.

### Decisions captured from the conversation

- **Linear scope**: existing project **Software Factory** (team `One True Name` / `ONE`); `place` lives there as a milestone, not a separate project.
- **Issue ↔ spec granularity**: 1 Linear issue ≈ 1 `.feature` file *initially*. The workflow must allow issues to span multiple features later (epics, refactors, bug fixes).
- **Spec source of truth**: Gherkin lives in repo `.feature` files. Linear issues hold prose + a link to the feature file path. Issue body structure comes from Linear's native templates feature, not from anything in this repo.
- **Skills are project-agnostic**: each skill describes *how to use a tool/process*, never repo-specific identifiers. Project-specific config (team IDs, milestone names, label vocab, repo paths) lives in `CLAUDE.md`.
- **Documentation philosophy**: CLAUDE.md is for agent-only context. Human-facing docs live in their natural home (Linear for tracker conventions, `plans/` for design decisions, READMEs for contributor docs); CLAUDE.md references them rather than duplicating their content.
- **Plans are durable**: technical plans live at `plans/<issue-key>.md` (top-level) — not buried under `.claude/`.
- **No issue-tracker references in code**: Linear keys/URLs go in commit messages and PR descriptions, never in source comments or `.feature` files.
- **Workflow shape mimics PM/PE/SWE collaboration at a real product team**:
  - Three agent roles: a **PM agent** (Grooming + Spec), a **PE agent** (feasibility check + Plan + reviewer of Spec and Implementation), and a **SWE agent** (Implementation).
  - Phases run sequentially: **Grooming → Spec → Plan → Implement**.
  - Cross-role review at each phase: PE reviews PM's Spec; PM reviews PE's Plan; PE reviews SWE's Implementation against the plan and code standards.
  - The human reviews **spec and plan together** as one consolidated gate before Implement starts.
- **Bootstrap deliverables**: (a) Linear seed (labels, two milestones, initial issues) and (b) three project-agnostic Claude Code skills — `linear`, `cucumber`, `workflow` — plus a project-specific `CLAUDE.md`.

## Deliverables

### 1. Repo files

```
/CLAUDE.md                              # project-specific config: Linear IDs, milestone names,
                                        # repo paths; pointers to the three skills
/.claude/skills/linear/SKILL.md         # generic Linear-via-MCP conventions
/.claude/skills/cucumber/SKILL.md       # generic cucumber-rs / Gherkin conventions
/.claude/skills/workflow/SKILL.md       # generic spec-driven loop with review gates
/plans/bootstrap-workflow.md            # this plan (durable record of bootstrap decisions)
```

#### `linear` skill — project-agnostic

Tool-and-process conventions for using the `linear-server` MCP, applicable to any project. **No** team/project/milestone IDs.

- **Issue structure**: Linear's native issue templates feature owns issue body shape — the skill points there rather than duplicating a template.
- **Phase via Linear workflow state, not labels**: Backlog → Planned (groomed; Spec/Plan/human-review happens here) → In Progress (implementation underway) → In Review (PR open) → Done (PR merged) → Cancelled.
- **Cross-linking**: mention the issue key in commit messages and PR titles/bodies; never in source comments.
- **MCP tool pointers**: when to use `list_issues`, `save_issue`, `save_comment`, `list_milestones`, `save_milestone`, `list_issue_labels`, `create_issue_label`. Always pass real newlines, never `\n` literals.
- **Querying patterns**: how to find "next issue assigned to me", "has this been done before", and "what's blocking phase X" without project-specific IDs.

#### `cucumber` skill — project-agnostic

Conventions for writing Gherkin and step definitions in cucumber-rs Rust projects.

- **File layout**: `features/<area>.feature`, `tests/cucumber.rs`, `tests/steps/` per area.
- **Gherkin style**: `Feature:` line is the user-facing capability name; scenarios use concrete example values rather than placeholders; one declarative `Given/When/Then` chain per scenario.
- **Run command**: `cargo test --test cucumber`.
- **Step definitions**: one module per area; small shared `World`; don't introduce fixtures until a third scenario needs them.

#### `workflow` skill — project-agnostic

The spec-driven loop, written so any project combining Linear + cucumber-rs can adopt it. Models a real product team: PM, PE, and SWE agents collaborating in sequence, with the human as the consolidated gatekeeper before implementation.

##### Roles

- **PM agent** — drives Grooming; authors the Spec; reviews the Plan.
- **PE agent** — feasibility check during Grooming; reviews the Spec; authors the Plan; reviews the Implementation against the plan and code standards.
- **SWE agent** — authors the Implementation.
- **Human** — final gate before Implement, and final gate at PR merge.

##### Phases (sequential)

1. **Grooming** (PM driver, PE feasibility check). Exit gate: human accepts the groomed issue and moves it to **Planned**.
2. **Spec** (PM authors `.feature` files, PE reviews). The cucumber spec *is* the acceptance criteria — no parallel "Acceptance bullets" list. Exit gate: PE review clean.
3. **Plan** (PE authors `plans/<issue-key>.md`, PM reviews). Exit gate: PM review clean; **human reviews spec and plan together** and accepts the pair before Phase 4.
4. **Implement** (SWE authors code *and documentation*; PE reviews against the plan, code standards, and the project's PR-template definition-of-done checklist). Issue moves to In Progress when SWE starts; In Review on PR open; Done on merge.

##### Meta-rules

- **Don't write code without a `.feature` to justify it.** Phase 4 is gated on Phases 2 and 3.
- **Bug fixes** start with a regression scenario in Phase 2 (red), then proceed normally.
- **Epics / cross-feature refactors**: the issue tracks a checklist of feature paths; Phase 2 lists them all; Phase 3 sequences them; Phase 4 runs per file; the issue stays open until all paths are green.
- **Phase collapse is explicit, not implicit**: trivial chores may collapse phases, but the agent must say so out loud and the human must accept.

### 2. `CLAUDE.md` — project-specific config

Holds everything the skills deliberately omit.

- One-paragraph description of `place` and the Claude-on-the-web context.
- **Documentation philosophy** statement (agent-only, references rather than duplicates).
- **Linear binding**:
  - Team: `One True Name` (`4298c63b-c726-415a-85d0-383e875f818e`)
  - Project: `Software Factory` (`fa52409f-b878-4f89-b34e-54c8907dc5e5`)
  - Active milestones: `Bootstrap`, `place: MVP`
  - Labels: pointer to Linear (`list_issue_labels`); not duplicated in CLAUDE.md.
- **Repo paths**: `features/`, `tests/cucumber.rs`, `tests/steps/`, `plans/<issue-key>.md`, `.claude/skills/`.
- **Skill index**: short pointer to each of the three skills and when to invoke them.

### 3. Linear seed (created live via MCP)

Inside the **Software Factory** project on team **One True Name**:

- **Labels** (create if missing): the project's chosen vocabulary, capitalized to match existing workspace conventions (`Feature`, `Epic`, `Bug`, `Refactor`, `Chore`).
- **Milestones** (two):
  1. `Bootstrap` — the work *of producing this workflow itself*. The bootstrap is the 0th iteration: tracked in Linear for traceability, but produced manually since the skills don't yet exist to drive it. Issues:
     - `Write the linear skill`
     - `Write the cucumber skill`
     - `Write the workflow skill`
     - `Write CLAUDE.md`
     - `Seed place: MVP issues`
  2. `place: MVP` — the first real run of the loop. Issues (each `Feature`-labeled, body following Linear's issue template; first time worked, drives Cargo/cucumber-rs scaffold creation as part of issue 1):
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
- `/Users/ungood/workspaces/personal/place/plans/bootstrap-workflow.md` (this file)

## Linear actions to perform

(All via `mcp__linear-server__*` against project `fa52409f-b878-4f89-b34e-54c8907dc5e5`.)

1. `list_issue_labels` → diff against the chosen vocabulary → `create_issue_label` for any missing.
2. `list_milestones` → if `Bootstrap` and/or `place: MVP` not present, `save_milestone` to create them.
3. `save_issue` ×5 for the Bootstrap milestone.
4. `save_issue` ×6 for the `place: MVP` milestone.

All bodies use real newlines (per the linear-server MCP instructions).

## Verification

End-to-end check that the bootstrap actually works:

1. **Skills load**: open a fresh Claude Code session in this repo; confirm `linear`, `cucumber`, and `workflow` appear in the available-skills list.
2. **Skills are project-agnostic**: grep the three SKILL.md files for `place`, `Software Factory`, `One True Name`, `ONE-`, `4298c63b`, `fa52409f` — expect zero matches. Repo-specific values appear only in `CLAUDE.md`.
3. **Linear state**: `list_issues` filtered to milestone `Bootstrap` returns 5 issues; filtered to `place: MVP` returns 6 issues with `Feature` label.
4. **Dogfood the loop on `place: MVP` / issue 1**: invoke the `workflow` skill on `place init`. The skill should walk us through Phase 1 (Grooming, PM-driven with PE feasibility check, then human approval to **Planned**), Phase 2 (PM-authored Spec, PE-reviewed; the spec phase also drives Cargo/cucumber-rs scaffold creation since issue 1 is the first to need it), and Phase 3 (PE-authored Plan in `plans/ONE-10.md`, PM-reviewed) until the human has approved the spec/plan pair. Stop there; Phase 4 (Implement) is out of scope for *this* plan. Any phase step ambiguous to a fresh agent is a defect in the corresponding skill — fix the skill.
