use std::collections::HashMap;

use indexmap::IndexMap;

use crate::processing::exiv2_backend::ExifTag;

/// Handles GPS coordinate conversion and formatting.
pub struct GPSConverter;

impl GPSConverter {
    /// Parse GPS coordinates out of the dict returned by EXIFProcessor.extract().
    ///
    /// Args:
    ///     exif_data: HashMap keyed by fully-qualified exiv2 keys, e.g.
    ///         {"Exif.GPSInfo.GPSLatitude": {value: "43/1 28/1 2814/1000", ...}, ...}
    ///
    /// Returns:
    ///     Some((latitude, longitude)) in decimal degrees, or None if invalid
    pub fn parse_exif_gps(
        exif_data: &IndexMap<String, ExifTag>,
    ) -> Option<(f64, f64)> {
        if exif_data.is_empty() {
            return None;
        }

        let lat_ref = Self::get_non_empty_value(exif_data, "Exif.GPSInfo.GPSLatitudeRef")?;
        let lat_raw = Self::get_non_empty_value(exif_data, "Exif.GPSInfo.GPSLatitude")?;
        let lon_ref = Self::get_non_empty_value(exif_data, "Exif.GPSInfo.GPSLongitudeRef")?;
        let lon_raw = Self::get_non_empty_value(exif_data, "Exif.GPSInfo.GPSLongitude")?;

        let latitude = Self::dms_to_decimal(lat_raw, lat_ref)?;
        let longitude = Self::dms_to_decimal(lon_raw, lon_ref)?;

        if latitude.is_none() || longitude.is_none() {
            return None;
        }
        let latitude = latitude.unwrap();
        let longitude = longitude.unwrap();

        if !(-90.0 <= latitude && latitude <= 90.0) || !(-180.0 <= longitude && longitude <= 180.0)
        {
            return None;
        }

        if latitude == 0.0 && longitude == 0.0 {
            return None;
        }

        Some((latitude, longitude))
    }

    /// Helper: get a non-empty value string for the given key from exif_data.
    fn get_non_empty_value<'a>(
        exif_data: &'a IndexMap<String, ExifTag>,
        key: &str,
    ) -> Option<&'a str> {
        exif_data
            .get(key)
            .map(|t| t.value.as_str())
            .filter(|s| !s.is_empty())
    }

    /// Parses a single exiv2 rational token like '2814/1000' or '43' into a float.
    fn parse_rational(token: &str) -> Option<f64> {
        if let Some((num_str, den_str)) = token.split_once('/') {
            let num: f64 = num_str.parse().ok()?;
            let den: f64 = den_str.parse().ok()?;
            if den == 0.0 {
                return None;
            }
            Some(num / den)
        } else {
            token.parse().ok()
        }
    }

    /// Convert an exiv2 raw GPS value to decimal degrees.
    ///
    /// exiv2's raw (untranslated) GPS value for a 3-component Rational tag
    /// comes back as a single space-separated string of fractions, e.g.
    /// "43/1 28/1 2814/1000" for 43 deg 28' 2.814".
    ///
    /// Args:
    ///     dms_raw: the raw value string from exiv2
    ///     ref: reference (N/S for latitude, E/W for longitude)
    ///
    /// Returns:
    ///     Some(decimal degree value) or None if invalid
    fn dms_to_decimal(dms_raw: &str, ref_str: &str) -> Option<Option<f64>> {
        let tokens: Vec<&str> = dms_raw.split_whitespace().collect();

        if tokens.len() < 3 {
            return None;
        }

        let degrees = Self::parse_rational(tokens[0])?;
        let minutes = Self::parse_rational(tokens[1])?;
        let seconds = Self::parse_rational(tokens[2])?;

        let mut decimal = degrees + (minutes / 60.0) + (seconds / 3600.0);

        if ref_str.eq_ignore_ascii_case("S") || ref_str.eq_ignore_ascii_case("W") {
            decimal = -decimal;
        }

        Some(Some(decimal))
    }

    /// Format coordinates as decimal degrees string,
    /// e.g. '40.712800, -74.006000'.
    pub fn to_decimal_degrees(latitude: f64, longitude: f64) -> String {
        format!("{:.6}, {:.6}", latitude, longitude)
    }

    /// Convert decimal degrees to DMS format.
    pub fn to_dms(latitude: f64, longitude: f64) -> String {
        fn decimal_to_dms(decimal_deg: f64, is_latitude: bool) -> String {
            let abs_deg = decimal_deg.abs();
            let degrees = abs_deg as i64;
            let minutes_float = (abs_deg - degrees as f64) * 60.0;
            let minutes = minutes_float as i64;
            let seconds = (minutes_float - minutes as f64) * 60.0;

            let ref_str = if is_latitude {
                if decimal_deg >= 0.0 { "N" } else { "S" }
            } else {
                if decimal_deg >= 0.0 { "E" } else { "W" }
            };

            format!("{}°{}'{:.2}\"{}", degrees, minutes, seconds, ref_str)
        }

        let lat_dms = decimal_to_dms(latitude, true);
        let lon_dms = decimal_to_dms(longitude, false);

        format!("{}, {}", lat_dms, lon_dms)
    }

    /// Convert decimal degrees to DMM (Degrees and Decimal Minutes) format.
    pub fn to_dmm(latitude: f64, longitude: f64) -> String {
        fn decimal_to_dmm(decimal_deg: f64, is_latitude: bool) -> String {
            let abs_deg = decimal_deg.abs();
            let degrees = abs_deg as i64;
            let minutes = (abs_deg - degrees as f64) * 60.0;

            let ref_str = if is_latitude {
                if decimal_deg >= 0.0 { "N" } else { "S" }
            } else {
                if decimal_deg >= 0.0 { "E" } else { "W" }
            };

            format!("{}°{:.4}'{}", degrees, minutes, ref_str)
        }

        let lat_dmm = decimal_to_dmm(latitude, true);
        let lon_dmm = decimal_to_dmm(longitude, false);

        format!("{}, {}", lat_dmm, lon_dmm)
    }

    /// Convert decimal degrees to UTM coordinates (simplified).
    pub fn to_utm(latitude: f64, longitude: f64) -> String {
        let zone = ((longitude + 180.0) / 6.0) as i64 + 1;
        let hemisphere = if latitude >= 0.0 { "N" } else { "S" };
        format!(
            "Zone {}{} (Approximate conversion - use specialized library for precise UTM)",
            zone, hemisphere
        )
    }

    /// Placeholder MGRS conversion.
    pub fn to_mgrs(latitude: f64, longitude: f64) -> String {
        format!(
            "MGRS conversion requires specialized library (lat: {:.6}, lon: {:.6})",
            latitude, longitude
        )
    }

    /// Convert decimal degrees to geohash (default precision = 12).
    pub fn to_geohash(latitude: f64, longitude: f64) -> String {
        Self::to_geohash_with_precision(latitude, longitude, 12)
    }

    /// Convert decimal degrees to geohash with explicit precision.
    pub fn to_geohash_with_precision(
        latitude: f64,
        longitude: f64,
        precision: usize,
    ) -> String {
        let base32 = "0123456789bcdefghjkmnpqrstuvwxyz";
        let mut lat_range = [-90.0_f64, 90.0_f64];
        let mut lon_range = [-180.0_f64, 180.0_f64];

        let mut geohash = String::new();
        let mut bit: usize = 0;
        let mut ch: usize = 0;
        let mut even = true;

        while geohash.len() < precision {
            if even {
                let mid = (lon_range[0] + lon_range[1]) / 2.0;
                if longitude >= mid {
                    ch |= 1 << (4 - bit);
                    lon_range[0] = mid;
                } else {
                    lon_range[1] = mid;
                }
            } else {
                let mid = (lat_range[0] + lat_range[1]) / 2.0;
                if latitude >= mid {
                    ch |= 1 << (4 - bit);
                    lat_range[0] = mid;
                } else {
                    lat_range[1] = mid;
                }
            }

            even = !even;
            if bit < 4 {
                bit += 1;
            } else {
                geohash.push(base32.as_bytes()[ch] as char);
                bit = 0;
                ch = 0;
            }
        }

        geohash
    }

    /// Placeholder Plus Code conversion.
    pub fn to_plus_code(latitude: f64, longitude: f64) -> String {
        format!(
            "Plus Code conversion requires specialized library (lat: {:.6}, lon: {:.6})",
            latitude, longitude
        )
    }

    /// Create a Google Maps URL for the given coordinates.
    pub fn create_google_maps_url(latitude: f64, longitude: f64) -> String {
        format!(
            "https://maps.google.com/maps?q={:.6},{:.6}",
            latitude, longitude
        )
    }

    /// Get coordinates in all supported formats.
    pub fn get_all_formats(latitude: f64, longitude: f64) -> HashMap<String, String> {
        let mut result = HashMap::new();
        result.insert(
            "decimal_degrees".to_string(),
            Self::to_decimal_degrees(latitude, longitude),
        );
        result.insert(
            "dms".to_string(),
            Self::to_dms(latitude, longitude),
        );
        result.insert(
            "dmm".to_string(),
            Self::to_dmm(latitude, longitude),
        );
        result.insert(
            "utm".to_string(),
            Self::to_utm(latitude, longitude),
        );
        result.insert(
            "mgrs".to_string(),
            Self::to_mgrs(latitude, longitude),
        );
        result.insert(
            "geohash".to_string(),
            Self::to_geohash(latitude, longitude),
        );
        result.insert(
            "plus_code".to_string(),
            Self::to_plus_code(latitude, longitude),
        );
        result.insert(
            "google_maps_url".to_string(),
            Self::create_google_maps_url(latitude, longitude),
        );
        result
    }
}
