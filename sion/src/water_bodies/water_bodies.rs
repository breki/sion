use crate::raster16::Raster16;
use crate::water_bodies::dem_tile_id::DemTileId;
use std::fs::File;
use std::io::Write;
use std::io::{self};
use std::path::Path;

pub const WATER_BODY_TILE_SIZE: u16 = 1800;

pub enum WaterBodyValue {
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

    pub fn downsample_from_worldcover_tile(
        tile_id: &DemTileId,
        raster: &Raster16,
    ) -> Self {
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
