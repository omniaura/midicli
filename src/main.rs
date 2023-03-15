use clap::Parser;
use clap_derive::{Parser, ValueEnum};
use midly;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(value_enum)]
    command: Cmd,

    #[arg(short, long, value_name = "MIDI FILE")]
    file: PathBuf,

    #[arg(short, long, value_name = "OSC PORT")]
    port: Option<u16>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Cmd {
    /// Play a MIDI file
    Play,
}
fn main() {
    let cli = Cli::parse();
    println!("Hello, world! {:?}", cli.command);

    // Load bytes first
    let data = std::fs::read(cli.file).unwrap();

    // Parse the raw bytes
    let mut smf = midly::Smf::parse(&data).unwrap();

    // Use the information
    println!("midi file has {} tracks!", smf.tracks.len());

    // Modify the file
    smf.header.format = midly::Format::SingleTrack;

    // Save it back
    smf.save("PiRewritten.mid").unwrap();

    // iter through the notes and play over osc
}
