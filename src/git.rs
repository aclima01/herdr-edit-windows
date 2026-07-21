//! git access for the diff tab and staging. Reads dominate: repo discovery and diffs never
//! touch the index. The one write is [`stage`] (`git add`), invoked only on the user's key.
//!
//! The binary runs inside the herdr pane's own console, so a `git` subprocess inherits it
//! and never flashes a separate window on Windows.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

/// Run `git -C <repo> <args>` and return stdout. Errors on a non-zero exit.
fn git(repo: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["-c", "core.quotepath=false"])
        .args(args)
        .output()
        .with_context(|| format!("running git {args:?}"))?;
    if !out.status.success() {
        bail!(
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// The work-tree root containing `dir`, or `None` when `dir` is outside any repo.
pub fn toplevel(dir: &Path) -> Option<PathBuf> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let line = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!line.is_empty()).then(|| PathBuf::from(line))
}

/// The unified diff of `file` against `HEAD` — every uncommitted change, staged or not.
/// Empty when the file matches `HEAD` (or is untracked, which `diff HEAD` does not show).
pub fn diff_uncommitted(repo: &Path, file: &Path) -> Result<String> {
    let path = file.to_string_lossy();
    git(repo, &["diff", "HEAD", "--", &path])
}

/// Stage `file` into the index (`git add`). The sole write in this module.
pub fn stage(repo: &Path, file: &Path) -> Result<()> {
    let path = file.to_string_lossy();
    git(repo, &["add", "--", &path])?;
    Ok(())
}
