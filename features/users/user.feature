Feature: User feature

  Scenario: Initial empty database
    Given I have initialized the user database
    When I list users
    Then the response's users count is 0

  Scenario: Adding a new user
    Given I have initialized the user database
    When I add a new user with username <username> and email <email>
    Then I can verify the username <username> in the response

    Examples:
      | username | email            |
      | alice    | alice@secret.org |

  Scenario: Adding a duplicate user
    Given I have a user with username <username> and email <email>
    When I add a new user with username <username> and email <email>
    Then I get a duplicate username error

    Examples:
      | username | email            |
      | alice    | alice@secret.org |

  Scenario: Adding a second user
    Given I have a user with username <username0> and email <email0>
    When I add a new user with username <username1> and email <email1>
    And I list users
    Then I can verify the response's users count is 2

    Examples:
      | username0 | email0           | username1 | email1           |
      | alice     | alice@secret.org | bob       | bob@secret.org   |

  Scenario: Searching a user by username
    Given I have a user with username <username0> and email <email0>
    And I have a user with username <username1> and email <email1>
    When I search for a user with username <username1>
    Then I can verify the username <username1> in the response


    Examples:
      | username0 | email0           | username1 | email1           |
      | alice     | alice@secret.org | bob       | bob@secret.org   |


