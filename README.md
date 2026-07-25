# herdr-edit

A simple text editor that runs in a [herdr](https://herdr.dev) pane, beside your coding agent. A file tree, a syntax-highlighted editor, and an uncommitted-diff tab — enough to read, tweak, and stage a file without leaving herdr. **Windows-only.**

![the editor pane: a file tree on the left, a highlighted buffer on the right](https://github.com/aclima01/herdr-edit-windows) <!-- screenshot placeholder -->

## What it does

- **File tree** of the pane's working directory. Arrow keys navigate; `Enter` expands a directory or opens a file.
- **Editor** with syntax highlighting (via [syntect](https://github.com/trishume/syntect)) over a [ropey](https://github.com/cessen/ropey) buffer. Modeless: type to insert, arrows to move, `Ctrl+S` to save.
- **Uncommitted diff tab** — `Ctrl+D` shows `git diff HEAD` for the open file, colored by line.
- **Stage** — `Ctrl+A` runs `git add` on the open file.

It is deliberately small: no LSP, no autocomplete, no multi-file search.

## Install

```
herdr plugin install aclima01/herdr-edit-windows
```

Then bind the toggle action to a key, or invoke it directly:

```
herdr plugin action invoke aclima.edit.toggle
```

## Keys

| Focus  | Key                       | Action                                  |
| ------ | ------------------------- | --------------------------------------- |
| Tree   | `↑` / `↓`                 | move the selection                      |
| Tree   | `→` / `Enter`             | expand a directory, or open a file      |
| Tree   | `←`                       | collapse a directory / go to parent     |
| Editor | printable keys            | insert text                             |
| Editor | `↑` `↓` `←` `→` `PgUp/Dn` | move the cursor                         |
| Editor | `Ctrl+S`                  | save                                    |
| Editor | `Ctrl+A`                  | `git add` the open file                 |
| Editor | `Ctrl+D`                  | toggle the uncommitted-diff tab         |
| Any    | `Tab`                     | switch focus between the tree and editor |
| Any    | `Esc`                     | (editor) return focus to the tree       |
| Any    | `Ctrl+Q`                  | quit the pane                           |

## Configuration

Optional. herdr passes a per-plugin config directory; drop a `config.toml` there:

```toml
# syntax color theme (default: catppuccin-mocha)
theme = "nord"
```

Recognized themes: `catppuccin-mocha`, `catppuccin-macchiato`, `catppuccin-frappe`, `catppuccin-latte`, `nord`, `dracula`, `gruvbox-dark`, `gruvbox-light`, `solarized-dark`, `solarized-light`, `one-half-dark`, `one-half-light`, `two-dark`, `github`, `monokai`, `zenburn`. An unknown name falls back to the default.

The theme colors the whole pane — syntax, borders, tree, title, gutter, and status all follow it. It is read when the pane starts, so **close and reopen the pane** after editing `config.toml`. The status line reports what it resolved (e.g. `theme: dracula`). If the theme is not taking, run the diagnostic:

```
herdr-edit --print-config
```

It prints the config directory it read and the outcome, so you can confirm the file is in the right place and the name is recognized.

## Build from source

Requires the Rust toolchain (`x86_64-pc-windows-msvc`).

```
cargo build --release
```

`herdr plugin link .` uses a local checkout; copy `target\release\herdr-edit.exe` into `bin\` so the pane can find it (a real `herdr plugin install` downloads the released binary instead).

## License

MIT — see [LICENSE](LICENSE). Patterns and acknowledgements in [CREDITS](CREDITS).
