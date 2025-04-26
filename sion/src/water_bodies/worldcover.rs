use crate::raster16::Raster16;
use crate::water_bodies::caching::ensure_file_in_cache;
use crate::water_bodies::dem_tile_id::DemTileId;
use crate::water_bodies::water_bodies::WaterBodyValue;
use flate2::read::ZlibDecoder;
use serde_json::{Map, Value};
use std::fs::File;
use std::io::Read;
use std::io::{self, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use tiff::decoder::Decoder;
use tiff::tags::{PlanarConfiguration, Tag};

const WORLD_COVER_S3_DOMAIN: &str =
    "https://esa-worldcover.s3.eu-central-1.amazonaws.com";

const WORLD_COVER_VERSION: &str = "v200";
const WORLD_COVER_YEAR: &str = "2021";

const WORLD_COVER_CACHE_DIR: &str = "WorldCover";

const WORLD_COVER_TILE_SIZE: u16 = 12000;
pub const WORLD_COVER_TILES_IN_BATCH: u16 = 3;
const WORLD_COVER_BITMAP_SIZE: u16 =
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

pub fn ensure_geojson_file(cache_dir: &Path) -> Result<PathBuf, String> {
    let cached_file_name = cache_dir
        .join(WORLD_COVER_CACHE_DIR)
        .join("esa_worldcover_2020_grid.geojson");
    ensure_file_in_cache(&geojson_url(), &cached_file_name)
}

pub fn ensure_world_cover_tile(
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

pub fn list_all_available_files(
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
                .map(|tile_name| {
                    tile_name.parse().unwrap_or_else(|_| {
                        panic!("Failed to parse tile name: {}", tile_name)
                    })
                })
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
) -> Result<Vec<Raster16>, String> {
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

    if image_width != WORLD_COVER_BITMAP_SIZE as u32
        || image_height != WORLD_COVER_BITMAP_SIZE as u32
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

    // Calculate number of tiles (rows and columns of tiles)
    let tiles_per_row = (image_width + tile_width - 1) / tile_width;
    let tiles_per_column = (image_height + tile_height - 1) / tile_height;

    let file_for_image_data = match File::open(&world_cover_tiff_file_name) {
        Ok(file) => file,
        Err(e) => return Err(format!("Failed to open TIFF file: {}", e)),
    };

    // create a 3x3 2D Vec of bitmaps
    let mut water_bodies_tiles: Vec<Raster16> = Vec::new();

    for _ in 0..WORLD_COVER_TILES_IN_BATCH {
        for _ in 0..WORLD_COVER_TILES_IN_BATCH {
            water_bodies_tiles.push(Raster16::new(
                WORLD_COVER_TILE_SIZE,
                WORLD_COVER_TILE_SIZE,
            ));
        }
    }

    let mut reader = BufReader::new(file_for_image_data);

    // Loop over each tile and decompress it
    for row in 0..tiles_per_column {
        let tile_y0 = (row * tile_height) as u16;

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
            let decompressed_tile_data =
                match decompress_tile_data(&mut reader, tile_byte_count) {
                    Ok(data) => data,
                    Err(e) => {
                        return Err(format!(
                            "Failed to decompress tile data: {}",
                            e
                        ))
                    }
                };

            if decompressed_tile_data.len() != tile_size_bytes as usize {
                return Err(format!(
                    "Decompressed data size does not match expected size: {}",
                    decompressed_tile_data.len()
                ));
            }

            // fill the main bitmap with the decompressed tile data
            let tile_x0 = (col * tile_width) as u16;

            for y in 0..tile_height {
                let abs_y = tile_y0 + y as u16;
                let tile_row = abs_y / WORLD_COVER_TILE_SIZE;

                if tile_row < WORLD_COVER_TILES_IN_BATCH {
                    for x in 0..tile_width {
                        let abs_x = tile_x0 + x as u16;

                        // determine which water bodies tile this pixel belongs to
                        let tile_col = abs_x / WORLD_COVER_TILE_SIZE;

                        if tile_col < WORLD_COVER_TILES_IN_BATCH {
                            let tile = &mut water_bodies_tiles[tile_row
                                as usize
                                * WORLD_COVER_TILES_IN_BATCH as usize
                                + tile_col as usize];

                            // calculate the tile-local coordinates
                            let local_x = abs_x % WORLD_COVER_TILE_SIZE;
                            let local_y = abs_y % WORLD_COVER_TILE_SIZE;

                            if local_x < WORLD_COVER_TILE_SIZE
                                && local_y < WORLD_COVER_TILE_SIZE
                            {
                                let pixel_value = decompressed_tile_data
                                    [(y * tile_width + x) as usize];

                                // the pixel value should be 0 for no data (/unknown),
                                // 1 for non-water, 2 for water
                                let water_body_pixel_value = match pixel_value {
                                    255 => WaterBodyValue::NoData,
                                    80 => WaterBodyValue::Water,
                                    _ => WaterBodyValue::NonWater,
                                };

                                tile.set_pixel(
                                    local_x,
                                    local_y,
                                    water_body_pixel_value as u16,
                                );
                            } else {
                                // these pixels are from edge tiles and reach beyond
                                // the main bitmap, so we skip them
                            }
                        }
                    }
                }
            }
        } // for col
    }

    Ok(water_bodies_tiles)
}

#[cfg(test)]
mod tests {
    use crate::water_bodies::worldcover::{
        ensure_geojson_file, ensure_world_cover_tile, list_all_available_files,
        read_world_cover_tiff_file, world_cover_tile_download_url, DemTileId,
        WORLD_COVER_TILES_IN_BATCH,
    };

    use crate::water_bodies::water_bodies::generate_water_bodies_processing_tiles_from_worldcover_ones;
    use dotenv::dotenv;
    use std::path::Path;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn initialize_logging() {
        INIT.call_once(|| {
            dotenv().ok();
        });
    }

    #[test]
    fn load_world_cover_tiff_file() {
        initialize_logging();

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
        assert_eq!(
            world_cover_tile_download_url(sample_tile_id),
            "https://esa-worldcover.s3.eu-central-1.amazonaws.com/v200/2021/map/\
            ESA_WorldCover_10m_2021_v200_S54E168_Map.tif");

        let tile_result = ensure_world_cover_tile(cache_dir, sample_tile_id);
        assert!(tile_result.is_ok(), "Error: {:?}", tile_result.unwrap_err());

        let tile_path = tile_result.unwrap();
        let tiff_file_reading_result =
            read_world_cover_tiff_file(tile_path.as_path());

        assert!(
            tiff_file_reading_result.is_ok(),
            "Error: {:?}",
            tiff_file_reading_result.unwrap_err()
        );

        let world_cover_tiles = tiff_file_reading_result.unwrap();

        assert_eq!(
            world_cover_tiles.len(),
            (WORLD_COVER_TILES_IN_BATCH * WORLD_COVER_TILES_IN_BATCH) as usize
        );

        generate_water_bodies_processing_tiles_from_worldcover_ones(
            sample_tile_id,
            &world_cover_tiles,
            cache_dir,
        );
    }
}
