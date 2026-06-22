// app_controller.rs

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use slint::{ComponentHandle, Image, Model, ModelRc, SharedString, VecModel, Weak};

// Import the generated types from the root crate (main.rs)
use crate::{ExifDataManager, ExifRow};

use crate::processing::exiv2_backend::{ExifValue, EXIFProcessor};
use crate::processing::geo_utils::GPSConverter;
use crate::dialogs::{open_file_dialog, FileDialogResult};

pub struct ExifBridge {
    ui: Weak<ExifDataManager>,
    current_path: String,
    batch_files: Vec<String>,
    all_exif_rows: Vec<ExifRow>,
    exif_edits: std::collections::HashMap<String, String>,
}

impl ExifBridge {
    pub fn new() -> (Arc<Mutex<Self>>, ExifDataManager) {
        let ui = ExifDataManager::new().unwrap();

        // Initialize Slint Models
        ui.set_batch_model(ModelRc::new(VecModel::<SharedString>::from(Vec::new())));
        ui.set_batch_status_model(ModelRc::new(VecModel::<SharedString>::from(Vec::new())));
        ui.set_exif_model(ModelRc::new(VecModel::<ExifRow>::from(Vec::new())));

        let bridge = Arc::new(Mutex::new(Self {
            ui: ui.as_weak(),
            current_path: String::new(),
            batch_files: Vec::new(),
            all_exif_rows: Vec::new(),
            exif_edits: std::collections::HashMap::new(),
        }));

        // Bind Callbacks from .slint to Rust methods
        {
            let b = bridge.clone();
            ui.on_request_open_image(move || {
                ExifBridge::request_open_image(&b);
            });
        }
        {
            let b = bridge.clone();
            ui.on_request_open_images(move || {
                ExifBridge::request_open_images(&b);
            });
        }
        {
            let b = bridge.clone();
            ui.on_load_image(move |path| {
                ExifBridge::load_image(&b, path.to_string());
            });
        }
        {
            let b = bridge.clone();
            ui.on_clear_batch_list(move || {
                b.lock().unwrap().clear_batch_list();
            });
        }
        {
            let b = bridge.clone();
            ui.on_start_batch_process(move || {
                ExifBridge::start_batch_process(&b);
            });
        }
        {
            let b = bridge.clone();
            ui.on_open_gps_location(move || {
                b.lock().unwrap().open_gps_location();
            });
        }
        {
            let b = bridge.clone();
            ui.on_save_exif_changes(move || {
                ExifBridge::save_exif_changes(&b);
            });
        }
        {
            let b = bridge.clone();
            ui.on_delete_exif(move || {
                ExifBridge::delete_exif(&b);
            });
        }
        {
            let b = bridge.clone();
            ui.on_apply_filter(move |text| {
                b.lock().unwrap().apply_filter(text.to_string());
            });
        }
        {
            let b = bridge.clone();
            ui.on_exif_value_edited(move |index, new_value| {
                b.lock().unwrap().exif_value_edited(index as usize, new_value.to_string());
            });
        }

        bridge.lock().unwrap().set_status("Ready");

        // Return both the bridge and the strong UI reference
        (bridge, ui)
    }

    pub fn run(this: &Arc<Mutex<Self>>, ui: &ExifDataManager) {
        // `this` is held by main until run() returns, keeping the bridge alive.
        // `ui` is the strong reference that keeps the Slint component alive.
        let _ = this;
        ui.run().unwrap();
    }

    fn ui(&self) -> ExifDataManager {
        self.ui.upgrade().expect("UI was dropped")
    }

    // ── Cross-thread UI helpers ───────────────────────────────────────
    fn run_on_ui_thread<F>(f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let _ = slint::invoke_from_event_loop(f);
    }

    // ── Status ─────────────────────────────────────────────────────────
    fn set_status(&self, msg: &str) {
        self.ui().set_status_message(msg.into());
    }

    // ── File dialogs ───────────────────────────────────────────────────
    fn request_open_image(this: &Arc<Mutex<Self>>) {
        match open_file_dialog(false, None) {
            Ok(FileDialogResult::Single(Some(path))) => {
                ExifBridge::load_image(this, path);
            }
            Ok(_) => {}
            Err(e) => this.lock().unwrap().set_status(&e.to_string()),
        }
    }

    fn request_open_images(this: &Arc<Mutex<Self>>) {
        match open_file_dialog(true, None) {
            Ok(FileDialogResult::Multiple(paths)) => {
                if !paths.is_empty() {
                    this.lock().unwrap().add_batch_files(paths);
                }
            }
            Ok(_) => {}
            Err(e) => this.lock().unwrap().set_status(&e.to_string()),
        }
    }

    // ── Single image ───────────────────────────────────────────────────
    fn load_image(this: &Arc<Mutex<Self>>, path: String) {
        let mut guard = this.lock().unwrap();
        if path.is_empty() || !Path::new(&path).exists() {
            guard.set_status("Invalid or missing file path");
            return;
        }

        guard.current_path = path.clone();
        guard.ui().set_current_file_name(
            Path::new(&path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default()
                .into(),
        );

        let img = Image::load_from_path(Path::new(&path)).unwrap_or_else(|_| Image::default());
        guard.ui().set_preview_source(img);
        drop(guard);

        ExifBridge::extract_exif(this);
    }

    fn extract_exif(this: &Arc<Mutex<Self>>) {
        let mut guard = this.lock().unwrap();
        if guard.current_path.is_empty() {
            guard.set_status("No image selected");
            return;
        }

        guard.exif_edits.clear();
        guard.set_status("Extracting EXIF...");
        let path = guard.current_path.clone();
        drop(guard);

        let b = this.clone();
        thread::spawn(move || {
            let processor = match EXIFProcessor::new(path.clone()) {
                Ok(p) => p,
                Err(e) => {
                    let err_msg = e.to_string();
                    ExifBridge::run_on_ui_thread(move || {
                        b.lock().unwrap().set_status(&format!("Error: {}", err_msg));
                    });
                    return;
                }
            };

            match processor.extract() {
                Ok(data) => {
                    let common_tags = std::collections::HashSet::from([
                        "Exif.Image.Make",
                        "Exif.Image.Model",
                        "Exif.Image.DateTime",
                        "Exif.Photo.DateTimeOriginal",
                        "Exif.Photo.ExposureTime",
                        "Exif.Photo.FNumber",
                        "Exif.Photo.ISOSpeedRatings",
                        "Exif.Photo.FocalLength",
                        "Exif.Image.Orientation",
                    ]);

                    let rows: Vec<ExifRow> = data.iter().map(|(tag, info)| {
                        let mut val_str = info.value.replace('\0', "");
                        const MAX_DISPLAY_LEN: usize = 300;
                        if val_str.chars().count() > MAX_DISPLAY_LEN {
                            let truncated: String = val_str.chars().take(MAX_DISPLAY_LEN).collect();
                            val_str = format!("{truncated}… [{} more chars]", val_str.chars().count() - MAX_DISPLAY_LEN);
                        }

                        let is_common = common_tags.contains(tag.as_str()) || tag.starts_with("Exif.GPSInfo.");
                        ExifRow {
                            tag: tag.into(),
                            value: val_str.into(),
                            is_common,
                        }
                    }).collect();

                    let coords = GPSConverter::parse_exif_gps(&data);
                    let has_gps = coords.is_some();
                    let count = data.len();

                    let b2 = b.clone();
                    ExifBridge::run_on_ui_thread(move || {
                        let mut bridge = b2.lock().unwrap();
                        bridge.all_exif_rows = rows.clone();
                        bridge.ui().set_exif_model(ModelRc::new(VecModel::from(rows)));
                        bridge.ui().set_has_gps(has_gps);
                        bridge.set_status(&format!("EXIF extracted ({}) tags", count));
                    });
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    ExifBridge::run_on_ui_thread(move || {
                        b.lock().unwrap().set_status(&format!("Error: {}", err_msg));
                    });
                }
            }
        });
    }

    // ── Editing ────────────────────────────────────────────────────────
    fn exif_value_edited(&mut self, index: usize, new_value: String) {
        if index < self.all_exif_rows.len() {
            let tag = self.all_exif_rows[index].tag.to_string();
            self.exif_edits.insert(tag, new_value.replace('\0', ""));
        }
    }

    fn overlay_edits(&self, rows: &[ExifRow]) -> Vec<ExifRow> {
        rows.iter().map(|row| {
            let tag = row.tag.to_string();
            if let Some(edited_val) = self.exif_edits.get(&tag) {
                ExifRow {
                    tag: row.tag.clone(),
                    value: edited_val.clone().into(),
                    is_common: row.is_common,
                }
            } else {
                row.clone()
            }
        }).collect()
    }

    fn save_exif_changes(this: &Arc<Mutex<Self>>) {
        let mut guard = this.lock().unwrap();
        if guard.current_path.is_empty() {
            guard.set_status("No image loaded");
            return;
        }

        if guard.exif_edits.is_empty() {
            guard.set_status("No changes to save");
            return;
        }

        let path = guard.current_path.clone();
        let edits = guard.exif_edits.clone();
        let preserve = guard.ui().get_preserve_original();

        guard.set_status("Saving changes...");
        drop(guard);

        let b = this.clone();
        thread::spawn(move || {
            let processor = match EXIFProcessor::new(path.clone()) {
                Ok(p) => p,
                Err(e) => {
                    let err_msg = e.to_string();
                    ExifBridge::run_on_ui_thread(move || {
                        b.lock().unwrap().set_status(&format!("Save error: {}", err_msg));
                    });
                    return;
                }
            };

            let exif_data: std::collections::HashMap<String, ExifValue> = edits.into_iter()
                .map(|(k, v)| (k, ExifValue::Plain(v)))
                .collect();

            match processor.edit(&exif_data, preserve) {
                Ok(result) => {
                    let result_msg = format!(
                        "Saved {} change(s){}",
                        result.applied.len(),
                        if !result.failed.is_empty() {
                            format!(" | {} skipped", result.failed.len())
                        } else {
                            String::new()
                        }
                    );

                    let b2 = b.clone();
                    ExifBridge::run_on_ui_thread(move || {
                        let mut bridge = b2.lock().unwrap();
                        bridge.exif_edits.clear();
                        bridge.set_status(&result_msg);
                        drop(bridge);
                        ExifBridge::extract_exif(&b2);
                    });
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    ExifBridge::run_on_ui_thread(move || {
                        b.lock().unwrap().set_status(&format!("Save error: {}", err_msg));
                    });
                }
            }
        });
    }

    // ── Filtering ──────────────────────────────────────────────────────
    fn apply_filter(&self, text: String) {
        if text.is_empty() {
            let rows = self.overlay_edits(&self.all_exif_rows);
            self.ui().set_exif_model(ModelRc::new(VecModel::from(rows)));
        } else {
            let text_lower = text.to_lowercase();
            let filtered: Vec<ExifRow> = self.all_exif_rows.iter().filter(|r| {
                let tag = r.tag.to_string().to_lowercase();
                let val = r.value.to_string().to_lowercase();
                tag.contains(&text_lower) || val.contains(&text_lower)
            }).cloned().collect();

            let overlayed = self.overlay_edits(&filtered);
            self.ui().set_exif_model(ModelRc::new(VecModel::from(overlayed)));
        }
    }

    // ── GPS ────────────────────────────────────────────────────────────
    fn open_gps_location(&self) {
        if !self.ui().get_has_gps() || self.current_path.is_empty() {
            return;
        }

        let processor = match EXIFProcessor::new(self.current_path.clone()) {
            Ok(p) => p,
            Err(e) => {
                self.set_status(&format!("GPS Error: {}", e));
                return;
            }
        };

        match processor.extract() {
            Ok(data) => {
                if let Some((lat, lon)) = GPSConverter::parse_exif_gps(&data) {
                    let url = GPSConverter::create_google_maps_url(lat, lon);
                    let _ = webbrowser::open(&url);
                }
            }
            Err(e) => self.set_status(&format!("GPS Error: {}", e)),
        }
    }

    // ── Delete ─────────────────────────────────────────────────────────
    fn delete_exif(this: &Arc<Mutex<Self>>) {
        let guard = this.lock().unwrap();
        if guard.current_path.is_empty() {
            return;
        }

        let preserve = guard.ui().get_preserve_original();
        let path = guard.current_path.clone();
        drop(guard);

        let b = this.clone();
        thread::spawn(move || {
            let processor = match EXIFProcessor::new(path.clone()) {
                Ok(p) => p,
                Err(e) => {
                    let err_msg = e.to_string();
                    ExifBridge::run_on_ui_thread(move || {
                        b.lock().unwrap().set_status(&format!("Error: {}", err_msg));
                    });
                    return;
                }
            };

            match processor.delete(preserve) {
                Ok(_) => {
                    let b2 = b.clone();
                    ExifBridge::run_on_ui_thread(move || {
                        b2.lock().unwrap().set_status("EXIF deleted");
                        ExifBridge::extract_exif(&b2);
                    });
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    ExifBridge::run_on_ui_thread(move || {
                        b.lock().unwrap().set_status(&format!("Error: {}", err_msg));
                    });
                }
            }
        });
    }

    // ── Batch processing ───────────────────────────────────────────────
    fn add_batch_files(&mut self, file_paths: Vec<String>) {
        let batch_model = self.ui().get_batch_model();
        if let Some(vec_model) = batch_model.as_any().downcast_ref::<VecModel<SharedString>>() {
            for path in file_paths {
                if !path.is_empty() && !self.batch_files.contains(&path) {
                    self.batch_files.push(path.clone());
                    let basename = Path::new(&path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    vec_model.push(basename.into());
                }
            }
        }
        self.set_status(&format!("Batch list: {} images", self.batch_files.len()));
    }

    fn clear_batch_list(&mut self) {
        self.batch_files.clear();
        self.ui().set_batch_model(ModelRc::new(VecModel::<SharedString>::from(Vec::new())));
        self.set_status("Batch list cleared");
    }

    fn start_batch_process(this: &Arc<Mutex<Self>>) {
        let guard = this.lock().unwrap();
        if guard.batch_files.is_empty() {
            guard.set_status("No images in batch list");
            return;
        }

        guard.ui().set_batch_status_model(ModelRc::new(VecModel::<SharedString>::from(Vec::new())));
        guard.ui().set_batch_progress(0.0);
        guard.set_status("Starting batch process...");

        let preserve = guard.ui().get_batch_preserve_original();
        let files = guard.batch_files.clone();
        drop(guard);

        let b = this.clone();
        thread::spawn(move || {
            let total = files.len();
            for (i, path) in files.iter().enumerate() {
                let status_msg = match EXIFProcessor::new(path.clone()) {
                    Ok(processor) => {
                        match processor.delete(preserve) {
                            Ok(_) => {
                                let basename = Path::new(path)
                                    .file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_default();
                                format!("({}/{}) {}: Deleted", i + 1, total, basename)
                            }
                            Err(e) => {
                                let basename = Path::new(path)
                                    .file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_default();
                                format!("({}/{}) {}: ERROR - {}", i + 1, total, basename, e)
                            }
                        }
                    }
                    Err(e) => {
                        let basename = Path::new(path)
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        format!("({}/{}) {}: ERROR - {}", i + 1, total, basename, e)
                    }
                };

                let progress = (i + 1) as f32 / total as f32;
                let b2 = b.clone();
                ExifBridge::run_on_ui_thread(move || {
                    let bridge = b2.lock().unwrap();
                    let status_model = bridge.ui().get_batch_status_model();
                    if let Some(vec_model) = status_model.as_any().downcast_ref::<VecModel<SharedString>>() {
                        vec_model.push(status_msg.into());
                    }
                    bridge.ui().set_batch_progress(progress);
                });
            }

            let b2 = b.clone();
            ExifBridge::run_on_ui_thread(move || {
                b2.lock().unwrap().set_status("Batch processing finished");
            });
        });
    }
}
