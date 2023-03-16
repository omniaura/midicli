use clap::Parser;
use clap_derive::{Parser, ValueEnum};
use midly::{self, MetaMessage};
use std::{path::PathBuf, thread::sleep, time::Duration};
use tracing::{debug, info, Level};
use tracing_subscriber::{fmt::time, FmtSubscriber};

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

    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        .with_timer(time::uptime())
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Load bytes first
    let data = std::fs::read(cli.file).unwrap();

    // Parse the raw bytes
    let smf = midly::Smf::parse(&data).unwrap();

    // Use the information
    info!("midi file has {} tracks!", smf.tracks.len());

    let mut ticks_per_beat = u32::MIN;
    // check the file timing
    match smf.header.timing {
        midly::Timing::Metrical(m) => {
            info!("metrical timing: {}", m);
            ticks_per_beat = u32::from(m.as_int());
        }
        midly::Timing::Timecode(fps, subframes) => {
            info!("timecode timing, fps: {:?}; subframes: {}", fps, subframes);
        }
    }

    debug!("header: {:?}", smf.header);

    let mut microseconds_per_beat = u32::MIN;
    // iter through the notes and play over osc
    for track in smf.tracks {
        for event in track {
            // debug!("{:?}", event);

            // grab the meta messages for timing
            use midly::TrackEventKind::*;
            match event.kind {
                Meta(meta) => {
                    use MetaMessage::*;
                    match meta {
                        Tempo(tempo) => {
                            microseconds_per_beat = tempo.as_int();
                            info!("microseconds per beat: {}", microseconds_per_beat);
                        }
                        TimeSignature(num, denom, per_tick, thirty_second) => {}
                        _ => {}
                    }
                }
                _ => {}
            }
            let Some(live_event) = event.kind.as_live_event()
            else {
                continue;
            };
            debug!("{:?}", live_event);
            if event.delta != 0 {
                let delta = u64::from(event.delta.as_int());
                let microseconds_per_tick = microseconds_per_beat / ticks_per_beat;
                let delta_micros = delta * u64::from(microseconds_per_tick);
                sleep(Duration::from_micros(delta_micros));
            }
        }
    }
}
