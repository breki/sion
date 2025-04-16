use crate::raster16::Raster16;
use flate2::read::ZlibDecoder;
use serde_json::{Map, Value};
use std::fmt;
use std::fs::File;
use std::io::{self, BufReader, Seek, SeekFrom};
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tiff::decoder::Decoder;
use tiff::tags::{PlanarConfiguration, Tag};

// todo 0: move stuff into their own modules

#[derive(Copy, Clone, Debug)]
pub struct DemTileId {
    pub lon: i16,
    pub lat: i16,
}

impl DemTileId {
    pub fn new(lon: i16, lat: i16) -> Self {
        DemTileId { lon, lat }
    }

    // parses file names like "S54E168", taking into account N-S and E-W
    pub fn from_tile_name(tile_name: &str) -> Result<DemTileId, String> {
        if tile_name.len() != 7 {
            return Err(format!("Invalid tile ID length: {}", tile_name));
        }

        let lat = tile_name[1..3]
            .parse::<i16>()
            .map_err(|e| format!("Failed to parse latitude: {}", e))?;
        let lon = tile_name[4..7]
            .parse::<i16>()
            .map_err(|e| format!("Failed to parse longitude: {}", e))?;

        // Adjust the sign of the coordinates based on the hemisphere
        let lon = if &tile_name[3..4] == "E" { lon } else { -lon };
        let lat = if &tile_name[0..1] == "N" { lat } else { -lat };

        Ok(DemTileId { lon, lat })
    }
}

impl fmt::Display for DemTileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            if self.lat >= 0 { 'N' } else { 'S' },
            self.lat.abs(),
            if self.lon >= 0 { 'E' } else { 'W' },
            self.lon.abs()
        )
    }
}

type _HeightsArray = Vec<i16>; // Replace with your actual HeightsArray type.

const WORLD_COVER_S3_DOMAIN: &str =
    "https://esa-worldcover.s3.eu-central-1.amazonaws.com";

const WORLD_COVER_VERSION: &str = "v200";
const WORLD_COVER_YEAR: &str = "2021";

const WORLD_COVER_CACHE_DIR: &str = "WorldCover";

const WORLD_COVER_TILE_SIZE: u16 = 12000;
const WORLD_COVER_TILES_IN_BATCH: u16 = 3;
const WORLD_COVER_BITMAP_SIZE: u16 =
    WORLD_COVER_TILE_SIZE * WORLD_COVER_TILES_IN_BATCH;

const WATER_BODY_TILE_SIZE: u16 = 1800;

enum WaterBodyValue {
    NoData = 0,
    NonWater = 1,
    Water = 2,
}

pub struct WaterBodiesProcessingTile {
    pub tile_id: DemTileId,
    data: Vec<Vec<u16>>,
}

impl WaterBodiesProcessingTile {
    pub fn new(tile_id: &DemTileId) -> Self {
        WaterBodiesProcessingTile {
            tile_id: tile_id.clone(),
            data: vec![
                vec![0; WATER_BODY_TILE_SIZE as usize];
                WATER_BODY_TILE_SIZE as usize
            ],
        }
    }

    pub fn set_pixel(&mut self, x: u16, y: u16, value: u16) {
        if x >= WATER_BODY_TILE_SIZE || y >= WATER_BODY_TILE_SIZE {
            panic!("Pixel coordinates out of bounds");
        }

        self.data[y as usize][x as usize] = value;
    }

    pub fn write_to_file(&self, file_name: &Path) -> Result<(), io::Error> {
        let mut file = File::create(file_name)?;
        for row in &self.data {
            for &value in row {
                file.write_all(&value.to_le_bytes())?;
            }
        }
        Ok(())
    }
}

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
                .map(|tile_name| {
                    DemTileId::from_tile_name(&tile_name).unwrap_or_else(|_| {
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
) -> Result<Vec<Vec<Raster16>>, String> {
    let start = Instant::now();

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
    let mut water_bodies_tiles: Vec<Vec<Raster16>> = Vec::new();

    for _ in 0..WORLD_COVER_TILES_IN_BATCH {
        let mut row: Vec<Raster16> = Vec::new();
        for _ in 0..WORLD_COVER_TILES_IN_BATCH {
            row.push(Raster16::new(
                WORLD_COVER_TILE_SIZE,
                WORLD_COVER_TILE_SIZE,
            ));
        }
        water_bodies_tiles.push(row);
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

            let duration = start.elapsed();

            println!(
                "{:2?}: Tile {}: Offset: {}, Byte Count: {}, Decompressed Size: {}",
                duration,
                tile_index,
                tile_offset,
                tile_byte_count,
                decompressed_tile_data.len()
            );

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
                            let tile = &mut water_bodies_tiles
                                [tile_row as usize]
                                [tile_col as usize];

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

pub fn downsample_to_water_bodies_tile(
    tile_id: &DemTileId,
    raster: &Raster16,
) -> WaterBodiesProcessingTile {
    let mut downsampled = WaterBodiesProcessingTile::new(tile_id);

    let x_ratio = raster.width as f32 / WATER_BODY_TILE_SIZE as f32;
    let y_ratio = raster.height as f32 / WATER_BODY_TILE_SIZE as f32;

    for y in 0..WATER_BODY_TILE_SIZE {
        for x in 0..WATER_BODY_TILE_SIZE {
            let src_x_start = x as f32 * x_ratio;
            let src_x_end = (x + 1) as f32 * x_ratio;
            let src_y_start = y as f32 * y_ratio;
            let src_y_end = (y + 1) as f32 * y_ratio;

            // Assuming 3 possible water body values
            let mut color_weights = [0.0; 3];

            for src_y in src_y_start.floor() as usize..src_y_end.ceil() as usize
            {
                for src_x in
                    src_x_start.floor() as usize..src_x_end.ceil() as usize
                {
                    if src_x < raster.width as usize
                        && src_y < raster.height as usize
                    {
                        let overlap_x = (src_x_end.min((src_x + 1) as f32)
                            - src_x_start.max(src_x as f32))
                        .max(0.0);
                        let overlap_y = (src_y_end.min((src_y + 1) as f32)
                            - src_y_start.max(src_y as f32))
                        .max(0.0);
                        let overlap_area = overlap_x * overlap_y;

                        let color =
                            raster.get_pixel(src_x as u16, src_y as u16);
                        if color < 3 {
                            color_weights[color as usize] += overlap_area;
                        }
                    }
                }
            }

            let dominant_color = color_weights
                .iter()
                .enumerate()
                .max_by(|&(_, a), &(_, b)| a.partial_cmp(b).unwrap())
                .map(|(color, _)| color as u16)
                .unwrap_or(0);

            downsampled.set_pixel(x, y, dominant_color);
        }
    }

    downsampled
}

#[cfg(test)]
mod tests {
    use crate::water_bodies::worldcover::{
        downsample_to_water_bodies_tile, ensure_geojson_file,
        ensure_world_cover_tile, list_all_available_files,
        read_world_cover_tiff_file, world_cover_tile_download_url, DemTileId,
        WORLD_COVER_CACHE_DIR,
    };

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
    fn parsing_and_formatting_tile_ids() {
        let tile_id = DemTileId::from_tile_name("N54E168").unwrap();
        assert_eq!(tile_id.lon, 168);
        assert_eq!(tile_id.lat, 54);
        assert_eq!(tile_id.to_string(), "N54E168");

        let tile_id = DemTileId::from_tile_name("S54W168").unwrap();
        assert_eq!(tile_id.lon, -168);
        assert_eq!(tile_id.lat, -54);
        assert_eq!(tile_id.to_string(), "S54W168");

        let tile_id = DemTileId::from_tile_name("N54W168").unwrap();
        assert_eq!(tile_id.lon, -168);
        assert_eq!(tile_id.lat, 54);
        assert_eq!(tile_id.to_string(), "N54W168");

        let tile_id = DemTileId::from_tile_name("S54E168").unwrap();
        assert_eq!(tile_id.lon, 168);
        assert_eq!(tile_id.lat, -54);
        assert_eq!(tile_id.to_string(), "S54E168");
    }

    // todo 5: extract the water bodies tiles generation logic into a separate function
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

        assert_eq!(world_cover_tiles.len(), 3);
        assert_eq!(world_cover_tiles[0].len(), 3);

        let processing_dir =
            cache_dir.join(WORLD_COVER_CACHE_DIR).join("processing");

        if !processing_dir.exists() {
            std::fs::create_dir_all(&processing_dir).unwrap();
        }

        // downsample the tiles to the WATER_BODY_TILE_SIZE
        for row in 0..world_cover_tiles.len() {
            for col in 0..world_cover_tiles[row].len() {
                let tile = &world_cover_tiles[row][col];

                let tile_id = DemTileId::new(
                    sample_tile_id.lon + col as i16,
                    sample_tile_id.lat + row as i16,
                );
                let downsampled_tile =
                    downsample_to_water_bodies_tile(&tile_id, &tile);

                let tile_file_name = processing_dir
                    .join(downsampled_tile.tile_id.to_string())
                    .with_extension("wbp");

                downsampled_tile.write_to_file(&tile_file_name).unwrap();
            }
        }
    }
}
