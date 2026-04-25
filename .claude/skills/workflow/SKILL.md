---
name: workflow
description: The PM/PE/SWE spec-driven loop for projects that combine Linear and cucumber-rs. Per Linear issue, runs Grooming → Spec → Plan → Implement, with a consolidated human review gate before Implement. Invoke whenever starting work on a Linear issue. Pairs with the `linear` and `cucumber` skills.
---

# workflow

Spec-driven loop modeled on PM/PE/SWE collaboration at a real product team. For each Linear issue, four phases run in order: **Grooming → Spec → Plan → Implement**. Pair with the `linear` and `cucumber` skills, and consult the project's `CLAUDE.md` for binding details.

## Roles

- **PM agent** (Product Manager) — drives **Grooming**; authors the **Spec** (Gherkin `.feature` files); reviews the **Plan**. Cares about: user-facing behavior, edge cases, scope.
- **PE agent** (Product Engineer) — provides feasibility check during Grooming; reviews the **Spec**; authors the **Plan**; reviews the **Implementation**. Cares about: feasibility, files-to-touch, risks, dependencies, code standards.
- **SWE agent** (Software Engineer) — authors the **Implementation**. Cares about: minimum code and documentation that turn the spec green; code quality.
- **Human** — final gate before Implement, and final gate at PR merge.

Each agent runs in its own Claude session via the `Agent` tool, so reviews are independent of the worker's context.

## Phase 1 — Grooming

- **Driver**: PM agent.
- **Artifact**: a refined Linear issue per the team's Linear issue template.
- **PE reviews**: feasibility-flag pass — surfaces anything expensive, unclear, or dependent on missing capability before the human is pulled in. Not approval — readiness-for-human-review.
- **Exit gate**: human accepts the groomed issue and moves it to **Planned**.

## Phase 2 — Spec

- **Driver**: PM agent.
- **Artifact**: one or more `.feature` files on a branch named `<issue-key>-<slug>`. The cucumber suite is **red** (or scenarios unimplemented). The cucumber spec *is* the acceptance criteria.
- **PE checks**:
  - Scenarios match the Story.
  - Scenarios are observable from where the code can sit.
  - Tests actually fail (no false-green).
  - No infeasible assumptions about the runtime/environment.
- **Exit gate**: PE review is clean; spec is ready for the consolidated human review at the end of Phase 3.

## Phase 3 — Plan

- **Driver**: PE agent.
- **Artifact**: a short technical plan in `plans/<issue-key>.md` — files to touch, approach, risks, dependency map, mapping from scenarios → planned changes. Plans are durable records of decisions and live in the repo, not in `.claude/`.
- **PM checks**:
  - The plan covers every scenario in the spec.
  - User-visible behavior matches the spec.
  - Nothing is silently descoped.
  - Risks the user should know about are surfaced.
- **Exit gate**: PM review is clean; the **human reviews spec and plan together** and accepts the pair before Phase 4 begins.

## Phase 4 — Implement

- **Driver**: SWE agent. Issue moves to **In Progress** when implementation starts.
- **Artifact**: code *and documentation* that turn the new scenarios green; PR opened against the project's PR template (which carries the project's definition-of-done checklist); issue moves to **In Review** on PR open.
- **PE checks**:
  - Correctness vs. the spec.
  - Code quality and adherence to project standards.
  - Documentation is updated alongside behavior changes.
  - The PR-template definition-of-done checklist is genuinely satisfied — not box-checked.
  - No issue-tracker references in source.
- **Exit gate**: human merges the PR; issue moves to **Done**. Any `@wip`-tagged scenarios become a follow-up issue.

## Meta-rules

- **Don't write code without a `.feature` to justify it.** Phase 4 is gated on Phases 2 and 3.
- **Bug fixes** start with a regression scenario in Phase 2 (red), then proceed normally.
- **Epics / cross-feature refactors**: the issue tracks a checklist of feature paths; Phase 2 lists them all; Phase 3 sequences them; Phase 4 runs per file; the issue stays open until all paths are green.
- **Phase collapse is explicit, not implicit**: trivial chores (typo fix, dep bump) may collapse phases, but the agent must say so out loud and the human must accept.

## How to spawn the agents

Use the `Agent` tool to start a fresh Claude session for each role. Pass the agent the relevant skill content (or refer it to the skill file by path), the issue body, and the artifact location. Brief each agent like a smart colleague who just walked into the room: state the goal, the artifact location, the criteria, and the expected response shape.
