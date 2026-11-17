//! Writes settings to the facer driver's character devices.
//! Byte layout mirrors `facer_rgb.py` in the repository root.

use crate::config::{Config, Effect, Rgb};
use std::{fs::OpenOptions, io, io::Write};

const DYNAMIC_DEV: &str = "/dev/acer-gkbbl-0";
const STATIC_DEV: &str = "/dev/acer-gkbbl-static-0";

/// Sends `config` to the keyboard.
pub fn apply(config: &Config) -> io::Result<()> {
    if config.effect == Effect::Static {
        for (zone, rgb) in config.zones.iter().enumerate() {
            write(STATIC_DEV, &static_payload(zone, *rgb))?;
        }
    }
    write(DYNAMIC_DEV, &dynamic_payload(config))
}

fn write(dev: &str, bytes: &[u8]) -> io::Result<()> {
    OpenOptions::new().write(true).open(dev)?.write_all(bytes)
}

fn dynamic_payload(c: &Config) -> [u8; 16] {
    let mut p = [0; 16];
    p[2] = c.brightness.min(100);
    p[9] = 1;
    if c.effect != Effect::Static {
        p[0] = c.effect.into();
        p[1] = c.speed.min(9);
        p[3] = if c.effect == Effect::Wave { 8 } else { 0 };
        p[4] = c.direction.into();
        p[5..8].copy_from_slice(&c.color);
    }
    p
}

/// `zone` is 0-based (left to right).
fn static_payload(zone: usize, [r, g, b]: Rgb) -> [u8; 4] {
    [1 << zone, r, g, b]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Direction;

    #[test]
    fn static_matches_facer_rgb_py() {
        let c = Config {
            brightness: 80,
            ..Config::default()
        };
        assert_eq!(
            dynamic_payload(&c),
            [0, 0, 80, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(static_payload(2, [1, 2, 3]), [4, 1, 2, 3]);
    }

    #[test]
    fn wave_matches_facer_rgb_py() {
        // ./facer_rgb.py -m 3 -s 5 -b 100 -d 2 -cR 50 -cG 255 -cB 50
        let c = Config {
            effect: Effect::Wave,
            speed: 5,
            direction: Direction::LeftToRight,
            color: [50, 255, 50],
            ..Config::default()
        };
        assert_eq!(
            dynamic_payload(&c),
            [3, 5, 100, 8, 2, 50, 255, 50, 0, 1, 0, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn clamps_out_of_range_values() {
        let c = Config {
            effect: Effect::Breath,
            speed: 50,
            brightness: 250,
            ..Config::default()
        };
        let p = dynamic_payload(&c);
        assert_eq!((p[1], p[2]), (9, 100));
    }
}
