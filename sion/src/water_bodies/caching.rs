use std::fs::File;
use std::io::Cursor;
use std::path::{Path, PathBuf};

/// Ensure that the file is in the cache directory. If it is not, download it
/// from the given URL and save it to the specified path in the cache directory.
pub fn ensure_file_in_cache(
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
