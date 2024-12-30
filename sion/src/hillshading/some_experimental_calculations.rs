use crate::trig::rad_to_deg;

pub struct Matrix3x3 {
    pub data: [i16; 9],
}

impl Matrix3x3 {
    pub fn new(elevations: [i16; 9]) -> Matrix3x3 {
        Matrix3x3 { data: elevations }
    }

    pub fn height_tl(&self) -> i16 {
        self.data[0]
    }

    pub fn height_tc(&self) -> i16 {
        self.data[1]
    }

    pub fn height_tr(&self) -> i16 {
        self.data[2]
    }

    pub fn height_cl(&self) -> i16 {
        self.data[3]
    }

    pub fn height_cr(&self) -> i16 {
        self.data[5]
    }

    pub fn height_bl(&self) -> i16 {
        self.data[6]
    }

    pub fn height_bc(&self) -> i16 {
        self.data[7]
    }

    pub fn height_br(&self) -> i16 {
        self.data[8]
    }
}

pub fn calculate_pq_1(d: i16, e: &Matrix3x3) -> (f32, f32) {
    let p = (((e.height_br() + 2 * e.height_cr() + e.height_tr())
        - (e.height_bl() + 2 * e.height_cl() + e.height_tl()))
        as f32)
        / ((d as f32) * 8.);
    let q = (((e.height_br() + 2 * e.height_bc() + e.height_bl())
        - (e.height_tr() + 2 * e.height_tc() + e.height_tl()))
        as f32)
        / ((d as f32) * 8.);
    (p, q)
}

pub fn calculate_pq_2(e: &Matrix3x3) -> (i16, i16) {
    let p_prime = (e.height_br() + 2 * e.height_cr() + e.height_tr())
        - (e.height_bl() + 2 * e.height_cl() + e.height_tl());
    let q_prime = (e.height_br() + 2 * e.height_bc() + e.height_bl())
        - (e.height_tr() + 2 * e.height_tc() + e.height_tl());
    (p_prime, q_prime)
}

pub fn calculate_slope_1(p: f32, q: f32) -> f32 {
    rad_to_deg((p * p + q * q).sqrt().atan())
}

pub fn calculate_slope_2(d: i16, p_prime: i16, q_prime: i16) -> f32 {
    let p_prime_32 = p_prime as f32;
    let q_prime_32 = q_prime as f32;
    let divisor = (d as f32) * 8.;
    let root = (p_prime_32 * p_prime_32 + q_prime_32 * q_prime_32).sqrt();
    let slope = root / divisor;

    rad_to_deg(slope.atan())
}

pub fn calculate_slope_3(zoom_level: i16, p_prime: i16, q_prime: i16) -> f32 {
    let p_prime_32 = p_prime as f32;
    let q_prime_32 = q_prime as f32;
    let root = (p_prime_32 * p_prime_32 + q_prime_32 * q_prime_32).sqrt();
    // slope needs to be a float value, otherwise atan() will not work properly
    let slope = root * 2_f32.powf(zoom_level as f32) / (45. * 8.);

    rad_to_deg(slope.atan())
}

pub fn calculate_aspect_1(p: f32, q: f32) -> f32 {
    let aspect = q.atan2(p);
    if aspect < 0. {
        rad_to_deg(aspect) + 360.
    } else {
        rad_to_deg(aspect)
    }
}

pub fn calculate_aspect_2(p_prime: i16, q_prime: i16) -> f32 {
    let aspect = (q_prime as f32).atan2(p_prime as f32);
    if aspect < 0. {
        rad_to_deg(aspect) + 360.
    } else {
        rad_to_deg(aspect)
    }
}

pub fn diff_between_angles_deg(a: i16, b: i16) -> i16 {
    let diff = (a - b).abs();
    if diff > 180 {
        360 - diff
    } else {
        diff
    }
}

pub fn hillshading_1(sun_azimuth: i16, slope: i16, aspect: i16) -> i16 {
    let slope_light_intensity = (90. - slope as f32) / 90.;
    let aspect_diff = diff_between_angles_deg(aspect, sun_azimuth);
    let aspect_light_intensity = (180. - aspect_diff as f32) / 180.;
    let light_intensity = slope_light_intensity * aspect_light_intensity;
    let color = (255. * light_intensity) as i16;
    color
}

pub fn hillshading_2(sun_azimuth: i16, slope: i16, aspect: i16) -> i16 {
    let slope_light_intensity = (90 - slope as i32) * 255 / 90;
    let aspect_diff = diff_between_angles_deg(aspect, sun_azimuth);
    let aspect_light_intensity = (180 - aspect_diff as i32) * 255 / 180;
    let light_intensity =
        (slope_light_intensity * aspect_light_intensity) / 255;
    let color = light_intensity as i16;
    color
}

pub fn hillshading_3(sun_azimuth: i16, slope: i16, aspect: i16) -> i16 {
    let slope_light_intensity = ((90 - slope as i32) << 8) / 90;
    let aspect_diff = diff_between_angles_deg(aspect, sun_azimuth);
    let aspect_light_intensity = ((180 - aspect_diff as i32) << 8) / 180;
    let light_intensity = (slope_light_intensity * aspect_light_intensity) >> 8;
    let color = light_intensity as i16;
    color
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::assert_eq_approx;

    #[test]
    fn test_slope_facing_west() {
        let elevations = Matrix3x3::new([
            1000, 1045, 1090, 1000, 1045, 1090, 1000, 1045, 1090,
        ]);

        assert_calculations(&elevations, 45, 0, 1., 0., 45., 0., 95);
        assert_calculations(&elevations, 90, -1, 0.5, 0., 26.56505, 0., 136);
        assert_calculations(
            &elevations,
            22,
            1,
            2.0454545,
            0.,
            63.946503,
            0.,
            57,
        );
    }

    #[test]
    fn test_slope_facing_east() {
        let elevations = Matrix3x3::new([
            1090, 1045, 1000, 1090, 1045, 1000, 1090, 1045, 1000,
        ]);

        assert_calculations(&elevations, 45, 0, -1., 0., 45., 180., 31);
        assert_calculations(&elevations, 90, -1, -0.5, 0., 26.56505, 180., 45);
        assert_calculations(
            &elevations,
            22,
            1,
            -2.0454545,
            0.,
            63.946503,
            180.,
            19,
        );
    }

    #[test]
    fn test_slope_facing_north() {
        let elevations = Matrix3x3::new([
            1000, 1000, 1000, 1045, 1045, 1045, 1090, 1090, 1090,
        ]);

        assert_calculations(&elevations, 45, 0, 0., 1., 45., 90., 95);
        assert_calculations(&elevations, 90, -1, 0., 0.5, 26.56505, 90., 136);
        assert_calculations(
            &elevations,
            22,
            1,
            0.,
            2.0454545,
            63.946503,
            90.,
            57,
        );
    }

    #[test]
    fn test_slope_facing_south() {
        let elevations = Matrix3x3::new([
            1090, 1090, 1090, 1045, 1045, 1045, 1000, 1000, 1000,
        ]);

        assert_calculations(&elevations, 45, 0, 0., -1., 45., 270., 31);
        assert_calculations(&elevations, 90, -1, 0., -0.5, 26.56505, 270., 45);
        assert_calculations(
            &elevations,
            22,
            1,
            0.,
            -2.0454545,
            63.946503,
            270.,
            19,
        );
    }

    #[test]
    fn test_natural_slope_sample_1() {
        let elevations = Matrix3x3::new([
            1250, 1256, 1265, 1271, 1280, 1297, 1274, 1303, 1318,
        ]);

        assert_calculations(
            &elevations,
            45,
            0,
            0.30833334,
            0.475,
            29.52283,
            57.011475,
            161,
        );
        assert_calculations(
            &elevations,
            90,
            -1,
            0.15416667,
            0.2375,
            15.809441,
            57.011475,
            198,
        );
        assert_calculations(
            &elevations,
            22,
            1,
            0.6306818,
            0.97159094,
            49.195778,
            57.011475,
            108,
        );
    }

    fn assert_calculations(
        elevations: &Matrix3x3,
        d: i16,
        zoom: i16,
        expected_p: f32,
        expected_q: f32,
        expected_slope: f32,
        expected_aspect: f32,
        expected_hillshading_color: i16,
    ) {
        let (p, q) = calculate_pq_1(d, &elevations);

        assert_eq!(p, expected_p);
        assert_eq!(q, expected_q);

        let slope = calculate_slope_1(p, q);
        assert_eq!(slope, expected_slope, "slope_1");
        let aspect = calculate_aspect_1(p, q);
        assert_eq_approx(aspect, expected_aspect, 0.001);

        let (p_prime, q_prime) = calculate_pq_2(&elevations);
        assert_eq!(
            calculate_slope_2(d, p_prime, q_prime),
            expected_slope,
            "slope_2"
        );
        assert_eq_approx(
            calculate_aspect_2(p_prime, q_prime),
            expected_aspect,
            0.0001,
        );

        assert_eq_approx(
            calculate_slope_3(zoom, p_prime, q_prime),
            expected_slope,
            1.,
        );

        let sun_azimuth = 45;
        let color = hillshading_1(sun_azimuth, slope as i16, aspect as i16);
        assert_eq!(color, expected_hillshading_color, "hillshading_1");
        let color = hillshading_2(sun_azimuth, slope as i16, aspect as i16);
        assert_eq_approx(color, expected_hillshading_color, 1);
        let color = hillshading_3(sun_azimuth, slope as i16, aspect as i16);
        assert_eq_approx(color, expected_hillshading_color, 1);
    }

    #[test]
    fn test_icebreaker() {
        fn latitude_to_world_cell_y_and_fraction(
            lat: f32,
            tile_size: i16,
        ) -> (i32, f32) {
            let lat_int = lat as i16;
            let fraction = lat - lat_int as f32;
            let cell_with_fraction = fraction * (tile_size as f32);
            let local_cell_y = cell_with_fraction.floor() as i16;
            let world_cell_y =
                (lat_int as i32) * (tile_size as i32) + (local_cell_y as i32);
            let mut cell_fraction =
                cell_with_fraction - (cell_with_fraction as i16) as f32;
            if local_cell_y < 0 {
                cell_fraction = 1. + cell_fraction;
            }
            (world_cell_y, cell_fraction)
        }

        fn from_world_cell_y_to_latitude_and_local_cell_y(
            world_cell_y: i32,
            tile_size: i16,
        ) -> (i16, i16) {
            let mut lat = world_cell_y / (tile_size as i32);
            if world_cell_y < 0 {
                lat = lat - 1;
            }

            let mut modulo = world_cell_y % (tile_size as i32);
            if modulo < 0 {
                modulo = (tile_size as i32) + modulo;
            }

            let local_cell_y = tile_size - 1 - (modulo as i16);
            (lat as i16, local_cell_y)
        }

        let tile_size = 1800;

        let lat = 0.177;
        let (world_cell_y, cell_fraction) =
            latitude_to_world_cell_y_and_fraction(lat, tile_size);
        assert_eq!(world_cell_y, 318);
        assert_eq!(cell_fraction, 0.6000061);
        assert_eq!(
            from_world_cell_y_to_latitude_and_local_cell_y(
                world_cell_y,
                tile_size
            ),
            (0, 1481)
        );

        let lat = 0.;
        let (world_cell_y, cell_fraction) =
            latitude_to_world_cell_y_and_fraction(lat, tile_size);
        assert_eq!(world_cell_y, 0);
        assert_eq!(cell_fraction, 0.);
        assert_eq!(
            from_world_cell_y_to_latitude_and_local_cell_y(
                world_cell_y,
                tile_size
            ),
            (0, 1799)
        );

        let lat = -0.0001 / (tile_size as f32);
        let (world_cell_y, cell_fraction) =
            latitude_to_world_cell_y_and_fraction(lat, tile_size);
        assert_eq!(world_cell_y, -1);
        assert_eq!(cell_fraction, 0.9999);
        assert_eq!(
            from_world_cell_y_to_latitude_and_local_cell_y(
                world_cell_y,
                tile_size
            ),
            (-1, 0)
        );

        let lat = 0.0001 / (tile_size as f32);
        let (world_cell_y, cell_fraction) =
            latitude_to_world_cell_y_and_fraction(lat, tile_size);
        assert_eq!(world_cell_y, 0);
        assert_eq!(cell_fraction, 0.0001);
        assert_eq!(
            from_world_cell_y_to_latitude_and_local_cell_y(
                world_cell_y,
                tile_size
            ),
            (0, 1799)
        );

        let lat = (-0.5001) / (tile_size as f32);
        let (world_cell_y, cell_fraction) =
            latitude_to_world_cell_y_and_fraction(lat, tile_size);
        assert_eq!(world_cell_y, -1);
        assert_eq_approx(cell_fraction, 0.4999, 0.0001);
        assert_eq!(
            from_world_cell_y_to_latitude_and_local_cell_y(
                world_cell_y,
                tile_size
            ),
            (-1, 0)
        );

        let lat = 0.5 / (tile_size as f32);
        let (world_cell_y, cell_fraction) =
            latitude_to_world_cell_y_and_fraction(lat, tile_size);
        assert_eq!(world_cell_y, 0);
        assert_eq!(cell_fraction, 0.5);
        assert_eq!(
            from_world_cell_y_to_latitude_and_local_cell_y(
                world_cell_y,
                tile_size
            ),
            (0, 1799)
        );

        let lat = 7.5;
        let (world_cell_y, cell_fraction) =
            latitude_to_world_cell_y_and_fraction(lat, tile_size);
        assert_eq!(world_cell_y, 13500);
        assert_eq!(cell_fraction, 0.);
        assert_eq!(
            from_world_cell_y_to_latitude_and_local_cell_y(
                world_cell_y,
                tile_size
            ),
            (7, 899)
        );

        let lat = 8.;
        let (world_cell_y, cell_fraction) =
            latitude_to_world_cell_y_and_fraction(lat, tile_size);
        assert_eq!(world_cell_y, 14400);
        assert_eq!(cell_fraction, 0.);
        assert_eq!(
            from_world_cell_y_to_latitude_and_local_cell_y(
                world_cell_y,
                tile_size
            ),
            (8, 1799)
        );
        assert_eq!(
            from_world_cell_y_to_latitude_and_local_cell_y(
                world_cell_y - 1,
                tile_size
            ),
            (7, 0)
        );

        let lat = 46.499889;
        let (world_cell_y, cell_fraction) =
            latitude_to_world_cell_y_and_fraction(lat, tile_size);
        assert_eq!(world_cell_y, 83699);
        assert_eq!(cell_fraction, 0.8009033);
        assert_eq!(
            from_world_cell_y_to_latitude_and_local_cell_y(
                world_cell_y,
                tile_size
            ),
            (46, 900)
        );
    }
}
