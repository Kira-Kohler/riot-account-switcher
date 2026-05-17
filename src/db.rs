use anyhow::Result;
use rusqlite::{params, Connection};
use std::path::PathBuf;

pub struct Account {
    pub id: i64,
    pub name: String,
    pub saved_at: String,
    pub token_data: String,
    pub riot_id: Option<String>,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open() -> Result<Self> {
        let conn = Connection::open(db_path())?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS accounts (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                name       TEXT    NOT NULL UNIQUE,
                token_data TEXT    NOT NULL,
                riot_id    TEXT,
                saved_at   TEXT    NOT NULL DEFAULT (datetime('now','localtime'))
            );",
        )?;
        let _ = conn.execute_batch("ALTER TABLE accounts ADD COLUMN riot_id TEXT");
        Ok(Self { conn })
    }

    pub fn list(&self) -> Result<Vec<Account>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, saved_at, token_data, riot_id FROM accounts ORDER BY name COLLATE NOCASE",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(Account {
                id: r.get(0)?,
                name: r.get(1)?,
                saved_at: r.get(2)?,
                token_data: r.get(3)?,
                riot_id: r.get(4)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn upsert(&self, name: &str, token_data: &str, riot_id: Option<&str>) -> Result<()> {
        self.conn.execute(
            "INSERT INTO accounts (name, token_data, riot_id) VALUES (?1, ?2, ?3)
             ON CONFLICT(name) DO UPDATE
             SET token_data = excluded.token_data,
                 riot_id    = COALESCE(excluded.riot_id, accounts.riot_id),
                 saved_at   = datetime('now','localtime')",
            params![name, token_data, riot_id],
        )?;
        Ok(())
    }

    pub fn rename(&self, id: i64, new_name: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE accounts SET name = ?1 WHERE id = ?2",
            params![new_name, id],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM accounts WHERE id = ?1", params![id])?;
        Ok(())
    }
}

pub fn db_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("accounts.db")))
        .unwrap_or_else(|| PathBuf::from("accounts.db"))
}
