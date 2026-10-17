Feature: place status

  status reports each repo's state from its working copy and the most recent
  fetch. It doesn't touch the network unless --fetch is given.

  Background:
    Given a remote repository "https://github.com/ungood/place.git"
    And a workplace with the file "place.toml":
      """
      [repos.place]
      url = "https://github.com/ungood/place.git"
      """

  Scenario: A repo that hasn't been cloned is missing
    When I run "place status"
    Then the command succeeds
    And place reports "place" as "missing"

  Scenario: A fresh clone is on its ref and clean
    Given "place" is a clone of "https://github.com/ungood/place.git"
    When I run "place status"
    Then the command succeeds
    And place reports "place" as "on ref"
    And place doesn't report "place" as "dirty"

  Scenario Outline: Local changes make a repo dirty
    Given "place" is a clone of "https://github.com/ungood/place.git"
    And "place" has <change>
    When I run "place status"
    Then the command succeeds
    And place reports "place" as "dirty"

    Examples:
      | change                |
      | an uncommitted change |
      | a staged change       |
      | an untracked file     |

  Scenario: A repo on another branch is elsewhere
    Given "place" is a clone of "https://github.com/ungood/place.git"
    And "place" is on a new branch "feature/x"
    When I run "place status"
    Then the command succeeds
    And place reports "place" as "elsewhere"

  Scenario: States combine
    Given "place" is a clone of "https://github.com/ungood/place.git"
    And "place" is on a new branch "feature/x"
    And "place" has an uncommitted change
    When I run "place status"
    Then the command succeeds
    And place reports "place" as "elsewhere"
    And place reports "place" as "dirty"

  Scenario: status doesn't fetch by default
    Given "place" is a clone of "https://github.com/ungood/place.git"
    And the remote "https://github.com/ungood/place.git" has a new commit on "main"
    When I run "place status"
    Then the command succeeds
    And place doesn't report "place" as "behind"

  Scenario: --fetch fetches before reporting
    Given "place" is a clone of "https://github.com/ungood/place.git"
    And the remote "https://github.com/ungood/place.git" has a new commit on "main"
    When I run "place status --fetch"
    Then the command succeeds
    And place reports "place" as "behind"

  Scenario: A local commit makes a repo ahead
    Given "place" is a clone of "https://github.com/ungood/place.git"
    And "place" has a local commit on "main"
    When I run "place status"
    Then the command succeeds
    And place reports "place" as "ahead"

  Scenario: Local and remote commits make a repo diverged
    Given "place" is a clone of "https://github.com/ungood/place.git"
    And "place" has a local commit on "main"
    And the remote "https://github.com/ungood/place.git" has a new commit on "main"
    When I run "place status --fetch"
    Then the command succeeds
    And place reports "place" as "diverged"

  Scenario: A directory that isn't a checkout is reported
    Given the workplace has a plain directory "place"
    When I run "place status"
    Then the command succeeds
    And place reports "place" as "not a repo"

  Scenario: A checkout of a different repository has the wrong remote
    Given "place" is a clone of "https://github.com/ungood/place.git"
    And the origin of "place" is "https://github.com/nobody/other.git"
    When I run "place status"
    Then the command succeeds
    And place reports "place" as "wrong remote"

  Scenario Outline: URLs that differ only in form name the same repository
    Given a workplace with the file "place.toml":
      """
      [repos.place]
      url = "<configured>"
      """
    And "place" is a clone of "https://github.com/ungood/place.git"
    And the origin of "place" is "<origin>"
    When I run "place status"
    Then the command succeeds
    And place doesn't report "place" as "wrong remote"

    Examples:
      | configured                          | origin                              |
      | https://github.com/ungood/place.git | git@github.com:ungood/place.git     |
      | https://github.com/ungood/place     | ssh://git@github.com/ungood/place   |
      | https://github.com/Ungood/Place/    | https://github.com/ungood/place     |
      | git@GitHub.com:ungood/place.git     | https://github.com/ungood/place.git |

  Scenario: A repo whose remote default branch changed reports it
    Given "place" is a clone of "https://github.com/ungood/place.git"
    And the remote "https://github.com/ungood/place.git" has a branch "trunk"
    And the remote "https://github.com/ungood/place.git" has the default branch "trunk"
    When I run "place status --fetch"
    Then the command succeeds
    And place reports "place" as "default moved"
    And "place" is checked out on "main"

  Scenario: Checkouts no configuration lists are unlisted
    Given a remote repository "https://github.com/nobody/stray.git"
    And "stray" is a clone of "https://github.com/nobody/stray.git"
    When I run "place status"
    Then the command succeeds
    And place reports "stray" as "unlisted"

  Scenario: Nested workplaces are included
    Given a remote repository "https://github.com/ungood/agents.git"
    And the remote "https://github.com/ungood/place.git" has the file "place.toml":
      """
      [repos.agents]
      url = "https://github.com/ungood/agents.git"
      """
    And "place" is a clone of "https://github.com/ungood/place.git"
    When I run "place status"
    Then the command succeeds
    And place reports "place/agents" as "missing"

  Scenario: --exit-code fails when any repo isn't clean and on its ref
    Given "place" is a clone of "https://github.com/ungood/place.git"
    And "place" has an uncommitted change
    When I run "place status --exit-code"
    Then the command fails with exit code 1
    And place reports "place" as "dirty"

  Scenario: --exit-code succeeds when every repo is clean and on its ref
    Given "place" is a clone of "https://github.com/ungood/place.git"
    When I run "place status --exit-code"
    Then the command succeeds
