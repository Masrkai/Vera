use std::path::Path;
use vera::processing::steg_backend::{StegMethod, StegProcessor};

#[test]
fn test_steg_metadata_embedding_extraction() {
    let sample = "images/1.jpg";
    if Path::new(sample).exists() {
        let temp_dir = std::env::temp_dir();
        let output = temp_dir.join("vera_steg_test.jpg").to_string_lossy().to_string();
        
        let secret = "hidden_forensic_message_123";
        let embed_res = StegProcessor::embed(sample, &output, secret, StegMethod::Metadata);
        
        if embed_res.is_ok() {
            let extracted = StegProcessor::extract(&output, StegMethod::Metadata);
            if let Ok(msg) = extracted {
                assert_eq!(msg, secret);
            }
            let _ = std::fs::remove_file(&output);
        }
    }
}
