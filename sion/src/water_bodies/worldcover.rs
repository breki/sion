use std::io::Cursor;
use std::path::{Path, PathBuf};

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

        let mut file = match std::fs::File::create(&cached_file_name) {
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

#[cfg(test)]
mod tests {
    use crate::water_bodies::worldcover::ensure_geojson_file;
    use std::path::Path;

    #[test]
    fn icebreaker() {
        let cache_dir = "cache";

        let result = ensure_geojson_file(Path::new(cache_dir));
        assert!(result.is_ok(), "Error: {:?}", result.unwrap_err());
        let result = ensure_geojson_file(Path::new(cache_dir));
        assert!(result.is_ok(), "Error: {:?}", result.unwrap_err());
    }
}
