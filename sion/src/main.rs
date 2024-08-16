#![deny(warnings)]

mod consts;
mod dem_tile;
mod errors;
mod geo;
mod grayscale_bitmap;
mod igor_hillshading;
mod mono_bitmap;
mod proj;
mod slopes;
mod testing;
mod trig;

fn main() {
    println!("Hello, world!");
}

// todo 10: implement 1:1 hillshading of a DEM tile, as the first step towards a full
//  hillshading algorithm

// todo 20: implement a first hillshading algorithm that reads an HGT file and
//   writes a grayscale PNG file
