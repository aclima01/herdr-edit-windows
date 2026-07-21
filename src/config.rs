//! Plugin configuration read from `$HERDR_PLUGIN_CONFIG_DIR/config.toml`.
//!
//! The only setting so far is the syntax `theme`. The file is a flat TOML with one
//! `key = "value"` per line; a missing file, missing key, or unknown theme falls back to
//! the default (Catppuccin Mocha). Parsing is deliberately minimal — one string key does
//! not justify a TOML dependency.

use std::env;
use std::path::PathBuf;

use two_face::theme::EmbeddedThemeName;

/// The resolved plugin configuration.
#[derive(Clone, Debug)]
pub struct Config {
    pub theme: EmbeddedThemeName,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: EmbeddedThemeName::CatppuccinMocha,
        }
    }
}

impl Config {
    /// Load from `$HERDR_PLUGIN_CONFIG_DIR/config.toml`, falling back to defaults.
    pub fn load() -> Self {
        let mut config = Self::default();
        let Some(dir) = env::var_os("HERDR_PLUGIN_CONFIG_DIR") else {
            return config;
        };
        let path = PathBuf::from(dir).join("config.toml");
        let Ok(text) = std::fs::read_to_string(&path) else {
            return config;
        };
        if let Some(name) = flat_value(&text, "theme")
            && let Some(theme) = resolve_theme(&name)
        {
            config.theme = theme;
        }
        config
    }
}

/// The value of `key` in a flat TOML: the first `key = "value"` line, quotes stripped,
/// comments and blank lines ignored.
fn flat_value(text: &str, key: &str) -> Option<String> {
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        if k.trim() != key {
            continue;
        }
        let v = v.trim().trim_matches('"').trim_matches('\'');
        return Some(v.to_string());
    }
    None
}

/// Map a config theme name to an embedded theme. Names are lowercase with `-` separators.
fn resolve_theme(name: &str) -> Option<EmbeddedThemeName> {
    use EmbeddedThemeName as T;
    let theme = match name.to_lowercase().as_str() {
        "catppuccin-mocha" | "catppuccin" => T::CatppuccinMocha,
        "catppuccin-macchiato" => T::CatppuccinMacchiato,
        "catppuccin-frappe" => T::CatppuccinFrappe,
        "catppuccin-latte" => T::CatppuccinLatte,
        "nord" => T::Nord,
        "dracula" => T::Dracula,
        "gruvbox-dark" => T::GruvboxDark,
        "gruvbox-light" => T::GruvboxLight,
        "solarized-dark" => T::SolarizedDark,
        "solarized-light" => T::SolarizedLight,
        "one-half-dark" => T::OneHalfDark,
        "one-half-light" => T::OneHalfLight,
        "two-dark" => T::TwoDark,
        "github" => T::Github,
        "monokai" => T::MonokaiExtended,
        "zenburn" => T::Zenburn,
        _ => return None,
    };
    Some(theme)
}

#[cfg(test)]
mod tests {
    use super::{flat_value, resolve_theme};
    use two_face::theme::EmbeddedThemeName;

    #[test]
    fn reads_a_quoted_value_ignoring_comments() {
        let text = "# a comment\n\ntheme = \"nord\"  \n";
        assert_eq!(flat_value(text, "theme").as_deref(), Some("nord"));
        assert_eq!(flat_value(text, "missing"), None);
    }

    #[test]
    fn resolves_known_themes_and_rejects_unknown() {
        assert_eq!(resolve_theme("nord"), Some(EmbeddedThemeName::Nord));
        assert_eq!(
            resolve_theme("Catppuccin-Mocha"),
            Some(EmbeddedThemeName::CatppuccinMocha)
        );
        assert_eq!(resolve_theme("nonsense"), None);
    }
}
