use serde_json::{Map, Value};
use std::fs::File;
use std::io::Cursor;
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};

type DemTileId = String;

const WORLD_COVER_S3_DOMAIN: &str =
    "https://esa-worldcover.s3.eu-central-1.amazonaws.com";

fn geojson_url() -> String {
    format!("{}/esa_worldcover_grid.geojson", WORLD_COVER_S3_DOMAIN)
}

fn ensure_geojson_file(cache_dir: &Path) -> Result<PathBuf, String> {
    let cached_file_name = cache_dir.join("esa_worldcover_2020_grid.geojson");

    if cached_file_name.exists() {
        Ok(cached_file_name)
    } else {
        // download the file from the geojson_url() to the cache_file_name
        let response = match reqwest::blocking::get(geojson_url()) {
            Ok(response) => response,
            Err(e) => {
                return Err(format!("Failed to download geojson file: {}", e))
            }
        };

        // if the cache directory does not exist, create it
        if let Some(parent) = cached_file_name.parent() {
            if !parent.exists() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    return Err(format!(
                        "Failed to create cache directory: {}",
                        e
                    ));
                }
            }
        }

        let mut file = match File::create(&cached_file_name) {
            Ok(file) => file,
            Err(e) => {
                return Err(format!("Failed to create geojson file: {}", e))
            }
        };

        let bytes = match response.bytes() {
            Ok(bytes) => bytes,
            Err(e) => {
                return Err(format!("Failed to read response bytes: {}", e))
            }
        };

        let mut cursor = Cursor::new(bytes);

        match std::io::copy(&mut cursor, &mut file) {
            Ok(_) => {
                println!(
                    "Downloaded geojson file to {}",
                    cached_file_name.display()
                );
                Ok(cached_file_name)
            }
            Err(e) => Err(format!("Failed to write geojson file: {}", e)),
        }
    }
}

fn list_all_available_files(
    geojson_file: &Path,
) -> Result<Vec<DemTileId>, String> {
    // Open the GeoJSON file directly.
    let file = match File::open(geojson_file) {
        Ok(file) => file,
        Err(e) => return Err(format!("Failed to open geojson file: {}", e)),
    };

    let reader = BufReader::new(file);

    // Parse the JSON data.
    let json_data: Value = match serde_json::from_reader(reader) {
        Ok(data) => data,
        Err(e) => return Err(format!("Failed to parse geojson file: {}", e)),
    };

    // Extract the "features" array.
    let features = match json_data["features"].as_array().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Missing or invalid 'features' array",
        )
    }) {
        Ok(features) => features,
        Err(e) => return Err(format!("Failed to extract features: {}", e)),
    };

    // Map each feature to its "ll_tile" property and parse it.
    let tiles = features
        .iter()
        .filter_map(|feature| {
            feature
                .get("properties")
                .and_then(|properties| properties.as_object())
                .and_then(|properties: &Map<String, Value>| {
                    properties.get("ll_tile")
                })
                .and_then(|ll_tile| ll_tile.as_str())
                .map(|tile_name| tile_name.to_string())
        })
        .collect();

    Ok(tiles)
}

#[cfg(test)]
mod tests {
    use crate::water_bodies::worldcover::{
        ensure_geojson_file, list_all_available_files,
    };
    use std::path::Path;

    #[test]
    fn icebreaker() {
        let cache_dir = "cache";

        let result = ensure_geojson_file(Path::new(cache_dir));
        assert!(result.is_ok(), "Error: {:?}", result.unwrap_err());
        let result = ensure_geojson_file(Path::new(cache_dir));
        assert!(result.is_ok(), "Error: {:?}", result.unwrap_err());

        let geojson_file = result.unwrap();

        let files_result = list_all_available_files(&geojson_file);
        assert!(
            files_result.is_ok(),
            "Error: {:?}",
            files_result.unwrap_err()
        );

        let files = files_result.unwrap();
        assert!(!files.is_empty(), "No files found in the GeoJSON.");
        assert!(files.len() > 100, "Too few files found in the GeoJSON.");
    }

    #[test]
    fn test2() {
        assert!(true);
    }
}
