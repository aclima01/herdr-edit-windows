//! Plugin configuration read from the plugin's config directory (`config.toml`).
//!
//! The only setting so far is the syntax `theme`. The file is a flat TOML with one
//! `key = "value"` per line; a missing file, missing key, or unknown theme falls back to
//! the default (Catppuccin Mocha). Parsing is deliberately minimal — one string key does
//! not justify a TOML dependency.
//!
//! Loading is tolerant and self-reporting: it strips a UTF-8 BOM (Notepad writes one),
//! resolves the config directory from `HERDR_PLUGIN_CONFIG_DIR` or, failing that, from
//! `%APPDATA%\herdr\plugins\config\<HERDR_PLUGIN_ID>`, and returns a human-readable `note`
//! describing the outcome so a silent fallback becomes visible in the status line.

use std::env;
use std::path::PathBuf;

use two_face::theme::EmbeddedThemeName;

/// The resolved plugin configuration and a note describing how it resolved.
#[derive(Clone, Debug)]
pub struct Config {
    pub theme: EmbeddedThemeName,
    /// Human-readable resolution outcome, shown in the status line on startup.
    pub note: String,
}

const DEFAULT_THEME_NAME: &str = "catppuccin-mocha";

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: EmbeddedThemeName::CatppuccinMocha,
            note: String::new(),
        }
    }
}

impl Config {
    /// Resolve the configuration, reporting the outcome in `note`.
    pub fn load() -> Self {
        let Some(dir) = config_dir() else {
            return Self::noted(format!(
                "theme: {DEFAULT_THEME_NAME} (no plugin config dir)"
            ));
        };
        let path = dir.join("config.toml");
        let Ok(raw) = std::fs::read_to_string(&path) else {
            return Self::noted(format!("theme: {DEFAULT_THEME_NAME} (no config.toml)"));
        };
        // Notepad writes a UTF-8 BOM; strip it so the first key still matches.
        let text = raw.strip_prefix('\u{feff}').unwrap_or(&raw);
        let Some(name) = flat_value(text, "theme") else {
            return Self::noted(format!("theme: {DEFAULT_THEME_NAME} (no theme set)"));
        };
        match resolve_theme(&name) {
            Some(theme) => Self {
                theme,
                note: format!("theme: {name}"),
            },
            None => Self::noted(format!(
                "theme '{name}' unknown — using {DEFAULT_THEME_NAME}"
            )),
        }
    }

    fn noted(note: String) -> Self {
        Self {
            note,
            ..Self::default()
        }
    }
}

/// The resolved config directory, for the `--print-config` diagnostic.
pub fn config_dir_debug() -> Option<PathBuf> {
    config_dir()
}

/// The plugin's config directory: `HERDR_PLUGIN_CONFIG_DIR` if herdr set it, else the
/// conventional `%APPDATA%\herdr\plugins\config\<HERDR_PLUGIN_ID>` so the theme still loads
/// on herdr builds that do not pass the variable to panes.
fn config_dir() -> Option<PathBuf> {
    if let Some(dir) = env::var_os("HERDR_PLUGIN_CONFIG_DIR") {
        return Some(PathBuf::from(dir));
    }
    let appdata = env::var_os("APPDATA")?;
    let plugin_id = env::var("HERDR_PLUGIN_ID").ok()?;
    Some(
        PathBuf::from(appdata)
            .join("herdr")
            .join("plugins")
            .join("config")
            .join(plugin_id),
    )
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
    let theme = match name.trim().to_lowercase().as_str() {
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
    fn a_utf8_bom_does_not_break_the_first_key() {
        // Notepad prepends a BOM; the loader strips it before parsing. Here we prove the
        // parser matches the key once the BOM is removed (as `Config::load` does).
        let raw = "\u{feff}theme = \"dracula\"\n";
        let text = raw.strip_prefix('\u{feff}').unwrap();
        assert_eq!(flat_value(text, "theme").as_deref(), Some("dracula"));
    }

    #[test]
    fn resolves_known_themes_and_rejects_unknown() {
        assert_eq!(resolve_theme("dracula"), Some(EmbeddedThemeName::Dracula));
        assert_eq!(resolve_theme("nord"), Some(EmbeddedThemeName::Nord));
        assert_eq!(
            resolve_theme("Catppuccin-Mocha"),
            Some(EmbeddedThemeName::CatppuccinMocha)
        );
        assert_eq!(resolve_theme("nonsense"), None);
    }
}
