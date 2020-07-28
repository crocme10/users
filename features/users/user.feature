Feature: Example feature

  Scenario: Initial empty scenario
    When I list users
    Then I have no user in the response

  Scenario: Adding a new user
    When I add alice
    Then I can verify the alice's details in the response

  Scenario: Adding a duplicate user
    When I add alice
    Then I get a duplicate username error

  Scenario: Adding a second user
    When I add bob
    Then I get two users

  Scenario: Searching a user by username
    When I search for bob
    Then I can find 
