use crate::consts::EARTH_RADIUS_METERS;
use std::f32::consts::TAU;

/// Calculates an approximate geodetic distance (in meters) between two points
/// on Earth. Not suitable for high-precision calculations.
#[allow(dead_code)]
pub fn geodetic_distance_approximate(
    lon1: f32,
    lat1: f32,
    lon2: f32,
    lat2: f32,
) -> f32 {
    let dlat2 = (lat2 - lat1) / 2.0;
    let dlon2 = (lon2 - lon1) / 2.0;

    let a = (dlat2.sin() * dlat2.sin())
        + (lat1.cos() * lat2.cos() * dlon2.sin() * dlon2.sin());

    let c = 2.0 * (a.sqrt() / (1.0 - a).sqrt()).atan();
    EARTH_RADIUS_METERS * c
}

pub fn normalize_angle(angle: f32) -> f32 {
    let angle_remainder = angle % TAU;

    if angle_remainder < 0.0 {
        angle_remainder + TAU
    } else {
        angle_remainder
    }
}

pub fn difference_between_angles(a1: f32, a2: f32) -> f32 {
    let diff = (a1 - a2) % TAU;
    if diff < -TAU / 2.0 {
        diff + TAU
    } else if diff > TAU / 2.0 {
        diff - TAU
    } else {
        diff.abs()
    }
}

#[cfg(test)]
mod tests {
    use crate::geo::difference_between_angles;
    use crate::testing::assert_eq_approx;
    use rstest::rstest;
    use std::f32::consts::PI;
    use std::f32::consts::TAU;

    #[rstest]
    #[case(0.0, 0.0, 0.0)]
    #[case(0.0, TAU, 0.0)]
    #[case(-TAU, TAU, 0.0)]
    #[case(-TAU, TAU * 2., 0.0)]
    #[case(-PI, PI, 0.0)]
    #[case(0., PI, PI)]
    #[case(PI, 0., PI)]
    #[case(-PI, 0., PI)]
    #[case(0., -PI, PI)]
    fn test_difference_between_angles(
        #[case] a1: f32,
        #[case] a2: f32,
        #[case] expected: f32,
    ) {
        assert_eq_approx(difference_between_angles(a1, a2), expected, 0.000001);
    }
}
