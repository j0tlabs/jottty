use serde::Deserialize;
use std::{
    env,
    fs::{self, File},
    io::{self, Read, Write},
    path::PathBuf,
};

#[derive(Debug, Deserialize)]
struct ConfigFile {
    bullet: Option<String>,
    editor: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub bullet: String,
    pub editor: String,
}

fn default_dir() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(format!("{}/.jottty", home))
}

fn default_bullet() -> String {
    "-".to_string()
}

fn default_editor() -> String {
    env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string())
}

fn config_file_path() -> PathBuf {
    if let Ok(path) = env::var("JOTTY_CONFIG") {
        return PathBuf::from(format!("{}/config.toml", path));
    }
    let mut base = default_dir();
    base.push("config.toml");
    base
}

impl Config {
    pub fn load() -> io::Result<Self> {
        let config_path = config_file_path();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut contents = String::new();
        if config_path.exists() {
            let mut file = File::open(&config_path)?;
            file.read_to_string(&mut contents)?;
        } else {
            let default = format!(
                "bullet = \"{}\"\\neditor = \"{}\"\\n",
                default_bullet(),
                default_editor()
            );
            let mut file = File::create(&config_path)?;
            file.write_all(default.as_bytes())?;
            contents = default;
        }

        let parsed: ConfigFile = toml::from_str(&contents).unwrap_or(ConfigFile {
            bullet: None,
            editor: None,
        });

        Ok(Config {
            bullet: parsed.bullet.unwrap_or_else(default_bullet),
            editor: parsed.editor.unwrap_or_else(default_editor),
        })
    }
}

