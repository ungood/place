# place features

These files specify place's behavior. Each one describes a command or capability, and cucumber-rs runs them as integration tests against the real binary.

## Writing scenarios

- Every feature starts with a short description of the capability and its rules.
- Every scenario has exactly one `When`, and the first `Then` says whether the command succeeded or failed.
- A failing scenario spells out the error and the help that the user sees. The wording is part of the spec, so review it as user-facing text.
- Reuse the steps below. If a scenario needs a new step, add it here in the same style.

## Remotes

Scenarios use real-looking URLs such as `https://github.com/ungood/place.git`, never placeholders. The test harness gives git a config that rewrites `https://github.com/` and `git@github.com:` to a directory of local bare repositories, so tests never touch the network. A URL with no matching remote fails to clone, the way a missing repository would.

Each remote is built once per run and copied into the scenarios that use it. Every remote starts with one commit on `main`, which is its default branch. Test commits use a fixed author, committer and date, so commit hashes are the same on every run and scenarios can write them out.

## Steps

Setup:

- `a remote repository "<url>"`
- `the remote "<url>" has a branch "<branch>"`, `has a tag "<tag>"`, `has a new commit on "<branch>"`, `has the default branch "<branch>"`
- `the remote "<url>" has the file "<file>":` with the content as a doc string, committed on `main`
- `an empty directory`, `an empty workplace` (a directory with an empty `place.toml`)
- `a workplace with the file "<file>":`, `the workplace has the file "<file>":`
- `the workplace has a plain directory "<dir>"`
- `the workplace is a git repository`
- `the environment variable "<name>" is "<value>"`
- `"<path>" is a clone of "<url>"`
- `"<path>" is on a new branch "<branch>"`
- `"<path>" has an uncommitted change`, `a staged change`, `an untracked file`, `a stash`, `a local commit on "<branch>"`, `a local branch "<branch>" with its own commit`, `a local branch "<branch>" at "<ref>"`
- `the origin of "<path>" is "<url>"`

Action:

- `I run "<command>"` runs in the workplace. `I run "<command>" in "<dir>"` runs in a directory inside it.

Outcome:

- `the command succeeds`, `the command fails`, `the command fails with exit code <n>`
- `the error says:` and `the help says:`, with the expected text as a doc string. Each line must appear in the error or help that place prints, after colors are removed.
- `place warns:` and `the output says:`, matched the same way.
- `place reports "<path>" as "<state>"` and `place doesn't report "<path>" as "<state>"` run `place status --json` without fetching and check the repo's states.
- `"<path>" is a clone of "<url>"`
- `"<path>" is checked out on "<branch>"`, `is detached at the tag "<tag>"`
- `"<path>" is at the remote's latest commit on "<branch>"`, `is not at the remote's latest commit on "<branch>"`
- `the local branch "<branch>" of "<path>" is at the remote's latest commit on "<branch>"`
- `"<path>" still has its uncommitted change`, `still has its local commit`
- `the workplace directory "<dir>" exists`, `does not exist`
- `the workplace file "<file>" is:`, `has no repos`, `lists "<path>"`, `is unchanged`, `does not exist`
