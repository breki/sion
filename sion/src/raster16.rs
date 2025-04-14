use image::{ImageBuffer, Luma};

/// Represents a 16-bit raster.
#[derive(Debug)]
pub struct Raster16 {
    pub width: u16,
    pub height: u16,
    data: Box<[u16]>,
}

impl Raster16 {
    /// Creates a new empty raster with the given width and height.
    pub fn new(width: u16, height: u16) -> Raster16 {
        Raster16 {
            width,
            height,
            data: vec![0; width as usize * height as usize].into_boxed_slice(),
        }
    }

    /// Gets the value of the pixel at the given coordinates.
    pub fn get_pixel(&self, x: u16, y: u16) -> u16 {
        if x >= self.width || y >= self.height {
            panic!("Pixel coordinates out of bounds");
        }

        let index = y as usize * self.width as usize + x as usize;
        self.data[index]
    }

    /// Sets the pixel at the given coordinates to the given value.
    pub fn set_pixel(&mut self, x: u16, y: u16, value: u16) {
        if x >= self.width || y >= self.height {
            panic!("Pixel coordinates out of bounds");
        }

        let index = y as usize * self.width as usize + x as usize;
        self.data[index] = value;
    }

    /// Writes the 16-bit raster to a PNG file.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The path to the output PNG file.
    pub fn write_to_png(
        &self,
        file_path: &str,
    ) -> Result<(), image::ImageError> {
        let mut img: ImageBuffer<Luma<u16>, Vec<u16>> =
            ImageBuffer::new(self.width.into(), self.height.into());
        for y in 0..self.height {
            for x in 0..self.width {
                let pixel_value = self.get_pixel(x, y);
                img.put_pixel(x.into(), y.into(), Luma([pixel_value]));
            }
        }
        img.save(file_path)
    }
}

#[cfg(test)]
mod tests {
    use super::Raster16;

    #[test]
    fn create_large_bitmap() {
        let bitmap = Raster16::new(1000, 1000);
        assert_eq!(bitmap.width, 1000);
        assert_eq!(bitmap.height, 1000);
        assert_eq!(bitmap.data.len(), 1000 * 1000);
    }

    /// A new bitmap is created with the correct dimensions and properties.
    #[test]
    fn create_bitmap() {
        let bitmap = Raster16::new(10, 15);
        assert_eq!(bitmap.width, 10);
        assert_eq!(bitmap.height, 15);
        assert_eq!(bitmap.data.len(), 150);
    }

    /// The pixels are black by default when the bitmap is created.
    #[test]
    fn pixels_are_black_by_default() {
        let bitmap = Raster16::new(10, 15);
        assert_eq!(bitmap.get_pixel(0, 0), 0);
        assert_eq!(bitmap.get_pixel(4, 5), 0);
    }

    /// Pixels can be set and then retrieved.
    #[test]
    fn set_and_get_pixel() {
        let mut bitmap = Raster16::new(10, 15);
        bitmap.set_pixel(3, 4, 123);
        assert_eq!(bitmap.get_pixel(3, 4), 123);
        bitmap.set_pixel(3, 4, 255);
        assert_eq!(bitmap.get_pixel(3, 4), 255);
    }

    /// The bitmap can be written to a PNG file.
    #[test]
    fn write_to_png() {
        let width = 100;
        let height = 150;
        let mut bitmap = Raster16::new(width, height);
        for y in 0..height {
            for x in 0..width {
                bitmap.set_pixel(x, y, (x + y) * 5);
            }
        }
        bitmap
            .write_to_png("target/debug/test-raster16.png")
            .unwrap();
    }
}
