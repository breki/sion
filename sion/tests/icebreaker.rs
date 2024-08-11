use byteorder::{BigEndian, ReadBytesExt};
use std::fs::File;
use std::io::BufReader;

#[test]
fn icebreaker() {
    // open the file in binary mode and read it whole into an array
    let file = File::open("tests/data/N46E006.hgt");

    match file {
        Ok(file) => {
            let mut reader = BufReader::new(file);
            let mut buffer: Vec<i16> = Vec::new();

            loop {
                let height = reader.read_i16::<BigEndian>();
                match height {
                    Ok(height) => {
                        buffer.push(height);
                    }
                    Err(_) => {
                        break;
                    }
                }
            }

            assert_eq!(buffer[3601*100 + 100], 732);
        }
        Err(error) => {
            // raise an error if the file cannot be opened
            panic!("Problem opening the file: {:?}", error);
        }
    }
}