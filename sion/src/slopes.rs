use crate::geo::geodetic_distance_approximate;

#[allow(dead_code)]
pub fn grid_size(coords: &Vec<(f32, f32)>) -> (f32, f32) {
    let (lon1, lat1) = coords[0];
    let (lon2, lat2) = coords[1];
    let (lon3, lat3) = coords[3];

    let width = geodetic_distance_approximate(lon1, lat1, lon2, lat2);
    let height = geodetic_distance_approximate(lon1, lat1, lon3, lat3);

    (width, height)
}

#[allow(dead_code)]
pub fn calculate_pq(coords: &Vec<(f32, f32)>, heights: &Vec<f32>) -> (f32, f32) {
    let (grid_width, grid_height) = grid_size(coords);

    let p = ((heights[8] + 2.0 * heights[5] + heights[2])
        - (heights[6] + 2.0 * heights[3] + heights[0]))
        / (8.0 * grid_width);
    let q = ((heights[8] + 2.0 * heights[7] + heights[6])
        - (heights[2] + 2.0 * heights[1] + heights[0]))
        / (8.0 * grid_height);
    (p, q)
}


