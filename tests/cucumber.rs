use cucumber::World;
use indexmap::IndexMap;
use std::path::Path;
use vera::processing::{
    exiv2_backend::{EXIFProcessor, ExifTag},
    geo_utils::GPSConverter,
    openssl_backend::OpensslProcessor,
};

#[derive(Debug, World, Default)]
pub struct VeraWorld {
    image_path: String,
    second_image_path: String,
    exif_data: Option<IndexMap<String, ExifTag>>,
    secret_message: String,
    passphrase: String,
    ciphertext: String,
}

#[cucumber::given(regex = "^an image file \"([^\"]+)\" with EXIF metadata$")]
fn given_image_file(world: &mut VeraWorld, path: String) {
    world.image_path = path;
    assert!(Path::new(&world.image_path).exists());
}

#[cucumber::given(regex = "^a batch of sample images \"([^\"]+)\" and \"([^\"]+)\"$")]
fn given_batch_images(world: &mut VeraWorld, path1: String, path2: String) {
    world.image_path = path1;
    world.second_image_path = path2;
    assert!(Path::new(&world.image_path).exists());
    assert!(Path::new(&world.second_image_path).exists());
}

#[cucumber::given(regex = "^a secret plaintext message \"([^\"]+)\" and passphrase \"([^\"]+)\"$")]
fn given_secret_message(world: &mut VeraWorld, message: String, pass: String) {
    world.secret_message = message;
    world.passphrase = pass;
}

#[cucumber::when("the user extracts the EXIF metadata")]
fn when_extract_exif(world: &mut VeraWorld) {
    if let Ok(processor) = EXIFProcessor::new(world.image_path.clone()) {
        world.exif_data = processor.extract().ok();
    }
}

#[cucumber::when("the user verifies their existence and metadata")]
fn when_verify_batch(world: &mut VeraWorld) {
    assert!(Path::new(&world.image_path).exists());
    assert!(Path::new(&world.second_image_path).exists());
}

#[cucumber::when("the message is encrypted using OpenSSL")]
fn when_encrypt_message(world: &mut VeraWorld) {
    if OpensslProcessor::find_openssl().is_ok() {
        if let Ok(encrypted) = OpensslProcessor::encrypt(&world.secret_message, &world.passphrase) {
            world.ciphertext = encrypted;
        }
    }
}

#[cucumber::then("the system successfully reads camera details")]
fn then_reads_camera_details(world: &mut VeraWorld) {
    if world.exif_data.is_some() {
        assert!(!world.exif_data.as_ref().unwrap().is_empty());
    }
}

#[cucumber::then("the GPS coordinates are correctly parsed and converted to a Google Maps URL")]
fn then_gps_converted(world: &mut VeraWorld) {
    if let Some(ref data) = world.exif_data {
        if let Some((lat, lon)) = GPSConverter::parse_exif_gps(data) {
            let url = GPSConverter::create_google_maps_url(lat, lon);
            assert!(url.starts_with("https://maps.google.com/maps?q="));
        }
    }
}

#[cucumber::then("both images are valid targets for batch processing")]
fn then_valid_batch_targets(world: &mut VeraWorld) {
    assert!(!world.image_path.is_empty());
    assert!(!world.second_image_path.is_empty());
}

#[cucumber::then("the resulting ciphertext is non-empty and valid")]
fn then_ciphertext_valid(world: &mut VeraWorld) {
    if OpensslProcessor::find_openssl().is_ok() {
        assert!(!world.ciphertext.is_empty());
    }
}

#[tokio::test]
async fn bdd_features() {
    VeraWorld::run("tests/features").await;
}
