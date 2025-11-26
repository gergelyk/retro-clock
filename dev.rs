#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! [dependencies]
//! anyhow = "1"
//! clap = { version = "4.5", features = ["derive"] }
//! dotenvy = "0.15"
//! duct = "1.1"
//! log = "0.4"
//! colog = "1.4"
//! pathexpand = "0.1"
//! tmp_env = "0.1"
//! ```

use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use duct::cmd;
use std::cell::RefCell;
use std::path::Path;

const RCLOCK_LOCAL_CONFIG_PORT_DEFAULT: &str = "8880";
const RCLOCK_SIM_SERVER_PORT_DEFAULT: &str = "3330";
const RCLOCK_ESP_SERVER_URL: &str = "https://retro-clock.local";

const RUN_KDL_SIM: &str = r#"// zellij layout for running this app
layout {
    pane split_direction="vertical" {
        pane split_direction="horizontal" {
            pane name="SimLogs" {
                cwd "retro-clock-sim"
                command "bash"
                args "-c" "cargo run -F no_cors > .display"
            }
            pane name="WebUiLogs" {
                cwd "retro-clock-webui"
                command "bash"
                args "-c" "trunk serve --port $RCLOCK_LOCAL_CONFIG_PORT"
            }
        }
        pane split_direction="horizontal" size="30" {
            pane name="SimDisplay" {
                cwd "retro-clock-sim"
                command "bash"
                args "-c" "cat .display"
            }
            pane name="ReferenceClock" {
                cwd "reference-clock"
                command "bash"
                args "-c" "cargo run"
            }
        }
    }
}
"#;

const RUN_KDL_MIX_FULL: &str = r#"// zellij layout for running this app
layout {
    pane split_direction="horizontal" {
        pane name="EspLogs" {
            cwd "retro-clock-esp"
            command "bash"
            args "-c" "cargo run -F no_cors -F no_file_server"
        }
        pane name="WebUiLogs" {
            cwd "retro-clock-webui"
            command "bash"
            args "-c" "trunk serve --port $RCLOCK_LOCAL_CONFIG_PORT"
        }
    }
}
"#;

const RUN_KDL_MIX_NO_FLASH: &str = r#"// zellij layout for running this app
layout {
    pane split_direction="horizontal" {
        pane name="EspLogs" {
            command "espflash"
            args "monitor"
        }
        pane name="WebUiLogs" {
            cwd "retro-clock-webui"
            command "bash"
            args "-c" "trunk serve --port $RCLOCK_LOCAL_CONFIG_PORT"
        }
    }
}
"#;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Build and run this project under selected target.
    #[command(subcommand)]
    Run(RunCommand),
    /// Create new asset.
    #[command(subcommand)]
    New(NewCommand),
    /// Format entire code.
    Fmt,
    /// Run any cargo sub-command against every sub-project.
    Cmd { cargo_cmd: String },
    /// List available ESP-IDF settings.
    Cfg,
    /// Connect terminal to the serial interface of the device.
    Term,
    /// Plot debug data from the serial interface of the device.
    #[command(subcommand)]
    Plot(PlotCommand),
}

#[derive(Subcommand)]
enum RunCommand {
    /// Simulated device and locally hosted UI
    Sim,
    /// Physical device with embedded UI
    Esp(RunEspArgs),
    /// Physical device with locally hosted UI
    Mix(RunMixArgs),
}

#[derive(Args)]
struct RunEspArgs {
    /// Enable panic on TX buffer overflow or thread error. Use for debugging.
    #[arg(short, long)]
    panic: bool,

    /// Compile in release mode.
    #[arg(short, long)]
    release: bool,

    /// Skip releasing config dist. Only makes sense if config dist was released before.
    #[arg(short, long)]
    skip_webui_release: bool,
}

#[derive(Args)]
struct RunMixArgs {
    /// Skip flashing ESP target. Assumes that it is already flashed with CORS disabled and that the debugger cable is connected.
    #[arg(short, long)]
    skip_flashing: bool,
}

#[derive(Subcommand)]
enum NewCommand {
    /// Local .env file.
    Env,
    /// Certificate for TLS for HTTP server.
    Cert,
}

#[derive(Subcommand)]
enum PlotCommand {
    /// Ambient light reading, before and after smoothing.
    Light,
    /// Heap memory usage.
    Heap,
    /// Millisecond when display was refreshed. Should be a small value (unless pages are being changed).
    Time,
    /// Time remaining after refreshing the display. Should be close to 1000 (unless pages are being changed).
    Delay,
}

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;
    colog::init();
    let cli = Cli::parse();

    let script_path_str = std::env::var("RUST_SCRIPT_PATH")?;
    let script_path = Path::new(&script_path_str);
    let script_dir = script_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Failed to get script dir"))?;
    log::info!("Setting current dir to {}", script_dir.to_string_lossy());
    std::env::set_current_dir(script_dir)?;

    match cli.command {
        Command::Run(subcommand) => {
            if !Path::new(".env").exists() {
                log::warn!("Missing .env file, consider running `dev.rs new env` first");
            }
            if !Path::new("retro-clock-esp/certs").exists() {
                log::warn!("TLS certificate not found, consider running `dev.rs new cert` first");
            }
            match subcommand {
                RunCommand::Sim => run_sim()?,
                RunCommand::Esp(RunEspArgs {
                    panic,
                    release,
                    skip_webui_release,
                }) => run_esp(panic, release, skip_webui_release)?,
                RunCommand::Mix(RunMixArgs { skip_flashing }) => run_mix(skip_flashing)?,
            }
        }
        Command::New(subcommand) => match subcommand {
            NewCommand::Cert => new_cert()?,
            NewCommand::Env => new_env()?,
        },
        Command::Fmt => {
            cmd!("leptosfmt", ".")
                .dir("retro-clock-webui")
                .run()
                .context("Failed to format retro-clock-webui (rust)")?;

            run_cargo_cmd("fmt")?;
        }
        Command::Cmd { cargo_cmd } => {
            run_cargo_cmd(&cargo_cmd)?;
        }
        Command::Cfg => {
            cmd!(
                "bash",
                "-c",
                "rg -oI 'CONFIG_[a-zA-Z0-9_]*' retro-clock-esp/.embuild | sort | uniq"
            )
            .run()
            .context("Failed to list ESP-IDF settings")?;
        }
        Command::Term => {
            cmd!("minicom", "-D", "/dev/ttyACM0", "--color=on")
                .env("HOME", ".") // minicom reads config files from $HOME/.minirc.*
                .run()
                .context("Failed to start minicom")?;
        }
        Command::Plot(subcommand) => match subcommand {
            PlotCommand::Light => {
                start_plotting("AmbientLight", &["ambient_light", "ambient_light_avr"])?;
            }
            PlotCommand::Heap => {
                start_plotting("HeapStats", &["current_free_kb", "minimum_free_kb"])?;
            }
            PlotCommand::Time => {
                start_plotting("WorldTime", &["millisecond"])?;
            }
            PlotCommand::Delay => {
                start_plotting(
                    "Delay",
                    &[
                        "time_to_round_second_ms",
                        "residual_time_to_round_second_ms",
                    ],
                )?;
            }
        },
    }

    Ok(())
}

fn start_plotting(key: &str, labels: &[&str]) -> anyhow::Result<()> {
    cmd!("stty", "-F", "/dev/ttyACM0", "115200", "raw", "-echo")
        .run()
        .context("Failed to configure serial port")?;

    let mut pipe = String::from("cat /dev/ttyACM0");
    pipe.push_str(r#" | stdbuf -oL grep -oE '"#);
    pipe.push_str(key);
    pipe.push_str(r#" [^}]*'"#);
    pipe.push_str(r#" | sed -u 's/[^0-9]\+/,/g; s/^,*//; s/,*$//'"#);
    pipe.push_str(r#" | wesplot -w 400 -t retro-clock"#);
    for label in labels.iter() {
        pipe.push_str(r#" -c "#);
        pipe.push_str(label);
    }
    pipe.push_str(r#" --ymin 0"#);

    cmd!("bash", "-c", pipe)
        .run()
        .context("Failed to start monitoring")?;
    Ok(())
}

/// Run cargo <cmd> in every sub-project, including dev.rs itself
fn run_cargo_cmd(cmd: &str) -> anyhow::Result<()> {
    let dev_rs_package = cmd!("rust-script", "-p", "dev.rs")
        .read()
        .context("Failed to package dev.rs")?;

    for dir in [
        "retro-clock-webui",
        "retro-clock-core",
        "retro-clock-sim",
        "retro-clock-esp",
        "reference-clock",
        &dev_rs_package,
    ] {
        cmd!("cargo", &cmd)
            .dir(dir)
            .run()
            .context(format!("Failed to run {} in {}", cmd, dir))?;
    }
    Ok(())
}

fn run_sim() -> anyhow::Result<()> {
    cmd!("trunk", "build")
        .dir("retro-clock-webui")
        .run()
        .context("Failed to build config app")?;

    if !Path::new("retro-clock-sim/.display").exists() {
        cmd!("mkfifo", ".display")
            .dir("retro-clock-sim")
            .run()
            .context("Failed to create FIFO for sim display")?;
    }

    let local_config_port = std::env::var("RCLOCK_LOCAL_CONFIG_PORT")
        .unwrap_or_else(|_| RCLOCK_LOCAL_CONFIG_PORT_DEFAULT.to_string());
    let sim_server_port = std::env::var("RCLOCK_SIM_SERVER_PORT")
        .unwrap_or_else(|_| RCLOCK_SIM_SERVER_PORT_DEFAULT.to_string());
    cmd!(
        "xdg-open",
        format!("http://127.0.0.1:{}", local_config_port)
    )
    .run()
    .context("Failed to open browser")?;

    std::fs::write(".run.kdl", RUN_KDL_SIM)?;
    cmd!("zellij", "-l", ".run.kdl")
        .env("RUST_LOG", "debug")
        .env(
            "UI_ORIGIN",
            format!("http://127.0.0.1:{}", local_config_port),
        )
        .env("API_URL", format!("http://127.0.0.1:{}", sim_server_port))
        .run()
        .context("Failed to start Zellij")?;

    Ok(())
}

fn run_esp(panic: bool, release: bool, skip_webui_release: bool) -> anyhow::Result<()> {
    if skip_webui_release {
        if !Path::new("retro-clock-esp/.webui_dist").exists() {
            log::warn!(
                "Web UI dist dir doesn't exist. It will be built despite of --skip-ui-release flag."
            );
            release_webui_app()?;
        } else {
            log::info!("Skipping releasing web UI app dist");
        }
    } else {
        release_webui_app()?;
    }
    let mut args = vec!["run"];
    if panic {
        args.push("-F");
        args.push("panic_on_tx_overflow");
        args.push("-F");
        args.push("panic_on_thread_error");
    }
    if release {
        args.push("--release");
    }

    cmd("cargo", args)
        .dir("retro-clock-esp")
        .run()
        .context("Failed to run ESP app")?;

    Ok(())
}

fn run_mix(skip_flashing: bool) -> anyhow::Result<()> {
    let local_config_port = std::env::var("RCLOCK_LOCAL_CONFIG_PORT")
        .unwrap_or_else(|_| RCLOCK_LOCAL_CONFIG_PORT_DEFAULT.to_string());

    cmd!(
        "xdg-open",
        format!("http://127.0.0.1:{}", local_config_port)
    )
    .run()
    .context("Failed to open browser")?;

    let run_kdl = if skip_flashing {
        RUN_KDL_MIX_NO_FLASH
    } else {
        RUN_KDL_MIX_FULL
    };

    std::fs::write(".run.kdl", run_kdl)?;
    cmd!("zellij", "-l", ".run.kdl")
        .env(
            "UI_ORIGIN",
            format!("http://127.0.0.1:{}", local_config_port),
        )
        .env("API_URL", RCLOCK_ESP_SERVER_URL)
        .run()
        .context("Failed to start Zellij")?;
    Ok(())
}

fn release_webui_app() -> anyhow::Result<()> {
    let webui_dir = "retro-clock-webui";
    let webui_dist_dir = format!("{webui_dir}/dist");
    let esp_webui_dist_dir = "retro-clock-esp/.webui_dist";

    // build web UI app in release mode
    cmd!("trunk", "build", "--release", "--minify", "true")
        .dir(webui_dir)
        .run()
        .context("Failed to build webui app")?;

    // copy webui app
    cmd!("rm", "-fr", esp_webui_dist_dir)
        .run()
        .context("Failed to remove old ESP webui dist")?;
    cmd!("cp", "-r", webui_dist_dir, esp_webui_dist_dir)
        .run()
        .context("Failed to copy webui app")?;
    cmd!(
        "cp",
        "-r",
        "retro-clock-esp/certs/rootCA.pem",
        format!("{esp_webui_dist_dir}/rootCA.pem")
    )
    .run()
    .context("Failed to copy config app")?;

    {
        let _dir_guard =
            tmp_env::set_current_dir(esp_webui_dist_dir).context("Failed to set current dir")?;

        // remove preload/modulepreload links
        cmd!(
            "sd",
            "<link[^>]*rel=(modulepreload|preload)[^>]*>",
            "",
            "index.html"
        )
        .run()
        .context("Failed to remove preload links")?;

        // create version file
        let version = cmd!("git", "rev-parse", "HEAD")
            .read()
            .context("Failed to read git version")?;
        std::fs::write("version.txt", version).context("Failed to write version file")?;

        // compress assets
        cmd!("bash", "-c", "brotli --rm *.{wasm,js,html}")
            .run()
            .context("Failed to compress assets")?;

        // print summary
        cmd!("exa", "-lars", "size")
            .run()
            .context("Failed to print summary")?;
    }

    Ok(())
}

fn new_cert() -> anyhow::Result<()> {
    let certs_dir = "retro-clock-esp/certs";

    cmd!("rm", "-fr", certs_dir)
        .run()
        .context("Failed to remove old certs dir")?;
    std::fs::create_dir_all(certs_dir).context("Failed to create cert dir")?;

    // This generates self-signed cert, this is fine if browser accepts such, but better way is to
    // generate cert using mkcert and then add CA used by mkcert to the list of CA in the browser.
    // openssl req -x509 -newkey rsa:2048 -keyout server.key -out server.crt -days 365 -nodes

    // generating it for an IP address would also work, but it is better to generate for a domain
    // and map IP to the domain in /etc/hosts (or use mDNS)
    {
        let _dir_guard =
            tmp_env::set_current_dir(certs_dir).context("Failed to set current dir")?;

        let mkcert_version = cmd!("mkcert", "-version")
            .read()
            .context("Failed to check mkcert version")?
            .trim()
            .to_owned();
        log::info!("Current mkcert version: {}", mkcert_version);
        if !mkcert_version.ends_with("+fork-gk") {
            log::warn!("We recommend using the forked version of mkcert that supports `-days`: https://github.com/gergelyk/mkcert");
            log::warn!("With current mkcert version the generated cert will be valid for only 825 days");
            cmd!("mkcert", "retro-clock.local")
        } else {
            cmd!("mkcert", "-days", "36524", "retro-clock.local")
        }.run().context("Failed to generate local certificate")?;

        cmd!(
            "openssl",
            "x509",
            "-in",
            "retro-clock.local.pem",
            "-out",
            "retro-clock.local.crt"
        )
        .run()
        .context("Failed to create x509 file")?;

        cmd!(
            "openssl",
            "pkey",
            "-in",
            "retro-clock.local-key.pem",
            "-out",
            "retro-clock.local.key"
        )
        .run()
        .context("Failed to create pkey file")?;
    }

    let cacert_dir = cmd!("mkcert", "-CAROOT")
        .read()
        .context("Failed to get mkcert CA root")?;

    std::fs::copy(
        format!("{}/rootCA.pem", cacert_dir),
        "retro-clock-esp/certs/rootCA.pem",
    )
    .context("Failed to copy CA cert")?;

    log::info!("This should add CA to the browsers (but doesn't work well):");
    log::info!("  mkcert -install");
    log::info!("");
    log::info!("Alternatively, in the browser settings you can manually add CA cert from:");
    log::info!("  retro-clock-esp/certs/rootCA.pem");

    Ok(())
}

fn new_env() -> anyhow::Result<()> {
    if Path::new(".env").exists() {
        anyhow::bail!(
            ".env file already exists. Consider making a backup, remove the file and run this command again."
        );
    }

    let content = RefCell::new(String::new());

    let var = |key: &str, value: &str| {
        content
            .borrow_mut()
            .push_str(&format!("{}={}\n", key, value));
    };

    let cvar = |key: &str, value: &str| {
        content
            .borrow_mut()
            .push_str(&format!("#{}={}\n", key, value));
    };

    let com = |comment: &str| {
        content.borrow_mut().push_str(&format!("# {}\n", comment));
    };

    let sep = || {
        content.borrow_mut().push('\n');
    };

    com("This file is a configuration for entire project. It was generated by dev.rs.");
    com("Feel free to edit it. Env variables shadow those provided here.");

    sep();
    com("If set to 1, time on the display will include milliseconds. Use for debugging.");
    var("DISPLAY_MILLISECONDS", "1");

    sep();
    com("Used for signing authentication tokens that are stored as HTTP cookies.");
    let jwt_secret_b64 = cmd!("sh", "-c", "head -c 32 /dev/urandom | base64")
        .read()
        .context("Failed to generate JWT secret")?;
    var("JWT_SECRET_BASE64", &jwt_secret_b64);

    sep();
    com("Used for hashing admin password that is stored in the device. Note that");
    com("changing this will result in the admin password being lost.");
    let hmac_salt_b64 = cmd!("sh", "-c", "head -c 32 /dev/urandom | base64")
        .read()
        .context("Failed to generate HMAC salt")?;
    var("HMAC_SALT_BASE64", &hmac_salt_b64);

    sep();
    com("Password to the config UI that will be used when reseting storage. Use it only");
    com("for debugging purposes. If not provided, device will generate a random password");
    com("that can be later changed in AP mode. Simulator uses '123' by default.");
    cvar("ADMIN_PASS", "123");

    sep();
    com("Credentials that will be used when reseting storage. Use it only for debugging");
    com("purposes. If not provided, empty values will be used. Empty SSID will make device");
    com("start in AP mode.");
    cvar("WIFI_ST_SSID", "mynetwork");
    cvar("WIFI_ST_PASS", "mypassword");

    sep();
    com("Provide it only for debugging purposes. If not provided,");
    com("random password will be generated by device.");
    cvar("WIFI_AP_PASS", "0123456789");

    sep();
    com("Time of inactivity before going back to the home screen.");
    cvar("HOME_SCREEN_TIMEOUT_SECONDS", "60");

    sep();
    com("Port at which trunk can serve retro-clock-webui when run on local host.");
    var("RCLOCK_LOCAL_CONFIG_PORT", "8880");

    sep();
    com("Port at which retro-clock-sim starts HTTP server.");
    var("RCLOCK_SIM_SERVER_PORT", "3330");

    sep();
    com("Tools needed for compiling C++ based crates for ESP32 target.");
    let bin_dir = pathexpand::expand(
        "~/.rustup/toolchains/esp/xtensa-esp-elf/esp-13.2.0_20230928/xtensa-esp-elf/bin/",
    )?;
    if !bin_dir.exists() {
        log::warn!(
            "Expected toolchain bin dir does not exist: {}",
            bin_dir.to_string_lossy()
        );
    }
    var(
        "CC_xtensa_esp32s3_espidf",
        &bin_dir.join("xtensa-esp32s3-elf-gcc").to_string_lossy(),
    );
    var(
        "CXX_xtensa_esp32s3_espidf",
        &bin_dir.join("xtensa-esp32s3-elf-g++").to_string_lossy(),
    );
    var(
        "AR_xtensa_esp32s3_espidf",
        &bin_dir.join("xtensa-esp32s3-elf-ar").to_string_lossy(),
    );

    sep();
    com("Below is the public token. Feel free to replace it with your own.");
    com("https://weather.krason.dev/api/v1 can be used too, but let's be prepared for");
    com("potential problems with DNS.");
    var(
        "WEATHER_API_URL",
        "https://weather.fermyon.app/api/v1?token=nnolm662M8cmjaIj",
    );
    var("FORECAST_API_URL", "https://api.open-meteo.com/v1");
    var("LOCATION_API_URL", "http://ip-api.com/json");

    std::fs::write(".env", content.borrow().as_bytes())?;
    log::info!("New .env file created");
    Ok(())
}
