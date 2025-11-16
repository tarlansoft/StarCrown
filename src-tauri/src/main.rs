mod database;
mod hotkey;
mod popup_manager;
mod text_expander;

use crate::database::Database;
use crate::popup_manager::PopupManager;
use crate::text_expander::TextExpander;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::Serialize;
use tauri::{AppHandle, Manager};

static POPUP_MANAGER: Lazy<Mutex<Option<PopupManager>>> = Lazy::new(|| Mutex::new(None));
static TEXT_EXPANDER: Lazy<Mutex<Option<TextExpander>>> = Lazy::new(|| Mutex::new(None));
static DB: Lazy<Mutex<Option<Database>>> = Lazy::new(|| Mutex::new(None));

#[derive(Serialize)]
struct ShowPopupPayload {
	query: String,
}

#[tauri::command]
fn cmd_list_tags() -> Result<Vec<String>, String> {
	let db = DB.lock();
	let db = db.as_ref().ok_or("DB not initialized")?;
	db.list_tags().map_err(|e| e.to_string())
}

#[tauri::command]
fn cmd_get_messages_for_tag(tag: String) -> Result<Vec<(i64, String)>, String> {
	let db = DB.lock();
	let db = db.as_ref().ok_or("DB not initialized")?;
	db.get_messages_for_tag(&tag).map_err(|e| e.to_string())
}

#[tauri::command]
fn cmd_expand_with_message(message_id: i64, app: AppHandle) -> Result<(), String> {
	let db = DB.lock();
	let db = db.as_ref().ok_or("DB not initialized")?;
	let msg = db.get_message_by_id(message_id).map_err(|e| e.to_string())?;
	let expander_guard = TEXT_EXPANDER.lock();
	let expander = expander_guard.as_ref().ok_or("Expander not initialized")?;
	expander.expand_replace_current_tag(&msg).map_err(|e| e.to_string())?;
	// Close popup if open
	let mut popup_guard = POPUP_MANAGER.lock();
	if let Some(pm) = popup_guard.as_mut() {
		let _ = pm.hide(&app);
	}
	Ok(())
}

#[tauri::command]
fn cmd_show_popup(query: Option<String>, app: AppHandle) -> Result<(), String> {
	let payload = ShowPopupPayload {
		query: query.unwrap_or_default(),
	};
	app.emit_all("show_popup", payload).map_err(|e| e.to_string())
}

fn main() {
	env_logger::init();
	tauri::Builder::default()
		.invoke_handler(tauri::generate_handler![
			cmd_list_tags,
			cmd_get_messages_for_tag,
			cmd_expand_with_message,
			cmd_show_popup
		])
		.setup(|app| {
			{
				let db = Database::new("expander.db")?;
				db.init_schema()?;
				db.seed_demo()?;
				*DB.lock() = Some(db);
			}
			{
				let mut pm = PopupManager::new();
				pm.init(app)?;
				*POPUP_MANAGER.lock() = Some(pm);
			}
			{
				let app_handle = app.handle();
				let expander = TextExpander::new(app_handle.clone());
				expander.start_listener();
				*TEXT_EXPANDER.lock() = Some(expander);
			}
			Ok(())
		})
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}


