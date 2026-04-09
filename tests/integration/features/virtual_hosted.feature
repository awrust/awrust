@virtual-hosted
Feature: Virtual-hosted bucket access

  When AWRUST_BASE_DOMAIN is set without a service prefix,
  virtual-hosted URLs like my-bucket.s3.{base_domain} must
  correctly extract the bucket name by stripping both the
  service prefix and the base domain.

  Scenario: Create and list buckets via virtual-hosted URL
    When I create bucket "vh-bucket"
    Then bucket "vh-bucket" should appear in the bucket list

  Scenario: Put and get object via virtual-hosted URL
    Given bucket "vh-obj" exists
    When I put object "vh-obj/hello.txt" with content "hello vhost"
    Then object "vh-obj/hello.txt" should contain "hello vhost"

  Scenario: Copy object via virtual-hosted URL
    Given bucket "vh-copy" exists
    And object "vh-copy/src.txt" contains "virtual source"
    When I copy "vh-copy/src.txt" to "vh-copy/dst.txt"
    Then object "vh-copy/dst.txt" should contain "virtual source"

  Scenario: Batch delete via virtual-hosted URL
    Given bucket "vh-batch" exists
    And object "vh-batch/a.txt" contains "a"
    And object "vh-batch/b.txt" contains "b"
    When I batch delete "a.txt,b.txt" from "vh-batch"
    Then object "vh-batch/a.txt" should not exist
    And object "vh-batch/b.txt" should not exist
