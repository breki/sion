use crate::maxx_sim::types::{Deg, GlobalCell, LocalCell, TileKey};
use proptest::prelude::ProptestConfig;
use std::cmp::{max, min};

pub struct CellKey {
    value: i32,
}

impl CellKey {
    pub fn from_cell_coords(x: GlobalCell, y: GlobalCell) -> Self {
        let cell_x = x.value;
        let cell_y = y.value;
        let value = cell_y * 100000 + cell_x;
        CellKey {
            value, // Assuming a unique encoding for the cell
        }
    }

    pub fn from_i32(value: i32) -> Self {
        CellKey { value }
    }

    pub fn to_cell_coords(&self) -> (GlobalCell, GlobalCell) {
        let cell_x = self.value % 100000;
        let cell_y = self.value / 100000;
        (GlobalCell::new(cell_x), GlobalCell::new(cell_y))
    }

    pub fn to_i32(&self) -> i32 {
        self.value
    }
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

#[derive(Debug)]
pub struct DemBuffer {
    pub buffer_width: i32,
    pub buffer_height: i32,
    pub dem_tile_size: i32,
    min_cell_distance_to_edge_before_refresh: i32,

    state: BufferState,

    // todo 0: the buffer data should be the global cell coordinates
    data: Box<[i32]>,
    center_global_cell_lon: GlobalCell,
    center_global_cell_lat: GlobalCell,

    buffer_west_edge: GlobalCell,
    buffer_east_edge: GlobalCell,
    buffer_north_edge: GlobalCell,
    buffer_south_edge: GlobalCell,

    pub slices_loaded: Vec<TileSlice>,
    pub block_move: Option<BlockMove>,
}

impl DemBuffer {
    pub fn new(
        width: i32,
        height: i32,
        dem_tile_size: i32,
        min_cell_distance_to_edge_before_refresh: i32,
    ) -> Self {
        let size = (width * height) as usize;
        let data = vec![0; size].into_boxed_slice();

        DemBuffer {
            buffer_width: width,
            buffer_height: height,
            dem_tile_size,
            min_cell_distance_to_edge_before_refresh,
            state: BufferState::Uninitialized,
            data,
            center_global_cell_lon: GlobalCell::new(0),
            center_global_cell_lat: GlobalCell::new(0),

            buffer_west_edge: GlobalCell::new(0),
            buffer_east_edge: GlobalCell::new(0),
            buffer_north_edge: GlobalCell::new(0),
            buffer_south_edge: GlobalCell::new(0),

            slices_loaded: Vec::new(),
            block_move: None,
        }
    }

    pub fn update_map_position(
        &mut self,
        lon: &Deg,
        lat: &Deg,
        visible_area_width: i32,
        visible_area_height: i32,
    ) {
        self.clear_update_log();

        let mut full_update_needed = self.state == BufferState::Uninitialized;

        if self.state == BufferState::Initialized {
            let update_required = self.is_buffer_update_required(
                lon,
                lat,
                visible_area_width,
                visible_area_height,
            );

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

    fn is_buffer_update_required(
        &self,
        lon: &Deg,
        lat: &Deg,
        visible_area_width: i32,
        visible_area_height: i32,
    ) -> bool {
        let map_center_lon_cell =
            GlobalCell::from_degrees(lon, self.dem_tile_size);
        let map_center_lat_cell =
            GlobalCell::from_degrees(lat, self.dem_tile_size);

        let visible_west_edge = &map_center_lon_cell - (visible_area_width / 2);
        let visible_east_edge = &visible_west_edge + visible_area_width;
        let visible_north_edge =
            &map_center_lat_cell + (visible_area_height / 2);
        let visible_south_edge = &visible_north_edge - visible_area_height;

        let west_edge_distance_in_cells =
            &visible_west_edge - &self.buffer_west_edge;
        let east_edge_distance_in_cells =
            &self.buffer_east_edge - &visible_east_edge;
        let north_edge_distance_in_cells =
            &self.buffer_north_edge - &visible_north_edge;
        let south_edge_distance_in_cells =
            &visible_south_edge - &self.buffer_south_edge;

        west_edge_distance_in_cells
            < self.min_cell_distance_to_edge_before_refresh
            || east_edge_distance_in_cells
                < self.min_cell_distance_to_edge_before_refresh
            || north_edge_distance_in_cells
                < self.min_cell_distance_to_edge_before_refresh
            || south_edge_distance_in_cells
                < self.min_cell_distance_to_edge_before_refresh
    }

    fn reload_entire_buffer(&mut self, lon: &Deg, lat: &Deg) {
        self.clear_data();

        self.center_global_cell_lon =
            GlobalCell::from_degrees(lon, self.dem_tile_size);
        self.center_global_cell_lat =
            GlobalCell::from_degrees(lat, self.dem_tile_size);

        self.buffer_west_edge =
            &self.center_global_cell_lon - self.buffer_width / 2;
        self.buffer_east_edge = &self.buffer_west_edge + self.buffer_width;
        self.buffer_north_edge =
            &self.center_global_cell_lat + (self.buffer_height) / 2;
        self.buffer_south_edge = &self.buffer_north_edge - self.buffer_height;

        self.update_buffer_area(0, 0, self.buffer_width, self.buffer_height);

        self.state = BufferState::Initialized;
    }

    fn try_partial_update(
        &mut self,
        lon: &Deg,
        lat: &Deg,
    ) -> BufferUpdateDecision {
        let new_buffer_west_edge_global_cell =
            &GlobalCell::from_degrees(lon, self.dem_tile_size)
                - self.buffer_width / 2;
        let new_buffer_east_edge_global_cell =
            &new_buffer_west_edge_global_cell + self.buffer_width;
        let new_buffer_north_edge_global_cell =
            &GlobalCell::from_degrees(lat, self.dem_tile_size)
                + self.buffer_height / 2;
        let new_buffer_south_edge_global_cell =
            &new_buffer_north_edge_global_cell - self.buffer_height;

        // Calculate the intersection area between the DEM buffer in its
        // current position and the new position.
        let intersection_west_edge = GlobalCell::new(max(
            self.buffer_west_edge.value,
            new_buffer_west_edge_global_cell.value,
        ));

        let intersection_east_edge = GlobalCell::new(min(
            self.buffer_east_edge.value,
            new_buffer_east_edge_global_cell.value,
        ));

        let intersection_north_edge = GlobalCell::new(min(
            self.buffer_north_edge.value,
            new_buffer_north_edge_global_cell.value,
        ));

        let intersection_south_edge = GlobalCell::new(max(
            self.buffer_south_edge.value,
            new_buffer_south_edge_global_cell.value,
        ));

        // Now calculate the top-left corner of the intersection in the
        // buffer's local coordinates.
        let source_x0 = &intersection_west_edge - &self.buffer_west_edge;
        let source_y0 = &self.buffer_north_edge - &intersection_north_edge;

        // Also calculate the width and height of the intersection.
        let block_width = &intersection_east_edge - &intersection_west_edge;
        let block_height = &intersection_north_edge - &intersection_south_edge;

        let update_decision;

        // is there actually any intersection?
        if block_width >= 0 && block_height >= 0 {
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
            self.center_global_cell_lon =
                GlobalCell::from_degrees(lon, self.dem_tile_size);
            self.center_global_cell_lat =
                GlobalCell::from_degrees(lat, self.dem_tile_size);

            self.buffer_north_edge = new_buffer_north_edge_global_cell;
            self.buffer_south_edge = new_buffer_south_edge_global_cell;
            self.buffer_west_edge = new_buffer_west_edge_global_cell;
            self.buffer_east_edge = new_buffer_east_edge_global_cell;

            // now load slices from DEM files
            self.fill_missing_data_after_move();
        } else {
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
        // The move is simulated in a such a way as to copy the block of data
        // from the source position to the destination position, and clear
        // all the remaining cells in the buffer. This is to ensure
        // that the partial update of those remaining cells always sets the
        // cells from an uninitialized state to the new values. If the update
        // tries to overwrite the already initialized cells, it will
        // indicate a bug in the update algorithm.

        // First, we need to copy the data from the buffer to a temporary
        // buffer
        let mut data_copy: Vec<i32> =
            vec![0; (self.buffer_width * self.buffer_height) as usize];
        for y in 0..self.buffer_height {
            for x in 0..self.buffer_width {
                let source_cell = self.get_cell(x, y);
                let dest_index = (y * self.buffer_width + x) as usize;
                data_copy[dest_index] = source_cell.to_i32();
            }
        }

        // now clean the original data
        self.clear_data();

        // ... and copy the block from the copy to the new position
        for y in 0..block_height {
            for x in 0..block_width {
                let source_index =
                    (source_y0 + y) * self.buffer_width + (source_x0 + x);
                self.set_cell(
                    dest_x0 + x,
                    dest_y0 + y,
                    &CellKey::from_i32(data_copy[source_index as usize]),
                );
            }
        }

        self.block_move = Some(BlockMove {
            source_x0,
            source_y0,
            block_width,
            block_height,
            dest_x0,
            dest_y0,
        });
    }

    fn fill_missing_data_after_move(&mut self) {
        match self.block_move {
            Some(ref block_move) => {
                // todo 6: now cover all the possible cases of missing areas

                let missing_area_x;
                let missing_area_y;
                let missing_area_width;
                let missing_area_height;

                // Calculate the missing area based on the block move
                if block_move.dest_x0 == 0 {
                    missing_area_x = block_move.block_width;
                    missing_area_y = 0;
                    missing_area_width =
                        self.buffer_width - block_move.block_width;
                    missing_area_height = self.buffer_height;
                } else {
                    missing_area_x = 0;
                    missing_area_y = 0;
                    missing_area_width = block_move.dest_x0;
                    missing_area_height = self.buffer_height;
                }

                // Update the buffer area with the missing area
                self.update_buffer_area(
                    missing_area_x,
                    missing_area_y,
                    missing_area_width,
                    missing_area_height,
                );
            }
            None => {
                panic!("No block move found, this method should not be called in this case.");
            }
        }
    }

    fn update_buffer_area(
        &mut self,
        area_x: i32,
        area_y: i32,
        area_width: i32,
        area_height: i32,
    ) {
        // the global cell coordinates of the left (west) edge of the buffer
        let area_west_edge_global_cell = &self.buffer_west_edge + area_x;

        // the top-left corner of the tile slice in the buffer coordinates
        let mut slice_west_edge_global_cell =
            area_west_edge_global_cell.clone();
        let mut slice_north_edge_global_cell = self.buffer_north_edge.clone();

        // the buffer coordinates of the top-left corner of the tile slice
        let mut slice_buffer_x0 = area_x;
        let mut slice_buffer_y0 = area_y;

        let mut next_slice_available = true;

        while next_slice_available {
            // calculate the tile ID of the current slice
            let tile_lon_deg = slice_west_edge_global_cell
                .to_tile_degrees(self.dem_tile_size)
                .to_int();
            let tile_lat_deg = slice_north_edge_global_cell
                .to_tile_degrees(self.dem_tile_size)
                .to_int();

            // now we know which tile it is
            let tile_key = TileKey::from_lon_lat(tile_lon_deg, tile_lat_deg);

            // calculate the local cell coordinates of the current slice's
            // top-left corner
            let slice_tile_x0 = slice_west_edge_global_cell
                .to_local_cell_lon(self.dem_tile_size);
            let slice_tile_y0 = slice_north_edge_global_cell
                .to_local_cell_lat(self.dem_tile_size);

            // calculate the size of the tile slice to be loaded
            let area_x1 = area_x + area_width;
            let slice_width = min(
                area_x1 - slice_buffer_x0,
                self.dem_tile_size - slice_tile_x0.value,
            );

            let area_y1 = area_y + area_height;
            let slice_height = min(
                area_y1 - slice_buffer_y0,
                self.dem_tile_size - slice_tile_y0.value,
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

            if slice_buffer_x0 >= area_width {
                // if we reached the right edge of the area...

                // "carriage return" to the left edge of the area...
                slice_buffer_x0 = area_x;
                slice_west_edge_global_cell =
                    area_west_edge_global_cell.clone();

                // ...and move to the next slice to the bottom
                slice_buffer_y0 += slice_height;
                slice_north_edge_global_cell -= slice_height;

                if slice_buffer_y0 >= area_height {
                    // if we reached the bottom edge of the buffer, we are done
                    next_slice_available = false;
                }
            }
        }
    }

    fn load_tile_slice(&mut self, slice: &TileSlice) {
        let lon_global_cell = GlobalCell::from_degrees(
            &Deg::new(slice.tile_key.lon as f32),
            self.dem_tile_size,
        );
        let lat_global_cell = GlobalCell::from_degrees(
            &Deg::new(slice.tile_key.lat as f32),
            self.dem_tile_size,
        );

        for y in 0..slice.slice_height {
            for x in 0..slice.slice_width {
                let dem_lon_global_cell =
                    &lon_global_cell + (slice.slice_tile_x0.value + x);
                let dem_lat_global_cell = &lat_global_cell
                    + (self.dem_tile_size
                        - 1
                        - (slice.slice_tile_y0.value + y));

                self.set_cell(
                    slice.slice_buffer_x0 + x,
                    slice.slice_buffer_y0 + y,
                    &CellKey::from_cell_coords(
                        dem_lon_global_cell,
                        dem_lat_global_cell,
                    ),
                );
            }
        }

        self.slices_loaded.push(slice.clone());
    }

    fn get_cell(&self, x: i32, y: i32) -> CellKey {
        let index = (y * self.buffer_width + x) as usize;
        if index < self.data.len() {
            CellKey::from_i32(self.data[index])
        } else {
            panic!("Index out of bounds");
        }
    }

    fn set_cell(&mut self, x: i32, y: i32, value: &CellKey) {
        let index = (y * self.buffer_width + x) as usize;
        if index < self.data.len() {
            if self.data[index] != 0 {
                // If the cell is already occupied, this indicates the buffer
                // update algorithm has a bug. Only empty cells should be
                // overwritten during the update.
                panic!(
                    "Bug: Trying to overwrite an already occupied cell at ({}, {})",
                    x, y
                );
            }

            self.data[index] = value.to_i32();
        } else {
            panic!("Index out of bounds");
        }
    }

    fn clear_data(&mut self) {
        for cell in self.data.iter_mut() {
            *cell = 0;
        }
    }

    fn clear_update_log(&mut self) {
        self.slices_loaded.clear();
        self.block_move = None;
    }

    pub fn prop_center_cell_is_correct_one(&self) -> bool {
        let center_x = self.buffer_width / 2;
        let center_y = self.buffer_height / 2;

        let center_cell = self.get_cell(center_x, center_y);
        let (cell_x, cell_y) = center_cell.to_cell_coords();

        cell_x == self.center_global_cell_lon
            && cell_y == self.center_global_cell_lat
    }

    /// Checks if all cells in the buffer are set (not empty).
    ///
    /// If not all cells are set, it indicates that the buffer update
    /// algorithm has a bug, as it did not cover all the buffer.
    pub fn prop_all_cells_are_set(&self) -> bool {
        for cell in self.data.iter() {
            if *cell == 0 {
                return false; // Found an empty cell
            }
        }
        true // All cells are set
    }

    pub fn prop_all_cells_are_good_neighbors(&self) -> bool {
        for y in 0..self.buffer_height - 1 {
            for x in 0..self.buffer_width - 1 {
                let cell = self.get_cell(x, y);
                let (cell_x, cell_y) = cell.to_cell_coords();

                let east_neighbor = self.get_cell(x + 1, y);
                let (east_neighbor_x, east_neighbor_y) =
                    east_neighbor.to_cell_coords();

                if east_neighbor_x.value != cell_x.value + 1
                    || east_neighbor_y.value != cell_y.value
                {
                    println!(
                        "{}, {}: ({}, {}) >> ({}, {})",
                        x,
                        y,
                        cell_x.value,
                        cell_y.value,
                        east_neighbor_x.value,
                        east_neighbor_y.value
                    );
                    return false; // East neighbor is not a good neighbor
                }

                let south_neighbor = self.get_cell(x, y + 1);
                let (south_neighbor_x, south_neighbor_y) =
                    south_neighbor.to_cell_coords();

                if south_neighbor_x.value != cell_x.value
                    || south_neighbor_y.value != cell_y.value - 1
                {
                    println!(
                        "{}, {}: ({}, {}) VV ({}, {})",
                        x,
                        y,
                        cell_x.value,
                        cell_y.value,
                        south_neighbor_x.value,
                        south_neighbor_y.value
                    );
                    return false; // South neighbor is not a good neighbor
                }
            }
        }

        true // All cells have good neighbors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_test_initial_loading() {
        let mut dem_buffer = DemBuffer::new(200, 200, 180, 30);

        dem_buffer.update_map_position(
            &Deg::new(7.65532),
            &Deg::new(46.64649),
            200,
            100,
        );

        assert!(dem_buffer.prop_center_cell_is_correct_one());
        assert!(dem_buffer.prop_all_cells_are_set());
        assert!(dem_buffer.prop_all_cells_are_good_neighbors());

        assert_eq!(dem_buffer.state, BufferState::Initialized);

        assert_eq!(dem_buffer.block_move, None);
        assert_eq!(dem_buffer.slices_loaded.len(), 4);
    }

    #[test]
    fn new_test_no_update_is_required_if_no_movement() {
        let visible_area_width = 80;
        let visible_area_height = 60;

        let mut dem_buffer = DemBuffer::new(200, 200, 180, 30);

        dem_buffer.update_map_position(
            &Deg::new(7.65532),
            &Deg::new(46.64649),
            visible_area_width,
            visible_area_height,
        );

        assert!(dem_buffer.prop_center_cell_is_correct_one());
        assert!(dem_buffer.prop_all_cells_are_set());
        assert!(dem_buffer.prop_all_cells_are_good_neighbors());

        // Simulate an update with no movement
        dem_buffer.update_map_position(
            &Deg::new(7.65532),
            &Deg::new(46.64649),
            visible_area_width,
            visible_area_height,
        );

        assert!(dem_buffer.prop_center_cell_is_correct_one());
        assert!(dem_buffer.prop_all_cells_are_set());
        assert!(dem_buffer.prop_all_cells_are_good_neighbors());

        assert_eq!(dem_buffer.state, BufferState::Initialized);

        assert_eq!(dem_buffer.block_move, None);
        assert_eq!(dem_buffer.slices_loaded.len(), 0);
    }

    #[test]
    fn new_test_moved_too_far_so_full_reload_is_needed() {
        let visible_area_width = 80;
        let visible_area_height = 60;

        let mut dem_buffer = DemBuffer::new(200, 200, 180, 30);
        dem_buffer.update_map_position(
            &Deg::new(7.65532),
            &Deg::new(46.64649),
            visible_area_width,
            visible_area_height,
        );

        assert!(dem_buffer.prop_center_cell_is_correct_one());
        assert!(dem_buffer.prop_all_cells_are_set());
        assert!(dem_buffer.prop_all_cells_are_good_neighbors());

        // Simulate a partial update
        dem_buffer.update_map_position(
            &Deg::new(9.0),
            &Deg::new(46.64649),
            visible_area_width,
            visible_area_height,
        );

        assert!(dem_buffer.prop_center_cell_is_correct_one());
        assert!(dem_buffer.prop_all_cells_are_set());
        assert!(dem_buffer.prop_all_cells_are_good_neighbors());

        assert_eq!(dem_buffer.state, BufferState::Initialized);

        assert_eq!(dem_buffer.block_move, None);
        assert_eq!(dem_buffer.slices_loaded.len(), 4);
    }

    #[test]
    fn new_test_partial_update_is_required_to_the_right() {
        let buffer_size = 200;
        let visible_area_width = 80;
        let visible_area_height = 60;

        let mut dem_buffer = DemBuffer::new(buffer_size, buffer_size, 180, 30);
        dem_buffer.update_map_position(
            &Deg::new(7.65532),
            &Deg::new(46.64649),
            visible_area_width,
            visible_area_height,
        );

        assert!(dem_buffer.prop_center_cell_is_correct_one());
        assert!(dem_buffer.prop_all_cells_are_set());
        assert!(dem_buffer.prop_all_cells_are_good_neighbors());

        // Simulate a partial update
        dem_buffer.update_map_position(
            &Deg::new(8.0),
            &Deg::new(46.64649),
            visible_area_width,
            visible_area_height,
        );

        assert!(dem_buffer.prop_center_cell_is_correct_one());
        assert!(dem_buffer.prop_all_cells_are_set());
        assert!(dem_buffer.prop_all_cells_are_good_neighbors());

        assert_eq!(dem_buffer.state, BufferState::Initialized);
        assert_ne!(dem_buffer.block_move, None);
        assert_eq!(dem_buffer.slices_loaded.len(), 2);
    }

    #[test]
    fn new_test_partial_update_is_required_to_the_left() {
        let buffer_size = 200;
        let visible_area_width = 80;
        let visible_area_height = 60;

        let mut dem_buffer = DemBuffer::new(buffer_size, buffer_size, 180, 30);
        dem_buffer.update_map_position(
            &Deg::new(7.65532),
            &Deg::new(46.64649),
            visible_area_width,
            visible_area_height,
        );

        assert!(dem_buffer.prop_center_cell_is_correct_one());
        assert!(dem_buffer.prop_all_cells_are_set());
        assert!(dem_buffer.prop_all_cells_are_good_neighbors());

        // Simulate a partial update
        dem_buffer.update_map_position(
            &Deg::new(7.2),
            &Deg::new(46.64649),
            visible_area_width,
            visible_area_height,
        );

        assert!(dem_buffer.prop_center_cell_is_correct_one());
        assert!(dem_buffer.prop_all_cells_are_set());
        assert!(dem_buffer.prop_all_cells_are_good_neighbors());

        assert_eq!(dem_buffer.state, BufferState::Initialized);
        assert_ne!(dem_buffer.block_move, None);
        assert_eq!(dem_buffer.slices_loaded.len(), 4);
    }

    #[test]
    fn test_cell_keys_1() {
        let cell_key = CellKey::from_cell_coords(
            GlobalCell::new(100),
            GlobalCell::new(200),
        );

        let (tile_lon_cell, tile_lat_cell) = cell_key.to_cell_coords();

        assert_eq!(tile_lon_cell.value, 100);
        assert_eq!(tile_lat_cell.value, 200);
    }

    #[test]
    fn test_cell_keys_2() {
        let cell_key = CellKey::from_cell_coords(
            GlobalCell::new(-100),
            GlobalCell::new(-200),
        );

        let (tile_lon_cell, tile_lat_cell) = cell_key.to_cell_coords();

        assert_eq!(tile_lon_cell.value, -100);
        assert_eq!(tile_lat_cell.value, -200);
    }
}

// use proptest::prelude::*;
//
// fn arb_dem_buffer() -> BoxedStrategy<DemBuffer> {
//     let buffer_size = 200;
//     let dem_tile_size = 180;
//     let min_cell_distance_to_edge_before_refresh = 30;
//
//     let mut buffer = DemBuffer::new(
//         buffer_size,
//         buffer_size,
//         dem_tile_size,
//         min_cell_distance_to_edge_before_refresh,
//     );
//
//     let visible_area_width = 80;
//     let visible_area_height = 60;
//
//     let lon = Deg::new(7.65532);
//     let lat = Deg::new(46.64649);
//
//     buffer.update_map_position(
//         &lon,
//         &lat,
//         visible_area_width,
//         visible_area_height,
//     );
//
//     buffer.boxed()
// }

// impl Strategy for DemBuffer {
//     type Value = Self;
//
//     fn new_tree(&self, runner: &mut TestRunner) -> NewTree<Self> {
//         let buffer_size = 200;
//         let dem_tile_size = runner.gen_range(180..=360);
//         let min_cell_distance_to_edge_before_refresh =
//             runner.gen_range(10..=50);
//
//         let buffer = DemBuffer::new(
//             buffer_size,
//             buffer_size,
//             dem_tile_size,
//             min_cell_distance_to_edge_before_refresh,
//         );
//
//         let visible_area_width = runner.gen_range(50..=200);
//         let visible_area_height = runner.gen_range(50..=200);
//
//         let lon = Deg::new(runner.gen_range(-180.0..=180.0));
//         let lat = Deg::new(runner.gen_range(-90.0..=90.0));
//
//         buffer.update_map_position(
//             &lon,
//             &lat,
//             visible_area_width,
//             visible_area_height,
//         )
//     }
// }

fn given_dem_buffer(lon_deg: f32, lat_deg: f32) -> DemBuffer {
    println!("Creating DEM buffer for lon: {}, lat: {}", lon_deg, lat_deg);

    let buffer_size = 200;
    let dem_tile_size = 180;
    let min_cell_distance_to_edge_before_refresh = 30;

    let mut buffer = DemBuffer::new(
        buffer_size,
        buffer_size,
        dem_tile_size,
        min_cell_distance_to_edge_before_refresh,
    );

    let visible_area_width = 80;
    let visible_area_height = 60;

    let lon = Deg::new(lon_deg);
    let lat = Deg::new(lat_deg);

    buffer.update_map_position(
        &lon,
        &lat,
        visible_area_width,
        visible_area_height,
    );

    buffer
}

use proptest::proptest;
use proptest::sample::select;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 1, timeout: 10000, .. ProptestConfig::default()
    })]
    #[test]
    fn test_dem_buffer_properties(
        lon_deg in select(vec![-10.5]),
        lat_deg in select(vec![-10.5])
        // lon_deg in select(vec![-10.5, -5.2, 0.0, 2.3, 10.0]),
        // lat_deg in select(vec![-10.5, -5.2, 0.0, 2.3, 10.0])
    ) {
        let dem_buffer = given_dem_buffer(lon_deg, lat_deg);

        assert!(dem_buffer.prop_center_cell_is_correct_one());
        assert!(dem_buffer.prop_all_cells_are_set());
        assert!(dem_buffer.prop_all_cells_are_good_neighbors());
    }
}
