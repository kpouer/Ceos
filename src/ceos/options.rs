use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Error;
use std::path::PathBuf;

const CONFIG_FILE: &str = "ceos.toml";

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Options {
    pub(crate) compression: bool,
}

impl Options {
    fn config_path() -> PathBuf {
        // Par défaut, on lit/écrit dans le répertoire courant
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(CONFIG_FILE)
    }

    pub(crate) fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(text) => match toml::from_str::<Options>(&text) {
                    Ok(opts) => {
                        info!("Options loaded from {path:?}");
                        opts
                    }
                    Err(e) => {
                        warn!("Invalid configuration file {path:?}, using défaut values: {e}");
                        Options::default()
                    }
                },
                Err(e) => {
                    warn!("Unable to read {path:?}, using default values: {e}");
                    Options::default()
                }
            }
        } else {
            let opts = Options::default();
            if let Err(e) = opts.save() {
                warn!("Unable to create {path:?}: {e}");
            }
            opts
        }
    }

    pub(crate) fn save(&self) -> Result<(), Error> {
        let path = Self::config_path();
        let parent = path.parent().unwrap_or_else(|| std::path::Path::new("."));
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
        let toml_text = toml::to_string_pretty(self)
            .map_err(|e| Error::other(format!("{}", e)))?;
        fs::write(&path, toml_text)?;
        info!("Options saved into {path:?}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let o = Options::default();
        let s = toml::to_string(&o)?;
        let back: Options = toml::from_str(&s)?;
        assert_eq!(o.compression, back.compression);
        Ok(())
    }
}
