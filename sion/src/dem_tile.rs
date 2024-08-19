use crate::errors::SionError;
use std::fs::File;
use std::io::{BufReader, Read};
use std::os::windows::fs::MetadataExt;
use std::path::Path;

pub struct DemTile {
    pub lon: i16,
    pub lat: i16,
    pub size: usize,
    data: Box<[u8]>,
}

impl DemTile {
    pub fn new(lon: i16, lat: i16, size: usize, data: Vec<u8>) -> DemTile {
        DemTile {
            lon,
            lat,
            size,
            data: data.into_boxed_slice(),
        }
    }

    // create a static constructor that reads the data from a file
    pub fn from_file(file: &str) -> DemTile {
        let tile_name = Path::new(file)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap();
        let (lon, lat) = DemTile::parse_tile_name(tile_name).unwrap();

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

                    let mut byte_array: Vec<u8> =
                        vec![0; metadata.file_size() as usize];
                    reader
                        .read_exact(&mut byte_array)
                        .expect("Failed to read the HGT file.");

                    DemTile::new(lon, lat, tile_size as usize, byte_array)
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
        let byte_offset = ((y as usize) * (self.size) + (x as usize)) << 1;
        (self.data[byte_offset] as i16) << 8
            | (self.data[byte_offset + 1] as i16)
    }

    pub fn height_at_index(&self, index: usize) -> i16 {
        let byte_offset = index << 1;
        (self.data[byte_offset] as i16) << 8
            | (self.data[byte_offset + 1] as i16)
    }

    pub fn parse_tile_name(tile_name: &str) -> Result<(i16, i16), SionError> {
        fn parse_lat_sign(tile_name: &str) -> Result<i16, SionError> {
            match tile_name.chars().nth(0) {
                Some('N') => Ok(1),
                Some('S') => Ok(-1),
                _ => Err(SionError::new("Invalid tile name")),
            }
        }

        fn parse_lat(tile_name: &str, lat_sign: i16) -> Result<i16, SionError> {
            match tile_name[1..3].parse::<i16>() {
                Ok(lat) => Ok(lat_sign * lat),
                Err(_) => Err(SionError::new("Invalid tile name")),
            }
        }

        fn parse_lon_sign(
            tile_name: &str,
            lat: i16,
        ) -> Result<(i16, i16), SionError> {
            match tile_name.chars().nth(3) {
                Some('E') => Ok((1, lat)),
                Some('W') => Ok((-1, lat)),
                _ => Err(SionError::new("Invalid tile name")),
            }
        }

        fn parse_lon(
            tile_name: &str,
            lon_sign: i16,
            lat: i16,
        ) -> Result<(i16, i16), SionError> {
            match tile_name[4..7].parse::<i16>() {
                Ok(lon) => Ok((lon_sign * lon, lat)),
                Err(_) => Err(SionError::new("Invalid tile name")),
            }
        }

        match tile_name.len() {
            7 => Ok(()),
            _ => Err(SionError::new("Invalid tile name")),
        }
        .and_then(|_| parse_lat_sign(&tile_name))
        .and_then(|lat_sign| parse_lat(&tile_name, lat_sign))
        .and_then(|lat| parse_lon_sign(&tile_name, lat))
        .and_then(|(lon_sign, lat)| parse_lon(&tile_name, lon_sign, lat))
    }
}

#[cfg(test)]
mod tests {
    use super::DemTile;
    use rstest::rstest;

    #[rstest]
    #[case("N46E006", 6, 46)]
    #[case("S46W123", -123, -46)]
    fn valid_tile_names(
        #[case] file_name: &str,
        #[case] expected_lon: i16,
        #[case] expected_lat: i16,
    ) {
        match DemTile::parse_tile_name(file_name) {
            Ok((lon, lat)) => {
                assert_eq!(lon, expected_lon);
                assert_eq!(lat, expected_lat);
            }
            Err(_) => {
                assert!(false, "Failed to parse tile name");
            }
        }
    }

    #[rstest]
    #[case("X", "Invalid tile name")]
    #[case("46E006", "Invalid tile name")]
    #[case("SX6W123", "Invalid tile name")]
    #[case("S16W1234", "Invalid tile name")]
    fn invalid_tile_names(
        #[case] file_name: &str,
        #[case] expected_error: &str,
    ) {
        match DemTile::parse_tile_name(file_name) {
            Ok(_) => {
                assert!(false, "Should not have parsed tile name");
            }
            Err(error) => {
                assert_eq!(error.message, expected_error);
            }
        }
    }

    #[test]
    fn read_from_file() {
        let tile = DemTile::from_file("tests/data/N46E006.hgt");
        assert_eq!(tile.lon, 6);
        assert_eq!(tile.lat, 46);
        assert_eq!(tile.size, 3601);
        assert_eq!(tile.height_at(100, 100), 732);
    }
}
