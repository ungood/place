---
name: cucumber
description: Conventions for writing Gherkin .feature files and cucumber-rs step definitions in Rust projects. Invoke when authoring or reviewing specs, scaffolding the cucumber runner, or adding step definitions.
---

# cucumber

Conventions for cucumber-rs (https://crates.io/crates/cucumber) in Rust projects.

## File layout

- `features/<area>.feature` — one feature file per top-level capability. Filename is kebab-case and matches the user-facing surface (subcommand, endpoint, screen).
- `tests/cucumber.rs` — single runner binary (Cargo `[[test]] name = "cucumber"`).
- `tests/steps/` — step definition modules, grouped by area to mirror the feature files.

## Gherkin style

- `Feature:` line is the user-facing capability name (matches the Linear issue title where applicable).
- One declarative `Given/When/Then` chain per scenario. Avoid imperative `And ... And ... And ...` chains that script implementation steps.
- **Concrete examples, not placeholders.** Prefer `Given the cart contains "Widget A" and "Widget B"` over `Given the cart contains <some> items`. Use `Scenario Outline` only when the variation is the test's point.

## Run command

`cargo test --test cucumber`

## Step definitions

- One module per feature area (`tests/steps/<area>.rs`).
- Share a single small `World` struct.
- Don't introduce fixtures until a third scenario needs the same setup. Two scenarios duplicating five lines is fine; abstracting too early hides intent.
- Step regexes match natural English — write them so the same step reads naturally in unrelated scenarios.
