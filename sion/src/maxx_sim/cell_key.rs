use crate::maxx_sim::types::GlobalCell;

pub struct CellKey {
    value: i32,
}

impl CellKey {
    pub fn from_cell_coords(x: &GlobalCell, y: &GlobalCell) -> Self {
        if x.value <= i16::MIN as i32
            || x.value >= i16::MAX as i32
            || y.value <= i16::MIN as i32
            || y.value >= i16::MAX as i32
        {
            panic!("Cell coordinates out of bounds");
        }

        let value = (y.value << 16) | (x.value & 0xFFFF);
        CellKey { value }
    }

    pub fn from_i32(value: i32) -> Self {
        CellKey { value }
    }

    pub fn to_cell_coords(&self) -> (GlobalCell, GlobalCell) {
        let x = ((self.value & 0xFFFF) as i16) as i32; // Extract lower 16 bits for x
        let y = (((self.value >> 16) & 0xFFFF) as i16) as i32; // Extract upper 16 bits for y

        if x <= i16::MIN as i32
            || x >= i16::MAX as i32
            || y <= i16::MIN as i32
            || y >= i16::MAX as i32
        {
            panic!("Cell coordinates out of bounds");
        }

        (GlobalCell::new(x), GlobalCell::new(y))
    }

    pub fn to_i32(&self) -> i32 {
        self.value
    }

    pub fn empty() -> Self {
        CellKey { value: i32::MIN }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maxx_sim::types::GlobalCell;


    #[test]
    fn test_cell_keys_1() {
        let cell_key = CellKey::from_cell_coords(
            &GlobalCell::new(100),
            &GlobalCell::new(200),
        );

        let (tile_lon_cell, tile_lat_cell) = cell_key.to_cell_coords();

        assert_eq!(tile_lon_cell.value, 100);
        assert_eq!(tile_lat_cell.value, 200);
    }

    #[test]
    fn test_cell_keys_2() {
        let cell_key = CellKey::from_cell_coords(
            &GlobalCell::new(-1239),
            &GlobalCell::new(195),
        );

        let (tile_lon_cell, tile_lat_cell) = cell_key.to_cell_coords();

        assert_eq!(tile_lon_cell.value, -1239);
        assert_eq!(tile_lat_cell.value, 195);
    }

    #[test]
    fn test_cell_keys_3() {
        let cell_key = CellKey::from_cell_coords(
            &GlobalCell::new(-1419),
            &GlobalCell::new(-180),
        );

        let (tile_lon_cell, tile_lat_cell) = cell_key.to_cell_coords();

        assert_eq!(tile_lon_cell.value, -1419);
        assert_eq!(tile_lat_cell.value, -180);
    }
}