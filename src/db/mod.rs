mod datom;
mod transit;

use std::str::FromStr;
use std::{env, fs, path::PathBuf};

use serde::Deserialize;
use sqlx::{ConnectOptions, Connection, SqliteConnection, sqlite::SqliteConnectOptions};

const DB_FILENAME: &str = "db.sqlite";
#[derive(Debug, Deserialize)]
struct DbConfigFile {
    db_path: Option<String>,
    db_dir: Option<String>,
}

fn default_db_dir() -> PathBuf {
    let home = env::var("HOME").unwrap_or(".".to_string());
    PathBuf::from(format!("{}/.jottty", home))
}

fn config_file_path() -> PathBuf {
    if let Ok(path) = env::var("JOTTTY_CONFIG") {
        return PathBuf::from(format!("{}/config.toml", path));
    }
    let home = env::var("HOME").unwrap_or(".".to_string());
    PathBuf::from(format!("{}/.jottty/config.toml", home))
}

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        return PathBuf::from(format!("{}/{}", home, rest));
    }
    PathBuf::from(path)
}

fn db_path_from_config(cfg: DbConfigFile) -> Option<PathBuf> {
    cfg.db_path.map(|p| expand_tilde(&p)).or_else(|| {
        cfg.db_dir.map(|dir| {
            let mut p = expand_tilde(&dir);
            p.push("db.sqlite");
            p
        })
    })
}
fn default_db_path() -> PathBuf {
    let mut dir = default_db_dir();
    dir.push(DB_FILENAME);
    dir
}
fn resolve_db_path() -> PathBuf {
    if let Ok(db_path) = env::var("JOTTTY_DB_PATH") {
        return PathBuf::from(db_path);
    }

    let config_path = config_file_path();
    fs::read_to_string(config_path)
        .ok()
        .and_then(|c| toml::from_str::<DbConfigFile>(&c).ok())
        .and_then(db_path_from_config)
        .unwrap_or_else(default_db_path)
}

fn db_url() -> String {
    let path = resolve_db_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    format!("sqlite://{}", path.to_string_lossy())
}

/// Connect to SQLite database.
///
/// # Errors
/// Returns an error if the connection fails.
async fn conn() -> sqlx::SqliteConnection {
    let url = db_url();
    SqliteConnectOptions::from_str(&url)
        .expect("Failed to create connection string")
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .locking_mode(sqlx::sqlite::SqliteLockingMode::Exclusive)
        .connect()
        .await
        .expect("Failed to connect to database")
}

async fn create_vaults_table(conn: &mut SqliteConnection) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS vaults (
            addr INTEGER PRIMARY KEY,
            content TEXT,
            addresses JSON
        );",
    )
    .execute(conn)
    .await
    .expect("Failed to create table");
}

async fn ensure_schema(conn: &mut SqliteConnection) {
    create_vaults_table(conn).await;
}

/// Initialize the database.
///
/// This function gets an connection ensure the database schema is set up.
/// This function is called on cli startup.
pub async fn init_db() {
    let mut conn = conn().await;
    ensure_schema(&mut conn).await;
    let _ = conn.close().await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Row;


    //TODO@chico: reuse this function
    fn unique_test_db_path(test_name: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let pid = std::process::id();
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("jottty_{}_{}_{}.db", test_name, pid, ts));
        path
    }

    #[tokio::test]
    async fn init_db_creates_db_file() {
        let db_path = unique_test_db_path("apply_datoms");
        unsafe {
            std::env::set_var("JOTTTY_DB_PATH", db_path.to_string_lossy().to_string());
        }
        init_db().await;

        assert!(db_path.exists());
    }

    #[tokio::test]
    async fn db_creates_schema() {
        let db = "sqlite::memory:";
        let mut conn = SqliteConnectOptions::from_str(db)
            .unwrap()
            .connect()
            .await
            .unwrap();

        ensure_schema(&mut conn).await;

        let row =
            sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='vaults';")
                .fetch_optional(&mut conn)
                .await
                .unwrap();

        for r in row.iter() {
            assert_eq!("vaults", r.get::<String, _>("name"));
        }

        assert!(row.is_some());
    }
}
