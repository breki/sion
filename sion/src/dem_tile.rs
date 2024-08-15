use std::fs::File;
use std::io::{BufReader, Read};
use std::os::windows::fs::MetadataExt;

pub struct DemTile {
    pub size: u16,
    data: Box<[u8]>,
}

impl DemTile {
    pub fn new(size: u16, data: Vec<u8>) -> DemTile {
        DemTile {
            size,
            data: data.into_boxed_slice(),
        }
    }

    // create a static constructor that reads the data from a file
    pub fn from_file(file: &str) -> DemTile {
        let file = File::open(file);
        match file {
            Ok(file) => {
                // get the file size
                let metadata = file.metadata().unwrap();
                let total_heights_count = metadata.file_size() / 2;

                let tile_size = (total_heights_count as f64).sqrt() as u64;
                if tile_size * tile_size == total_heights_count {
                    // read the whole file into a byte array
                    let mut reader = BufReader::new(file);

                    let mut byte_array: Vec<u8> = vec![0; metadata.file_size() as usize];
                    reader.read_exact(&mut byte_array).expect("Failed to read the HGT file.");

                    println!("Read: {}", byte_array[(100 * 3601 + 100) * 2]);

                    DemTile::new(tile_size as u16, byte_array)
                } else {
                    panic!("The HGT file does not contain a square number of heights");
                }
            }
            Err(error) => {
                // raise an error if the file cannot be opened
                panic!("Problem opening the HGT file: {:?}", error);
            }
        }
    }


    pub fn height_at(&self, x: u16, y: u16) -> i16 {
        let index = ((y as usize) * (self.size as usize) + (x as usize)) << 1;
        (self.data[index] as i16) << 8 | (self.data[index + 1] as i16)
    }
}


#[cfg(test)]
mod tests {
    use super::DemTile;

    #[test]
    fn read_from_file() {
        let tile = DemTile::from_file("tests/data/N46E006.hgt");
        assert_eq!(tile.height_at(100, 100), 732);
    }
}