use std::cmp::min;
use std::ops::{Add, Sub};

pub const GRID_UNITS_PER_DEM_CELL_BITS: i32 = 8;
pub const GRID_UNITS_PER_DEM_CELL: i32 = 1 << GRID_UNITS_PER_DEM_CELL_BITS;

pub const DEM_TILE_SIZE: i32 = 1800;
pub const DEM_TILE_SIZE_IN_GRID_CELLS: i32 =
    DEM_TILE_SIZE * GRID_UNITS_PER_DEM_CELL;

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

impl Add<i32> for &GlobalCell {
    type Output = GlobalCell;

    fn add(self, other: i32) -> GlobalCell {
        GlobalCell {
            value: self.value + other,
        }
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

#[derive(Clone)]
pub struct LocalCell {
    pub value: i32,
}

impl LocalCell {
    pub fn new(value: i32) -> LocalCell {
        LocalCell { value }
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

#[derive(Clone)]
pub struct TileSlice {
    pub tile_key: TileKey,
    pub slice_buffer_x0: i32,
    pub slice_buffer_y0: i32,
    pub slice_tile_x0: LocalCell,
    pub slice_tile_y0: LocalCell,
    pub slice_width: i32,
    pub slice_height: i32,
}

#[derive(PartialEq)]
enum BufferState {
    Uninitialized,
    Initialized,
}

#[derive(PartialEq)]
enum BufferUpdateDecision {
    _None,
    _PartialUpdatePerformed,
    EntireBufferReloadRequired,
}

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

    pub slices_loaded: Vec<TileSlice>,
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
            slices_loaded: Vec::new(),
        }
    }

    pub fn update_map_position(&mut self, lon: &Deg, lat: &Deg) {
        let mut full_update_needed = self.state == BufferState::Uninitialized;

        if self.state == BufferState::Initialized {
            let update_required = self.is_buffer_update_required(lon, lat);
            if update_required {
                let update_decision = self.update_buffer(lon, lat);
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

    fn is_buffer_update_required(&self, _lon: &Deg, _lat: &Deg) -> bool {
        true
    }

    fn update_buffer(
        &mut self,
        _lon: &Deg,
        _lat: &Deg,
    ) -> BufferUpdateDecision {
        panic!("Not implemented yet");
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

    fn load_tile_slice(&mut self, slice: &TileSlice) {
        self.slices_loaded.push(slice.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_loading() {
        let mut dem_buffer = DemBuffer::new(2000, 2000);

        dem_buffer.update_map_position(&Deg::new(7.65532), &Deg::new(46.64649));

        assert_eq!(dem_buffer.slices_loaded.len(), 4);

        let slice = &dem_buffer.slices_loaded[0];
        assert_eq!(slice.tile_key, TileKey::from_lon_lat(7, 47));
        assert_eq!(slice.slice_buffer_x0, 0);
        assert_eq!(slice.slice_buffer_y0, 0);
        assert_eq!(slice.slice_tile_x0.value, 179);
        assert_eq!(slice.slice_tile_y0.value, 1436);
        assert_eq!(slice.slice_width, 1621);
        assert_eq!(slice.slice_height, 364);

        let slice = &dem_buffer.slices_loaded[1];
        assert_eq!(slice.tile_key, TileKey::from_lon_lat(8, 47));
        assert_eq!(slice.slice_buffer_x0, 1621);
        assert_eq!(slice.slice_buffer_y0, 0);
        assert_eq!(slice.slice_tile_x0.value, 0);
        assert_eq!(slice.slice_tile_y0.value, 1436);
        assert_eq!(slice.slice_width, 379);
        assert_eq!(slice.slice_height, 364);

        let slice = &dem_buffer.slices_loaded[2];
        assert_eq!(slice.tile_key, TileKey::from_lon_lat(7, 46));
        assert_eq!(slice.slice_buffer_x0, 0);
        assert_eq!(slice.slice_buffer_y0, 364);
        assert_eq!(slice.slice_tile_x0.value, 179);
        assert_eq!(slice.slice_tile_y0.value, 0);
        assert_eq!(slice.slice_width, 1621);
        assert_eq!(slice.slice_height, 1636);

        let slice = &dem_buffer.slices_loaded[3];
        assert_eq!(slice.tile_key, TileKey::from_lon_lat(8, 46));
        assert_eq!(slice.slice_buffer_x0, 1621);
        assert_eq!(slice.slice_buffer_y0, 364);
        assert_eq!(slice.slice_tile_x0.value, 0);
        assert_eq!(slice.slice_tile_y0.value, 0);
        assert_eq!(slice.slice_width, 379);
        assert_eq!(slice.slice_height, 1636);
    }
}
