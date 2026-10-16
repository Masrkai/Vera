use std::path::Path;
use vera::processing::exiv2_backend::EXIFProcessor;

#[test]
fn test_batch_sample_images_validation() {
    let image1 = "images/1.jpg";
    let image2 = "images/2.jpg";

    if Path::new(image1).exists() && Path::new(image2).exists() {
        let processor1 = EXIFProcessor::new(image1.to_string());
        let processor2 = EXIFProcessor::new(image2.to_string());

        assert!(processor1.is_ok(), "Failed to initialize EXIFProcessor for image 1");
        assert!(processor2.is_ok(), "Failed to initialize EXIFProcessor for image 2");

        let extracted1 = processor1.unwrap().extract();
        let extracted2 = processor2.unwrap().extract();

        assert!(extracted1.is_ok(), "Failed to extract metadata from image 1");
        assert!(extracted2.is_ok(), "Failed to extract metadata from image 2");
    }
}
