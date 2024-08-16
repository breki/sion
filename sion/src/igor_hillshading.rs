use crate::consts::EARTH_CIRCUMFERENCE_METERS;
use crate::dem_tile::DemTile;
use crate::geo::normalize_angle;
use crate::grayscale_bitmap::GrayscaleBitmap;
use crate::trig::deg_to_rad;
use std::f32::consts::FRAC_PI_2;

#[allow(dead_code)]
pub struct HillshadingParameters {
    pub sun_azimuth: f32,
    pub intensity: f32,
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
    #[allow(dead_code)]
    fn new(sun_azimuth: f32, intensity: f32) -> Self {
        Self {
            sun_azimuth,
            intensity,
        }
    }
}

pub fn calculate_pq(
    dem_tile: &DemTile,
    x: u16,
    y: u16,
    horizontal_spacing_mul8: f32,
    vertical_spacing_mul8: f32,
) -> (f32, f32) {
    let center_index = y as usize * dem_tile.size as usize + x as usize;
    let top_center_index = center_index - dem_tile.size as usize;
    let bottom_center_index = center_index + dem_tile.size as usize;

    let height_tl = dem_tile.height_at_index(top_center_index - 1) as f32;
    let height_bl = dem_tile.height_at_index(bottom_center_index - 1) as f32;
    let height_br = dem_tile.height_at_index(bottom_center_index + 1) as f32;
    let height_tr = dem_tile.height_at_index(top_center_index + 1) as f32;

    let p = ((height_br
        + 2. * dem_tile.height_at_index(center_index + 1) as f32
        + height_tr)
        - (height_bl
            + 2. * dem_tile.height_at_index(center_index - 1) as f32
            + height_tl))
        / horizontal_spacing_mul8;

    let q = ((height_br
        + 2. * dem_tile.height_at_index(bottom_center_index) as f32
        + height_bl)
        - (height_tr
            + 2. * dem_tile.height_at_index(top_center_index) as f32
            + height_tl))
        / vertical_spacing_mul8;
    (p, q)
}

// todo 5: switch to f16?
pub fn calculate_slope_and_aspect(p: f32, q: f32) -> (f32, f32) {
    let max_slope = (p * p + q * q).sqrt();
    let slope = max_slope.atan();
    let aspect = normalize_angle(FRAC_PI_2 * 3.0 - q.atan2(p));

    (slope, aspect)
}

pub fn hillshade(
    dem: &DemTile,
    _parameters: &HillshadingParameters,
    bitmap: &mut GrayscaleBitmap,
) {
    if bitmap.width != dem.size || bitmap.height != dem.size {
        panic!("bitmap size does not match DEM size");
    }

    // Calculate the (approximate) horizontal and vertical grid
    // spacing (in meters). Note that for latitude, we add 0.5 degrees
    // to the calculation so the spacing is calculated for the center
    // of the DEM tile.
    let horizontal_grid_spacing_meters = deg_to_rad(dem.lon as f32 + 0.5).cos()
        * EARTH_CIRCUMFERENCE_METERS
        / 360.
        / dem.size as f32;
    let horizontal_spacing_mul8 = 8.0 * horizontal_grid_spacing_meters;

    let vertical_grid_spacing_meters =
        EARTH_CIRCUMFERENCE_METERS / 360. / dem.size as f32;
    let vertical_spacing_mul8 = 8.0 * vertical_grid_spacing_meters;

    for y in 1..dem.size - 1 {
        for x in 1..dem.size - 1 {
            let (_p, _q) = calculate_pq(
                dem,
                x,
                y,
                horizontal_spacing_mul8,
                vertical_spacing_mul8,
            );

            let (_slope, _aspect) = calculate_slope_and_aspect(_p, _q);
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
