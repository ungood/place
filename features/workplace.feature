Feature: Workplace configuration

  A workplace is the directory a command runs in, or the one given with -C.
  place reads *.repos files, place.toml, place.<env>.toml, place.local.toml
  and place.<env>.local.toml there, from lowest to highest precedence, and
  merges them per repo path. Environment files are read only for the
  environments selected with -E or PLACE_ENV.

  Background:
    Given a remote repository "https://github.com/ungood/place.git"
    And a remote repository "https://github.com/nobody/place.git"
    And a remote repository "https://github.com/ungood/agents.git"

  Scenario: A directory without configuration is not a workplace
    Given an empty directory
    When I run "place status"
    Then the command fails
    And the error says:
      """
      This directory isn't a workplace.
      """
    And the help says:
      """
      A workplace has a place.toml or a .repos file. Run "place init" to create one, or use -C <dir> to run place in another directory.
      """

  Scenario: Commands don't search parent directories, but the help points to the workplace
    Given a workplace with the file "place.toml":
      """
      [repos.place]
      url = "https://github.com/ungood/place.git"
      """
    And "place" is a clone of "https://github.com/ungood/place.git"
    When I run "place status" in "place"
    Then the command fails
    And the error says:
      """
      This directory isn't a workplace.
      """
    And the help says:
      """
      It's inside the workplace at "..". Run place there, or use "place -C ..".
      """

  Scenario: -C runs a command in another directory
    Given a workplace with the file "place.toml":
      """
      [repos.place]
      url = "https://github.com/ungood/place.git"
      """
    And "place" is a clone of "https://github.com/ungood/place.git"
    When I run "place -C .. status" in "place"
    Then the command succeeds
    And place reports "place" as "on ref"

  Scenario: place.local.toml replaces a whole entry from place.toml
    Given the remote "https://github.com/ungood/place.git" has a branch "develop"
    And a workplace with the file "place.toml":
      """
      [repos.place]
      url = "https://github.com/ungood/place.git"
      ref = "develop"
      """
    And the workplace has the file "place.local.toml":
      """
      [repos.place]
      url = "https://github.com/nobody/place.git"
      """
    When I run "place sync"
    Then the command succeeds
    And "place" is a clone of "https://github.com/nobody/place.git"
    And "place" is checked out on "main"

  Scenario: An environment file is ignored unless its environment is selected
    Given a workplace with the file "place.toml":
      """
      [repos.place]
      url = "https://github.com/ungood/place.git"
      """
    And the workplace has the file "place.ci.toml":
      """
      [repos.agents]
      url = "https://github.com/ungood/agents.git"
      """
    When I run "place sync"
    Then the command succeeds
    And the workplace directory "agents" does not exist

  Scenario: -E selects an environment file
    Given a workplace with the file "place.toml":
      """
      [repos.place]
      url = "https://github.com/ungood/place.git"
      """
    And the workplace has the file "place.ci.toml":
      """
      [repos.agents]
      url = "https://github.com/ungood/agents.git"
      """
    When I run "place -E ci sync"
    Then the command succeeds
    And "place" is a clone of "https://github.com/ungood/place.git"
    And "agents" is a clone of "https://github.com/ungood/agents.git"

  Scenario: PLACE_ENV selects environments like -E
    Given a workplace with the file "place.ci.toml":
      """
      [repos.agents]
      url = "https://github.com/ungood/agents.git"
      """
    And the environment variable "PLACE_ENV" is "ci"
    When I run "place sync"
    Then the command succeeds
    And "agents" is a clone of "https://github.com/ungood/agents.git"

  Scenario: place.local.toml wins over an environment file
    Given a workplace with the file "place.ci.toml":
      """
      [repos.place]
      url = "https://github.com/ungood/place.git"
      """
    And the workplace has the file "place.local.toml":
      """
      [repos.place]
      url = "https://github.com/nobody/place.git"
      """
    When I run "place -E ci sync"
    Then the command succeeds
    And "place" is a clone of "https://github.com/nobody/place.git"

  Scenario: Repos are read from a .repos file
    Given a workplace with the file "deps.repos":
      """
      repositories:
        vendor/place:
          type: git
          url: https://github.com/ungood/place.git
          version: main
      """
    When I run "place sync"
    Then the command succeeds
    And "vendor/place" is checked out on "main"

  Scenario: place.toml wins over a .repos file for the same path
    Given a workplace with the file "deps.repos":
      """
      repositories:
        place:
          type: git
          url: https://github.com/ungood/place.git
      """
    And the workplace has the file "place.toml":
      """
      [repos.place]
      url = "https://github.com/nobody/place.git"
      """
    When I run "place sync"
    Then the command succeeds
    And "place" is a clone of "https://github.com/nobody/place.git"

  Scenario: .repos files are read in lexicographic order
    Given a workplace with the file "a.repos":
      """
      repositories:
        place:
          type: git
          url: https://github.com/ungood/place.git
      """
    And the workplace has the file "b.repos":
      """
      repositories:
        place:
          type: git
          url: https://github.com/nobody/place.git
      """
    When I run "place sync"
    Then the command succeeds
    And "place" is a clone of "https://github.com/nobody/place.git"

  Scenario: A .repos entry that isn't git is rejected
    Given a workplace with the file "deps.repos":
      """
      repositories:
        tools/legacy:
          type: svn
          url: https://svn.example.com/legacy
      """
    When I run "place status"
    Then the command fails
    And the error says:
      """
      deps.repos lists "tools/legacy" as an svn repository, and place only supports git.
      """
    And the help says:
      """
      Remove the entry from deps.repos, or manage that checkout with another tool.
      """
