Feature: place doctor

  doctor checks a workplace's configuration and layout, and explains each
  problem it finds: what's wrong, where, and how to fix it. Errors fail the
  command. Warnings are reported and don't.

  Background:
    Given a remote repository "https://github.com/ungood/place.git"

  Scenario: A healthy workplace passes
    Given a workplace with the file "place.toml":
      """
      [repos.place]
      url = "https://github.com/ungood/place.git"
      """
    And "place" is a clone of "https://github.com/ungood/place.git"
    When I run "place doctor"
    Then the command succeeds
    And the output says:
      """
      No problems found.
      """

  Scenario: A file that doesn't parse is an error that points at the mistake
    Given a workplace with the file "place.toml":
      """
      [repos.place
      url = "https://github.com/ungood/place.git"
      """
    When I run "place doctor"
    Then the command fails
    And the error says:
      """
      place.toml has a syntax error on line 1.
      """

  Scenario: An entry without a URL is an error
    Given a workplace with the file "place.toml":
      """
      [repos.place]
      ref = "main"
      """
    When I run "place doctor"
    Then the command fails
    And the error says:
      """
      "place" in place.toml has no url.
      """
    And the help says:
      """
      Every repo needs a url, for example: url = "https://github.com/ungood/place.git"
      """

  Scenario: A path defined by a .repos file and place.toml is a warning
    Given a workplace with the file "place.toml":
      """
      [repos.place]
      url = "https://github.com/ungood/place.git"
      """
    And the workplace has the file "deps.repos":
      """
      repositories:
        place:
          type: git
          url: https://github.com/ungood/place.git
      """
    When I run "place doctor"
    Then the command succeeds
    And place warns:
      """
      "place" is listed in both deps.repos and place.toml. The entry in place.toml is used.
      """

  Scenario: An unlisted checkout is a warning
    Given a remote repository "https://github.com/nobody/stray.git"
    And an empty workplace
    And "stray" is a clone of "https://github.com/nobody/stray.git"
    When I run "place doctor"
    Then the command succeeds
    And place warns:
      """
      "stray" is a git checkout that no workplace file lists. Run "place add stray" to list it.
      """

  Scenario: A child checkout the workplace repository doesn't ignore is an error
    Given a workplace with the file "place.toml":
      """
      [repos.place]
      url = "https://github.com/ungood/place.git"
      """
    And the workplace is a git repository
    And "place" is a clone of "https://github.com/ungood/place.git"
    When I run "place doctor"
    Then the command fails
    And the error says:
      """
      The workplace repository doesn't ignore "place", so git shows it as untracked.
      """
    And the help says:
      """
      Add "/place/" to the workplace's .gitignore.
      """

  Scenario: A child checkout the workplace repository ignores passes
    Given a workplace with the file "place.toml":
      """
      [repos.place]
      url = "https://github.com/ungood/place.git"
      """
    And the workplace is a git repository
    And the workplace has the file ".gitignore":
      """
      /place/
      """
    And "place" is a clone of "https://github.com/ungood/place.git"
    When I run "place doctor"
    Then the command succeeds

  Scenario: Every checkout the workplace repository doesn't ignore is in one error
    Given a remote repository "https://github.com/ungood/agents.git"
    And a workplace with the file "place.toml":
      """
      [repos.place]
      url = "https://github.com/ungood/place.git"

      [repos.agents]
      url = "https://github.com/ungood/agents.git"
      """
    And the workplace is a git repository
    And "place" is a clone of "https://github.com/ungood/place.git"
    And "agents" is a clone of "https://github.com/ungood/agents.git"
    When I run "place doctor"
    Then the command fails
    And the error says:
      """
      The workplace repository doesn't ignore "agents" and "place", so git shows them as untracked.
      """
    And the help says:
      """
      Add "/agents/" and "/place/" to the workplace's .gitignore.
      """

  Scenario: A nested workplace's repository must ignore its checkouts too
    Given a remote repository "https://github.com/ungood/agents.git"
    And the remote "https://github.com/ungood/place.git" has the file "place.toml":
      """
      [repos.agents]
      url = "https://github.com/ungood/agents.git"
      """
    And a workplace with the file "place.toml":
      """
      [repos.place]
      url = "https://github.com/ungood/place.git"
      """
    And "place" is a clone of "https://github.com/ungood/place.git"
    And "place/agents" is a clone of "https://github.com/ungood/agents.git"
    When I run "place doctor"
    Then the command fails
    And the error says:
      """
      The repository "place" doesn't ignore "agents", so git shows it as untracked.
      """
    And the help says:
      """
      Add "/agents/" to the .gitignore of "place".
      """
