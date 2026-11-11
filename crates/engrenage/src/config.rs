use std::{fs, net::SocketAddr, path::Path, path::PathBuf};

use anyhow::{Context, bail};
use serde::Deserialize;

/// Manager configuration, loaded from `engrenage.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Address the dashboard listens on.
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,
    /// Root directory for manager state (SQLite db, version store, ...).
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_addr: default_listen_addr(),
            data_dir: default_data_dir(),
        }
    }
}

fn default_listen_addr() -> String {
    "127.0.0.1:8080".to_owned()
}

fn default_data_dir() -> PathBuf {
    PathBuf::from("./data")
}

impl Config {
    /// Load from an explicit path, or from `./engrenage.toml` when it exists,
    /// or fall back to defaults.
    pub fn load(path: Option<&Path>) -> anyhow::Result<Self> {
        match path {
            Some(path) => {
                let raw = fs::read_to_string(path)
                    .with_context(|| format!("reading config file {}", path.display()))?;
                toml::from_str(&raw).with_context(|| format!("parsing {}", path.display()))
            }
            None => {
                let default_path = Path::new("engrenage.toml");
                if default_path.exists() {
                    Self::load(Some(default_path))
                } else {
                    Ok(Self::default())
                }
            }
        }
    }

    pub fn override_listen_addr(&mut self, listen_addr: String) {
        self.listen_addr = listen_addr;
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        self.listen_addr
            .parse::<SocketAddr>()
            .map_err(|e| anyhow::anyhow!("invalid listen_addr '{}': {e}", self.listen_addr))?;
        if self.data_dir.as_os_str().is_empty() {
            bail!("data_dir must not be empty");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let config = Config::default();
        config.validate().expect("defaults must be valid");
        assert_eq!(config.listen_addr, "127.0.0.1:8080");
        assert_eq!(config.data_dir, PathBuf::from("./data"));
    }

    #[test]
    fn parses_toml() {
        let raw = r#"
            listen_addr = "0.0.0.0:9090"
            data_dir = "/var/lib/engrenage"
        "#;
        let config: Config = toml::from_str(raw).expect("must parse");
        assert_eq!(config.listen_addr, "0.0.0.0:9090");
        assert_eq!(config.data_dir, PathBuf::from("/var/lib/engrenage"));
    }

    #[test]
    fn validate_rejects_bad_listen_addr() {
        let config = Config {
            listen_addr: "not-an-addr".to_owned(),
            ..Config::default()
        };
        assert!(config.validate().is_err());
    }
}
