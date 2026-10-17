Feature: place add

  add lists a repo in the workplace. Given a URL, it clones the repo and
  records the URL as given. Given the path of a checkout that's already in the
  workplace, it adopts that checkout.

  Background:
    Given a remote repository "https://github.com/ungood/place.git"
    And the remote "https://github.com/ungood/place.git" has a branch "develop"
    And an empty workplace

  Scenario: Adding a URL clones the repo and records the URL as given
    When I run "place add https://github.com/ungood/place.git"
    Then the command succeeds
    And "place" is a clone of "https://github.com/ungood/place.git"
    And "place" is checked out on "main"
    And the workplace file "place.toml" is:
      """
      [repos.place]
      url = "https://github.com/ungood/place.git"
      """

  Scenario: The path can be given explicitly
    When I run "place add https://github.com/ungood/place.git vendor/place"
    Then the command succeeds
    And "vendor/place" is a clone of "https://github.com/ungood/place.git"
    And the workplace file "place.toml" is:
      """
      [repos."vendor/place"]
      url = "https://github.com/ungood/place.git"
      """

  Scenario Outline: --branch records the ref and checks it out
    When I run "place add https://github.com/ungood/place.git <flag> develop"
    Then the command succeeds
    And "place" is checked out on "develop"
    And the workplace file "place.toml" is:
      """
      [repos.place]
      url = "https://github.com/ungood/place.git"
      ref = "develop"
      """

    Examples:
      | flag     |
      | --branch |
      | -b       |

  Scenario Outline: --env writes to an environment file
    When I run "place add https://github.com/ungood/place.git <flag> <env>"
    Then the command succeeds
    And the workplace file "<file>" is:
      """
      [repos.place]
      url = "https://github.com/ungood/place.git"
      """
    And the workplace file "place.toml" has no repos

    Examples:
      | flag  | env   | file             |
      | --env | local | place.local.toml |
      | -e    | local | place.local.toml |
      | --env | ci    | place.ci.toml    |

  Scenario: add keeps the comments and formatting of place.toml
    Given the workplace has the file "place.toml":
      """
      # Tools for the software factory.

      [repos.agents]   # keep me
      url = "https://github.com/ungood/agents.git"
      """
    When I run "place add https://github.com/ungood/place.git"
    Then the command succeeds
    And the workplace file "place.toml" is:
      """
      # Tools for the software factory.

      [repos.agents]   # keep me
      url = "https://github.com/ungood/agents.git"

      [repos.place]
      url = "https://github.com/ungood/place.git"
      """

  Scenario: Adopting a checkout records its origin without a ref
    Given "place" is a clone of "https://github.com/ungood/place.git"
    And "place" is on a new branch "feature/x"
    When I run "place add place"
    Then the command succeeds
    And "place" is checked out on "feature/x"
    And the workplace file "place.toml" is:
      """
      [repos.place]
      url = "https://github.com/ungood/place.git"
      """

  Scenario: Adopting a directory that isn't a git checkout fails
    Given the workplace has a plain directory "notes"
    When I run "place add notes"
    Then the command fails
    And the error says:
      """
      "notes" isn't a git checkout, so there's nothing to adopt.
      """
    And the help says:
      """
      To clone a repo there instead, move "notes" aside and run "place add <url> notes".
      """
    And the workplace file "place.toml" has no repos

  Scenario: A path that's already listed can't be added again
    Given the workplace has the file "place.toml":
      """
      [repos.place]
      url = "https://github.com/ungood/place.git"
      """
    When I run "place add https://github.com/ungood/place.git"
    Then the command fails
    And the error says:
      """
      "place" is already listed in place.toml.
      """
    And the help says:
      """
      To clone it again under another name, run "place add <url> <path>".
      """
    And the workplace file "place.toml" is unchanged

  Scenario: Cloning into an existing directory fails
    Given the workplace has a plain directory "place"
    When I run "place add https://github.com/ungood/place.git"
    Then the command fails
    And the error says:
      """
      Can't clone into "place": that directory already exists.
      """
    And the help says:
      """
      Choose another path with "place add <url> <path>". If "place" is a checkout, "place add place" adopts it.
      """
    And the workplace file "place.toml" has no repos

  Scenario: A path outside the workplace can't be adopted
    When I run "place add ../place"
    Then the command fails
    And the error says:
      """
      "../place" is outside the workplace.
      """
    And the help says:
      """
      place add lists checkouts inside the workplace. To clone a repo from a local path, give it as a URL, such as file:///path/to/repo.git.
      """
    And the workplace file "place.toml" has no repos
