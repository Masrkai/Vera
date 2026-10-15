use vera::processing::geo_utils::GPSConverter;

#[test]
fn test_decimal_degrees_formatting() {
    let formatted = GPSConverter::to_decimal_degrees(40.7128, -74.0060);
    assert_eq!(formatted, "40.712800, -74.006000");
}

#[test]
fn test_google_maps_url() {
    let url = GPSConverter::create_google_maps_url(40.7128, -74.0060);
    assert_eq!(url, "https://maps.google.com/maps?q=40.712800,-74.006000");
}

#[test]
fn test_esri_satellite_url() {
    let url = GPSConverter::create_esri_satellite_url(40.7128, -74.0060);
    assert_eq!(url, "https://livingatlas.arcgis.com/wayback/#mapCenter=-74.006000,40.712800,18");
}

#[test]
fn test_geohash() {
    // New York City coordinates roughly
    let geohash = GPSConverter::to_geohash_with_precision(40.7128, -74.0060, 6);
    assert!(!geohash.is_empty());
    assert_eq!(geohash.len(), 6);
}

#[test]
fn test_dms_conversion() {
    let dms = GPSConverter::to_dms(40.0, -74.0);
    assert!(dms.contains("40°0'0.00\"N"));
    assert!(dms.contains("74°0'0.00\"W"));
}
