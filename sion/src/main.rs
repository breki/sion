#![deny(warnings)]

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "water-bodies")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    GenerateTile {
        #[arg(String)]
        tile_id: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::GenerateTile { tile_id } => {
            println!("Tile data generated for tile ID: {}", tile_id);
        }
    }
}

// todo: profile the code (maybe using http://www.codersnotes.com/sleepy/
//   or https://superluminal.eu/rust or VC Code (https://dev.to/jambochen/profiling-rust-with-vs-on-windows-3m4l))
