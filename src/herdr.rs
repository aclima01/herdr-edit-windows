//! herdr host integration: the pane's environment context.
//!
//! herdr sets `HERDR_*` env vars for every plugin process (see the kickoff and
//! `specs/herdr-host.md` in reviewr). Later milestones add CLI calls (`herdr pane
//! list`, etc.); for now we only surface the identifying context so the pane can prove
//! it is running inside herdr.

use std::env;
use std::path::PathBuf;

/// The identifying context herdr hands this pane through the environment. Several fields
/// are read by later milestones (the tree's cwd, CLI calls keyed by workspace); they are
/// captured now so the whole context resolves in one place.
#[derive(Clone, Debug, Default)]
#[allow(dead_code)]
pub struct Context {
    pub workspace_id: Option<String>,
    pub tab_id: Option<String>,
    pub pane_id: Option<String>,
    pub plugin_root: Option<PathBuf>,
    /// The pane's working directory (the repo under edit), normalized.
    pub cwd: PathBuf,
}

impl Context {
    /// Read the context from the `HERDR_*` environment.
    pub fn from_env() -> Self {
        Self {
            workspace_id: env::var("HERDR_WORKSPACE_ID").ok(),
            tab_id: env::var("HERDR_TAB_ID").ok(),
            pane_id: env::var("HERDR_PANE_ID").ok(),
            plugin_root: env::var("HERDR_PLUGIN_ROOT").ok().map(strip_extended_prefix),
            cwd: env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }

    /// A short one-line summary for the status bar, proving the herdr context resolved.
    pub fn summary(&self) -> String {
        let ws = self.workspace_id.as_deref().unwrap_or("-");
        let pane = self.pane_id.as_deref().unwrap_or("-");
        format!("herdr ws={ws} pane={pane}")
    }
}

/// Strip a Windows `\\?\` extended-length path prefix so the path resolves for tools that
/// do not accept it. herdr hands `HERDR_PLUGIN_ROOT` with this prefix on Windows.
fn strip_extended_prefix(raw: String) -> PathBuf {
    PathBuf::from(raw.strip_prefix(r"\\?\").unwrap_or(&raw).to_string())
}

#[cfg(test)]
mod tests {
    use super::strip_extended_prefix;

    #[test]
    fn strips_the_extended_length_prefix() {
        let p = strip_extended_prefix(r"\\?\E:\apps\herdr-edit-windows".to_string());
        assert_eq!(p.to_string_lossy(), r"E:\apps\herdr-edit-windows");
    }

    #[test]
    fn leaves_a_plain_path_untouched() {
        let p = strip_extended_prefix(r"E:\apps".to_string());
        assert_eq!(p.to_string_lossy(), r"E:\apps");
    }
}
