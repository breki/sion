use crate::consts::EARTH_CIRCUMFERENCE_METERS;
use crate::dem_tile::DemTile;
use crate::geo::{difference_between_angles, normalize_angle};
use crate::grayscale_bitmap::GrayscaleBitmap;
use crate::hillshading::parameters::HillshadingParameters;
use crate::trig::deg_to_rad;
use std::f32::consts::{FRAC_PI_2, PI};

pub fn calculate_pq(
    dem_tile: &DemTile,
    x: usize,
    y: usize,
    horizontal_spacing_mul8: f32,
    vertical_spacing_mul8: f32,
) -> (f32, f32) {
    let center_index = y * dem_tile.size + x;
    let top_center_index = center_index - dem_tile.size;
    let bottom_center_index = center_index + dem_tile.size;

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

pub fn calculate_slope_and_aspect(p: f32, q: f32) -> (f32, f32) {
    let max_slope = (p * p + q * q).sqrt();
    let slope = max_slope.atan();
    let aspect = normalize_angle(q.atan2(p) - FRAC_PI_2);

    (slope, aspect)
}

pub fn hillshade(
    dem: &DemTile,
    parameters: &HillshadingParameters,
    bitmap: &mut GrayscaleBitmap,
) {
    if bitmap.width as usize != dem.size || bitmap.height as usize != dem.size {
        panic!("bitmap size does not match DEM size");
    }

    let sun_azimuth = deg_to_rad(parameters.sun_azimuth);

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
            let (p, q) = calculate_pq(
                dem,
                x,
                y,
                horizontal_spacing_mul8,
                vertical_spacing_mul8,
            );

            let (slope, aspect) = calculate_slope_and_aspect(p, q);

            let aspect_diff = difference_between_angles(aspect, sun_azimuth);
            let aspect_darkness = aspect_diff / PI;
            let slope_darkness = slope / FRAC_PI_2;
            let darkness = 1.
                - (slope_darkness * aspect_darkness * parameters.intensity)
                    .min(1.);
            let darkness_shade = (255.0 * darkness) as u8;

            bitmap.set_pixel(x, y, darkness_shade);
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
        let dem = DemTile::from_hgt_file("tests/data/N46E006.hgt");
        let mut bitmap = GrayscaleBitmap::new(dem.size as u16, dem.size as u16);
        let parameters = HillshadingParameters::default();
        hillshade(&dem, &parameters, &mut bitmap);

        bitmap
            .write_to_png("target/debug/igor_hillshading_orig.png")
            .unwrap()
    }
}
