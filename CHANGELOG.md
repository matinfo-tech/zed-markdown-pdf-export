# Changelog

All notable changes to this extension. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/). Changes to the exporter itself are in the
[markdown-pdf-export changelog](https://github.com/matinfo-tech/markdown-pdf-export/blob/main/CHANGELOG.md).

## [Unreleased]

## [0.2.0] - 2026-11-22

### Changed

- The server starts without a network call when a working copy of the exporter is installed:
  the npm update check now runs at most once a day instead of on every first start of a Zed
  session. The time of the last check is kept in `.last-update-check` in the extension folder.
- Updates stay on major version 2 of `markdown-pdf-export`. A future 3.x is not installed
  automatically; a note in the Zed log says a newer extension is needed.
- A failed or skipped update is reported in the Zed log (lines starting with `[markdown-pdf]`)
  instead of being silent.
- `settings_file` may start with `~` or `$HOME`.
- README: install as a dev extension (or with `markdown-pdf-export setup`) until Zed lists MCP
  servers from the official MCP registry; new MCP extensions are paused in the Zed extension list.
- Clearer install/update logic in `src/lib.rs`.
- README: keyboard shortcut for exporting without the Agent, set up by
  `markdown-pdf-export setup zed` (2.3.0 or later).

## [0.1.0] - 2026-11-22

### Added

- `markdown-pdf` context server: installs and updates the `markdown-pdf-export` npm package
  (2.2.0 or later) and runs its MCP server with Zed's Node. Works offline once installed.
- Optional settings `browser`, `settings_file` and `debug`, with a schema and installation
  instructions in the Configure dialog.

[Unreleased]: https://github.com/matinfo-tech/zed-markdown-pdf-export/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/matinfo-tech/zed-markdown-pdf-export/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/matinfo-tech/zed-markdown-pdf-export/releases/tag/v0.1.0
