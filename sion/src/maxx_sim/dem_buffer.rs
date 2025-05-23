use std::cmp::{max, min};
use std::ops::{Add, Sub};

pub const EARTH_RADIUS_METERS: f32 = 6378137.0;
pub const EARTH_CIRCUMFERENCE_METERS: f32 = 2.0 * PI * EARTH_RADIUS_METERS;

pub const GRID_UNITS_PER_DEM_CELL_BITS: i32 = 8;
pub const GRID_UNITS_PER_DEM_CELL: i32 = 1 << GRID_UNITS_PER_DEM_CELL_BITS;

pub const DEM_TILE_SIZE: i32 = 1800;
pub const DEM_TILE_SIZE_IN_GRID_CELLS: i32 =
    DEM_TILE_SIZE * GRID_UNITS_PER_DEM_CELL;

pub const MAX_PIXELS_PER_METER: f32 = 45.0;

#[derive(Clone)]
pub struct Deg {
    pub value: f32,
}

impl Deg {
    pub fn new(value: f32) -> Deg {
        Deg { value }
    }

    pub fn to_int(&self) -> i32 {
        self.value as i32
    }

    pub fn to_radians(&self) -> f32 {
        self.value * (PI / 180.0)
    }
}

#[derive(Clone)]
pub struct GlobalCell {
    pub value: i32,
}

impl GlobalCell {
    pub fn new(value: i32) -> GlobalCell {
        GlobalCell { value }
    }

    pub fn from_degrees(value: &Deg) -> GlobalCell {
        GlobalCell::new((value.value * DEM_TILE_SIZE as f32) as i32)
    }

    pub fn to_tile_degrees(&self) -> Deg {
        Deg::new(self.value as f32 / DEM_TILE_SIZE as f32)
    }

    pub fn to_local_cell_lon(&self) -> LocalCell {
        LocalCell::new(self.value % DEM_TILE_SIZE)
    }

    pub fn to_local_cell_lat(&self) -> LocalCell {
        LocalCell::new(DEM_TILE_SIZE - 1 - self.value % DEM_TILE_SIZE)
    }
}

use std::cmp::Ordering;
use std::f32::consts::PI;

impl PartialEq for GlobalCell {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for GlobalCell {}

impl PartialOrd for GlobalCell {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GlobalCell {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl Add<GlobalCell> for &GlobalCell {
    type Output = GlobalCell;

    fn add(self, other: GlobalCell) -> GlobalCell {
        GlobalCell {
            value: self.value + other.value,
        }
    }
}

impl Add<i32> for &GlobalCell {
    type Output = GlobalCell;

    fn add(self, other: i32) -> GlobalCell {
        GlobalCell {
            value: self.value + other,
        }
    }
}

impl Sub<&GlobalCell> for &GlobalCell {
    type Output = i32;

    fn sub(self, other: &GlobalCell) -> i32 {
        self.value - other.value
    }
}

impl Sub for GlobalCell {
    type Output = i32;

    fn sub(self, other: GlobalCell) -> i32 {
        self.value - other.value
    }
}

impl Sub<i32> for &GlobalCell {
    type Output = GlobalCell;

    fn sub(self, other: i32) -> GlobalCell {
        GlobalCell {
            value: self.value - other,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LocalCell {
    pub value: i32,
}

impl LocalCell {
    pub fn new(value: i32) -> LocalCell {
        LocalCell { value }
    }
}

impl PartialEq for LocalCell {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

#[derive(Clone)]
pub struct Grid {
    pub value: i32,
}

impl Grid {
    pub fn new(value: i32) -> Grid {
        Grid { value }
    }

    pub fn from_degrees(value: &Deg) -> Grid {
        Grid::new((value.value * DEM_TILE_SIZE_IN_GRID_CELLS as f32) as i32)
    }

    pub fn to_global_cell(&self) -> GlobalCell {
        GlobalCell::new(self.value >> GRID_UNITS_PER_DEM_CELL_BITS)
    }
}

impl Add<i32> for &Grid {
    type Output = Grid;

    fn add(self, other: i32) -> Grid {
        Grid {
            value: self.value + other,
        }
    }
}

impl Sub<&Grid> for &Grid {
    type Output = i32;

    fn sub(self, other: &Grid) -> i32 {
        self.value - other.value
    }
}

impl Sub<i32> for &Grid {
    type Output = Grid;

    fn sub(self, other: i32) -> Grid {
        Grid {
            value: self.value - other,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TileKey {
    pub lon: i32,
    pub lat: i32,
}

impl TileKey {
    pub fn from_lon_lat(lon: i32, lat: i32) -> TileKey {
        TileKey { lon, lat }
    }
}

impl PartialEq for TileKey {
    fn eq(&self, other: &Self) -> bool {
        self.lon == other.lon && self.lat == other.lat
    }
}

fn calculate_pixel_size_in_grid_units(
    latitude_rad: f32,
    zoom_meters_per_pixel: f32,
) -> (i32, i32) {
    let latitude_cos = latitude_rad.cos();
    let circumference_at_latitude = EARTH_CIRCUMFERENCE_METERS * latitude_cos;
    let longitude_degree_length_in_meters_at_latitude =
        circumference_at_latitude / 360.0;
    let dem_cell_horiz_length_in_meters_at_latitude =
        longitude_degree_length_in_meters_at_latitude / DEM_TILE_SIZE as f32;
    let grid_unit_horiz_length_in_meters_at_latitude =
        dem_cell_horiz_length_in_meters_at_latitude / 256.0;
    let pixel_size_in_meters = zoom_meters_per_pixel;

    let horizontal = (pixel_size_in_meters
        / grid_unit_horiz_length_in_meters_at_latitude)
        .round() as i32;
    let vertical = (horizontal as f32 * latitude_cos).round() as i32;

    (horizontal, vertical)
}

#[derive(Clone, Debug)]
pub struct TileSlice {
    pub tile_key: TileKey,
    pub slice_buffer_x0: i32,
    pub slice_buffer_y0: i32,
    pub slice_tile_x0: LocalCell,
    pub slice_tile_y0: LocalCell,
    pub slice_width: i32,
    pub slice_height: i32,
}

impl PartialEq for TileSlice {
    fn eq(&self, other: &Self) -> bool {
        self.tile_key == other.tile_key
            && self.slice_buffer_x0 == other.slice_buffer_x0
            && self.slice_buffer_y0 == other.slice_buffer_y0
            && self.slice_tile_x0 == other.slice_tile_x0
            && self.slice_tile_y0 == other.slice_tile_y0
            && self.slice_width == other.slice_width
            && self.slice_height == other.slice_height
    }
}

#[derive(Clone, Debug)]
pub struct BlockMove {
    pub source_x0: i32,
    pub source_y0: i32,
    pub block_width: i32,
    pub block_height: i32,
    pub dest_x0: i32,
    pub dest_y0: i32,
}

impl PartialEq for BlockMove {
    fn eq(&self, other: &Self) -> bool {
        self.source_x0 == other.source_x0
            && self.source_y0 == other.source_y0
            && self.block_width == other.block_width
            && self.block_height == other.block_height
            && self.dest_x0 == other.dest_x0
            && self.dest_y0 == other.dest_y0
    }
}

#[derive(PartialEq, Debug)]
enum BufferState {
    Uninitialized,
    Initialized,
}

#[derive(PartialEq)]
enum BufferUpdateDecision {
    PartialUpdatePerformed,
    EntireBufferReloadRequired,
}

const MIN_PIXEL_DISTANCE_TO_EDGE_BEFORE_REFRESH: i32 = 500;

pub struct DemBuffer {
    pub width: i32,
    pub height: i32,

    state: BufferState,

    // data: Box<[u8]>,
    center_global_cell_lon: GlobalCell,
    center_global_cell_lat: GlobalCell,

    buffer_west_edge_grid: Grid,
    buffer_east_edge_grid: Grid,
    buffer_north_edge_grid: Grid,
    buffer_south_edge_grid: Grid,

    horizontal_max_pixel_size_in_grids: Grid,
    vertical_max_pixel_size_in_grids: Grid,

    pub slices_loaded: Vec<TileSlice>,
    pub block_move: Option<BlockMove>,
}

impl DemBuffer {
    pub fn new(width: i32, height: i32) -> Self {
        // let size = (width * height) as usize;
        // let data = vec![0; size].into_boxed_slice();
        DemBuffer {
            width,
            height,
            state: BufferState::Uninitialized,
            // data,
            center_global_cell_lon: GlobalCell::new(0),
            center_global_cell_lat: GlobalCell::new(0),

            buffer_west_edge_grid: Grid::new(0),
            buffer_east_edge_grid: Grid::new(0),
            buffer_north_edge_grid: Grid::new(0),
            buffer_south_edge_grid: Grid::new(0),

            horizontal_max_pixel_size_in_grids: Grid::new(0),
            vertical_max_pixel_size_in_grids: Grid::new(0),

            slices_loaded: Vec::new(),
            block_move: None,
        }
    }

    pub fn update_map_position(&mut self, lon: &Deg, lat: &Deg) {
        self.clear_update_log();

        let mut full_update_needed = self.state == BufferState::Uninitialized;

        println!("full_update_needed: {}", full_update_needed);

        if self.state == BufferState::Initialized {
            let update_required = self.is_buffer_update_required(lon, lat);

            println!("update_required: {}", update_required);

            if update_required {
                let update_decision = self.try_partial_update(lon, lat);
                if update_decision
                    == BufferUpdateDecision::EntireBufferReloadRequired
                {
                    full_update_needed = true;
                }
            }
        }

        if full_update_needed {
            self.reload_entire_buffer(lon, lat);
        }
    }

    fn is_buffer_update_required(&self, lon: &Deg, lat: &Deg) -> bool {
        let map_center_lon_grid = Grid::from_degrees(lon);
        let map_center_lat_grid = Grid::from_degrees(lat);

        // calculate the distance of the new map center from the edges
        // of the buffer
        let west_edge_distance_in_pixels = (&map_center_lon_grid
            - &self.buffer_west_edge_grid)
            / self.horizontal_max_pixel_size_in_grids.value;
        let east_edge_distance_in_pixels = (&self.buffer_east_edge_grid
            - &map_center_lon_grid)
            / self.horizontal_max_pixel_size_in_grids.value;
        let north_edge_distance_in_pixels = (&self.buffer_north_edge_grid
            - &map_center_lat_grid)
            / self.vertical_max_pixel_size_in_grids.value;
        let south_edge_distance_in_pixels = (&map_center_lat_grid
            - &self.buffer_south_edge_grid)
            / self.vertical_max_pixel_size_in_grids.value;

        west_edge_distance_in_pixels < MIN_PIXEL_DISTANCE_TO_EDGE_BEFORE_REFRESH
            || east_edge_distance_in_pixels
                < MIN_PIXEL_DISTANCE_TO_EDGE_BEFORE_REFRESH
            || north_edge_distance_in_pixels
                < MIN_PIXEL_DISTANCE_TO_EDGE_BEFORE_REFRESH
            || south_edge_distance_in_pixels
                < MIN_PIXEL_DISTANCE_TO_EDGE_BEFORE_REFRESH
    }

    fn reload_entire_buffer(&mut self, lon: &Deg, lat: &Deg) {
        self.center_global_cell_lon = GlobalCell::from_degrees(lon);
        self.center_global_cell_lat = GlobalCell::from_degrees(lat);

        self.buffer_west_edge_grid = &Grid::from_degrees(lon)
            - ((self.width / 2) << GRID_UNITS_PER_DEM_CELL_BITS);
        self.buffer_east_edge_grid = &self.buffer_west_edge_grid
            + (self.width << GRID_UNITS_PER_DEM_CELL_BITS);
        self.buffer_north_edge_grid = &Grid::from_degrees(lat)
            + ((self.height) / 2 << GRID_UNITS_PER_DEM_CELL_BITS);
        self.buffer_south_edge_grid = &self.buffer_north_edge_grid
            - (self.height << GRID_UNITS_PER_DEM_CELL_BITS);

        // Calculate the maximum pixel size so we can use it when map position
        // is moved to determine whether the DEM buffer needs to be updated.
        let (hpixel_size, vpixel_size) = calculate_pixel_size_in_grid_units(
            lat.to_radians(),
            MAX_PIXELS_PER_METER,
        );

        self.horizontal_max_pixel_size_in_grids = Grid::new(hpixel_size);
        self.vertical_max_pixel_size_in_grids = Grid::new(vpixel_size);

        // the global cell coordinates of the left (west) edge of the buffer
        let buffer_west_edge_global_cell =
            &self.buffer_west_edge_grid.to_global_cell();

        // the top-left corner of the tile slice in the buffer coordinates
        let mut slice_west_edge_global_cell =
            buffer_west_edge_global_cell.clone();
        let mut slice_north_edge_global_cell =
            self.buffer_north_edge_grid.to_global_cell();

        // the buffer coordinates of the top-left corner of the tile slice
        let mut slice_buffer_x0 = 0;
        let mut slice_buffer_y0 = 0;

        let mut next_slice_available = true;

        while next_slice_available {
            // calculate the tile ID of the current slice
            let tile_lon_deg =
                slice_west_edge_global_cell.to_tile_degrees().to_int();
            let tile_lat_deg =
                slice_north_edge_global_cell.to_tile_degrees().to_int();

            // now we know which tile it is
            let tile_key = TileKey::from_lon_lat(tile_lon_deg, tile_lat_deg);

            // calculate the local cell coordinates of the current slice's
            // top-left corner
            let slice_tile_x0 = slice_west_edge_global_cell.to_local_cell_lon();
            let slice_tile_y0 =
                slice_north_edge_global_cell.to_local_cell_lat();

            // calculate the size of the tile slice to be loaded
            let slice_width = min(
                self.width - slice_buffer_x0,
                DEM_TILE_SIZE - slice_tile_x0.value,
            );

            let slice_height = min(
                self.height - slice_buffer_y0,
                DEM_TILE_SIZE - slice_tile_y0.value,
            );

            self.load_tile_slice(&TileSlice {
                tile_key,
                slice_buffer_x0,
                slice_buffer_y0,
                slice_tile_x0,
                slice_tile_y0,
                slice_width,
                slice_height,
            });

            // try to move to the next slice to the right
            slice_buffer_x0 += slice_width;
            slice_west_edge_global_cell =
                &slice_west_edge_global_cell + slice_width;

            if slice_buffer_x0 >= self.width {
                // if we reached the right edge of the buffer...

                // "carriage return" to the left edge of the buffer...
                slice_buffer_x0 = 0;
                slice_west_edge_global_cell =
                    buffer_west_edge_global_cell.clone();

                // ...and move to the next slice to the bottom
                slice_buffer_y0 += slice_height;
                slice_north_edge_global_cell =
                    &slice_north_edge_global_cell - slice_height;

                if slice_buffer_y0 >= self.height {
                    // if we reached the bottom edge of the buffer, we are done
                    next_slice_available = false;
                }
            }
        }

        self.state = BufferState::Initialized;
    }

    fn try_partial_update(
        &mut self,
        lon: &Deg,
        lat: &Deg,
    ) -> BufferUpdateDecision {
        let new_buffer_west_edge_grid =
            &Grid::from_degrees(lon) - ((self.width / 2) << 8);
        let new_buffer_east_edge_grid =
            &new_buffer_west_edge_grid + (self.width << 8);
        let new_buffer_north_edge_grid =
            &Grid::from_degrees(lat) + ((self.height / 2) << 8);
        let new_buffer_south_edge_grid =
            &new_buffer_north_edge_grid - (self.height << 8);

        // Get the coordinates of the edges of the buffer once the buffer
        // has been moved to the new position.
        let new_buffer_west_edge_global_cell =
            new_buffer_west_edge_grid.to_global_cell();
        let new_buffer_east_edge_global_cell =
            new_buffer_east_edge_grid.to_global_cell();
        let new_buffer_north_edge_global_cell =
            new_buffer_north_edge_grid.to_global_cell();
        let new_buffer_south_edge_global_cell =
            new_buffer_south_edge_grid.to_global_cell();

        // Calculate the intersection area between the DEM buffer in its
        // current position and the new position.
        let intersection_west_edge = GlobalCell::new(max(
            self.buffer_west_edge_grid.to_global_cell().value,
            new_buffer_west_edge_global_cell.value,
        ));

        let intersection_east_edge = GlobalCell::new(min(
            self.buffer_east_edge_grid.to_global_cell().value,
            new_buffer_east_edge_global_cell.value,
        ));

        let intersection_north_edge = GlobalCell::new(min(
            self.buffer_north_edge_grid.to_global_cell().value,
            new_buffer_north_edge_global_cell.value,
        ));

        let intersection_south_edge = GlobalCell::new(max(
            self.buffer_south_edge_grid.to_global_cell().value,
            new_buffer_south_edge_global_cell.value,
        ));

        // Now calculate the top-left corner of the intersection in the
        // buffer's local coordinates.
        let source_x0 = &intersection_west_edge
            - &self.buffer_west_edge_grid.to_global_cell();
        let source_y0 = &self.buffer_north_edge_grid.to_global_cell()
            - &intersection_north_edge;

        // Also calculate the width and height of the intersection.
        let block_width = &intersection_east_edge - &intersection_west_edge;
        let block_height = &intersection_north_edge - &intersection_south_edge;

        let update_decision;

        // is there actually any intersection?
        if block_width >= 0 && block_height >= 0 {
            println!("Intersection found!");

            // if there is an intersection, we can perform a partial update
            update_decision = BufferUpdateDecision::PartialUpdatePerformed;

            // And now calculate the coordinates of the intersection, but this
            // time in the local coordinates of the buffer after it has been moved
            // to a new location. This will serve as the destination coordinates
            // where the intersection area will be copied to.
            let dest_x0 =
                &intersection_west_edge - &new_buffer_west_edge_global_cell;
            let dest_y0 =
                &new_buffer_north_edge_global_cell - &intersection_north_edge;

            self.move_dem_block(
                source_x0,
                source_y0,
                block_width,
                block_height,
                dest_x0,
                dest_y0,
            );

            // update the buffer's fields
            self.center_global_cell_lon = GlobalCell::from_degrees(lon);
            self.center_global_cell_lat = GlobalCell::from_degrees(lat);

            let (hpixel_size, vpixel_size) = calculate_pixel_size_in_grid_units(
                lat.to_radians(),
                MAX_PIXELS_PER_METER,
            );

            self.horizontal_max_pixel_size_in_grids = Grid::new(hpixel_size);
            self.vertical_max_pixel_size_in_grids = Grid::new(vpixel_size);

            self.buffer_north_edge_grid = new_buffer_north_edge_grid;
            self.buffer_south_edge_grid = new_buffer_south_edge_grid;
            self.buffer_west_edge_grid = new_buffer_west_edge_grid;
            self.buffer_east_edge_grid = new_buffer_east_edge_grid;
        } else {
            println!("No intersection found!");

            // if there is no intersection, we need to do a full buffer reload
            update_decision = BufferUpdateDecision::EntireBufferReloadRequired;
        }

        update_decision
    }

    fn move_dem_block(
        &mut self,
        source_x0: i32,
        source_y0: i32,
        block_width: i32,
        block_height: i32,
        dest_x0: i32,
        dest_y0: i32,
    ) {
        // Here we would implement the logic to move the DEM block
        // from the source coordinates to the destination coordinates.
        // This is a placeholder implementation.
        self.block_move = Some(BlockMove {
            source_x0,
            source_y0,
            block_width,
            block_height,
            dest_x0,
            dest_y0,
        });
    }

    fn load_tile_slice(&mut self, slice: &TileSlice) {
        self.slices_loaded.push(slice.clone());
    }

    fn clear_update_log(&mut self) {
        self.slices_loaded.clear();
        self.block_move = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_loading() {
        let mut dem_buffer = DemBuffer::new(2000, 2000);

        dem_buffer.update_map_position(&Deg::new(7.65532), &Deg::new(46.64649));

        assert_eq!(dem_buffer.state, BufferState::Initialized);

        assert_eq!(dem_buffer.block_move, None);
        assert_eq!(dem_buffer.slices_loaded.len(), 4);

        assert_eq!(
            dem_buffer.slices_loaded[0],
            TileSlice {
                tile_key: TileKey::from_lon_lat(7, 47),
                slice_buffer_x0: 0,
                slice_buffer_y0: 0,
                slice_tile_x0: LocalCell::new(179),
                slice_tile_y0: LocalCell::new(1436),
                slice_width: 1621,
                slice_height: 364,
            }
        );

        assert_eq!(
            dem_buffer.slices_loaded[1],
            TileSlice {
                tile_key: TileKey::from_lon_lat(8, 47),
                slice_buffer_x0: 1621,
                slice_buffer_y0: 0,
                slice_tile_x0: LocalCell::new(0),
                slice_tile_y0: LocalCell::new(1436),
                slice_width: 379,
                slice_height: 364,
            }
        );

        assert_eq!(
            dem_buffer.slices_loaded[2],
            TileSlice {
                tile_key: TileKey::from_lon_lat(7, 46),
                slice_buffer_x0: 0,
                slice_buffer_y0: 364,
                slice_tile_x0: LocalCell::new(179),
                slice_tile_y0: LocalCell::new(0),
                slice_width: 1621,
                slice_height: 1636,
            }
        );

        assert_eq!(
            dem_buffer.slices_loaded[3],
            TileSlice {
                tile_key: TileKey::from_lon_lat(8, 46),
                slice_buffer_x0: 1621,
                slice_buffer_y0: 364,
                slice_tile_x0: LocalCell::new(0),
                slice_tile_y0: LocalCell::new(0),
                slice_width: 379,
                slice_height: 1636,
            }
        );
    }

    #[test]
    fn test_no_update_is_required_if_no_movement() {
        let mut dem_buffer = DemBuffer::new(2000, 2000);
        dem_buffer.update_map_position(&Deg::new(7.65532), &Deg::new(46.64649));

        // Simulate an update with no movement
        dem_buffer.update_map_position(&Deg::new(7.65532), &Deg::new(46.64649));

        assert_eq!(dem_buffer.state, BufferState::Initialized);

        assert_eq!(dem_buffer.block_move, None);
        assert_eq!(dem_buffer.slices_loaded.len(), 0);
    }

    #[test]
    fn test_partial_update_is_required() {
        let mut dem_buffer = DemBuffer::new(2000, 2000);
        dem_buffer.update_map_position(&Deg::new(7.65532), &Deg::new(46.64649));

        // Simulate a partial update
        dem_buffer.update_map_position(&Deg::new(8.0), &Deg::new(46.64649));

        assert_eq!(dem_buffer.state, BufferState::Initialized);

        // todo 2: this condition will be true once we implement the partial
        // loading of DEM data
        // assert!(dem_buffer.slices_loaded.len() > 0);
        assert_eq!(
            dem_buffer.block_move,
            Some(BlockMove {
                source_x0: 621,
                source_y0: 0,
                block_width: 1379,
                block_height: 2000,
                dest_x0: 0,
                dest_y0: 0
            })
        );
    }

    #[test]
    fn test_moved_too_far_so_full_reload_is_needed() {
        let mut dem_buffer = DemBuffer::new(2000, 2000);
        dem_buffer.update_map_position(&Deg::new(7.65532), &Deg::new(46.64649));

        // Simulate a partial update
        dem_buffer.update_map_position(&Deg::new(9.0), &Deg::new(46.64649));

        assert_eq!(dem_buffer.state, BufferState::Initialized);

        assert_eq!(dem_buffer.block_move, None);
        assert_eq!(dem_buffer.slices_loaded.len(), 4);
    }
}
