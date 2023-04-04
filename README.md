# Setup

[Install rust](https://www.rust-lang.org/tools/install)

# Usage

- `play`: The only subcommand, playback a MIDI file over OSC
- `-f`: Filepath to the MIDI to play
- `-t`: Target port to broadcast OSC MIDI
- `-s`: Sender port to bind the UDP socket to

# Examples

- Basic usage

        cargo run -- play -f midi_files/Nola__Arndt__Arndt_1915_DA.mid  -t 57101

- Play osc from specific sender port

        cargo run -- play -f midi_files/Nola__Arndt__Arndt_1915_DA.mid -s 57120 -t 57101
