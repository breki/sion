use image::{GrayImage, Luma};

/// Represents a monochrome bitmap that can be used to draw on and then
/// be sent to the display.
pub struct MonoBitmap {
    pub width: u16,
    pub height: u16,
    data: Box<[u8]>,
    /// The number of bytes per row in the bitmap.
    width_bytes: u16,
}

impl MonoBitmap {
    /// Creates a new empty monochrome bitmap with the given width and height.
    #[allow(dead_code)]
    fn new(width: u16, height: u16) -> MonoBitmap {
        let width_bytes = (width + 7) / 8;

        MonoBitmap {
            width,
            height,
            data: vec![0; width_bytes as usize * height as usize]
                .into_boxed_slice(),
            width_bytes,
        }
    }

    /// Sets the pixel at the given coordinates to the given value (on or off).
    #[allow(dead_code)]
    fn set_pixel(&mut self, x: u16, y: u16, value: bool) {
        let byte_index = (y * self.width_bytes + x / 8) as usize;
        let bit_index = x % 8;
        let mask = 1 << bit_index;

        if value {
            self.data[byte_index] |= mask;
        } else {
            self.data[byte_index] &= !mask;
        }
    }

    /// Gets the value of the pixel at the given coordinates.
    fn get_pixel(&self, x: u16, y: u16) -> bool {
        let byte_index = (y * self.width_bytes + x / 8) as usize;
        let bit_index = x % 8;
        let mask = 1 << bit_index;

        self.data[byte_index] & mask != 0
    }

    /// Writes the monochrome bitmap to a PNG file.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The path to the output PNG file.
    pub fn write_to_png(
        &self,
        file_path: &str,
    ) -> Result<(), image::ImageError> {
        let mut img = GrayImage::new(self.width.into(), self.height.into());
        for y in 0..self.height {
            for x in 0..self.width {
                let pixel_value = if self.get_pixel(x, y) { 255 } else { 0 };
                img.put_pixel(x.into(), y.into(), Luma([pixel_value]));
            }
        }
        img.save(file_path)
    }
}

#[cfg(test)]
mod tests {
    use super::MonoBitmap;

    #[test]
    fn create_large_bitmap() {
        let bitmap = MonoBitmap::new(1000, 1000);
        assert_eq!(bitmap.width, 1000);
        assert_eq!(bitmap.height, 1000);
        assert_eq!(bitmap.data.len(), 125000);
    }

    /// A new bitmap is created with the correct dimensions and properties.
    #[test]
    fn create_bitmap() {
        let bitmap = MonoBitmap::new(10, 15);
        assert_eq!(bitmap.width, 10);
        assert_eq!(bitmap.height, 15);
        assert_eq!(bitmap.width_bytes, 2);
        assert_eq!(bitmap.data.len(), 30);
    }

    /// The pixels are off by default when the bitmap is created.
    #[test]
    fn pixels_are_off_by_default() {
        let bitmap = MonoBitmap::new(10, 15);
        assert_eq!(bitmap.get_pixel(0, 0), false);
        assert_eq!(bitmap.get_pixel(4, 5), false);
    }

    /// Pixels can be set/reset and then retrieved.
    #[test]
    fn set_and_get_pixel() {
        let mut bitmap = MonoBitmap::new(10, 15);
        bitmap.set_pixel(3, 4, true);
        assert_eq!(bitmap.get_pixel(3, 4), true);
        bitmap.set_pixel(3, 4, false);
        assert_eq!(bitmap.get_pixel(3, 4), false);
    }

    /// The bitmap can be written to a PNG file.
    #[test]
    fn write_to_png() {
        let width = 100;
        let height = 150;
        let mut bitmap = MonoBitmap::new(width, height);
        for y in 0..height {
            for x in 0..width {
                bitmap.set_pixel(x, y, (x + y) % 2 == 0);
            }
        }
        bitmap.write_to_png("target/debug/test-mono.png").unwrap();
    }
}
