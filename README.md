# place

place manages workplaces: directories of independent git repositories, plus the configuration they share. The name completes _mise-en-place_: [mise](https://mise.jdx.dev) sets up what happens inside a directory, and place sets up which directories exist.

Unlike git submodules, each repo keeps its own history, and a change inside one never needs a commit in the workplace. place clones, fetches and fast-forwards. It never switches branches, rebases or stashes.

## Install

With mise:

```sh
mise use packslip:github.com/ungood/place
```

With Nix:

```sh
nix profile install github:ungood/place
```

Or add `github:ungood/place` as a flake input and use `inputs.place.packages.${system}.default`, or `overlays.default` for `pkgs.place`.

Prebuilt archives for Linux and macOS are on the [releases page](https://github.com/ungood/place/releases).

## Usage

List the repos in `place.toml`, keyed by their path in the workplace:

```toml
[repos.place]
url = "https://github.com/ungood/place.git"

[repos.agents]
url = "https://github.com/ungood/agents.git"
ref = "main"
```

Without `ref`, a repo tracks the remote's default branch. place also reads vcstool `.repos` files, and `place.local.toml` for machine-specific repos.

```sh
place init                    # make the current directory a workplace
place add <url> [<path>]      # clone a repo and list it
place sync                    # clone missing repos, fast-forward clean ones
place status                  # show where every repo stands
place remove <path>           # unlist a repo, deleting it only if nothing would be lost
place doctor                  # check the configuration and layout
```

If the workplace is itself a git repository, ignore each checkout in its `.gitignore`. `place doctor` checks that.

Run `place help <command>` for the details.

## Docs

- [Design](docs/design.md): the principles, configuration and behavior, and what's planned.
- [features/](features/): the behavior of each command, as cucumber scenarios.

## Development

mise provides the Rust toolchain and the tasks:

```sh
mise run build
mise run test
mise run lint
```

Every merge to `main` with a `feat:` or `fix:` commit releases a new version. See [AGENTS.md](AGENTS.md) for the conventions.

## License

[MIT](LICENSE)
