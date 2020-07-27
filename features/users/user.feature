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
