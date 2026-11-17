//! User settings: what effect to show and whether to restore it at boot.

use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// Lighting effects understood by the facer driver. Discriminants are the driver's mode ids.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub enum Effect {
    #[default]
    Static = 0,
    Breath = 1,
    Neon = 2,
    Wave = 3,
    Shifting = 4,
    Zoom = 5,
}

impl Effect {
    pub const ALL: [Self; 6] = [
        Self::Static,
        Self::Breath,
        Self::Neon,
        Self::Wave,
        Self::Shifting,
        Self::Zoom,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Static => "Static",
            Self::Breath => "Breath",
            Self::Neon => "Neon",
            Self::Wave => "Wave",
            Self::Shifting => "Shifting",
            Self::Zoom => "Zoom",
        }
    }

    /// Breath, Shifting and Zoom take one color; Neon and Wave are rainbow effects.
    pub fn uses_color(self) -> bool {
        matches!(self, Self::Breath | Self::Shifting | Self::Zoom)
    }
}

impl From<Effect> for u8 {
    fn from(e: Effect) -> Self {
        e as u8
    }
}

impl TryFrom<u8> for Effect {
    type Error = String;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        Self::ALL
            .get(usize::from(v))
            .copied()
            .ok_or_else(|| format!("unknown effect id {v}"))
    }
}

/// Animation direction. Discriminants are the driver's direction ids.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub enum Direction {
    #[default]
    RightToLeft = 1,
    LeftToRight = 2,
}

impl From<Direction> for u8 {
    fn from(d: Direction) -> Self {
        d as u8
    }
}

impl TryFrom<u8> for Direction {
    type Error = String;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            1 => Ok(Self::RightToLeft),
            2 => Ok(Self::LeftToRight),
            _ => Err(format!("unknown direction id {v}")),
        }
    }
}

pub type Rgb = [u8; 3];

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(default)]
pub struct Config {
    #[serde(rename = "mode")]
    pub effect: Effect,
    /// 0 (frozen) ..= 9
    pub speed: u8,
    /// 0 (off) ..= 100
    pub brightness: u8,
    pub direction: Direction,
    /// Color for single-color effects.
    pub color: Rgb,
    /// Per-zone colors for the static effect, left to right.
    pub zones: [Rgb; 4],
    pub apply_on_boot: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            effect: Effect::Static,
            speed: 4,
            brightness: 100,
            direction: Direction::RightToLeft,
            color: [255; 3],
            zones: [[255; 3]; 4],
            apply_on_boot: true,
        }
    }
}

/// `$XDG_CONFIG_HOME/predator-rgb/config.json`, falling back to `~/.config`.
pub fn default_path() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map_or_else(
            || PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".config"),
            PathBuf::from,
        )
        .join("predator-rgb/config.json")
}

/// Loads the config; a missing file gives defaults, an unreadable or invalid one is reported and gives defaults.
pub fn load(path: &Path) -> Config {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Config::default(),
        Err(e) => {
            eprintln!(
                "predator-rgb: cannot read {}: {e}; using defaults",
                path.display()
            );
            return Config::default();
        }
    };
    serde_json::from_str(&text).unwrap_or_else(|e| {
        eprintln!(
            "predator-rgb: invalid {}: {e}; using defaults",
            path.display()
        );
        Config::default()
    })
}

pub fn save(path: &Path, config: &Config) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(path, serde_json::to_string_pretty(config)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let c = Config {
            effect: Effect::Zoom,
            direction: Direction::LeftToRight,
            apply_on_boot: false,
            ..Config::default()
        };
        assert_eq!(
            serde_json::from_str::<Config>(&serde_json::to_string(&c).unwrap()).unwrap(),
            c
        );
    }

    #[test]
    fn reads_numeric_ids_and_fills_missing_fields() {
        let c: Config = serde_json::from_str(r#"{"mode":3,"direction":2}"#).unwrap();
        assert_eq!(c.effect, Effect::Wave);
        assert_eq!(c.direction, Direction::LeftToRight);
        assert_eq!(c.brightness, 100);
    }

    #[test]
    fn rejects_unknown_ids() {
        assert!(serde_json::from_str::<Config>(r#"{"mode":9}"#).is_err());
        assert!(serde_json::from_str::<Config>(r#"{"direction":0}"#).is_err());
    }

    #[test]
    fn load_falls_back_to_defaults() {
        let dir = std::env::temp_dir().join(format!("predator-rgb-test-{}", std::process::id()));
        let path = dir.join("config.json");
        assert_eq!(load(&path), Config::default());
        fs::create_dir_all(&dir).unwrap();
        fs::write(&path, "not json").unwrap();
        assert_eq!(load(&path), Config::default());
        let c = Config {
            speed: 7,
            ..Config::default()
        };
        save(&path, &c).unwrap();
        assert_eq!(load(&path), c);
        fs::remove_dir_all(dir).unwrap();
    }
}
