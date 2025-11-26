use crc32fast::Hasher;
use esp_idf_hal::io::Write;
use esp_idf_svc::http::server as esp_http_server;
use include_dir::{include_dir, Dir};
use retro_clock_core::channel_utils::SyncSenderBlocking;

static FILES: Dir = include_dir!("$CARGO_MANIFEST_DIR/.webui_dist");

fn get_file(name: &str) -> Option<&'static [u8]> {
    FILES.get_file(name).map(|f| f.contents())
}

pub fn get_file_handler(
    req: esp_http_server::Request<&mut esp_http_server::EspHttpConnection<'_>>,
    message_tx: &SyncSenderBlocking<String>,
) -> Result<(), anyhow::Error> {
    let path = req.uri();
    log::info!("Requested static file: {}", path);
    let path = if path == "/" {
        "index.html"
    } else {
        &path[1..]
    };
    let mut path = path.to_string();
    let mut compress = false;
    let content_type = if path.ends_with(".html") {
        compress = true;
        "text/html"
    } else if path.ends_with(".js") {
        compress = true;
        "application/javascript"
    } else if path.ends_with(".css") {
        compress = true;
        "text/css"
    } else if path.ends_with(".wasm") {
        compress = true;
        "application/wasm"
    } else if path.ends_with(".png") {
        compress = true;
        "image/png"
    } else if path.ends_with(".svg") {
        compress = true;
        "image/svg+xml"
    } else if path.ends_with(".txt") {
        "text/plain"
    } else {
        "application/octet-stream"
    };

    let mut content_encoding = "identity";
    if compress {
        content_encoding = "br";
        path = format!("{}.br", path);
    }

    if let Some(file) = get_file(path.as_str()) {
        if path.ends_with(".pem") {
            let mut hasher = Hasher::new();
            hasher.update(file);
            let crc32 = hasher.finalize();
            message_tx.send(format!("crc32 {}\n    {:08X}", path, crc32))?;
        }

        let status = http::StatusCode::OK;
        let mut resp = req.into_response(
            status.into(),
            None,
            &[
                ("Content-Type", content_type),
                ("Content-Encoding", content_encoding),
            ],
        )?;
        resp.write_all(file)?;
    } else {
        let status = http::StatusCode::NOT_FOUND;
        let mut resp = req.into_response(status.into(), None, &[])?;
        resp.write_all(status.to_string().as_bytes())?;
    }
    Ok(())
}
