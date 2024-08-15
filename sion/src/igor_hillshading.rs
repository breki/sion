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
    _bitmap: &mut GrayscaleBitmap) {
    for _y in 0..dem.size {
        for _x in 0..dem.size {
            panic!("not implemented yet")
        }
    }
}