/// Represents a bitmap that can be used to draw on and then sent to the
/// display.
struct Bitmap {
    width: u16,
    height: u16,
    data: Box<[u8]>,
    /// The number of bytes per row in the bitmap.
    width_bytes: u16,
}


impl Bitmap {
    /// Creates a new empty bitmap with the given width and height.
    fn new(width: u16, height: u16) -> Bitmap {
        let width_bytes = (width + 7) / 8;

        Bitmap {
            width,
            height,
            data: vec![0; (width_bytes * height) as usize].into_boxed_slice(),
            width_bytes,
        }
    }

    /// Sets the pixel at the given coordinates to the given value (on or off).
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
}

#[cfg(test)]
mod tests {
    use super::Bitmap;

    /// A new bitmap is created with the correct dimensions and properties.
    #[test]
    fn create_bitmap() {
        let bitmap = Bitmap::new(10, 15);
        assert_eq!(bitmap.width, 10);
        assert_eq!(bitmap.height, 15);
        assert_eq!(bitmap.width_bytes, 2);
        assert_eq!(bitmap.data.len(), 30);
    }

    /// The pixels are off by default when the bitmap is created.
    #[test]
    fn pixels_are_off_by_default() {
        let bitmap = Bitmap::new(10, 15);
        assert_eq!(bitmap.get_pixel(0, 0), false);
        assert_eq!(bitmap.get_pixel(4, 5), false);
    }

    /// Pixels can be set/reset and then retrieved.
    #[test]
    fn set_and_get_pixel() {
        let mut bitmap = Bitmap::new(10, 15);
        bitmap.set_pixel(3, 4, true);
        assert_eq!(bitmap.get_pixel(3, 4), true);
        bitmap.set_pixel(3, 4, false);
        assert_eq!(bitmap.get_pixel(3, 4), false);
    }
}