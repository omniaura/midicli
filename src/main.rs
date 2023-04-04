use clap::Parser;
use clap_derive::{Parser, Subcommand};
use midly::{self, MetaMessage};
use rosc::{OscMessage, OscMidiMessage, OscPacket, OscType};
use std::{net::SocketAddr, path::PathBuf, thread::sleep, time::Duration, vec};
use tracing::{debug, info, warn, Level};
use tracing_subscriber::{fmt::time, FmtSubscriber};
mod osc_udp_client;
use osc_udp_client::OscUdpClient;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,

    #[arg(short, long)]
    log_level: Option<Level>
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Subcommand)]
#[command(author, version, about, long_about = None)]
enum Cmd {
    /// Play a MIDI file
    #[command(author, version, about, long_about = None)]
    Play {
        /// Path to the MIDI file to play
        #[arg(short, long, value_name = "MIDI FILE")]
        file: PathBuf,

        /// Set the socket addres that the sender binds to.
        /// Use if the receiver filters to receive from a specific socket address.
        #[arg(short, long, value_name = "OSC FROM PORT")]
        sender: Option<u16>,

        /// The destination port to play MIDI over osc.
        #[arg(short, long, value_name = "OSC TO PORT")]
        to: u16,
    },
}

fn main() {
    let cli = Cli::parse();

    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(cli.log_level.unwrap_or(Level::WARN))
        .with_timer(time::uptime())
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    match cli.command {
        Cmd::Play { file, to, sender } => play_osc(file, to, sender),
    }
}

fn play_osc(file: PathBuf, to: u16, sender: Option<u16>) {
    let sender: SocketAddr = match sender {
        Some(port) => format!("0.0.0.0:{}", port).parse().unwrap(),
        None => "0.0.0.0:0".parse().unwrap(),
    };
    let to: SocketAddr = format!("0.0.0.0:{}", to).parse().unwrap();

    // create osc client
    let osc_client = OscUdpClient::new(sender, to).unwrap();

    // Load bytes first
    let data = std::fs::read(file).unwrap();

    // Parse the raw bytes
    let smf = midly::SmfBytemap::parse(&data).unwrap();

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

            match event.1.kind {
                Meta(meta) => {
                    use MetaMessage::*;
                    match meta {
                        Tempo(tempo) => {
                            microseconds_per_beat = tempo.as_int();
                            info!("microseconds per beat: {}", microseconds_per_beat);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }

            let Some(live_event) = event.1.kind.as_live_event()
            else {
                continue;
            };
            debug!("{:?}", live_event);
            let mut buf = Vec::new();
            live_event.write(&mut buf).unwrap();
            match buf.len() {
                3 => {
                    use midly::live::LiveEvent;
                    let port = match live_event {
                        LiveEvent::Midi { channel, .. } => channel.as_int(),
                        _ => 0,
                    };

                    let addr = "/midi".to_string();
                    let msg = OscMidiMessage {
                        port,
                        status: buf[0],
                        data1: buf[1],
                        data2: buf[2],
                    };
                    let msg_print = msg.clone();
                    let args = vec![OscType::Midi(msg)];
                    let packet = &OscPacket::Message(OscMessage { addr, args });
                    match osc_client.send(&packet) {
                        Ok(_res) => {
                            debug!("send: {:?}; port: {:?}", &msg_print, &osc_client.to);
                        }
                        Err(err) => warn!("error: {}", err),
                    }
                }
                len => {
                    debug!("buffer len: {}", len)
                }
            }

            // send messages thru osc
            if event.1.delta != 0 {
                let delta = u64::from(event.1.delta.as_int());
                let microseconds_per_tick = microseconds_per_beat / ticks_per_beat;
                let delta_micros = delta * u64::from(microseconds_per_tick);
                sleep(Duration::from_micros(delta_micros));
            }
        }
    }
}
