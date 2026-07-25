# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
