use crate::database::Database;
use crate::POPUP_MANAGER;
use crate::DB;
use anyhow::{anyhow, Result};
use enigo::{Enigo, KeyboardControllable};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use rdev::{listen, Event, EventType, Key};
use std::thread;
use std::time::{Duration, Instant};
use tauri::AppHandle;

static INPUT_BUFFER: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::with_capacity(128)));

pub struct TextExpander {
	app: AppHandle,
}

impl TextExpander {
	pub fn new(app: AppHandle) -> Self {
		Self { app }
	}

	pub fn start_listener(&self) {
		let app = self.app.clone();
		thread::spawn(move || {
			let _ = listen(move |event| {
				if let Err(err) = handle_event(&app, event) {
					log::debug!("expander listener error: {err:?}");
				}
			});
		});
	}

	fn delete_typed(len: usize) -> Result<()> {
		let mut enigo = Enigo::new();
		for _ in 0..len {
			enigo.key_down(enigo::Key::Backspace);
			enigo.key_up(enigo::Key::Backspace);
			thread::sleep(Duration::from_millis(2));
		}
		Ok(())
	}

	pub fn expand_replace_current_tag(&self, message: &str) -> Result<()> {
		// figure out the last parsed trigger to know how many chars to delete
		let trigger = {
			let mut buf = INPUT_BUFFER.lock();
			let trig = extract_last_trigger(&buf).unwrap_or_default();
			// clear buffer after expansion to avoid re-trigger
			*buf = String::new();
			trig
		};
		if !trigger.is_empty() {
			Self::delete_typed(trigger.len())?;
		}
		// Вводим текст напрямую через Enigo
		let mut enigo = Enigo::new();
		enigo.text(message);
		Ok(())
	}
}

fn is_word_char(k: Key) -> bool {
	matches!(k,
		Key::KeyA
		| Key::KeyB
		| Key::KeyC
		| Key::KeyD
		| Key::KeyE
		| Key::KeyF
		| Key::KeyG
		| Key::KeyH
		| Key::KeyI
		| Key::KeyJ
		| Key::KeyK
		| Key::KeyL
		| Key::KeyM
		| Key::KeyN
		| Key::KeyO
		| Key::KeyP
		| Key::KeyQ
		| Key::KeyR
		| Key::KeyS
		| Key::KeyT
		| Key::KeyU
		| Key::KeyV
		| Key::KeyW
		| Key::KeyX
		| Key::KeyY
		| Key::KeyZ
		| Key::Num1
		| Key::Num2
		| Key::Num3
		| Key::Num4
		| Key::Num5
		| Key::Num6
		| Key::Num7
		| Key::Num8
		| Key::Num9
		| Key::Num0
		| Key::Slash
		| Key::Space
	)
}

fn handle_event(app: &AppHandle, event: Event) -> Result<()> {
	static LAST_INPUT: Lazy<Mutex<Instant>> = Lazy::new(|| Mutex::new(Instant::now()));

	match event.event_type {
		EventType::KeyPress(key) => {
			{
				let mut last = LAST_INPUT.lock();
				*last = Instant::now();
			}
			if !is_word_char(key) {
				return Ok(());
			}
			let ch = match key {
				Key::Space => Some(' '),
				Key::Slash => Some('/'),
				Key::Num0 => Some('0'),
				Key::Num1 => Some('1'),
				Key::Num2 => Some('2'),
				Key::Num3 => Some('3'),
				Key::Num4 => Some('4'),
				Key::Num5 => Some('5'),
				Key::Num6 => Some('6'),
				Key::Num7 => Some('7'),
				Key::Num8 => Some('8'),
				Key::Num9 => Some('9'),
				Key::KeyA => Some('a'),
				Key::KeyB => Some('b'),
				Key::KeyC => Some('c'),
				Key::KeyD => Some('d'),
				Key::KeyE => Some('e'),
				Key::KeyF => Some('f'),
				Key::KeyG => Some('g'),
				Key::KeyH => Some('h'),
				Key::KeyI => Some('i'),
				Key::KeyJ => Some('j'),
				Key::KeyK => Some('k'),
				Key::KeyL => Some('l'),
				Key::KeyM => Some('m'),
				Key::KeyN => Some('n'),
				Key::KeyO => Some('o'),
				Key::KeyP => Some('p'),
				Key::KeyQ => Some('q'),
				Key::KeyR => Some('r'),
				Key::KeyS => Some('s'),
				Key::KeyT => Some('t'),
				Key::KeyU => Some('u'),
				Key::KeyV => Some('v'),
				Key::KeyW => Some('w'),
				Key::KeyX => Some('x'),
				Key::KeyY => Some('y'),
				Key::KeyZ => Some('z'),
				_ => None,
			};
			if let Some(c) = ch {
				let mut buf = INPUT_BUFFER.lock();
				buf.push(c);
				if buf.len() > 256 {
					buf.drain(..buf.len() - 256);
				}
				drop(buf);

				if let Some((trigger_len, tag)) = detect_trigger() {
					// Query DB
					let (count, first_msg) = {
						let db_guard = DB.lock();
						let db = db_guard.as_ref().ok_or(anyhow!("DB not ready"))?;
						if let Some((_tid, msgs)) = db.find_tag_and_messages(&tag)? {
							let cnt = msgs.len();
							let first = msgs.get(0).cloned().unwrap_or_default();
							(cnt, first)
						} else {
							(0, String::new())
						}
					};
					if count == 1 {
						// Replace directly
						{
							let mut buf = INPUT_BUFFER.lock();
							// store only the trigger to let expander delete it
							*buf = format!("/{tag} ");
						}
						let expander = crate::TEXT_EXPANDER.lock();
						if let Some(exp) = expander.as_ref() {
							let _ = exp.expand_replace_current_tag(&first_msg);
						}
					} else if count > 1 {
						// open popup with query = tag
						let mut pm = POPUP_MANAGER.lock();
						if let Some(p) = pm.as_mut() {
							let _ = p.show_with_query(app, &tag);
						} else {
							let _ = app.emit_all(
								"show_popup",
								serde_json::json!({ "query": tag }),
							);
						}
					}
				}
			}
		}
		EventType::KeyRelease(key) => {
			// handle backspace to keep buffer realistic
			if matches!(key, Key::Backspace) {
				let mut buf = INPUT_BUFFER.lock();
				let _ = buf.pop();
			}
		}
		_ => {}
	}
	Ok(())
}

fn detect_trigger() -> Option<(usize, String)> {
	let buf = INPUT_BUFFER.lock();
	extract_last_trigger(&buf).map(|t| {
		let tag = t.trim().trim_start_matches('/').trim_end().to_string();
		(t.len(), tag)
	})
}

fn extract_last_trigger(buffer: &str) -> Option<String> {
	// Looks for "/tag " pattern at the end
	let s = buffer.trim_end_matches(|c| c == '\n' || c == '\r');
	if !s.ends_with(' ') {
		return None;
	}
	let mut idx = s.len();
	// go back to previous whitespace or line start
	while idx > 0 {
		let ch = s[..idx].chars().last().unwrap();
		if ch.is_whitespace() {
			break;
		}
		idx = s[..idx].chars().rev().skip(1).count();
	}
	let candidate = &s[idx..];
	if candidate.starts_with('/') && candidate.len() > 2 {
		Some(candidate.to_string())
	} else {
		None
	}
}


