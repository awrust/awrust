@init
Feature: Init script execution

  Init scripts in /etc/awrust/init/ready.d/ run after services are healthy.
  The health endpoint reports "initializing" until all scripts complete,
  so Docker depends_on with service_healthy waits for provisioning to finish.

  Scenario: Init scripts provision S3 resources on startup
    Then bucket "init-provisioned" should exist
    And object "init-provisioned/greeting.txt" should contain "hello from init"
