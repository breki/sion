use crate::consts::EARTH_RADIUS_METERS;

/// Calculates an approximate geodetic distance (in meters) between two points
/// on Earth. Not suitable for high-precision calculations.
#[allow(dead_code)]
pub fn geodetic_distance_approximate(lon1: f32, lat1: f32, lon2: f32, lat2: f32) -> f32 {
    let dlat2 = (lat2 - lat1) / 2.0;
    let dlon2 = (lon2 - lon1) / 2.0;

    let a = (dlat2.sin() * dlat2.sin())
        + (lat1.cos() * lat2.cos() * dlon2.sin() * dlon2.sin());

    let c = 2.0 * (a.sqrt() / (1.0 - a).sqrt()).atan();
    EARTH_RADIUS_METERS * c
}
