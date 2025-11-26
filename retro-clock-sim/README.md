# Retro Clock Sim

Provides partial host-based simulation of *retro-clock* application.

## Setup

Get the toolset below (or compatible):
- cargo 1.92.0-nightly (f2932725b 2025-09-24)
- rustc 1.92.0-nightly (c8905eaa6 2025-09-28)

- Consider installing other tools mentioned in `../README.md`
- Initialize environment as described in `../README.md`

## Building & Running

The easiest way is to use `../dev.rs` script. Alternatively you can do the following:

- Open two shell sessions
- In the 1st session invoke `tty` and note the output, e.g. /dev/pts/3
- In the 2nd session invoke `cargo run -F no_cors > /dev/pts/3`, with the pts from the previous step
- `-F no_cors` can be skipped if you don't need to use Web UI, or disable CORS rules in the browser
- Web UI can be started independently. Refer to `../retro-clock-webui/READEM.md`
- For more fetatures check `Cargo.toml`
- Hit '?' in the console to see the full list of keyboard shortcuts
