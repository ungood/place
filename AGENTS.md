# place

`place` is a Rust CLI that manages workplaces: directories of independent git repos plus shared config. The design is in `docs/design.md`. Read it before changing behavior.

## Documentation

This file is for agent-only context: paths, identifiers and conventions an agent needs to act. Human-facing docs live in `docs/` and the README, and this file points to them instead of repeating them.

## Skills

Skills live in `.agents/skills/`. `.claude/skills` is a symlink to that directory for Claude Code.

- `cucumber`: conventions for `.feature` files and cucumber-rs step definitions.

## Repo paths

- `docs/design.md`: the design, and the source of truth for behavior.
- `features/`: Gherkin `.feature` files, one per command or capability, kebab-case. `features/README.md` holds the step vocabulary.
- `tests/cucumber.rs`: the single cucumber-rs runner.
- `tests/steps/`: step definitions, grouped by area.

## Conventions

- Work is tracked in GitHub Issues on `ungood/place`.
- Behavior is specified in `features/` before it's implemented. Integration tests are cucumber-rs. Unit tests are ordinary Rust tests next to the code.
- Integration tests use local bare repos as remotes and never touch the network.
- `features/README.md` defines the step vocabulary and placeholders. Reuse those steps, and add new ones there.
- Issue numbers and tracker URLs appear in commit messages and PR descriptions only, never in source, comments or `.feature` files.
- `@wip` is the only Gherkin tag. CI excludes `@wip` scenarios.
- The Cargo project and the cucumber runner aren't scaffolded yet. They arrive with the first implementation PR.
