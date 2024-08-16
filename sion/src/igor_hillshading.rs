use crate::grayscale_bitmap::GrayscaleBitmap;
use crate::dem_tile::DemTile;

pub struct HillshadingParameters {
    sun_azimuth: f32,
    intensity: f32,
}

impl Default for HillshadingParameters {
    fn default() -> Self {
        Self {
            sun_azimuth: 315.0,
            intensity: 1.0,
        }
    }
}

impl HillshadingParameters {
    fn new(sun_azimuth: f32, intensity: f32) -> Self {
        Self {
            sun_azimuth,
            intensity,
        }
    }
}

pub fn hillshade(
    dem: &DemTile,
    _parameters: &HillshadingParameters,
    bitmap: &mut GrayscaleBitmap) {
    if bitmap.width != dem.size || bitmap.height != dem.size {
        panic!("bitmap size does not match DEM size");
    }

    let horizontal_grid_spacing_meters = 1.0;

    for y in 1..dem.size - 1 {
        for x in 1..dem.size - 1 {
            panic!("not implemented yet")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dem_tile::DemTile;
    use crate::grayscale_bitmap::GrayscaleBitmap;

    #[test]
    fn hillshade_of_whole_dem() {
        let dem = DemTile::from_file("tests/data/N46E006.hgt");
        let mut bitmap = GrayscaleBitmap::new(dem.size, dem.size);
        let parameters = HillshadingParameters::default();
        hillshade(&dem, &parameters, &mut bitmap);
    }
}