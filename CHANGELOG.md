# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.3] - 2026-07-25

### Fixed

- The `theme` setting was ignored when `config.toml` was saved with a UTF-8 BOM
  (Notepad's default), because the BOM stuck to the first key. The BOM is now stripped
  before parsing.

### Added

- The config directory now falls back to `%APPDATA%\herdr\plugins\config\<plugin-id>`
  when `HERDR_PLUGIN_CONFIG_DIR` is not set, so the theme loads on any herdr build.
- The status line reports how the theme resolved on startup (`theme: dracula`,
  `theme 'x' unknown — using catppuccin-mocha`, or `no config.toml`), so a bad or
  missing setting is no longer silent.
- `herdr-edit --print-config` prints the resolved config directory and theme outcome,
  for diagnosing a config that will not take.

## [0.1.2] - 2026-07-25

### Fixed

- The install build step still failed on PowerShell 3 (Windows 8): `Get-FileHash`
  needs PowerShell 4 and `Expand-Archive` needs PowerShell 5. Replaced both with
  .NET calls (`System.Security.Cryptography.SHA256` and `System.IO.Compression.ZipFile`)
  so `herdr/install.ps1` runs on PowerShell 3 and up.

## [0.1.1] - 2026-07-21

### Fixed

- The Windows install build step failed to parse: `"$Name: ..."` in `herdr/install.ps1`
  is read by PowerShell as a drive-qualified variable reference. Delimited the name
  as `"${Name}: ..."` so `herdr plugin install` runs the download step.

## [0.1.0] - 2026-07-21

### Added

- File tree of the pane's working directory, with lazy expand/collapse.
- Syntax-highlighted, editable buffer (ropey + syntect): insert, delete, cursor
  movement, and `Ctrl+S` save.
- Uncommitted-diff tab (`Ctrl+D`): `git diff HEAD` of the open file, colored by line.
- Stage the open file with `Ctrl+A` (`git add`).
- Optional `theme` setting read from the plugin config directory.
- Windows plugin packaging: pane, toggle/open/close actions, and a GitHub release
  workflow that publishes the `x86_64-pc-windows-msvc` binary.
