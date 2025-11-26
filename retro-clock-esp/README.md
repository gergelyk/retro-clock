# Retro Clock ESP

Application for ESP32-S3 target.

## Setup

Get the toolset below (or compatible):
- cargo 1.84.0-nightly (66221abde 2024-11-19)
- rustc 1.84.0-nightly (b0bd11bcc 2025-01-10) (1.84.0.0)

- Install depenencies
    ```sh
    cargo install espflash
    cargo install ldproxy
    cargo install cargo-espflash # Optional
    cargo install espup
    espup install -v 1.84.0 -c 13.2.0_20230928
    ```

- Consider installing other tools mentioned in `../README.md`
- Initialize environment as described in `../README.md`
  Note that `../env`  should contain paths to your xtensa toolchain installed above. These are important only for compiling crates that contain C/C++ files.

## Building

It is best to build using `dev.rs` in the top directory. Alternatively you can build and run as any other cargo package:
```sh
cargo build
cargo run
```

- WebUI must be already built and available under `./.webui_dist`. Otherwise you can use `-F no_file_server`.
- Host must be connected over USB cable with debug port of ESP32. Power switch must be in position DBG. Remaining cables must be disconnected.
- For testing disable CORS by adding `-F no_cors` (make sure to pass the same features to `build` and `run` if both commands are used).
- For available features refer to `Cargo.toml`
- For available application parameters refer to `../.env`.
- `cargo run` uploads executable to the device. Alternatively you can do this using `espflash` directly:
```sh
espflash flash target/xtensa-esp32s3-espidf/debug/retro_clock_esp --partition-table partition_table.csv
espflash monitor
```

Complete cleaning:
```sh
cargo clean
rm -fr .embuild
```

## Development

- Essential ESP-IDF parameters can be set in `sdkconfig.defaults`. Complete list of parameters can be obtained like: `../dev.rs cfg`

- Version of ESP-IDF is specified in `./cargo/config.toml`

- Adding ESP-IDF components can be done via `Cargo.toml`, section `[[package.metadata.esp-idf-sys.extra_components]]`

- Size of partitions available for the app and non-volatile storage can be adjusted in `./partition_table.csv`

## Usage

- For application usage refer to `../docs/user-manual.md`
- For observability options check `../dev.rs plot --help`
- Whenever console is closed you can re-open it using `./dev.rs term`
- Hit '?' in the console to see the full list of keyboard shortcuts

## TLS Cert

- Install TLS cert as described in the User Manual.
- Theoretically root CA cert can be also installed using: `mkcert -install`, but it is more reliable to manually add it to the browser from `$(mkcert -CAROOT)/rootCA.pem`.
- Also add CA cert manually to bruno, if you plan to use it.
