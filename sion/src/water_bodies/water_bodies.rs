use crate::raster16::Raster16;
use crate::water_bodies::dem_tile_id::DemTileId;
use std::fs::File;
use std::io::Write;
use std::io::{self};
use std::path::Path;

pub const WATER_BODIES_TILE_SIZE: u16 = 1800;

pub const WATER_BODIES_CACHE_DIR: &str = "WaterBodies";
pub const WATER_BODIES_PROCESSING_DIR: &str = "processing";

pub enum WaterBodyValue {
    NoData = 0,
    NonWater = 1,
    Water = 2,
}

pub struct WaterBodiesProcessingTile {
    pub tile_id: DemTileId,
    pub tile_size: u16,
    data: Vec<u16>,
}

impl WaterBodiesProcessingTile {
    pub fn new(tile_id: &DemTileId, tile_size: u16) -> Self {
        WaterBodiesProcessingTile {
            tile_id: tile_id.clone(),
            tile_size,
            data: vec![
                0;
                WATER_BODIES_TILE_SIZE as usize
                    * WATER_BODIES_TILE_SIZE as usize
            ],
        }
    }

    pub fn downsample_from_worldcover_tile(
        tile_id: &DemTileId,
        raster: &Raster16,
    ) -> Self {
        let mut downsampled =
            WaterBodiesProcessingTile::new(tile_id, WATER_BODIES_TILE_SIZE);

        let x_ratio = raster.width as f32 / WATER_BODIES_TILE_SIZE as f32;
        let y_ratio = raster.height as f32 / WATER_BODIES_TILE_SIZE as f32;

        for y in 0..WATER_BODIES_TILE_SIZE {
            for x in 0..WATER_BODIES_TILE_SIZE {
                let src_x_start = x as f32 * x_ratio;
                let src_x_end = (x + 1) as f32 * x_ratio;
                let src_y_start = y as f32 * y_ratio;
                let src_y_end = (y + 1) as f32 * y_ratio;

                // Assuming 3 possible water body values
                let mut color_weights = [0.0; 3];

                for src_y in
                    src_y_start.floor() as usize..src_y_end.ceil() as usize
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

    pub fn get_pixel(&self, x: u16, y: u16) -> u16 {
        if x >= self.tile_size || y >= self.tile_size {
            panic!("Pixel coordinates out of bounds");
        }

        self.data[y as usize * self.tile_size as usize + x as usize]
    }

    pub fn set_pixel(&mut self, x: u16, y: u16, value: u16) {
        if x >= self.tile_size || y >= self.tile_size {
            panic!("Pixel coordinates out of bounds");
        }

        self.data[y as usize * self.tile_size as usize + x as usize] = value;
    }

    pub fn write_to_file(&self, file_name: &Path) -> Result<(), io::Error> {
        let mut file = File::create(file_name)?;
        for y in 0..self.tile_size {
            for x in 0..self.tile_size {
                let pixel_value = self.get_pixel(x, y);
                file.write_all(&pixel_value.to_le_bytes())?;
            }
        }
        Ok(())
    }
}

pub fn generate_water_bodies_processing_tiles_from_worldcover_ones(
    world_cover_base_tile_id: &DemTileId,
    world_cover_tiles: &Vec<Vec<Raster16>>,
    cache_dir: &Path,
) {
    let processing_dir = cache_dir
        .join(WATER_BODIES_CACHE_DIR)
        .join(WATER_BODIES_PROCESSING_DIR);

    if !processing_dir.exists() {
        std::fs::create_dir_all(&processing_dir).unwrap();
    }

    // downsample the tiles to the WATER_BODY_TILE_SIZE
    for row in 0..world_cover_tiles.len() {
        for col in 0..world_cover_tiles[row].len() {
            let tile = &world_cover_tiles[row][col];

            let tile_id = DemTileId::new(
                world_cover_base_tile_id.lon + col as i16,
                world_cover_base_tile_id.lat + row as i16,
            );
            let downsampled_tile =
                WaterBodiesProcessingTile::downsample_from_worldcover_tile(
                    &tile_id, &tile,
                );

            let tile_file_name = processing_dir
                .join(downsampled_tile.tile_id.to_string())
                .with_extension("wbp");

            downsampled_tile.write_to_file(&tile_file_name).unwrap();
        }
    }
}

// todo 3: implement water bodies coloring algorithm

#[cfg(test)]
pub mod tests {
    use crate::water_bodies::dem_tile_id::DemTileId;
    use crate::water_bodies::water_bodies::WaterBodiesProcessingTile;

    #[derive(Debug)]
    struct Scene {
        scene: String,
    }

    impl Scene {
        fn new(scene: &str) -> Self {
            Scene {
                scene: scene.to_string(),
            }
        }

        fn to_tile(&self) -> WaterBodiesProcessingTile {
            let tile_id = DemTileId::from_tile_name("N54E168").unwrap();

            let lines = self
                .scene
                .lines()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty())
                .collect::<Vec<&str>>();

            let tile_size = lines.len() as u16;

            let mut tile = WaterBodiesProcessingTile::new(&tile_id, tile_size);

            for (y, line) in lines.iter().enumerate() {
                if line.len() != tile_size as usize {
                    panic!("Line length does not match tile size");
                }

                for (x, c) in line.chars().enumerate() {
                    tile.set_pixel(x as u16, y as u16, (c as u16) - '0' as u16);
                }
            }

            tile
        }

        fn from_tile(tile: &WaterBodiesProcessingTile) -> Scene {
            let mut scene = String::new();
            for y in 0..tile.tile_size {
                for x in 0..tile.tile_size {
                    let pixel_value = tile.get_pixel(x, y);
                    scene.push_str(&pixel_value.to_string());
                }
                scene.push('\n');
            }
            Scene::new(&scene)
        }
    }

    impl PartialEq for Scene {
        fn eq(&self, other: &Self) -> bool {
            self.scene.replace("\n", "") == other.scene.replace("\n", "")
        }
    }

    #[test]
    fn icebreaker() {
        let scene = Scene::new(
            r#"
0000100
0010100
1011110
1111100
0011100
0011000
0001000"#,
        );

        let tile = scene.to_tile();
        assert_eq!(tile.get_pixel(0, 0), 0);
        assert_eq!(tile.get_pixel(4, 0), 1);

        assert_eq!(Scene::from_tile(&tile), scene);
    }
}
