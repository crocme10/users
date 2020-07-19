Feature: Example feature

  Scenario: An example scenario
    Given I have seeded the user database
    When I list users
    Then I have as many users in the database as in the response

  Scenario: Adding a new user
    Given I have seeded the user database
    When I add a new user
    Then I can verify the user has been succesfully added
