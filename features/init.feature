Feature: place init

  init turns the current directory into a workplace by creating an empty
  place.toml. It refuses a directory that's already a workplace, so it never
  overwrites configuration.

  Scenario: init creates an empty place.toml
    Given an empty directory
    When I run "place init"
    Then the command succeeds
    And the workplace file "place.toml" has no repos

  Scenario: init refuses to overwrite an existing place.toml
    Given a workplace with the file "place.toml":
      """
      [repos.place]
      url = "https://github.com/ungood/place.git"
      """
    When I run "place init"
    Then the command fails
    And the error says:
      """
      This directory is already a workplace: it has a place.toml.
      """
    And the help says:
      """
      Use "place add" to list more repos in it.
      """
    And the workplace file "place.toml" is unchanged

  Scenario: init refuses a directory that already has a .repos file
    Given a workplace with the file "deps.repos":
      """
      repositories: {}
      """
    When I run "place init"
    Then the command fails
    And the error says:
      """
      This directory is already a workplace: it has deps.repos.
      """
    And the help says:
      """
      place reads deps.repos as it is. "place add" creates place.toml when it needs to.
      """
    And the workplace file "place.toml" does not exist
