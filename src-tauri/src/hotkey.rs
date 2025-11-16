use anyhow::Result;
use tauri::AppHandle;

pub fn register_global_hotkeys(app: AppHandle) -> Result<()> {
	// Перенесено на фронтенд через @tauri-apps/api/globalShortcut
	let _ = app;
	Ok(())
}


