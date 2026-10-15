Feature: EXIF Metadata Extraction and Geolocation
  As a forensic investigator or user
  I want to view image metadata and location coordinates
  So that I can verify where and when an image was taken

  Scenario: Extracting metadata and converting GPS coordinates
    Given an image file "images/1.jpg" with EXIF metadata
    When the user extracts the EXIF metadata
    Then the system successfully reads camera details
    And the GPS coordinates are correctly parsed and converted to a Google Maps URL
