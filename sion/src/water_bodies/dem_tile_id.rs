use std::fmt;

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

    // parses file names like "S54E168", taking into account N-S and E-W
    pub fn from_tile_name(tile_name: &str) -> Result<DemTileId, String> {
        if tile_name.len() != 7 {
            return Err(format!("Invalid tile ID length: {}", tile_name));
        }

        let lat = tile_name[1..3]
            .parse::<i16>()
            .map_err(|e| format!("Failed to parse latitude: {}", e))?;
        let lon = tile_name[4..7]
            .parse::<i16>()
            .map_err(|e| format!("Failed to parse longitude: {}", e))?;

        // Adjust the sign of the coordinates based on the hemisphere
        let lon = if &tile_name[3..4] == "E" { lon } else { -lon };
        let lat = if &tile_name[0..1] == "N" { lat } else { -lat };

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
