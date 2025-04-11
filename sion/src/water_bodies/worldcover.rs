use serde_json::{Map, Value};
use std::fs::File;
use std::io::Cursor;
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};

type DemTileId = String;

const WORLD_COVER_S3_DOMAIN: &str =
    "https://esa-worldcover.s3.eu-central-1.amazonaws.com";

const WORLD_COVER_VERSION: &str = "v200";
const WORLD_COVER_YEAR: &str = "2021";

const WORLD_COVER_CACHE_DIR: &str = "WorldCover";

fn geojson_url() -> String {
    format!("{}/esa_worldcover_grid.geojson", WORLD_COVER_S3_DOMAIN)
}

fn world_cover_tile_download_url(tile_id: &DemTileId) -> String {
    format!(
        "{}/{}/{}/map/ESA_WorldCover_10m_{}_{}_{}_Map.tif",
        WORLD_COVER_S3_DOMAIN,
        WORLD_COVER_VERSION,
        WORLD_COVER_YEAR,
        WORLD_COVER_YEAR,
        WORLD_COVER_VERSION,
        tile_id
    )
}

/// Ensure that the file is in the cache directory. If it is not, download it
/// from the given URL and save it to the specified path in the cache directory.
fn ensure_file_in_cache(
    url: &str,
    cached_file_name: &Path,
) -> Result<PathBuf, String> {
    if cached_file_name.exists() {
        Ok(cached_file_name.to_path_buf())
    } else {
        let response = match reqwest::blocking::get(url) {
            Ok(response) => response,
            Err(e) => {
                return Err(format!("Failed to download the file: {}", e))
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
                return Err(format!(
                    "Failed to create the file in the cache: {}",
                    e
                ))
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
            Ok(_) => Ok(cached_file_name.to_path_buf()),
            Err(e) => {
                Err(format!("Failed to write the file in the cache: {}", e))
            }
        }
    }
}

fn ensure_geojson_file(cache_dir: &Path) -> Result<PathBuf, String> {
    let cached_file_name = cache_dir
        .join(WORLD_COVER_CACHE_DIR)
        .join("esa_worldcover_2020_grid.geojson");
    ensure_file_in_cache(&geojson_url(), &cached_file_name)
}

fn ensure_world_cover_tile(
    cache_dir: &Path,
    tile_id: &DemTileId,
) -> Result<PathBuf, String> {
    let cached_file_name = cache_dir
        .join(WORLD_COVER_CACHE_DIR)
        .join(format!("{}.tif", tile_id));
    ensure_file_in_cache(
        &world_cover_tile_download_url(&tile_id),
        &cached_file_name,
    )
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
        ensure_geojson_file, ensure_world_cover_tile, list_all_available_files,
        world_cover_tile_download_url, DemTileId,
    };
    use std::path::Path;

    #[test]
    fn icebreaker() {
        let cache_dir = Path::new("cache");

        let result = ensure_geojson_file(cache_dir);
        assert!(result.is_ok(), "Error: {:?}", result.unwrap_err());
        let result = ensure_geojson_file(cache_dir);
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

        let sample_tile_id: &DemTileId = &files[0];
        println!("Example file: {:?}", files[0]);
        assert_eq!(
            world_cover_tile_download_url(sample_tile_id),
            "https://esa-worldcover.s3.eu-central-1.amazonaws.com/v200/2021/map/\
            ESA_WorldCover_10m_2021_v200_S54E168_Map.tif");

        let tile_result = ensure_world_cover_tile(cache_dir, sample_tile_id);
        assert!(tile_result.is_ok(), "Error: {:?}", tile_result.unwrap_err());
    }
}
