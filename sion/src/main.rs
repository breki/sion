#![deny(warnings)]

use clap::{Parser, Subcommand};
use sion::water_bodies::command::generate_water_bodies_tile;
use sion::water_bodies::dem_tile_id::DemTileId;

#[derive(Parser)]
#[command(name = "water-bodies")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    GenerateTile { tile_id: DemTileId },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::GenerateTile { tile_id } => {
            // todo 0: prepare function skeleton for generating water bodies tile
            match generate_water_bodies_tile(tile_id) {
                Ok(_) => println!("Water bodies tile generated successfully."),
                Err(e) => {
                    eprintln!("Error generating water bodies tile: {}", e)
                }
            }
        }
    }
}

// todo: profile the code (maybe using http://www.codersnotes.com/sleepy/
//   or https://superluminal.eu/rust or VC Code (https://dev.to/jambochen/profiling-rust-with-vs-on-windows-3m4l))
