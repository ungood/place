# place design

Status: draft, 2026-10-15.

`place` manages workplaces. A workplace is a directory that holds several independent git repositories, plus the shared configuration that applies to all of them. The name completes _mise-en-place_: mise sets up what happens inside a directory (tools, environment, tasks), and place sets up which directories exist.

## Principles

- **One job.** place decides which repositories belong in a workplace and keeps the checkouts in line with that list. Tools, environment and tasks belong to mise, secrets to fnox, and agent orchestration to other tools. place exposes its state through `--json` output and exit codes so those tools can build on it.
- **Polyrepo, not monorepo.** Each repository keeps its own history, CI and releases. A change inside a child repository never requires a commit in the workplace. This is the main difference from git submodules.
- **Never disturb work.** place clones, fetches and fast-forwards. It never switches branches, rebases or stashes. It deletes a checkout only when asked to, and only when nothing would be lost.
- **Familiar names.** Commands and flags follow git first, mise second and vcstool third.
- **Friendly errors.** Every error says what went wrong in plain words, names the file, repo or path involved, and gives help that says how to fix it. A configuration error points at the line that caused it. Exact wording is specified in `features/`, so it gets reviewed like any other behavior.
- **Output like mise.** place's output looks and behaves like mise's and fnox's, and uses the same libraries where it can: clap for arguments, miette for errors with help and source snippets, and console for color and terminal handling.
- **Compatible input.** place reads vcstool `.repos` files alongside its own `place.toml`, the way mise reads `.tool-versions` alongside `mise.toml`.
- **Git first.** Git is the only version control system supported for now, but it sits behind an interface so others can be added later.

## Concepts

- **Workplace.** A directory containing place configuration: `place.toml`, any `place.*.toml`, or a `*.repos` file. Commands act on the workplace in the current directory, or in `-C <dir>`. They don't search parent directories, so running place inside a repo, or in a directory with no configuration, is an error. When there's a workplace further up, the error's help names it.
- **Repo.** A git checkout listed in the workplace configuration, keyed by its path relative to the workplace.
- **Ref.** What a repo should track: a branch (the normal case), a tag, or a commit. A repo with no ref tracks the remote's default branch as it was when the repo was cloned.
- **Nested workplace.** A repo that contains its own place configuration. place recurses into it, and its repos are resolved relative to its own directory. Parents never need to know about grandchildren.
- **Unlisted repo.** A git checkout directly inside the workplace that no configuration lists. place reports it and leaves it alone.

A workplace may itself be a git repository, as `ungood/factory` is. That repository tracks the workplace's own files (configuration, `AGENTS.md`, `mise.toml`) and ignores the child checkouts.

## Configuration

### place.toml

```toml
[repos.place]
url = "https://github.com/onetruename/place.git"

[repos.agents]
url = "https://github.com/ungood/agents.git"
ref = "main"

[repos."vendor/vcs2l"]
url = "https://github.com/ros-infrastructure/vcs2l.git"
ref = "1.1.7"
```

Each table under `repos` is keyed by the repo's relative path. Keys containing `/` or `.` must be quoted.

- `url` (required): the clone URL. git's own `url.<base>.insteadOf` rewriting applies, because place runs the git CLI.
- `ref` (optional): a branch, a tag, or a commit hash, resolved against the remote the way `git rev-parse` resolves names locally. The qualified forms `refs/heads/<name>` and `refs/tags/<name>` make the choice explicit when a branch and a tag share a name. Without `ref`, the repo tracks the remote's default branch.

M0 has no other keys. Groups, shallow clones and per-repo recursion settings can come later when a real workplace needs them.

### Files and precedence

Within a workplace directory, place reads these files, from lowest to highest precedence:

1. `*.repos`, in lexicographic order.
2. `place.toml`
3. `place.<env>.toml`, for each selected environment.
4. `place.local.toml`, for machine-specific changes kept out of version control.
5. `place.<env>.local.toml`, for each selected environment.

Environments are selected with the global `-E/--env <env>` flag or the `PLACE_ENV` variable, as `-E` and `MISE_ENV` work in mise. With several environments, a later one wins over an earlier one.

Merging works per repo path. The highest-precedence file that defines a path supplies the whole entry, which matches how mise merges `[tools]`. Later versions can add mise's other locations, such as `.config/place.toml`, without changing these rules.

Global settings live in `~/.config/place/config.toml`. They never list repos.

### .repos compatibility

place reads the vcstool format:

```yaml
repositories:
  vendor/vcs2l:
    type: git
    url: https://github.com/ros-infrastructure/vcs2l.git
    version: 1.1.7
```

Entries whose `type` isn't `git` are rejected with an error that names the file and path. A vcstool `version` is read as a `ref`, and both are resolved the same way. vcs2l's `extends` key is not supported in M0.

place never writes `.repos` files. A later `place export --format repos` can produce one.

## Repo state

`place status` computes each repo's state from the working copy and the most recent fetch. Nothing is stored between runs.

- **missing**: the path doesn't exist.
- **not a repo**: the path exists but isn't a git checkout.
- **wrong remote**: no remote matches the configured URL.
- **on ref**: HEAD is on the ref's branch, or at its tag or commit.
- **elsewhere**: HEAD is on another branch, or detached away from the ref.
- **dirty**: the working tree has uncommitted changes. Untracked files count, except the checkouts of a nested workplace inside the repo.
- **ahead / behind / diverged**: the ref's branch compared with its remote-tracking branch.
- **default moved**: the repo has no ref, and the remote's default branch is no longer the one it was cloned with. place reports this and never switches branches because of it. The branch it was cloned with is git's `refs/remotes/<remote>/HEAD`, which fetch leaves alone. When place fetches a repo with no ref, it also records the remote's current default branch as the symbolic ref `refs/place/<remote>/HEAD`, so status can compare the two without touching the network.
- **unlisted**: a git checkout directly inside the workplace that no configuration lists.

These combine. A repo can be elsewhere and dirty at the same time.

place records URLs exactly as given, the way Cargo and Go do. Each machine can choose SSH or HTTPS with git's `insteadOf`. URLs are compared after normalization. The comparison ignores the scheme (`https://`, `ssh://` and scp-style `git@host:` are equivalent), the user, a trailing `.git`, a trailing slash, and case. Without this, `MaintainX/maintainx` and `maintainx/maintainx.git` would count as different repositories.

## Commands (M0)

Global flags: `-C <dir>`, `-E/--env <env>`, `-j/--jobs <n>`, `-q/--quiet`, `-v/--verbose`. `-E/--env` goes before the command, as in `place -E ci sync`, because `place add` has its own `-e/--env`.

### place init

Creates an empty `place.toml` in the current directory. It fails if the directory already has place configuration.

### place add

```text
place add <url> [<path>] [-b/--branch <ref>] [-e/--env <env>]
place add <path>...
```

With a URL, `add` clones the repository and adds an entry. The path defaults to the repository name, as in `git clone`. `-b/--branch` takes a branch, a tag or a commit, as `git clone -b` does, and records it as the entry's `ref`.

With an existing path, `add` adopts the checkout. It records the URL of `origin` (or of the only remote) and adds no ref, so the repo tracks the remote's default branch. Adopting a checkout that's currently on a feature branch doesn't record that branch.

Entries go to `place.toml`. `-e/--env <env>` writes to `place.<env>.toml` instead, so `-e local` writes `place.local.toml`, as `mise use --env` does. If `place.toml` doesn't exist yet, as in a workplace that only has `.repos` files, `add` creates it. place edits the file in place and keeps its comments and formatting.

### place remove

`place remove <path>... [--keep] [--force]`, with the alias `rm`. Like `git rm`, it deletes the entry and the checkout. It deletes the checkout only when nothing would be lost:

- the working tree is clean, untracked files included;
- every local branch is contained in some remote-tracking branch;
- there are no stashes.

If any check fails, place removes nothing, neither the entry nor the checkout, and says why. `--keep` removes only the entry and leaves the checkout on disk. `--force` skips the checks. A later `place prune` will delete checkouts that no configuration lists, with the same checks.

If the entry comes from a `.repos` file, `remove` fails, because place doesn't write `.repos` files.

### place status

`place status [<path>...] [--fetch] [--json] [--exit-code]`

Prints the state of every repo, including nested workplaces. By default it doesn't touch the network, like `git status`. `--fetch` fetches first.

`--json` prints one object with a `repos` array. Each repo has its `path` relative to the workplace, its `url` and `ref` (`null` for an unlisted repo, or when there's no ref), and its `states`, named exactly as in [Repo state](#repo-state):

```json
{"repos": [{"path": "place", "url": "https://github.com/onetruename/place.git", "ref": null, "states": ["on ref", "dirty"]}]}
```

The exit code is 0 unless place itself fails. With `--exit-code`, it is 1 when any repo isn't clean and on its ref, like `git diff --exit-code`.

### place sync

`place sync [<path>...]`

For each repo, in parallel:

1. **missing**: clone it and check out the ref.
2. **wrong remote** or **not a repo**: report an error and skip it.
3. Otherwise, fetch. Then, if the repo is on its ref's branch, clean and strictly behind, fast-forward it.
4. If the ref's branch exists locally but isn't checked out, fast-forward that local branch when it's strictly behind. This touches no files in the working tree.
5. Anything else (dirty, elsewhere, ahead, diverged, default moved, a different tag or commit checked out) is reported and left alone. `sync` never changes which branch is checked out.

After a repo is synced, place recurses into it if it is a nested workplace.

The exit code is non-zero if any clone or fetch failed, if any repo had the wrong remote or wasn't a repo, or if a ref's branch had diverged from its remote. Repos that are dirty or elsewhere don't affect the exit code, because that is normal work in progress.

### place doctor

Checks the workplace and exits non-zero when it finds problems:

- every configuration file parses, and every entry has a valid URL;
- the same path defined in a `.repos` file and in `place.toml` (a warning, because the merge is intended but may surprise);
- a `.repos` entry whose type isn't git;
- git is installed and recent enough;
- if the workplace is a git repository, every child checkout is ignored by it (`git check-ignore`), and likewise for each nested workplace;
- unlisted repos (a warning).

doctor checks nested workplaces too, and reports all the checkouts a repository doesn't ignore in one error.

## Recursion

A repo is a nested workplace when its root contains place configuration, as defined under [Concepts](#concepts). `sync` and `status` recurse into nested workplaces by default. A repo can't be listed by two workplaces, because every path lies inside exactly one parent, so no conflict resolution is needed.

Running place inside a nested workplace acts on that workplace and its descendants only. Parent workplaces aren't loaded.

Treating every `*.repos` file as a nested workplace may be too broad. Some repositories ship `.repos` files meant for a different layout. If that comes up, add a per-repo `recursive = false`.

## Implementation

- **Language:** Rust. clap for the CLI, toml_edit for editing `place.toml` without losing formatting, and a maintained YAML parser for `.repos` (serde_yaml is archived).
- **Version control:** one interface covers what place needs from a version control system: clone, fetch, fast-forward, inspect state, delete safely. Git is the only implementation.
- **Git:** shell out to the git CLI. That gives place the user's credential helpers, SSH agent, `insteadOf` rules and hooks for free, which is also why vcstool does it. A library such as gix can speed up read-only `status` later if it proves slow.
- **Concurrency:** a worker pool of `--jobs` threads (default: the number of CPUs), each running git subprocesses.
- **Tests:** unit tests in Rust. Integration tests in cucumber-rs, which run the real binary against temporary directories with local bare repositories as remotes, so they need no network.
- **Distribution:** GitHub releases, a nix flake, and installation through mise (`mise use github:onetruename/place`).

## Later

These are deliberately out of M0. They are listed in rough priority order.

- `foreach`, `config list`, `where`, `prune`, and shell completions.
- `import` and `export` between `place.toml` and `.repos`.
- A shared repo cache, so that short-lived task workplaces don't clone the same repositories again and again. The options are a bare mirror per normalized URL, with checkouts made as clones that borrow objects from it, as worktrees of it, or as copy-on-write copies. All clones in M0 go through one function so the cache can be added there.
- `place lock` writes `place.lock` with exact commits, and `sync --locked` checks them out. The lock file is optional, and the workplace never needs a commit when a child repository changes.
- mise integration, one feature at a time, up to roughly the level fnox reaches:
  - installation through mise;
  - mise's remaining config locations, such as `.config/place.toml`;
  - a mise `enter` hook that runs `place status --quiet`;
  - feeding mise monorepo tasks, so that `mise run //...:test` covers every repo in a workplace. place won't have a task runner of its own.
- Local dependency overrides in the spirit of Amazon's Peru: generate `go.work`, Cargo `[patch]` sections, pnpm overrides and similar from the repos in a workplace, so that each repo builds against its siblings. This probably belongs in a separate tool that reads `place status --json`.
- A `vcs` shim that maps vcstool's CLI onto place for existing scripts.

## Non-goals

- Running tasks or builds. mise does that.
- Managing tools, environment variables or secrets.
- Loading agent context or launching agent harnesses. Claude Code already reads ancestor `CLAUDE.md` files. Anything more belongs in a separate tool.
- Keeping state between runs, other than the optional lock file.

## Open questions

1. How should place adopt every unlisted repo at once, for example `add --all` or an option on `init`? Deferred until after M0.
