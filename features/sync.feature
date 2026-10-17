Feature: place sync

  sync clones missing repos, fetches, and fast-forwards. It never changes
  which branch is checked out, and it leaves work in progress alone. It fails
  only for problems that need attention, not for normal work in progress.

  Background:
    Given a remote repository "https://github.com/ungood/place.git"
    And a workplace with the file "place.toml":
      """
      [repos.place]
      url = "https://github.com/ungood/place.git"
      """

  Scenario: Missing repos are cloned onto the remote's default branch
    When I run "place sync"
    Then the command succeeds
    And "place" is a clone of "https://github.com/ungood/place.git"
    And "place" is checked out on "main"

  Scenario Outline: Missing repos are cloned at their ref
    Given the remote "https://github.com/ungood/place.git" has a branch "develop"
    And the remote "https://github.com/ungood/place.git" has a tag "v1.0"
    And the workplace has the file "place.toml":
      """
      [repos.place]
      url = "https://github.com/ungood/place.git"
      ref = "<ref>"
      """
    When I run "place sync"
    Then the command succeeds
    And "place" is <checkout>

    Examples:
      | ref                                      | checkout                   |
      | develop                                  | checked out on "develop"   |
      | refs/heads/develop                       | checked out on "develop"   |
      | v1.0                                     | detached at the tag "v1.0" |
      | fa853e769fdc374139e7677cd78006997bb679bc | detached at the tag "v1.0" |

  Scenario: A clean repo on its ref is fast-forwarded
    Given "place" is a clone of "https://github.com/ungood/place.git"
    And the remote "https://github.com/ungood/place.git" has a new commit on "main"
    When I run "place sync"
    Then the command succeeds
    And "place" is at the remote's latest commit on "main"

  Scenario: A dirty repo is fetched but not moved
    Given "place" is a clone of "https://github.com/ungood/place.git"
    And "place" has an uncommitted change
    And the remote "https://github.com/ungood/place.git" has a new commit on "main"
    When I run "place sync"
    Then the command succeeds
    And "place" still has its uncommitted change
    And place reports "place" as "behind"

  Scenario: A repo on another branch stays there, and its ref's branch is fast-forwarded
    Given "place" is a clone of "https://github.com/ungood/place.git"
    And "place" is on a new branch "feature/x"
    And the remote "https://github.com/ungood/place.git" has a new commit on "main"
    When I run "place sync"
    Then the command succeeds
    And "place" is checked out on "feature/x"
    And the local branch "main" of "place" is at the remote's latest commit on "main"

  Scenario: A repo that's ahead is left alone
    Given "place" is a clone of "https://github.com/ungood/place.git"
    And "place" has a local commit on "main"
    When I run "place sync"
    Then the command succeeds
    And place reports "place" as "ahead"

  Scenario: A diverged repo is left alone and fails the sync
    Given "place" is a clone of "https://github.com/ungood/place.git"
    And "place" has a local commit on "main"
    And the remote "https://github.com/ungood/place.git" has a new commit on "main"
    When I run "place sync"
    Then the command fails
    And the error says:
      """
      "place": "main" has diverged from "origin/main", so place left it alone.
      """
    And the help says:
      """
      Rebase or merge "main" in "place", then run "place sync" again.
      """
    And "place" still has its local commit

  Scenario: A wrong remote fails the sync without stopping other repos
    Given a remote repository "https://github.com/ungood/agents.git"
    And the workplace has the file "place.toml":
      """
      [repos.place]
      url = "https://github.com/ungood/place.git"

      [repos.agents]
      url = "https://github.com/ungood/agents.git"
      """
    And "place" is a clone of "https://github.com/ungood/place.git"
    And the origin of "place" is "https://github.com/nobody/other.git"
    When I run "place sync"
    Then the command fails
    And the error says:
      """
      "place" is a checkout of https://github.com/nobody/other.git, but place.toml expects https://github.com/ungood/place.git.
      """
    And the help says:
      """
      Point its origin at the expected URL with "git -C place remote set-url origin https://github.com/ungood/place.git", or change the entry in place.toml.
      """
    And "agents" is a clone of "https://github.com/ungood/agents.git"

  Scenario: A clone that fails fails the sync
    Given the workplace has the file "place.toml":
      """
      [repos.missing]
      url = "https://github.com/nobody/missing.git"
      """
    When I run "place sync"
    Then the command fails
    And the error says:
      """
      Couldn't clone "missing" from https://github.com/nobody/missing.git.
      """
    And the help says:
      """
      Check the URL and that you have access to it. git's own message is below.
      """

  Scenario: sync doesn't follow a moved default branch
    Given "place" is a clone of "https://github.com/ungood/place.git"
    And the remote "https://github.com/ungood/place.git" has a branch "trunk"
    And the remote "https://github.com/ungood/place.git" has the default branch "trunk"
    When I run "place sync"
    Then the command succeeds
    And "place" is checked out on "main"
    And place reports "place" as "default moved"

  Scenario: sync recurses into nested workplaces
    Given a remote repository "https://github.com/ungood/agents.git"
    And the remote "https://github.com/ungood/place.git" has the file "place.toml":
      """
      [repos.agents]
      url = "https://github.com/ungood/agents.git"
      """
    When I run "place sync"
    Then the command succeeds
    And "place/agents" is a clone of "https://github.com/ungood/agents.git"

  Scenario: Unlisted checkouts are left alone
    Given a remote repository "https://github.com/nobody/stray.git"
    And "stray" is a clone of "https://github.com/nobody/stray.git"
    And the remote "https://github.com/nobody/stray.git" has a new commit on "main"
    When I run "place sync"
    Then the command succeeds
    And "stray" is not at the remote's latest commit on "main"
