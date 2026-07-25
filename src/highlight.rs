//! Syntax highlighting via `syntect`, using the broad two-face syntax set and an
//! embedded theme. Produces per-line foreground spans; the pane keeps the terminal's
//! own background, so only token colors come from the theme.

use std::fmt;
use std::sync::OnceLock;

use syntect::easy::HighlightLines;
use syntect::highlighting::Theme;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use two_face::theme::EmbeddedThemeName;

/// One run of same-colored text on a line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub text: String,
    pub color: (u8, u8, u8),
}

/// Fallback text color when a token carries none (Catppuccin Mocha text).
const DEFAULT_FG: (u8, u8, u8) = (0xcd, 0xd6, 0xf4);

/// UI chrome colors derived from the active theme, so the whole pane (borders, tree,
/// title, gutter, selection, status) follows the configured theme, not just the syntax.
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    /// Default text: file names and body text.
    pub text: (u8, u8, u8),
    /// Focused borders and titles, directory names — the theme's keyword color.
    pub accent: (u8, u8, u8),
    /// Gutter, key hints, unfocused chrome — the theme's comment color.
    pub muted: (u8, u8, u8),
    /// Selected tree row background — the theme's selection color.
    pub selection: (u8, u8, u8),
    /// Transient status messages — the theme's string color.
    pub status: (u8, u8, u8),
}

/// The broad two-face/bat syntax set, built once per process (deserializing it is
/// expensive) and shared across the process.
fn syntaxes() -> &'static SyntaxSet {
    static SYNTAXES: OnceLock<SyntaxSet> = OnceLock::new();
    SYNTAXES.get_or_init(two_face::syntax::extra_newlines)
}

/// Holds the active theme and highlights file content into per-line spans.
pub struct Highlighter {
    theme: Theme,
    default_fg: (u8, u8, u8),
}

impl fmt::Debug for Highlighter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Highlighter").finish_non_exhaustive()
    }
}

impl Default for Highlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl Highlighter {
    /// Build with the default embedded theme (Catppuccin Mocha).
    pub fn new() -> Self {
        Self::with_theme(EmbeddedThemeName::CatppuccinMocha)
    }

    /// Build with a specific embedded theme, chosen by plugin config (`config.rs`).
    pub fn with_theme(name: EmbeddedThemeName) -> Self {
        let theme = two_face::theme::extra().get(name).clone();
        let default_fg = theme
            .settings
            .foreground
            .map_or(DEFAULT_FG, |c| (c.r, c.g, c.b));
        Self { theme, default_fg }
    }

    /// Derive UI chrome colors from the theme. Accent, muted, and status come from the
    /// theme's own keyword, comment, and string colors (sampled by highlighting a tiny Rust
    /// snippet), so the chrome always matches the syntax. Selection comes from the theme
    /// settings, dimmed toward the default when absent.
    pub fn palette(&self) -> Palette {
        // Distinct tokens: `fn` (keyword), `"s"` (string), `// c` (comment).
        let sample = "fn a() { let x = \"s\"; }\n// c\n";
        let spans: Vec<Span> = self
            .highlight(sample, Some("rs"))
            .into_iter()
            .flatten()
            .collect();
        let color_of =
            |pred: &dyn Fn(&str) -> bool| spans.iter().find(|s| pred(&s.text)).map(|s| s.color);

        let accent = color_of(&|t| t.trim() == "fn").unwrap_or(self.default_fg);
        let status = color_of(&|t| t.contains('s') && t.contains('"'))
            .or_else(|| color_of(&|t| t.trim() == "s"))
            .unwrap_or(self.default_fg);
        let muted = color_of(&|t| t.contains("//")).unwrap_or_else(|| dim(self.default_fg));
        let selection = self
            .theme
            .settings
            .selection
            .map_or_else(|| dim(accent), |c| (c.r, c.g, c.b));

        Palette {
            text: self.default_fg,
            accent,
            muted,
            selection,
            status,
        }
    }

    /// Highlight `content` line by line. Each inner `Vec` is one line's spans. With no
    /// resolvable `extension`, every line is one plain span in the default color.
    /// `extension` matches by file extension (e.g. `rs`, `toml`).
    pub fn highlight(&self, content: &str, extension: Option<&str>) -> Vec<Vec<Span>> {
        let syntaxes = syntaxes();
        let syntax = extension.and_then(|ext| syntaxes.find_syntax_by_extension(ext));
        let Some(syntax) = syntax else {
            return content
                .lines()
                .map(|l| {
                    vec![Span {
                        text: l.to_string(),
                        color: self.default_fg,
                    }]
                })
                .collect();
        };
        let mut h = HighlightLines::new(syntax, &self.theme);
        let mut out = Vec::new();
        for line in LinesWithEndings::from(content) {
            let spans = match h.highlight_line(line, syntaxes) {
                Ok(regions) => regions
                    .into_iter()
                    .map(|(style, text)| Span {
                        text: text.trim_end_matches('\n').to_string(),
                        color: (style.foreground.r, style.foreground.g, style.foreground.b),
                    })
                    .collect(),
                // A grammar error degrades to plain text rather than blocking the view.
                Err(_) => vec![Span {
                    text: line.trim_end_matches('\n').to_string(),
                    color: self.default_fg,
                }],
            };
            out.push(spans);
        }
        out
    }
}

/// A dimmed variant of a color (~40% toward black), for a muted fallback.
fn dim(c: (u8, u8, u8)) -> (u8, u8, u8) {
    (
        (c.0 as u16 * 2 / 5) as u8,
        (c.1 as u16 * 2 / 5) as u8,
        (c.2 as u16 * 2 / 5) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::Highlighter;

    #[test]
    fn highlights_rust_into_colored_spans() {
        let h = Highlighter::new();
        let lines = h.highlight("let x = 1;\n", Some("rs"));
        assert_eq!(lines.len(), 1);
        let spans = &lines[0];
        assert!(spans.len() > 1, "rust tokenizes into several spans");
        assert_eq!(
            spans.iter().map(|s| s.text.as_str()).collect::<String>(),
            "let x = 1;"
        );
        // A keyword color differs from the default text color.
        assert!(
            spans
                .iter()
                .any(|s| s.text == "let" && s.color != super::DEFAULT_FG)
        );
    }

    #[test]
    fn palette_follows_the_theme() {
        use two_face::theme::EmbeddedThemeName as T;
        let dracula = Highlighter::with_theme(T::Dracula).palette();
        let zenburn = Highlighter::with_theme(T::Zenburn).palette();
        // Two themes yield different chrome, and the accent is a real (non-default) color.
        assert_ne!(dracula.accent, zenburn.accent);
        assert_ne!(dracula.text, zenburn.text);
        assert_ne!(dracula.accent, super::DEFAULT_FG);
    }

    #[test]
    fn unknown_extension_is_one_plain_span_per_line() {
        let h = Highlighter::new();
        let lines = h.highlight("alpha\nbeta\n", None);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].len(), 1);
        assert_eq!(lines[0][0].text, "alpha");
    }
}
