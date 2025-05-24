pub const EARTH_RADIUS_METERS: f32 = 6378137.0;
pub const EARTH_CIRCUMFERENCE_METERS: f32 = 2.0 * PI * EARTH_RADIUS_METERS;

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
use std::ops::{Add, Sub, SubAssign};

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

impl SubAssign<i32> for GlobalCell {
    fn sub_assign(&mut self, other: i32) {
        self.value -= other;
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

    pub fn from_i16(value: i16) -> TileKey {
        let lon = value & 0xFF;
        let lat = (value >> 8) & 0xFF;
        TileKey {
            lon: lon as i32,
            lat: lat as i32,
        }
    }

    pub fn to_i16(&self) -> i16 {
        let lon = self.lon as i16;
        let lat = self.lat as i16;
        (lat << 8) | (lon & 0xFF)
    }
}

impl PartialEq for TileKey {
    fn eq(&self, other: &Self) -> bool {
        self.lon == other.lon && self.lat == other.lat
    }
}

pub fn calculate_pixel_size_in_grid_units(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_size() {
        let (horizontal, vertical) =
            calculate_pixel_size_in_grid_units(0.0, 1.0);
        assert_eq!(horizontal, 4);
        assert_eq!(vertical, 4);
    }
}
