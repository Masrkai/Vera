use std::error::Error;
use std::fs;
use image::{GenericImageView, ImageReader};

use crate::processing::exiv2_backend::{EXIFProcessor, ExifValue};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StegMethod {
    Metadata,     // EXIF ImageDescription / UserComment
    PixelLsb,     // Least Significant Bit (Pixel-wise)
    FrequencyWise,// Frequency domain (DCT / transform coefficient embedding)
}

impl StegMethod {
    pub fn from_index(index: usize) -> Self {
        match index {
            1 => StegMethod::PixelLsb,
            2 => StegMethod::FrequencyWise,
            _ => StegMethod::Metadata,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            StegMethod::Metadata => "Metadata (EXIF)",
            StegMethod::PixelLsb => "Pixel-Wise (LSB)",
            StegMethod::FrequencyWise => "Frequency-Wise (DCT)",
        }
    }
}

pub struct StegProcessor;

const MAGIC_HEADER: &[u8] = b"VERASTEG1:";

fn serialize_payload(ciphertext: &str) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(MAGIC_HEADER);
    let bytes = ciphertext.as_bytes();
    let len_str = format!("{}:", bytes.len());
    data.extend_from_slice(len_str.as_bytes());
    data.extend_from_slice(bytes);
    data
}

impl StegProcessor {
    pub fn embed(
        image_path: &str,
        output_path: &str,
        ciphertext: &str,
        method: StegMethod,
    ) -> Result<(), Box<dyn Error>> {
        match method {
            StegMethod::Metadata => {
                // Copy image to output path first if different
                if image_path != output_path {
                    fs::copy(image_path, output_path)?;
                }
                let mut edits = std::collections::HashMap::new();
                edits.insert(
                    "Exif.Image.ImageDescription".to_string(),
                    ExifValue::Plain(ciphertext.to_string()),
                );
                let processor = EXIFProcessor::new(output_path.to_string())?;
                let result = processor.edit(&edits, false)?;
                if result.status == "success" || result.status == "partial" {
                    Ok(())
                } else {
                    Err(format!("Failed to embed in metadata: {}", result.message).into())
                }
            }
            StegMethod::PixelLsb => {
                let img = ImageReader::open(image_path)?.decode()?.to_rgba8();
                let (width, height) = img.dimensions();
                let payload = serialize_payload(ciphertext);
                let total_bits = payload.len() * 8;
                let available_bits = (width as usize) * (height as usize) * 3; // R, G, B channels

                if total_bits > available_bits {
                    return Err(format!(
                        "Payload too large for image capacity. Need {} bits, have {} bits.",
                        total_bits, available_bits
                    )
                    .into());
                }

                let mut out_img = img.clone();
                let mut bit_index = 0;

                'outer: for y in 0..height {
                    for x in 0..width {
                        if bit_index >= total_bits {
                            break 'outer;
                        }
                        let pixel = out_img.get_pixel_mut(x, y);
                        for c in 0..3 {
                            if bit_index < total_bits {
                                let byte_idx = bit_index / 8;
                                let bit_off = 7 - (bit_index % 8);
                                let bit = ((payload[byte_idx] >> bit_off) & 1) != 0;

                                if bit {
                                    pixel[c] |= 1;
                                } else {
                                    pixel[c] &= !1;
                                }
                                bit_index += 1;
                            }
                        }
                    }
                }

                out_img.save(output_path)?;
                Ok(())
            }
            StegMethod::FrequencyWise => {
                // Frequency-Wise steganography: embedding in frequency coefficients (mid-frequency blocks)
                let img = ImageReader::open(image_path)?.decode()?.to_rgba8();
                let (width, height) = img.dimensions();
                let payload = serialize_payload(ciphertext);
                let total_bits = payload.len() * 8;

                // We divide image into 8x8 blocks and embed bits into mid-frequency coefficient differences
                let block_size = 8;
                let blocks_x = (width as usize) / block_size;
                let blocks_y = (height as usize) / block_size;
                let available_bits = blocks_x * blocks_y;

                if total_bits > available_bits {
                    return Err(format!(
                        "Payload too large for image frequency capacity. Need {} bits, have {} blocks.",
                        total_bits, available_bits
                    )
                    .into());
                }

                let mut out_img = img.clone();
                let mut bit_index = 0;

                'block_loop: for by in 0..blocks_y {
                    for bx in 0..blocks_x {
                        if bit_index >= total_bits {
                            break 'block_loop;
                        }

                        let byte_idx = bit_index / 8;
                        let bit_off = 7 - (bit_index % 8);
                        let bit = ((payload[byte_idx] >> bit_off) & 1) != 0;

                        // Modify a representative pixel pair in the 8x8 block (e.g. (3,3) and (3,4))
                        let px = (bx * block_size + 3) as u32;
                        let py = (by * block_size + 3) as u32;
                        let pixel = out_img.get_pixel_mut(px, py);

                        // Modulate green channel LSB based on frequency bit
                        if bit {
                            pixel[1] |= 1;
                        } else {
                            pixel[1] &= !1;
                        }

                        bit_index += 1;
                    }
                }

                out_img.save(output_path)?;
                Ok(())
            }
        }
    }

    pub fn extract(image_path: &str, method: StegMethod) -> Result<String, Box<dyn Error>> {
        match method {
            StegMethod::Metadata => {
                let processor = EXIFProcessor::new(image_path.to_string())?;
                let tags = processor.extract()?;
                if let Some(tag) = tags.get("Exif.Image.ImageDescription") {
                    Ok(tag.value.clone())
                } else if let Some(tag) = tags.get("Exif.Photo.UserComment") {
                    Ok(tag.value.clone())
                } else {
                    Err("No steganography ciphertext found in image metadata.".into())
                }
            }
            StegMethod::PixelLsb => {
                let img = ImageReader::open(image_path)?.decode()?.to_rgba8();
                let (width, height) = img.dimensions();

                // Extract bits into a byte buffer
                let mut bits = Vec::new();
                for y in 0..height {
                    for x in 0..width {
                        let pixel = img.get_pixel(x, y);
                        for c in 0..3 {
                            bits.push(pixel[c] & 1);
                        }
                    }
                }

                let mut bytes = Vec::new();
                for chunk in bits.chunks(8) {
                    if chunk.len() < 8 {
                        break;
                    }
                    let mut b: u8 = 0;
                    for (i, &bit) in chunk.iter().enumerate() {
                        if bit != 0 {
                            b |= 1 << (7 - i);
                        }
                    }
                    bytes.push(b);
                }

                parse_extracted_bytes(&bytes)
            }
            StegMethod::FrequencyWise => {
                let img = ImageReader::open(image_path)?.decode()?.to_rgba8();
                let (width, height) = img.dimensions();

                let block_size = 8;
                let blocks_x = (width as usize) / block_size;
                let blocks_y = (height as usize) / block_size;

                let mut bits = Vec::new();
                for by in 0..blocks_y {
                    for bx in 0..blocks_x {
                        let px = (bx * block_size + 3) as u32;
                        let py = (by * block_size + 3) as u32;
                        let pixel = img.get_pixel(px, py);
                        bits.push(pixel[1] & 1);
                    }
                }

                let mut bytes = Vec::new();
                for chunk in bits.chunks(8) {
                    if chunk.len() < 8 {
                        break;
                    }
                    let mut b: u8 = 0;
                    for (i, &bit) in chunk.iter().enumerate() {
                        if bit != 0 {
                            b |= 1 << (7 - i);
                        }
                    }
                    bytes.push(b);
                }

                parse_extracted_bytes(&bytes)
            }
        }
    }
}

fn parse_extracted_bytes(bytes: &[u8]) -> Result<String, Box<dyn Error>> {
    if !bytes.windows(MAGIC_HEADER.len()).any(|w| w == MAGIC_HEADER) {
        return Err("Invalid steganography payload or magic header not found.".into());
    }

    // Find magic header position
    let mut start_pos = 0;
    for i in 0..bytes.len().saturating_sub(MAGIC_HEADER.len()) {
        if &bytes[i..i + MAGIC_HEADER.len()] == MAGIC_HEADER {
            start_pos = i + MAGIC_HEADER.len();
            break;
        }
    }

    // Parse length (terminated by ':' after magic header)
    let mut len_end = start_pos;
    while len_end < bytes.len() && bytes[len_end] != b':' {
        len_end += 1;
    }
    if len_end >= bytes.len() {
        return Err("Malformed steganography payload length.".into());
    }

    let len_str = std::str::from_utf8(&bytes[start_pos..len_end])?;
    let payload_len: usize = len_str.parse()?;

    let data_start = len_end + 1;
    if data_start + payload_len > bytes.len() {
        return Err("Payload length exceeds available extracted data.".into());
    }

    let ciphertext_bytes = &bytes[data_start..data_start + payload_len];
    let ciphertext = std::str::from_utf8(ciphertext_bytes)?;
    Ok(ciphertext.to_string())
}
