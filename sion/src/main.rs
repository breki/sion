#![deny(warnings)]

mod dem_tile;
mod mono_bitmap;
mod grayscale_bitmap;
mod igor_hillshading;
mod proj;
mod consts;
mod trig;
mod slopes;
mod geo;
mod errors;

fn main() {
    println!("Hello, world!");
}

// todo 5: implement 1:1 hillshading of a DEM tile, as the first step towards a full
//  hillshading algorithm

// todo 20: implement a first hillshading algorithm that reads an HGT file and
//   writes a grayscale PNG file