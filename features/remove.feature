Feature: place remove

  remove deletes a repo's entry and its checkout, but only when nothing would
  be lost. When a check fails, it removes nothing and explains what it found.

  Background:
    Given a remote repository "https://github.com/ungood/place.git"
    And a workplace with the file "place.toml":
      """
      [repos.place]
      url = "https://github.com/ungood/place.git"
      """
    And "place" is a clone of "https://github.com/ungood/place.git"

  Scenario: Removing a clean repo deletes the entry and the checkout
    When I run "place remove place"
    Then the command succeeds
    And the workplace directory "place" does not exist
    And the workplace file "place.toml" has no repos

  Scenario: rm is an alias for remove
    When I run "place rm place"
    Then the command succeeds
    And the workplace directory "place" does not exist

  Scenario Outline: Nothing is removed when work would be lost
    Given "place" has <work>
    When I run "place remove place"
    Then the command fails
    And the error says:
      """
      "place" has <found>, so place didn't remove it.
      """
    And the help says:
      """
      Push or discard that work first. "place remove place --keep" removes only the entry, and --force deletes the checkout anyway.
      """
    And the workplace directory "place" exists
    And the workplace file "place.toml" lists "place"

    Examples:
      | work                                       | found                                        |
      | an uncommitted change                      | uncommitted changes                          |
      | an untracked file                          | untracked files                              |
      | a local commit on "main"                   | commits on "main" that aren't on any remote  |
      | a local branch "spike" with its own commit | commits on "spike" that aren't on any remote |
      | a stash                                    | stashed changes                              |

  Scenario: A local branch whose commits are all pushed doesn't block removal
    Given "place" has a local branch "spike" at "main"
    When I run "place remove place"
    Then the command succeeds
    And the workplace directory "place" does not exist

  Scenario: --keep removes only the entry
    Given "place" has an uncommitted change
    When I run "place remove place --keep"
    Then the command succeeds
    And the workplace directory "place" exists
    And the workplace file "place.toml" has no repos

  Scenario: --force removes even when work would be lost
    Given "place" has an uncommitted change
    When I run "place remove place --force"
    Then the command succeeds
    And the workplace directory "place" does not exist

  Scenario: An entry is removed from the file that defines it
    Given a remote repository "https://github.com/ungood/agents.git"
    And the workplace has the file "place.local.toml":
      """
      [repos.agents]
      url = "https://github.com/ungood/agents.git"
      """
    And "agents" is a clone of "https://github.com/ungood/agents.git"
    When I run "place remove agents"
    Then the command succeeds
    And the workplace file "place.local.toml" has no repos
    And the workplace file "place.toml" lists "place"

  Scenario: Entries from .repos files can't be removed
    Given a remote repository "https://github.com/ungood/agents.git"
    And the workplace has the file "deps.repos":
      """
      repositories:
        agents:
          type: git
          url: https://github.com/ungood/agents.git
      """
    And "agents" is a clone of "https://github.com/ungood/agents.git"
    When I run "place remove agents"
    Then the command fails
    And the error says:
      """
      "agents" is listed in deps.repos, and place doesn't edit .repos files.
      """
    And the help says:
      """
      Delete the entry from deps.repos yourself, then delete the checkout.
      """
    And the workplace directory "agents" exists

  Scenario: Removing the first entry leaves the rest of the file as it was
    Given a remote repository "https://github.com/ungood/agents.git"
    And the workplace has the file "place.toml":
      """
      [repos.place]
      url = "https://github.com/ungood/place.git"

      [repos.agents]
      url = "https://github.com/ungood/agents.git"
      """
    When I run "place remove place"
    Then the command succeeds
    And the workplace file "place.toml" is:
      """
      [repos.agents]
      url = "https://github.com/ungood/agents.git"
      """

  Scenario: Directories left empty by a removed checkout are deleted
    Given the workplace has the file "place.toml":
      """
      [repos."vendor/place"]
      url = "https://github.com/ungood/place.git"
      """
    And "vendor/place" is a clone of "https://github.com/ungood/place.git"
    When I run "place remove vendor/place"
    Then the command succeeds
    And the workplace directory "vendor" does not exist
