use std::cmp::Ordering;
use std::f32::consts::PI;
use std::fmt;
use std::ops::{Add, Sub, SubAssign};

const EARTH_RADIUS_METERS: f32 = 6378137.0;
const EARTH_CIRCUMFERENCE_METERS: f32 = 2.0 * PI * EARTH_RADIUS_METERS;

pub const GRID_UNITS_PER_DEM_CELL_BITS: i32 = 8;
pub const GRID_UNITS_PER_DEM_CELL: i32 = 1 << GRID_UNITS_PER_DEM_CELL_BITS;

#[derive(Clone)]
pub struct Deg {
    pub value: f32,
}

impl Deg {
    pub fn new(value: f32) -> Deg {
        if !(value > -181.0 && value < 180.0) {
            panic!("Invalid degree value: {}", value);
        }

        Deg { value }
    }

    pub fn to_int_floor(&self) -> i32 {
        self.value.floor() as i32
    }

    pub fn to_radians(&self) -> f32 {
        self.value * (PI / 180.0)
    }
}

impl Add<f32> for &Deg {
    type Output = Deg;

    fn add(self, other: f32) -> Deg {
        Deg {
            value: self.value + other,
        }
    }
}

impl Sub<f32> for &Deg {
    type Output = Deg;
    fn sub(self, other: f32) -> Deg {
        Deg {
            value: self.value - other,
        }
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

    pub fn from_degrees(value: &Deg, dem_tile_size: i32) -> GlobalCell {
        GlobalCell::new((value.value * dem_tile_size as f32) as i32)
    }

    pub fn from_local_cell_lat(
        lat: &Deg,
        cell_y: LocalCell,
        dem_tile_size: i32,
    ) -> GlobalCell {
        if cell_y.value < 0 || cell_y.value >= dem_tile_size {
            panic!("Invalid local cell Y value: {}", cell_y.value);
        }

        let mut value = (lat.value * dem_tile_size as f32) as i32;
        value += dem_tile_size - 1 - cell_y.value;
        GlobalCell::new(value)
    }

    pub fn to_tile_degrees(&self, dem_tile_size: i32) -> Deg {
        let degrees = self.value as f32 / dem_tile_size as f32;
        if degrees <= -181.0 {
            Deg::new(degrees + 360.0)
        } else if degrees >= 180.0 {
            Deg::new(degrees - 360.0)
        } else {
            Deg::new(degrees)
        }
    }

    pub fn to_local_cell_lon(&self, dem_tile_size: i32) -> LocalCell {
        let m = self.value % dem_tile_size;
        if m < 0 {
            LocalCell::new(dem_tile_size + m)
        } else {
            LocalCell::new(m)
        }
    }

    pub fn to_local_cell_lat(&self, dem_tile_size: i32) -> LocalCell {
        if self.value < 0 {
            let mut a = -1 - self.value % dem_tile_size;
            if a < 0 {
                a += dem_tile_size;
            }
            LocalCell::new(a)
        } else {
            LocalCell::new(dem_tile_size - 1 - self.value % dem_tile_size)
        }
    }
}

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

impl fmt::Debug for GlobalCell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GlobalCell {{ value: {} }}", self.value)
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

impl Add<i32> for &LocalCell {
    type Output = LocalCell;

    fn add(self, other: i32) -> LocalCell {
        LocalCell {
            value: self.value + other,
        }
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

    pub fn from_degrees(value: &Deg, dem_tile_size: i32) -> Grid {
        Grid::new(
            (value.value * (dem_tile_size * GRID_UNITS_PER_DEM_CELL) as f32)
                as i32,
        )
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
    dem_tile_size: i32,
) -> (i32, i32) {
    let latitude_cos = latitude_rad.cos();
    let circumference_at_latitude = EARTH_CIRCUMFERENCE_METERS * latitude_cos;
    let longitude_degree_length_in_meters_at_latitude =
        circumference_at_latitude / 360.0;
    let dem_cell_horiz_length_in_meters_at_latitude =
        longitude_degree_length_in_meters_at_latitude / dem_tile_size as f32;
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
    fn test_to_local_cell_lon_negative_1() {
        let global_cell = GlobalCell::new(-1);
        let dem_tile_size = 256;
        let local_cell = global_cell.to_local_cell_lon(dem_tile_size);
        assert_eq!(local_cell.value, 255);
    }

    #[test]
    fn test_to_local_cell_lon_negative_2() {
        let global_cell = GlobalCell::new(-10);
        let dem_tile_size = 10;
        let local_cell = global_cell.to_local_cell_lon(dem_tile_size);
        assert_eq!(local_cell.value, 0);
    }

    #[test]
    fn test_to_local_cell_lon_negative_3() {
        let global_cell = GlobalCell::new(-11);
        let dem_tile_size = 10;
        let local_cell = global_cell.to_local_cell_lon(dem_tile_size);
        assert_eq!(local_cell.value, 9);
    }

    #[test]
    fn test_to_local_cell_lat_positive() {
        let global_cell = GlobalCell::new(1);
        let dem_tile_size = 10;
        let local_cell = global_cell.to_local_cell_lat(dem_tile_size);
        assert_eq!(local_cell.value, 8);
    }

    #[test]
    fn test_to_local_cell_lat_zero() {
        let global_cell = GlobalCell::new(0);
        let dem_tile_size = 10;
        let local_cell = global_cell.to_local_cell_lat(dem_tile_size);
        assert_eq!(local_cell.value, 9);
    }

    #[test]
    fn test_to_local_cell_lat_negative_1() {
        let global_cell = GlobalCell::new(-1);
        let dem_tile_size = 10;
        let local_cell = global_cell.to_local_cell_lat(dem_tile_size);
        assert_eq!(local_cell.value, 0);
    }

    #[test]
    fn test_to_local_cell_lat_negative_2() {
        let global_cell = GlobalCell::new(-2);
        let dem_tile_size = 10;
        let local_cell = global_cell.to_local_cell_lat(dem_tile_size);
        assert_eq!(local_cell.value, 1);
    }

    #[test]
    fn test_to_local_cell_lat_negative_3() {
        let global_cell = GlobalCell::new(-9);
        let dem_tile_size = 10;
        let local_cell = global_cell.to_local_cell_lat(dem_tile_size);
        assert_eq!(local_cell.value, 8);
    }

    #[test]
    fn test_to_local_cell_lat_negative_4() {
        let global_cell = GlobalCell::new(-10);
        let dem_tile_size = 10;
        let local_cell = global_cell.to_local_cell_lat(dem_tile_size);
        assert_eq!(local_cell.value, 9);
    }

    #[test]
    fn test_to_local_cell_lat_negative_5() {
        let global_cell = GlobalCell::new(-11);
        let dem_tile_size = 10;
        let local_cell = global_cell.to_local_cell_lat(dem_tile_size);
        assert_eq!(local_cell.value, 0);
    }

    #[test]
    fn test_pixel_size() {
        let (horizontal, vertical) =
            calculate_pixel_size_in_grid_units(0.0, 1.0, 1800);
        assert_eq!(horizontal, 4);
        assert_eq!(vertical, 4);
    }

    #[test]
    fn test_from_local_cell_lat_1() {
        let g = GlobalCell::from_local_cell_lat(
            &Deg::new(0.0),
            LocalCell::new(1799),
            1800,
        );

        assert_eq!(g.value, 0);
    }

    #[test]
    fn test_from_local_cell_lat_2() {
        let g = GlobalCell::from_local_cell_lat(
            &Deg::new(0.0),
            LocalCell::new(0),
            1800,
        );

        assert_eq!(g.value, 1799);
    }

    #[test]
    fn test_from_local_cell_lat_3() {
        let g = GlobalCell::from_local_cell_lat(
            &Deg::new(1.0),
            LocalCell::new(1799),
            1800,
        );

        assert_eq!(g.value, 1800);
    }

    #[test]
    fn test_from_local_cell_lat_4() {
        let g = GlobalCell::from_local_cell_lat(
            &Deg::new(-1.0),
            LocalCell::new(0),
            1800,
        );

        assert_eq!(g.value, -1);
    }

    #[test]
    fn test_from_local_cell_lat_5() {
        let g = GlobalCell::from_local_cell_lat(
            &Deg::new(-1.0),
            LocalCell::new(1799),
            1800,
        );

        assert_eq!(g.value, -1800);
    }

    #[test]
    fn test_from_local_cell_lat_6() {
        let g = GlobalCell::from_local_cell_lat(
            &Deg::new(-2.0),
            LocalCell::new(0),
            1800,
        );

        assert_eq!(g.value, -1801);
    }

    #[test]
    fn test_from_local_cell_lat_7() {
        let g = GlobalCell::from_local_cell_lat(
            &Deg::new(0.0),
            LocalCell::new(22),
            157,
        );

        assert_eq!(g.value, 1777);
    }
}
