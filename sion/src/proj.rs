use crate::consts::{DPI, EARTH_RADIUS_METERS, INCHES_PER_METER};
use std::f32::consts::FRAC_PI_4;

pub const MIN_LAT: f32 = -1.48442222974533;
pub const MAX_LAT: f32 = 1.48442222974533;

pub fn proj_scale_factor(map_scale: f32) -> f32 {
    EARTH_RADIUS_METERS / map_scale * INCHES_PER_METER * DPI
}

/// Project lon/lat (in radians) to x/y (in meters) using the Web Mercator projection.
pub fn web_mercator_proj(
    lon: f32,
    lat: f32,
    proj_scale_factor: f32,
) -> (f32, f32) {
    if lat < MIN_LAT || lat > MAX_LAT {
        panic!("Latitude out of bounds: {}", lat);
    } else {
    }

    let x = lon * proj_scale_factor;
    let y = (FRAC_PI_4 + lat / 2.0).tan().ln() * proj_scale_factor;
    (x, y)
}

#[cfg(test)]
mod tests {
    use crate::consts::{DPI, EARTH_RADIUS_METERS, INCHES_PER_METER};
    use crate::testing::assert_eq_approx;
    use crate::trig::deg_to_rad;

    #[test]
    fn mercator_projection() {
        let scale_factor = super::proj_scale_factor(1.);
        let (x, y) = super::web_mercator_proj(0., 0., scale_factor);
        assert_eq!(x, 0.);
        assert_eq!(y, 0.);

        let (x, y) = super::web_mercator_proj(
            deg_to_rad(10.),
            deg_to_rad(10.),
            scale_factor,
        );
        assert_eq_approx(scale_to_1_dpi(x), 0.17453293, 0.0000001);
        assert_eq_approx(scale_to_1_dpi(y), 0.17542583, 0.0000001);

        let (x, y) = super::web_mercator_proj(
            deg_to_rad(10.),
            deg_to_rad(80.),
            scale_factor,
        );
        assert_eq_approx(scale_to_1_dpi(x), 0.17453293, 0.000001);
        assert_eq_approx(scale_to_1_dpi(y), 2.43624605, 0.000001);

        let (x, y) = super::web_mercator_proj(
            deg_to_rad(180.),
            deg_to_rad(-80.),
            scale_factor,
        );
        assert_eq_approx(scale_to_1_dpi(x), std::f32::consts::PI, 0.000001);
        assert_eq_approx(scale_to_1_dpi(y), -2.43624605, 0.000001);
    }

    /// Scales a value to 1 DPI so we can use the test values from Demeton.
    fn scale_to_1_dpi(value: f32) -> f32 {
        value / (DPI / (1. / (EARTH_RADIUS_METERS * INCHES_PER_METER)))
    }
}
