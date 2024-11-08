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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::assert_eq_approx;

    #[test]
    fn test_slope_facing_west() {
        let elevations = Matrix3x3::new([
            1000, 1045, 1090, 1000, 1045, 1090, 1000, 1045, 1090,
        ]);

        assert_calculations(&elevations, 45, 1., 0., 45., 0., 95);
        assert_calculations(&elevations, 90, 0.5, 0., 26.56505, 0., 136);
    }

    #[test]
    fn test_slope_facing_east() {
        let elevations = Matrix3x3::new([
            1090, 1045, 1000, 1090, 1045, 1000, 1090, 1045, 1000,
        ]);

        assert_calculations(&elevations, 45, -1., 0., 45., 180., 31);
        assert_calculations(&elevations, 90, -0.5, 0., 26.56505, 180., 45);
    }

    #[test]
    fn test_slope_facing_north() {
        let elevations = Matrix3x3::new([
            1000, 1000, 1000, 1045, 1045, 1045, 1090, 1090, 1090,
        ]);

        assert_calculations(&elevations, 45, 0., 1., 45., 90., 95);
        assert_calculations(&elevations, 90, 0., 0.5, 26.56505, 90., 136);
    }

    #[test]
    fn test_slope_facing_south() {
        let elevations = Matrix3x3::new([
            1090, 1090, 1090, 1045, 1045, 1045, 1000, 1000, 1000,
        ]);

        assert_calculations(&elevations, 45, 0., -1., 45., 270., 31);
        assert_calculations(&elevations, 90, 0., -0.5, 26.56505, 270., 45);
    }

    #[test]
    fn test_natural_slope_sample_1() {
        let elevations = Matrix3x3::new([
            1250, 1256, 1265, 1271, 1280, 1297, 1274, 1303, 1318,
        ]);

        assert_calculations(
            &elevations,
            45,
            0.30833334,
            0.475,
            29.52283,
            57.011475,
            161,
        );
        assert_calculations(
            &elevations,
            90,
            0.15416667,
            0.2375,
            15.809441,
            57.011475,
            198,
        );
    }

    fn assert_calculations(
        elevations: &Matrix3x3,
        d: i16,
        expected_p: f32,
        expected_q: f32,
        expected_slope: f32,
        expected_aspect: f32,
        expected_igor_hillshading_color: i16,
    ) {
        let (p, q) = calculate_pq_1(d, &elevations);

        assert_eq!(p, expected_p);
        assert_eq!(q, expected_q);

        let slope = calculate_slope_1(p, q);
        assert_eq!(slope, expected_slope);
        let aspect = calculate_aspect_1(p, q);
        assert_eq!(aspect, expected_aspect);

        let (p_prime, q_prime) = calculate_pq_2(&elevations);
        assert_eq!(calculate_slope_2(d, p_prime, q_prime), expected_slope);
        assert_eq_approx(
            calculate_aspect_2(p_prime, q_prime),
            expected_aspect,
            0.0001,
        );

        let sun_azimuth = 45;
        let color = hillshading_1(sun_azimuth, slope as i16, aspect as i16);
        assert_eq!(color, expected_igor_hillshading_color, "sun_azimuth",);
    }
}
