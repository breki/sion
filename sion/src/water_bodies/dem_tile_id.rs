use std::fmt;
use std::str::FromStr;
// todo X: in the long run, we should merge this into the dem_tile module

#[derive(Copy, Clone, Debug)]
pub struct DemTileId {
    pub lon: i16,
    pub lat: i16,
}

impl DemTileId {
    pub fn new(lon: i16, lat: i16) -> Self {
        DemTileId { lon, lat }
    }
}

impl FromStr for DemTileId {
    type Err = String;

    fn from_str(tile_name: &str) -> Result<Self, Self::Err> {
        if tile_name.len() != 7 {
            return Err(format!("Invalid tile ID length: {}", tile_name));
        }

        let lat = tile_name[1..3]
            .parse::<i16>()
            .map_err(|e| format!("Failed to parse latitude: {}", e))?;
        let lon = tile_name[4..7]
            .parse::<i16>()
            .map_err(|e| format!("Failed to parse longitude: {}", e))?;

        // todo 2: this should be case insensitive
        // Adjust the sign of the coordinates based on the hemisphere
        let lon = if &tile_name[3..4].to_uppercase() == "E" {
            lon
        } else {
            -lon
        };
        let lat = if &tile_name[0..1].to_uppercase() == "N" {
            lat
        } else {
            -lat
        };

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

#[cfg(test)]
mod tests {
    use crate::water_bodies::dem_tile_id::DemTileId;

    #[test]
    fn parsing_and_formatting_tile_ids() {
        let tile_id: DemTileId = "N54E168".parse().unwrap();
        assert_eq!(tile_id.lon, 168);
        assert_eq!(tile_id.lat, 54);
        assert_eq!(tile_id.to_string(), "N54E168");

        let tile_id: DemTileId = "n54e168".parse().unwrap();
        assert_eq!(tile_id.lon, 168);
        assert_eq!(tile_id.lat, 54);
        assert_eq!(tile_id.to_string(), "N54E168");

        let tile_id: DemTileId = "S54W168".parse().unwrap();
        assert_eq!(tile_id.lon, -168);
        assert_eq!(tile_id.lat, -54);
        assert_eq!(tile_id.to_string(), "S54W168");

        let tile_id: DemTileId = "N54W168".parse().unwrap();
        assert_eq!(tile_id.lon, -168);
        assert_eq!(tile_id.lat, 54);
        assert_eq!(tile_id.to_string(), "N54W168");

        let tile_id: DemTileId = "S54E168".parse().unwrap();
        assert_eq!(tile_id.lon, 168);
        assert_eq!(tile_id.lat, -54);
        assert_eq!(tile_id.to_string(), "S54E168");
    }

    // todo 1: negative tests
}
