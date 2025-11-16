use anyhow::Result;
use tauri::{App, AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

pub struct PopupManager {
	initialized: bool,
}

impl PopupManager {
	pub fn new() -> Self {
		Self { initialized: false }
	}

	pub fn init(&mut self, app: &mut App) -> Result<()> {
		self.ensure_popup_window(app.handle())?;
		self.initialized = true;
		Ok(())
	}

	fn ensure_popup_window(&self, app: &AppHandle) -> Result<WebviewWindow> {
		if let Some(win) = app.get_webview_window("popup") {
			return Ok(win);
		}
		let url = WebviewUrl::App("index.html".into());
		let win = WebviewWindowBuilder::new(app, "popup", url)
			.title("Expander Popup")
			.decorations(true)
			.resizable(false)
			.always_on_top(true)
			.visible(false)
			.inner_size(560.0, 380.0)
			.build()?;
		Ok(win)
	}

	pub fn show_with_query(&mut self, app: &AppHandle, query: &str) -> Result<()> {
		let win = self.ensure_popup_window(app)?;
		win.show()?;
		win.set_focus()?;
		app.emit_all(
			"show_popup",
			serde_json::json!({ "query": query.to_string() }),
		)?;
		Ok(())
	}

	pub fn hide(&mut self, app: &AppHandle) -> Result<()> {
		let win = self.ensure_popup_window(app)?;
		win.hide()?;
		Ok(())
	}
}


