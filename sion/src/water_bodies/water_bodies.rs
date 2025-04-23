use crate::raster16::Raster16;
use crate::water_bodies::dem_tile_id::DemTileId;
use std::collections::VecDeque;
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
    cells: Vec<u16>,
}

impl WaterBodiesProcessingTile {
    pub fn new(tile_id: &DemTileId, tile_size: u16) -> Self {
        WaterBodiesProcessingTile {
            tile_id: tile_id.clone(),
            tile_size,
            cells: vec![
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

                downsampled.set_cell(x, y, dominant_color);
            }
        }

        downsampled
    }

    pub fn get_cell(&self, x: u16, y: u16) -> u16 {
        if x >= self.tile_size || y >= self.tile_size {
            panic!("Cell coordinates out of bounds");
        }

        self.cells[y as usize * self.tile_size as usize + x as usize]
    }

    pub fn set_cell(&mut self, x: u16, y: u16, value: u16) {
        if x >= self.tile_size || y >= self.tile_size {
            panic!("Cell coordinates out of bounds");
        }

        self.cells[y as usize * self.tile_size as usize + x as usize] = value;
    }

    pub fn write_to_file(&self, file_name: &Path) -> Result<(), io::Error> {
        let mut file = File::create(file_name)?;
        for y in 0..self.tile_size {
            for x in 0..self.tile_size {
                let pixel_value = self.get_cell(x, y);
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

#[derive(Debug, Clone, Default, PartialEq)]
struct Rect {
    min_x: u16,
    min_y: u16,
    width: u16,
    height: u16,
}

impl Rect {
    fn extend(&mut self, point: (u16, u16)) {
        if self.width == 0 && self.height == 0 {
            self.min_x = point.0;
            self.min_y = point.1;
            self.width = 1;
            self.height = 1;
        } else {
            let max_x = self.min_x + self.width - 1;
            let max_y = self.min_y + self.height - 1;

            self.min_x = self.min_x.min(point.0);
            self.min_y = self.min_y.min(point.1);
            let new_max_x = max_x.max(point.0);
            let new_max_y = max_y.max(point.1);

            self.width = new_max_x - self.min_x + 1;
            self.height = new_max_y - self.min_y + 1;
        }
    }
}

// todo 5: water body colors should start from 2

#[derive(Debug)]
pub struct WaterBody {
    color: u16,
    surface_area: usize,
    coverage: Rect,
}

fn try_color_next_water_body(
    color: u16,
    starting_point: (u16, u16),
    tile: &mut WaterBodiesProcessingTile,
) -> Option<(WaterBody, Option<(u16, u16)>)> {
    let mut current_point = Some(starting_point);
    let mut water_body = None;

    while let Some((x, y)) = current_point {
        let pixel_color = tile.get_cell(x, y);

        let next_point = if x < tile.tile_size - 1 {
            Some((x + 1, y))
        } else if y < tile.tile_size - 1 {
            Some((0, y + 1))
        } else {
            None
        };

        if pixel_color == 1 {
            let mut points_to_color = VecDeque::new();
            points_to_color.push_back((x, y));
            let mut surface_area = 0;
            let mut coverage = Rect::default();

            while let Some((px, py)) = points_to_color.pop_front() {
                let point_index: usize =
                    py as usize * tile.tile_size as usize + px as usize;
                if tile.cells[point_index] == 1 {
                    tile.cells[point_index] = color;
                    surface_area += 1;
                    coverage.extend((px, py as u16));

                    // Neighbor left
                    if px > 0 && tile.cells[point_index - 1] == 1 {
                        points_to_color.push_back((px - 1, py));
                    }
                    // Neighbor right
                    if px < tile.tile_size - 1
                        && tile.cells[point_index + 1] == 1
                    {
                        points_to_color.push_back((px + 1, py));
                    }
                    // Neighbor up
                    if py > 0
                        && tile.cells[point_index - tile.tile_size as usize]
                            == 1
                    {
                        points_to_color.push_back((px, py - 1));
                    }
                    // Neighbor down
                    if py < tile.tile_size - 1
                        && tile.cells[point_index + tile.tile_size as usize]
                            == 1
                    {
                        points_to_color.push_back((px, py + 1));
                    }
                }
            }

            water_body = Some((
                WaterBody {
                    color,
                    surface_area,
                    coverage,
                },
                next_point,
            ));
            current_point = None;
        } else {
            current_point = next_point;
        }
    }

    water_body
}

pub fn color_water_bodies(
    tile: &mut WaterBodiesProcessingTile,
) -> Vec<WaterBody> {
    let mut color = 2;
    let mut water_bodies = Vec::new();
    let mut starting_point = (0, 0);

    while let Some((water_body, next_point)) =
        try_color_next_water_body(color, starting_point, tile)
    {
        if color == u16::MAX {
            panic!("Too many water bodies for u16 index... we need to start using u32");
        }

        water_bodies.push(water_body);
        color += 1;

        if let Some(next) = next_point {
            starting_point = next;
        } else {
            break;
        }
    }

    water_bodies
}

#[cfg(test)]
pub mod tests {
    use crate::water_bodies::dem_tile_id::DemTileId;
    use crate::water_bodies::water_bodies::{Rect, WaterBodiesProcessingTile};

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
                    tile.set_cell(x as u16, y as u16, (c as u16) - '0' as u16);
                }
            }

            tile
        }

        fn from_tile(tile: &WaterBodiesProcessingTile) -> Scene {
            let mut scene = String::new();
            for y in 0..tile.tile_size {
                for x in 0..tile.tile_size {
                    let pixel_value = tile.get_cell(x, y);
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
    fn color_scene_1() {
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

        let mut tile = scene.to_tile();

        let water_bodies = super::color_water_bodies(&mut tile);

        let expected_scene = Scene::new(
            r#"
0000200
0020200
2022220
2222200
0022200
0022000
0002000"#,
        );

        assert_eq!(Scene::from_tile(&tile), expected_scene);

        assert_eq!(water_bodies.len(), 1);
        assert_eq!(water_bodies[0].color, 2);
        assert_eq!(water_bodies[0].surface_area, 19);
        assert_eq!(
            water_bodies[0].coverage,
            Rect {
                min_x: 0,
                min_y: 0,
                width: 6,
                height: 7,
            }
        );
    }

    #[test]
    fn color_scene_2() {
        let scene = Scene::new(
            r#"
0000100
0010100
1011110
1111100
0011100
0011010
0001001"#,
        );

        let mut tile = scene.to_tile();

        let water_bodies = super::color_water_bodies(&mut tile);

        let expected_scene = Scene::new(
            r#"
0000200
0020200
2022220
2222200
0022200
0022030
0002004"#,
        );

        assert_eq!(Scene::from_tile(&tile), expected_scene);

        assert_eq!(water_bodies.len(), 3);
        assert_eq!(water_bodies[1].color, 3);
        assert_eq!(water_bodies[1].surface_area, 1);
        assert_eq!(water_bodies[2].color, 4);
        assert_eq!(water_bodies[2].surface_area, 1)
    }

    #[test]
    fn color_scene_3() {
        let scene = Scene::new(
            r#"
0000
0001
0011
0000"#,
        );

        let mut tile = scene.to_tile();

        let water_bodies = super::color_water_bodies(&mut tile);

        let expected_scene = Scene::new(
            r#"
0000
0002
0022
0000"#,
        );

        assert_eq!(Scene::from_tile(&tile), expected_scene);

        assert_eq!(water_bodies.len(), 1);
        assert_eq!(water_bodies[0].color, 2);
        assert_eq!(water_bodies[0].surface_area, 3);
    }
}
