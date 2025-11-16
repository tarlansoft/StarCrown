use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};

pub struct Database {
	conn: Connection,
}

impl Database {
	pub fn new(path: &str) -> Result<Self> {
		let conn = Connection::open(path)?;
		Ok(Self { conn })
	}

	pub fn init_schema(&self) -> Result<()> {
		self.conn.execute_batch(
			r#"
			PRAGMA journal_mode = WAL;
			CREATE TABLE IF NOT EXISTS tags (
				id INTEGER PRIMARY KEY AUTOINCREMENT,
				name TEXT NOT NULL UNIQUE
			);
			CREATE TABLE IF NOT EXISTS messages (
				id INTEGER PRIMARY KEY AUTOINCREMENT,
				tag_id INTEGER NOT NULL,
				content TEXT NOT NULL,
				FOREIGN KEY(tag_id) REFERENCES tags(id) ON DELETE CASCADE
			);
		"#,
		)?;
		Ok(())
	}

	pub fn seed_demo(&self) -> Result<()> {
		let mut stmt = self.conn.prepare("SELECT id FROM tags WHERE name = ?1")?;
		let exists: Option<i64> = stmt.query_row(params!["hello"], |r| r.get(0)).optional()?;
		if exists.is_none() {
			self.conn.execute("INSERT INTO tags(name) VALUES (?1)", params!["hello"])?;
			let tag_id: i64 = self
				.conn
				.query_row("SELECT id FROM tags WHERE name=?1", params!["hello"], |r| r.get(0))?;
			self.conn.execute(
				"INSERT INTO messages(tag_id, content) VALUES (?1, ?2)",
				params![tag_id, "Hola!"],
			)?;
			self.conn.execute(
				"INSERT INTO messages(tag_id, content) VALUES (?1, ?2)",
				params![tag_id, "Buen día!"],
			)?;
			self.conn.execute(
				"INSERT INTO messages(tag_id, content) VALUES (?1, ?2)",
				params![tag_id, "¿Cómo estás?"],
			)?;
		}
		Ok(())
	}

	pub fn list_tags(&self) -> Result<Vec<String>> {
		let mut stmt = self.conn.prepare("SELECT name FROM tags ORDER BY name ASC")?;
		let rows = stmt
			.query_map([], |r| r.get::<_, String>(0))?
			.collect::<rusqlite::Result<Vec<_>>>()?;
		Ok(rows)
	}

	pub fn get_messages_for_tag(&self, tag: &str) -> Result<Vec<(i64, String)>> {
		let mut stmt = self.conn.prepare(
			"SELECT m.id, m.content
			 FROM messages m
			 JOIN tags t ON t.id = m.tag_id
			 WHERE t.name = ?1
			 ORDER BY m.id ASC",
		)?;
		let rows = stmt
			.query_map(params![tag], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?
			.collect::<rusqlite::Result<Vec<_>>>()?;
		Ok(rows)
	}

	pub fn get_message_by_id(&self, id: i64) -> Result<String> {
		let msg: String = self
			.conn
			.query_row("SELECT content FROM messages WHERE id=?1", params![id], |r| r.get(0))?;
		Ok(msg)
	}

	pub fn find_tag_and_messages(&self, tag: &str) -> Result<Option<(i64, Vec<String>)>> {
		let tag_id: Option<i64> = self
			.conn
			.query_row(
				"SELECT id FROM tags WHERE name=?1",
				params![tag],
				|r| r.get::<_, i64>(0),
			)
			.optional()?;
		if let Some(tid) = tag_id {
			let mut stmt = self
				.conn
				.prepare("SELECT content FROM messages WHERE tag_id=?1 ORDER BY id ASC")?;
			let msgs = stmt
				.query_map(params![tid], |r| r.get::<_, String>(0))?
				.collect::<rusqlite::Result<Vec<_>>>()?;
			return Ok(Some((tid, msgs)));
		}
		Ok(None)
	}
}


