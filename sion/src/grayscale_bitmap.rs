/// Represents a 8-bit grayscale bitmap that can be used to draw on and then
/// be sent to the display.
struct GrayscaleBitmap {
    width: u16,
    height: u16,
    data: Box<[u8]>,
}


impl GrayscaleBitmap {
    /// Creates a new empty grayscale bitmap with the given width and height.
    fn new(width: u16, height: u16) -> GrayscaleBitmap {
        GrayscaleBitmap {
            width,
            height,
            data: vec![0; (width * height) as usize].into_boxed_slice(),
        }
    }

    /// Sets the pixel at the given coordinates to the given value (on or off).
    fn set_pixel(&mut self, x: u16, y: u16, value: u8) {
        let index = (y * self.width + x) as usize;
        self.data[index] = value;
    }

    /// Gets the value of the pixel at the given coordinates.
    fn get_pixel(&self, x: u16, y: u16) -> u8 {
        let index = (y * self.width + x) as usize;
        self.data[index]
    }
}

#[cfg(test)]
mod tests {
    use super::GrayscaleBitmap;

    /// A new bitmap is created with the correct dimensions and properties.
    #[test]
    fn create_bitmap() {
        let bitmap = GrayscaleBitmap::new(10, 15);
        assert_eq!(bitmap.width, 10);
        assert_eq!(bitmap.height, 15);
        assert_eq!(bitmap.data.len(), 150);
    }

    /// The pixels are black by default when the bitmap is created.
    #[test]
    fn pixels_are_black_by_default() {
        let bitmap = GrayscaleBitmap::new(10, 15);
        assert_eq!(bitmap.get_pixel(0, 0), 0);
        assert_eq!(bitmap.get_pixel(4, 5), 0);
    }

    /// Pixels can be set and then retrieved.
    #[test]
    fn set_and_get_pixel() {
        let mut bitmap = GrayscaleBitmap::new(10, 15);
        bitmap.set_pixel(3, 4, 123);
        assert_eq!(bitmap.get_pixel(3, 4), 123);
        bitmap.set_pixel(3, 4, 255);
        assert_eq!(bitmap.get_pixel(3, 4), 255);
    }
}