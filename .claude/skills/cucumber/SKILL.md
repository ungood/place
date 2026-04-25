---
name: cucumber
description: Conventions for writing Gherkin .feature files and cucumber-rs step definitions in Rust projects. Invoke when authoring or reviewing specs, scaffolding the cucumber runner, or adding step definitions. Project-specific tag vocabulary and any path overrides live in the project's CLAUDE.md.
---

# cucumber

Conventions for cucumber-rs (https://crates.io/crates/cucumber) in Rust projects. Project-specific paths and tag vocabularies live in `CLAUDE.md`.

## File layout

- `features/<area>.feature` — one feature file per top-level capability. Filename is kebab-case and matches the user-facing surface (subcommand, endpoint, screen).
- `tests/cucumber.rs` — single runner binary (Cargo `[[test]] name = "cucumber"`).
- `tests/steps/` — step definition modules, grouped by area to mirror the feature files.

## Gherkin style

- `Feature:` line is the user-facing capability name (matches the Linear issue title where applicable).
- One declarative `Given/When/Then` chain per scenario. Avoid imperative `And ... And ... And ...` chains that script implementation steps.
- **Concrete examples, not placeholders.** Prefer `Given the cart contains "Widget A" and "Widget B"` over `Given the cart contains <some> items`. Use `Scenario Outline` only when the variation is the test's point.
- No issue-tracker keys, URLs, or commit refs in `.feature` files. Traceability lives in Linear's Spec field and commit/PR text.

## Tags

- `@wip` — committed but not yet implemented. The runner excludes `@wip` by default; CI fails if any non-`@wip` scenario fails.
- All other tags are project-specific; declare them in `CLAUDE.md` as they're adopted.

## Run command

`cargo test --test cucumber`

## When one issue spans multiple feature files

List all paths in the issue's **Spec** section. The workflow loop runs per file: each file gets its own red→green pass, but the issue stays open until all files are green.

## Step definitions

- One module per feature area (`tests/steps/<area>.rs`).
- Share a single small `World` struct.
- Don't introduce fixtures until a third scenario needs the same setup. Two scenarios duplicating five lines is fine; abstracting too early hides intent.
- Step regexes match natural English — write them so the same step reads naturally in unrelated scenarios.
