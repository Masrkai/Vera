use std::path::Path;
use vera::processing::exiv2_backend::EXIFProcessor;
use vera::processing::geo_utils::GPSConverter;

#[test]
fn test_exif_processor_on_sample_image() {
    let image_path = "images/1.jpg";
    if Path::new(image_path).exists() {
        if let Ok(processor) = EXIFProcessor::new(image_path.to_string()) {
            let extracted = processor.extract();
            assert!(extracted.is_ok());
            let tags = extracted.unwrap();
            
            // Test GPS parsing if GPS info exists
            let gps = GPSConverter::parse_exif_gps(&tags);
            if let Some((lat, lon)) = gps {
                assert!(lat >= -90.0 && lat <= 90.0);
                assert!(lon >= -180.0 && lon <= 180.0);
            }
        }
    }
}
