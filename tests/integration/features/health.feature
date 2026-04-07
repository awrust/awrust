Feature: Facade health check

  Scenario: Health endpoint returns service status
    When I check the facade health
    Then the health status should be "ok"
    And the health response should include service "s3" with status "ok"

  Scenario: S3 requests with auth headers bypass facade health
    Given bucket "my-health-bucket" exists
    Then bucket "my-health-bucket" should exist
