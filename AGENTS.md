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
- `tests/steps/`: step definitions, grouped by subject (remotes, workplace files, repos, running place, outcomes). `world.rs` holds the test world and its isolated git environment.
- `src/main.rs`: the CLI arguments and dispatch. Each command is in `src/commands/`.
- `src/error.rs`: every error and warning place prints. Their wording is spec, so keep it in step with `features/`.
- `src/git.rs`: the only module that runs git. Every clone goes through `git::clone`.
- `src/config.rs` reads, merges and edits workplace files. `src/workplace.rs` finds workplaces, nested workplaces and unlisted checkouts. `src/state.rs` computes repo states.

## Conventions

- Work is tracked in GitHub Issues on `ungood/place`.
- Behavior is specified in `features/` before it's implemented. Integration tests are cucumber-rs. Unit tests are ordinary Rust tests next to the code.
- Integration tests use local bare repos as remotes and never touch the network.
- `features/README.md` defines the step vocabulary and placeholders. Reuse those steps, and add new ones there.
- Command output is for people, and must make sense to someone who doesn't know place's internals. Prefer git's own output and git's wording (`git status`, `git status --short`) over place's own vocabulary, and only add words where place knows something git doesn't, such as which branch a repo should be on. Commands that report on many repos print a block per repo by default and one line per repo with `-s`/`--short`. `--json` output is for machines and can be as concise as it likes.
- Every merge to main releases if it brings a `feat:` or `fix:` commit: `.github/workflows/release.yml` bumps the version, commits and tags it, and publishes with packslip. Commit messages are conventional commits, and their type decides the release (see `cliff.toml`). Use `chore:`, `docs:`, `test:` and the like for changes that shouldn't release. `mise run release:version` and `mise run release:notes` preview the next release. place isn't published to crates.io.
- Issue numbers and tracker URLs appear in commit messages and PR descriptions only, never in source, comments or `.feature` files.
- `@wip` is the only Gherkin tag. CI excludes `@wip` scenarios.
- mise provides the Rust toolchain and the tasks: `mise run build`, `mise run test` (or `test:unit` and `test:features`), `mise run lint`, `mise run fmt`. Run `mise run lint` and `mise run test` before committing.
- Machine-specific settings, such as a pinned `SDKROOT` on macOS, go in `mise.local.toml`, which git ignores.
