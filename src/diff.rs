//! Parsing a unified `git diff` into typed lines for rendering. The diff tab shows the diff
//! as-is (not side by side); each line is classified so the renderer can color it.

/// The role of one diff line, which decides its color.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffKind {
    /// `diff`, `index`, `+++`/`---` headers, mode lines: file-level metadata.
    Meta,
    /// A `@@ ... @@` hunk header.
    Hunk,
    /// An added line (`+`).
    Add,
    /// A removed line (`-`).
    Remove,
    /// An unchanged context line.
    Context,
}

/// One classified diff line, without its trailing newline.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: DiffKind,
    pub text: String,
}

/// Classify each line of a unified diff. `+++`/`---` are matched before `+`/`-` so file
/// headers are metadata, not added or removed content.
pub fn parse(unified: &str) -> Vec<DiffLine> {
    unified
        .lines()
        .map(|line| {
            // `+++`/`---` are file headers, matched here before the `+`/`-` content checks.
            let kind = if line.starts_with("@@") {
                DiffKind::Hunk
            } else if line.starts_with("+++")
                || line.starts_with("---")
                || line.starts_with("diff ")
                || line.starts_with("index ")
                || line.starts_with("new file")
                || line.starts_with("deleted file")
                || line.starts_with("similarity ")
                || line.starts_with("rename ")
                || line.starts_with("old mode")
                || line.starts_with("new mode")
            {
                DiffKind::Meta
            } else if line.starts_with('+') {
                DiffKind::Add
            } else if line.starts_with('-') {
                DiffKind::Remove
            } else {
                DiffKind::Context
            };
            DiffLine {
                kind,
                text: line.to_string(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{DiffKind, parse};

    #[test]
    fn classifies_a_unified_diff() {
        let unified = "diff --git a/f b/f\n\
index 111..222 100644\n\
--- a/f\n\
+++ b/f\n\
@@ -1,2 +1,2 @@\n\
 keep\n\
-old\n\
+new\n";
        let kinds: Vec<_> = parse(unified).iter().map(|l| l.kind).collect();
        assert_eq!(
            kinds,
            vec![
                DiffKind::Meta,    // diff --git
                DiffKind::Meta,    // index
                DiffKind::Meta,    // --- a/f
                DiffKind::Meta,    // +++ b/f
                DiffKind::Hunk,    // @@
                DiffKind::Context, // ' keep'
                DiffKind::Remove,  // -old
                DiffKind::Add,     // +new
            ]
        );
    }
}
