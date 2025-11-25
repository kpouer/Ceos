use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

const CONFIG_FILE: &str = "ceos.toml";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Options {
    pub(crate) compression: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self { compression: false }
    }
}

impl Options {
    fn config_path() -> PathBuf {
        // Par défaut, on lit/écrit dans le répertoire courant
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
            .join(CONFIG_FILE)
    }

    pub(crate) fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(text) => match toml::from_str::<Options>(&text) {
                    Ok(opts) => {
                        info!("Options chargées depuis {:?}", path);
                        opts
                    }
                    Err(e) => {
                        warn!(
                            "Fichier de configuration invalide {:?}, utilisation des valeurs par défaut: {}",
                            path, e
                        );
                        Options::default()
                    }
                },
                Err(e) => {
                    warn!(
                        "Impossible de lire {:?}, utilisation des valeurs par défaut: {}",
                        path, e
                    );
                    Options::default()
                }
            }
        } else {
            let opts = Options::default();
            // Essayer de créer un fichier initial pour l'utilisateur
            if let Err(e) = opts.save() {
                warn!("Impossible de créer {:?}: {}", path, e);
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
            .map_err(|e| Error::new(ErrorKind::Other, format!("{}", e)))?;
        fs::write(&path, toml_text)?;
        info!("Options enregistrées dans {:?}", path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_roundtrip() {
        let o = Options::default();
        let s = toml::to_string(&o).unwrap();
        let back: Options = toml::from_str(&s).unwrap();
        assert_eq!(o.compression, back.compression);
    }
}
