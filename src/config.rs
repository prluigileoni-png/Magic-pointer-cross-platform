use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub provider: Provider,
    pub model: String,
    pub api_key: String,
    pub base_url: String,
    pub hotkey: String,
    pub sensitivity: u8,
    pub fullscreen: bool,
    pub ocr: bool,
    pub timeout_seconds: u64,
    pub language: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Gemini,
    Openai,
    Anthropic,
    Nvidia,
}

impl Default for Provider {
    fn default() -> Self {
        Self::Gemini
    }
}

impl Provider {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gemini => "gemini",
            Self::Openai => "openai",
            Self::Anthropic => "anthropic",
            Self::Nvidia => "nvidia",
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            provider: Provider::Gemini,
            model: "gemini-2.5-flash".into(),
            api_key: String::new(),
            base_url: String::new(),
            hotkey: "Ctrl+Shift+G".into(),
            sensitivity: 5,
            fullscreen: true,
            ocr: true,
            timeout_seconds: 30,
            language: String::new(),
        }
    }
}

impl Config {
    pub fn path() -> Result<PathBuf> {
        let dirs = ProjectDirs::from("com", "MagicPointer", "MagicPointer")
            .context("cannot determine platform configuration directory")?;
        Ok(dirs.config_dir().join("config.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        if !path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }
        let text =
            fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        serde_json::from_str(&text).context("invalid Magic Pointer configuration")
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}
