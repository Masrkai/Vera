use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

use crate::processing::exiv2_backend::{ExifValue, EXIFProcessor};

/// Processes a batch of images.
pub struct BatchProcessor {
    pub file_paths: Vec<String>,
    pub operation: String,
    pub preserve_original: bool,
    pub exif_data: Option<HashMap<String, ExifValue>>,
}

impl BatchProcessor {
    pub fn new(
        file_paths: Vec<String>,
        operation: String,
        preserve_original: bool,
        exif_data: Option<HashMap<String, ExifValue>>,
    ) -> Self {
        Self {
            file_paths,
            operation,
            preserve_original,
            exif_data,
        }
    }

    /// Executes the batch operation.
    ///
    /// `progress_callback` is called once per file with:
    ///   (current_index (1-based), total_files, basename, status_string)
    pub fn run<F>(&self, mut progress_callback: F)
    where
        F: FnMut(usize, usize, &str, &str),
    {
        let total_files = self.file_paths.len();
        for (i, file_path) in self.file_paths.iter().enumerate() {
            let mut status = "Success".to_string();

            let result: Result<(), Box<dyn Error>> = (|| {
                let working_path: String;

                if self.preserve_original {
                    let base_path = Path::new(file_path);
                    let parent = base_path.parent().unwrap_or_else(|| Path::new(""));
                    let stem = base_path
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let suffix = base_path
                        .extension()
                        .map(|e| format!(".{}", e.to_string_lossy()))
                        .unwrap_or_default();
                    let new_path = parent.join(format!("{}_modified{}", stem, suffix));
                    fs::copy(file_path, &new_path)?;
                    working_path = new_path.to_string_lossy().to_string();
                } else {
                    working_path = file_path.clone();
                }

                let processor = EXIFProcessor::new(working_path)?;

                match self.operation.as_str() {
                    "delete" => {
                        // preserve_original=False because backup already handled above
                        processor.delete(false)?;
                    }
                    "edit" => {
                        let data = self.exif_data.as_ref().ok_or_else(|| -> Box<dyn Error> {
                            "No EXIF data provided for edit operation".into()
                        })?;
                        processor.edit(data, false)?;
                    }
                    _ => {}
                }

                Ok(())
            })();

            if let Err(e) = result {
                status = format!("Error: {}", e);
            }

            let basename = Path::new(file_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            progress_callback(i + 1, total_files, &basename, &status);
        }
    }
}
