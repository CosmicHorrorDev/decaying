use std::{
    collections::{btree_map, BTreeMap},
    fs, io,
    path::Path,
};

use bytesize::ByteSize;
use humantime_serde::re::humantime::Duration;
use serde::{de::Error as DeError, Deserialize, Deserializer};

pub fn try_read(path: &Path) -> Result<Config, Error> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(err) => {
            return match err.kind() {
                io::ErrorKind::NotFound => {
                    log::debug!("Config not found. Falling back to default");
                    Ok(Config::default())
                }
                _other => Err(err.into()),
            };
        }
    };

    let config: Config =
        basic_toml::from_str(&contents).map_err(|e| Error::ParseFailure(e.to_string()))?;

    Ok(config)
}

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(deserialize_with = "limits_de")]
    pub limits: BTreeMap<(ByteSize, ByteSize), Duration>,
}

impl Default for Config {
    fn default() -> Self {
        let limits = [(
            (ByteSize(u64::MIN), ByteSize(u64::MAX)),
            Duration::from(std::time::Duration::from_secs(60 * 60 * 24)),
        )]
        .into_iter()
        .collect();
        Self { limits }
    }
}

fn limits_de<'de, D>(deserializer: D) -> Result<BTreeMap<(ByteSize, ByteSize), Duration>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Simple(Vec<(ByteSize, ByteSize, String)>);

    let simple = Simple::deserialize(deserializer)?;

    let mut limits = BTreeMap::default();
    for (lower, upper, duration_str) in simple.0 {
        if lower > upper {
            return Err(DeError::custom(format!(
                "Lower bound greater than upper bound. Lower: {lower} Upper: {upper}"
            )));
        }
        let size = (lower, upper);

        let duration = duration_str.parse().map_err(|e| DeError::custom(e))?;
        let entry = limits.entry(size);

        if let btree_map::Entry::Occupied(_) = entry {
            return Err(DeError::custom(format!("Duplicate entry for {size:?}")));
        }

        entry.or_insert(duration);
    }

    Ok(limits)
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("I/O error reading config file - {0}")]
    Io(#[from] io::Error),
    #[error("Failed parsing config file - {0}")]
    ParseFailure(String),
}
