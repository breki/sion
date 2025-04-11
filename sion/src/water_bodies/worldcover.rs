use flate2::read::ZlibDecoder;
use serde_json::{Map, Value};
use std::fs::File;
use std::io::{self, BufReader, Seek, SeekFrom};
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use tiff::decoder::Decoder;
use tiff::tags::{CompressionMethod, PlanarConfiguration, Tag};

type DemTileId = String;

type _HeightsArray = Vec<i16>; // Replace with your actual HeightsArray type.

const WORLD_COVER_S3_DOMAIN: &str =
    "https://esa-worldcover.s3.eu-central-1.amazonaws.com";

const WORLD_COVER_VERSION: &str = "v200";
const WORLD_COVER_YEAR: &str = "2021";

const WORLD_COVER_CACHE_DIR: &str = "WorldCover";

const WORLD_COVER_TILE_SIZE: u32 = 12000;
const WORLD_COVER_TILES_IN_BATCH: u32 = 3;
const WORLD_COVER_BITMAP_SIZE: u32 =
    WORLD_COVER_TILE_SIZE * WORLD_COVER_TILES_IN_BATCH;

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

fn decompress_tile_data(
    compressed_data: &mut BufReader<File>,
    compressed_size: u32,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Limit the reader to only read `compressed_size` bytes
    let limited_reader = compressed_data.take(compressed_size as u64);

    // Create a ZlibDecoder to decompress the data
    let mut decompressor = ZlibDecoder::new(limited_reader);

    // Prepare a buffer for the decompressed data
    let mut decompressed_data = Vec::new();

    // Decompress the data into the buffer
    decompressor.read_to_end(&mut decompressed_data)?;

    Ok(decompressed_data)
}

pub fn read_world_cover_tiff_file(
    world_cover_tiff_file_name: &Path,
) -> Result<(), String> {
    let file = match File::open(&world_cover_tiff_file_name) {
        Ok(file) => file,
        Err(e) => return Err(format!("Failed to open TIFF file: {}", e)),
    };

    let mut decoder = match Decoder::new(file) {
        Ok(decoder) => decoder,
        Err(e) => return Err(format!("Failed to create TIFF decoder: {}", e)),
    };

    // Validate raster dimensions
    let (image_width, image_height) = match decoder.dimensions() {
        Ok((width, height)) => (width, height),
        Err(e) => return Err(format!("Failed to get TIFF dimensions: {}", e)),
    };

    if image_width != WORLD_COVER_BITMAP_SIZE
        || image_height != WORLD_COVER_BITMAP_SIZE
    {
        return Err(format!(
            "Expected dimensions {}x{}, but got {}x{}",
            WORLD_COVER_BITMAP_SIZE,
            WORLD_COVER_BITMAP_SIZE,
            image_width,
            image_height
        )
        .into());
    }

    match decoder.get_tag_u32(Tag::PlanarConfiguration) {
        Ok(v) => {
            if (v as u16) != PlanarConfiguration::Chunky.to_u16() {
                return Err(format!(
                    "Expected CONTIG/Chunky planar configuration, but got {:?}",
                    v
                ))
                .into();
            }
        }
        Err(e) => {
            return Err(format!(
                "Failed to get TIFF planar configuration: {}",
                e
            ))
        }
    }

    let tile_width = match decoder.get_tag_u32(Tag::TileWidth) {
        Ok(v) => v,
        Err(e) => {
            return Err(format!(
            "Failed to get tile width, looks like this TIFF is not tiled: {}",
            e
        ))
        }
    };

    let tile_height = match decoder.get_tag_u32(Tag::TileLength) {
        Ok(v) => v,
        Err(e) => {
            return Err(format!(
            "Failed to get tile length, looks like this TIFF is not tiled: {}",
            e
        ))
        }
    };

    let tile_size_bytes = tile_width * tile_height;

    if let Some(tile_offsets) = decoder.get_tag(Tag::TileOffsets).ok() {
        println!("Tile Offsets: {:?}", tile_offsets);
    }

    // Compression tag
    match decoder.get_tag_u32(Tag::Compression) {
        Ok(comp) => {
            let compression_method = CompressionMethod::from_u16(comp as u16)
                .unwrap_or(CompressionMethod::Unknown(comp as u16));
            println!("Compression: {:?}", compression_method);
        }
        _ => println!("Compression tag not found"),
    };

    // Calculate number of tiles (rows and columns of tiles)
    let tiles_per_row = (image_width + tile_width - 1) / tile_width;
    let tiles_per_column = (image_height + tile_height - 1) / tile_height;

    // let mut _all_tiles_data: Vec<u8> =
    //     Vec::with_capacity((image_width * image_height) as usize);

    let file_for_image_data = match File::open(&world_cover_tiff_file_name) {
        Ok(file) => file,
        Err(e) => return Err(format!("Failed to open TIFF file: {}", e)),
    };

    let mut reader = BufReader::new(file_for_image_data);

    // Loop over each tile and decompress it
    for row in 0..tiles_per_column {
        for col in 0..tiles_per_row {
            // Get the index of the tile
            let tile_index = (row * tiles_per_row + col) as usize;

            // Get the offset and byte count for the tile
            let tile_offset: u32 = match decoder
                .get_tag_u32_vec(Tag::TileOffsets)
            {
                Ok(v) => *v.get(tile_index).unwrap(),
                Err(e) => {
                    return Err(format!("Failed to get tile offsets: {}", e))
                }
            };

            let tile_byte_count: u32 =
                match decoder.get_tag_u32_vec(Tag::TileByteCounts) {
                    Ok(v) => *v.get(tile_index).unwrap(),
                    Err(e) => {
                        return Err(format!(
                            "Failed to get tile byte counts: {}",
                            e
                        ))
                    }
                };

            match reader.seek(SeekFrom::Start(tile_offset as u64)) {
                Ok(_) => {}
                Err(e) => {
                    return Err(format!("Failed to seek to tile offset: {}", e))
                }
            }

            // Decompress the tile data
            let decompressed_data =
                match decompress_tile_data(&mut reader, tile_byte_count) {
                    Ok(data) => data,
                    Err(e) => {
                        return Err(format!(
                            "Failed to decompress tile data: {}",
                            e
                        ))
                    }
                };

            println!(
                "Tile {}: Offset: {}, Byte Count: {}, Decompressed Size: {}",
                tile_index,
                tile_offset,
                tile_byte_count,
                decompressed_data.len()
            );

            if decompressed_data.len() != tile_size_bytes as usize {
                return Err(format!(
                    "Decompressed data size does not match expected size: {}",
                    decompressed_data.len()
                ));
            }

            // todo 0: fill the main array with the decompressed tile data
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::water_bodies::worldcover::{
        ensure_geojson_file, ensure_world_cover_tile, list_all_available_files,
        read_world_cover_tiff_file, world_cover_tile_download_url, DemTileId,
    };
    use std::path::Path;

    #[test]
    fn load_world_cover_tiff_tile() {
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

        let tile_path = tile_result.unwrap();
        let tile_reading_result =
            read_world_cover_tiff_file(tile_path.as_path());

        assert!(
            tile_reading_result.is_ok(),
            "Error: {:?}",
            tile_reading_result.unwrap_err()
        );
    }
}
