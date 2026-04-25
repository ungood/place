---
name: workflow
description: The PM/PE spec-driven loop for projects that combine Linear and cucumber-rs. Per Linear issue, runs Grooming → (Spec ∥ Plan) → Human review → Implement, with cross-role agent reviews at each phase. Invoke whenever starting work on a Linear issue. Pairs with the `linear` and `cucumber` skills.
---

# workflow

Spec-driven loop modeled on PM/PE collaboration at a real product team. For each Linear issue, four phases run in order: **Grooming → (Spec ∥ Plan) → Human review → Implement**. This skill is project-agnostic; pair it with the `linear` and `cucumber` skills, and consult the project's `CLAUDE.md` for binding details.

## Roles

- **PM agent** (Product Manager) — authors the **spec** (Gherkin `.feature` files) and reviews the **plan**. Cares about: user-facing behavior, edge cases, acceptance criteria, naming, scope.
- **PE agent** (Product Engineer) — authors the **plan** (technical approach) and reviews the **spec**. Cares about: feasibility, files-to-touch, risks, dependencies, test coverage of acceptance.
- **Reviewer agent (Implement)** — used in Phase 3. A code-review-style agent (project-defined `code-reviewer` subagent_type, or `general-purpose` with explicit reviewer instructions).
- **Human** — final gate before Implement and final gate at PR merge.

Each agent runs in its own Claude session via the `Agent` tool, so reviews are independent of the worker's context.

## Phase 1 — Grooming

- **Driver**: PM agent.
- **Artifact**: a refined Linear issue (Story, Acceptance, scope, type label, milestone) per the `linear` skill template.
- **PM produces**: a concrete user story; testable Acceptance bullets; an explicit scope decision (single feature vs. multi-feature epic); type label; milestone.
- **PE reviews**: feasibility-flag pass — calls out anything expensive, unclear, or dependent on missing capability. Surfaces surprises before the human is pulled in. Not approval — readiness-for-human-review.
- **Exit gate**: human accepts the groomed issue and moves it to **In Progress**.

## Phase 2 — Spec ∥ Plan (parallel)

After Grooming, two branches run **concurrently**, each authored by one role and reviewed by the other.

### Spec branch (PM authors, PE reviews)

- **Artifact**: one or more `.feature` files on a branch named `<issue-key>-<slug>`; the issue's **Spec** field updated to the path(s); `cargo test --test cucumber` is **red** (or scenarios unimplemented).
- **PM produces**: Gherkin scenarios with concrete example values covering every Acceptance bullet. Layout per the `cucumber` skill.
- **PE checks**:
  - Every Acceptance bullet has at least one scenario.
  - Scenarios are testable from where the code can observe them.
  - Tests actually fail (no false-green).
  - No infeasible assumptions about the runtime/environment.

### Plan branch (PE authors, PM reviews)

- **Artifact**: a short technical plan in the issue (or `.claude/plans/<issue-key>.md` for larger work) — files to touch, approach, risks, dependency map.
- **PE produces**: file-level plan; risks; alternatives considered; mapping from Acceptance bullets → planned scenarios → planned code changes.
- **PM checks**:
  - The plan covers every Acceptance bullet.
  - User-visible behavior matches the spec.
  - Nothing is silently descoped.
  - Risks the user should know about are surfaced.

### Coordination

The two branches share the same Linear issue and feature branch. If one branch surfaces a finding that invalidates the other, comment on the issue and the affected branch reworks. **Both** cross-reviews must be clean before Phase 2 ends.

### Exit gate

The human reviews **spec and plan together**. They accept the pair, send one or both back for rework, or abort.

## Phase 3 — Implement

- **Driver**: PE agent.
- **Artifact**: code that turns the new scenarios green; PR opened; issue moved to **In Review**.
- **PE produces**: minimum code to satisfy the spec; incremental commits; green test run; PR referencing the issue key in title and body.
- **Reviewer agent checks**: correctness vs. the spec; code quality; no regressions in other features; no issue-tracker references in source.
- **Exit gate**: human merges the PR; issue moves to **Done**. Any `@wip`-tagged scenarios become a follow-up issue.

## Meta-rules

- **Don't write code without a `.feature` to justify it.** Phase 3 is gated on Phase 2.
- **Bug fixes** start with a regression scenario in the Spec branch (red), then Implement.
- **Epics / cross-feature refactors**: the issue tracks a checklist of feature paths; the Spec branch lists them all; the Plan branch sequences them; Implement runs per file; the issue stays open until all paths are green.
- **Phase collapse is explicit, not implicit**: trivial chores (typo fix, dep bump) may collapse the PM/PE split, but the agent must say so out loud and the human must accept.

## How to spawn the agents

Use the `Agent` tool to start a fresh Claude session for each role. Pass the agent the relevant skill content (or refer it to the skill file by path), the issue body, and the artifact location. Suggested mappings — projects can override:

- **PM (worker or reviewer)**: `Agent({subagent_type: "general-purpose", description: "PM: <task> for <issue-key>", prompt: "..."})`
- **PE (worker)**: `Agent({subagent_type: "Plan", description: "PE: plan for <issue-key>", prompt: "..."})` — the `Plan` subagent's design focus fits planning. For spec review use `general-purpose`.
- **PE (reviewer of spec)**: `Agent({subagent_type: "general-purpose", description: "PE: review spec for <issue-key>", prompt: "..."})`
- **Implement reviewer**: prefer a project-defined `code-reviewer` subagent_type if available; otherwise `general-purpose` with explicit reviewer instructions.

In every prompt, brief the agent like a smart colleague who just walked into the room: state the goal, the artifact location, the criteria, and the expected response shape.
