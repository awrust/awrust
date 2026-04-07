Feature: Service routing through facade

  Scenario: Create and list buckets through facade
    When I create bucket "route-bucket"
    Then bucket "route-bucket" should appear in the bucket list

  Scenario: Put and get object through facade
    Given bucket "obj-bucket" exists
    When I put object "obj-bucket/hello.txt" with content "hello facade"
    Then object "obj-bucket/hello.txt" should contain "hello facade"

  Scenario: Multipart upload through facade
    Given bucket "mp-bucket" exists
    When I upload "mp-bucket/big.bin" as multipart with 2 parts
    Then object "mp-bucket/big.bin" should exist

  Scenario: Copy object through facade
    Given bucket "copy-bucket" exists
    And object "copy-bucket/src.txt" contains "source data"
    When I copy "copy-bucket/src.txt" to "copy-bucket/dst.txt"
    Then object "copy-bucket/dst.txt" should contain "source data"

  Scenario: Batch delete through facade
    Given bucket "batch-bucket" exists
    And object "batch-bucket/a.txt" contains "a"
    And object "batch-bucket/b.txt" contains "b"
    When I batch delete "a.txt,b.txt" from "batch-bucket"
    Then object "batch-bucket/a.txt" should not exist
    And object "batch-bucket/b.txt" should not exist
