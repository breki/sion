use image::{GrayImage, Luma};

/// Represents a 8-bit grayscale bitmap that can be used to draw on and then
/// be sent to the display.
#[derive(Debug)]
pub struct Grayscale8Bitmap {
    pub width: u16,
    pub height: u16,
    data: Box<[u8]>,
}

impl Grayscale8Bitmap {
    /// Creates a new empty grayscale bitmap with the given width and height.
    pub fn new(width: u16, height: u16) -> Grayscale8Bitmap {
        Grayscale8Bitmap {
            width,
            height,
            data: vec![0; width as usize * height as usize].into_boxed_slice(),
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Gets the value of the pixel at the given coordinates.
    pub fn get_pixel(&self, x: u16, y: u16) -> u8 {
        if x >= self.width || y >= self.height {
            panic!("Pixel coordinates out of bounds");
        }

        let index = y as usize * self.width as usize + x as usize;
        self.data[index]
    }

    /// Sets the pixel at the given coordinates to the given value (on or off).
    pub fn set_pixel(&mut self, x: u16, y: u16, value: u8) {
        if x >= self.width || y >= self.height {
            panic!("Pixel coordinates out of bounds");
        }

        let index = (x as u32)
            .checked_mul(self.width as u32)
            .and_then(|w| w.checked_add(y as u32))
            .expect("Overflow in pixel index calculation");
        self.data[index as usize] = value;
    }

    /// Extracts a sub-region of the bitmap as a new `GrayscaleBitmap`.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the top-left corner of the region.
    /// * `y` - The y-coordinate of the top-left corner of the region.
    /// * `width` - The width of the region to extract.
    /// * `height` - The height of the region to extract.
    ///
    /// # Panics
    ///
    /// Panics if the specified region is out of bounds.
    pub fn extract(
        &self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
    ) -> Grayscale8Bitmap {
        if x + width > self.width || y + height > self.height {
            panic!("Extract region is out of bounds");
        }

        let mut extracted_data =
            vec![0; (width as usize) * (height as usize)].into_boxed_slice();

        for row in 0..height {
            let src_start =
                (y + row) as usize * self.width as usize + x as usize;
            let src_end = src_start + width as usize;

            let dest_start = row as usize * width as usize;
            let dest_end = dest_start + width as usize;

            extracted_data[dest_start..dest_end]
                .copy_from_slice(&self.data[src_start..src_end]);
        }

        Grayscale8Bitmap {
            width,
            height,
            data: extracted_data,
        }
    }

    /// Writes the grayscale bitmap to a PNG file.
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
                let pixel_value = self.get_pixel(x, y);
                img.put_pixel(x.into(), y.into(), Luma([pixel_value]));
            }
        }
        img.save(file_path)
    }
}

#[cfg(test)]
mod tests {
    use super::Grayscale8Bitmap;

    #[test]
    fn create_large_bitmap() {
        let bitmap = Grayscale8Bitmap::new(1000, 1000);
        assert_eq!(bitmap.width, 1000);
        assert_eq!(bitmap.height, 1000);
        assert_eq!(bitmap.data.len(), 1000 * 1000);
    }

    /// A new bitmap is created with the correct dimensions and properties.
    #[test]
    fn create_bitmap() {
        let bitmap = Grayscale8Bitmap::new(10, 15);
        assert_eq!(bitmap.width, 10);
        assert_eq!(bitmap.height, 15);
        assert_eq!(bitmap.data.len(), 150);
    }

    /// The pixels are black by default when the bitmap is created.
    #[test]
    fn pixels_are_black_by_default() {
        let bitmap = Grayscale8Bitmap::new(10, 15);
        assert_eq!(bitmap.get_pixel(0, 0), 0);
        assert_eq!(bitmap.get_pixel(4, 5), 0);
    }

    /// Pixels can be set and then retrieved.
    #[test]
    fn set_and_get_pixel() {
        let mut bitmap = Grayscale8Bitmap::new(10, 15);
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
        let mut bitmap = Grayscale8Bitmap::new(width, height);
        for y in 0..height {
            for x in 0..width {
                bitmap.set_pixel(x, y, ((x + y) * 5) as u8);
            }
        }
        bitmap
            .write_to_png("target/debug/test-grayscale.png")
            .unwrap();
    }
}
