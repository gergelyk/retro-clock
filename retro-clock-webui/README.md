# Retro Clock WebUI

Provides configuration interface for *retro-clock* application.

## Setup

Get the toolset below (or compatible):
- cargo 1.92.0-nightly (f2932725b 2025-09-24)
- rustc 1.92.0-nightly (c8905eaa6 2025-09-28)
- trunk 0.21.14

- Add WASM target:
    ```sh
    rustup target add wasm32-unknown-unknown
    ```

- Consider installing other tools mentioned in `../README.md`
- Initialize environment as described in `../README.md`

## Development

Use extra step for formatting:
```sh
leptosfmt .
cargo fmt
```

## Building

This package can be build as `trunk build`. Application can be run locally for embedded in ESP32 and served by the device.
For the further option we do some optimizations of the distribution. `../dev.rs` script will do this for you and prepare separate distribution in `../retro-clock-esp/.webui_dist`.

## Running

Application can be run locally against `retro-clock-sim` or `retro-clock-esp`. Best option is to use `../dev.rs` script. Check `../dev.rs run --help` Alternatively you can:

```sh
# set this for retro-clock-sim
export API_URL=http://127.0.0.1:3330
# ..or this for retro-clock-esp
export API_URL=https://retro-clock.local
# serve UI
trunk serve --port 8880
```

Open your browser at: http://127.0.0.1:8880

`retro-clock-sim or` `retro-clock-esp` must be run in advance according to the instructions in their directories.

CORS rules must be either disabled in the browser, or on the server side (configurable by the features of corresponding packages).
