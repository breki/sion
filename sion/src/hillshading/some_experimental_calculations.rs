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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::assert_eq_approx;

    #[test]
    fn test_eastward_slope() {
        let d = 45;
        let elevations = Matrix3x3::new([
            1000, 1045, 1090, 1000, 1045, 1090, 1000, 1045, 1090,
        ]);

        let (p, q) = calculate_pq_1(d, &elevations);

        assert_eq!(p, 1.);
        assert_eq!(q, 0.);
        assert_eq!(calculate_slope_1(p, q), 45.);
        assert_eq!(calculate_aspect_1(p, q), 0.);

        let (p_prime, q_prime) = calculate_pq_2(&elevations);
        assert_eq!(calculate_slope_2(d, p_prime, q_prime), 45.);
        assert_eq!(calculate_aspect_2(p_prime, q_prime), 0.);
    }

    #[test]
    fn test_westward_slope() {
        let d = 45;
        let elevations = Matrix3x3::new([
            1090, 1045, 1000, 1090, 1045, 1000, 1090, 1045, 1000,
        ]);

        let (p, q) = calculate_pq_1(d, &elevations);

        assert_eq!(p, -1.);
        assert_eq!(q, 0.);
        assert_eq!(calculate_slope_1(p, q), 45.);
        assert_eq!(calculate_aspect_1(p, q), 180.);

        let (p_prime, q_prime) = calculate_pq_2(&elevations);
        assert_eq!(calculate_slope_2(d, p_prime, q_prime), 45.);
        assert_eq!(calculate_aspect_2(p_prime, q_prime), 180.);
    }

    #[test]
    fn test_southward_slope() {
        let d = 45;
        let elevations = Matrix3x3::new([
            1000, 1000, 1000, 1045, 1045, 1045, 1090, 1090, 1090,
        ]);

        let (p, q) = calculate_pq_1(d, &elevations);

        assert_eq!(p, 0.);
        assert_eq!(q, 1.);
        assert_eq!(calculate_slope_1(p, q), 45.);
        assert_eq!(calculate_aspect_1(p, q), 90.);

        let (p_prime, q_prime) = calculate_pq_2(&elevations);
        assert_eq!(calculate_slope_2(d, p_prime, q_prime), 45.);
        assert_eq!(calculate_aspect_2(p_prime, q_prime), 90.);
    }

    #[test]
    fn test_northward_slope() {
        let d = 45;
        let elevations = Matrix3x3::new([
            1090, 1090, 1090, 1045, 1045, 1045, 1000, 1000, 1000,
        ]);

        let (p, q) = calculate_pq_1(d, &elevations);

        assert_eq!(p, 0.);
        assert_eq!(q, -1.);
        assert_eq!(calculate_slope_1(p, q), 45.);
        assert_eq!(calculate_aspect_1(p, q), 270.);

        let (p_prime, q_prime) = calculate_pq_2(&elevations);
        assert_eq!(calculate_slope_2(d, p_prime, q_prime), 45.);
        assert_eq!(calculate_aspect_2(p_prime, q_prime), 270.);
    }

    #[test]
    fn test_natural_1() {
        let d = 45;
        let elevations = Matrix3x3::new([
            1250, 1256, 1265, 1271, 1280, 1297, 1274, 1303, 1318,
        ]);

        let (p, q) = calculate_pq_1(d, &elevations);

        assert_eq!(p, 0.30833334);
        assert_eq!(q, 0.475);
        assert_eq!(calculate_slope_1(p, q), 29.52283);
        assert_eq!(calculate_aspect_1(p, q), 57.011475);

        let (p_prime, q_prime) = calculate_pq_2(&elevations);
        assert_eq!(calculate_slope_2(d, p_prime, q_prime), 29.52283);
        assert_eq_approx(
            calculate_aspect_2(p_prime, q_prime),
            57.011475,
            0.0001,
        );
    }
}
