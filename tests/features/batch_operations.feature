Feature: Batch Metadata Operations
  As a user with multiple images
  I want to process multiple files simultaneously
  So that I can efficiently clean or update metadata in batch

  Scenario: Batch checking sample images
    Given a batch of sample images "images/1.jpg" and "images/2.jpg"
    When the user verifies their existence and metadata
    Then both images are valid targets for batch processing
