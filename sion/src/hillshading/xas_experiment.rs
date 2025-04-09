pub struct XasTile {
    pub size: usize,
    data: Box<[u8]>,
}

impl XasTile {
    pub fn new(size: usize) -> XasTile {
        XasTile {
            size,
            data: vec![0; size * size * 2].into_boxed_slice(),
        }
    }

    pub fn set_aspect_and_slope(
        &mut self,
        x: u16,
        y: u16,
        aspect: f32,
        slope: f32,
    ) {
        let byte_offset = ((y as usize) * (self.size) + (x as usize)) << 1;
        let aspect_int = aspect.round() as u16;
        let slope_int = slope.round() as u16;
        let encoded_value = slope_int << 7 | aspect_int;
        self.data[byte_offset] = (encoded_value >> 8) as u8;
        self.data[byte_offset + 1] = (encoded_value & 0xff) as u8;
    }

    pub fn get_aspect_and_slope(&self, x: u16, y: u16) -> (f32, f32) {
        let byte_offset = ((y as usize) * (self.size) + (x as usize)) << 1;
        let encoded_value = (self.data[byte_offset] as u16) << 8
            | (self.data[byte_offset + 1] as u16);
        let aspect = (encoded_value & 0x7f) as f32;
        let slope = (encoded_value >> 7) as f32;
        (aspect, slope)
    }
}

#[cfg(test)]
mod tests {
    use crate::consts::EARTH_CIRCUMFERENCE_METERS;
    use crate::dem_tile::DemTile;
    use crate::grayscale_bitmap::GrayscaleBitmap;
    use crate::hillshading::igor_hillshading_orig::{
        calculate_pq, calculate_slope_and_aspect,
    };
    use crate::hillshading::xas_experiment::XasTile;
    use crate::testing::assert_eq_approx;
    use crate::trig::deg_to_rad;

    #[test]
    fn xas_experiment() {
        let dem = DemTile::from_hgt_file("tests/data/N46E006.hgt");

        let mut xas_tile = XasTile::new(dem.size);

        // Calculate the (approximate) horizontal and vertical grid
        // spacing (in meters). Note that for latitude, we add 0.5 degrees
        // to the calculation so the spacing is calculated for the center
        // of the DEM tile.
        let horizontal_grid_spacing_meters =
            deg_to_rad(dem.lon as f32 + 0.5).cos() * EARTH_CIRCUMFERENCE_METERS
                / 360.
                / dem.size as f32;
        let horizontal_spacing_mul8 = 8.0 * horizontal_grid_spacing_meters;

        let vertical_grid_spacing_meters =
            EARTH_CIRCUMFERENCE_METERS / 360. / dem.size as f32;
        let vertical_spacing_mul8 = 8.0 * vertical_grid_spacing_meters;

        for y in 1..dem.size - 1 {
            for x in 1..dem.size - 1 {
                let (p, q) = calculate_pq(
                    &dem,
                    x,
                    y,
                    horizontal_spacing_mul8,
                    vertical_spacing_mul8,
                );

                let (slope, aspect) = calculate_slope_and_aspect(p, q);

                xas_tile
                    .set_aspect_and_slope(x as u16, y as u16, aspect, slope);

                // just making sure the get/set methods work
                let (aspect2, slope2) =
                    xas_tile.get_aspect_and_slope(x as u16, y as u16);

                assert_eq_approx(aspect, aspect2, 0.5);
                assert_eq_approx(slope, slope2, 0.5);
            }
        }

        let display_width = 800;
        let display_height = 600;
        let display_image = GrayscaleBitmap::new(display_width, display_height);

        let map_center_lon = 6.5;
        let map_center_lat = 46.5;
        let map_zoom_level = 0;

        for y in 0..display_height - 1 {
            for x in 0..display_width - 1 {
                // calculate the pixel position in DEM cell coordinates
            }
        }
    }
}
