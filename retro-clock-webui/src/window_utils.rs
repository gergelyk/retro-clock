use anyhow::Context;
use leptos::logging;
use web_sys::window;

fn get_window() -> anyhow::Result<web_sys::Window> {
    web_sys::window().context("window does not exist")
}

fn get_document() -> anyhow::Result<web_sys::Document> {
    get_window()?
        .document()
        .context("document does not exist on window")
}

pub fn refresh_page() {
    if let Some(win) = window() {
        win.location().reload().unwrap_or_else(|e| {
            logging::error!("Failed to reload page: {:?}", e);
        });
    }
}

fn set_wait_cursor() -> anyhow::Result<()> {
    let body = get_document()?.body().context("document has no body")?;
    body.class_list()
        .add_1("waiting")
        .map_err(|e| anyhow::anyhow!("Failed to set 'waiting' class: {:?}", e))?;
    Ok(())
}

fn set_default_cursor() -> anyhow::Result<()> {
    let body = get_document()?.body().context("document has no body")?;
    body.class_list()
        .remove_1("waiting")
        .map_err(|e| anyhow::anyhow!("Failed to remove 'waiting' class: {:?}", e))?;
    Ok(())
}

pub struct WaitCursorGuard {}

impl WaitCursorGuard {
    pub fn new() -> Self {
        set_wait_cursor().unwrap_or_else(|e| {
            logging::error!("Failed to set wait cursor: {:?}", e);
        });
        Self {}
    }
}

impl Drop for WaitCursorGuard {
    fn drop(&mut self) {
        set_default_cursor().unwrap_or_else(|e| {
            logging::error!("Failed to unset wait cursor: {:?}", e);
        });
    }
}
