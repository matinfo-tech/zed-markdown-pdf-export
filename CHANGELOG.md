# Changelog

All notable changes to this extension. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/). Changes to the exporter itself are in the
[markdown-pdf-export changelog](https://github.com/matinfo-tech/markdown-pdf-export/blob/main/CHANGELOG.md).

## [Unreleased]

### Changed

- README: install as a dev extension (or with `markdown-pdf-export setup`) until Zed lists MCP
  servers from the official MCP registry; new MCP extensions are paused in the Zed extension list.
- Clearer install/update logic in `src/lib.rs`, same behaviour.
- README: keyboard shortcut for exporting without the Agent, set up by
  `markdown-pdf-export setup zed` (2.3.0 or later).

## [0.1.0] - 2026-11-22

### Added

- `markdown-pdf` context server: installs and updates the `markdown-pdf-export` npm package
  (2.2.0 or later) and runs its MCP server with Zed's Node. Works offline once installed.
- Optional settings `browser`, `settings_file` and `debug`, with a schema and installation
  instructions in the Configure dialog.

[Unreleased]: https://github.com/matinfo-tech/zed-markdown-pdf-export/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/matinfo-tech/zed-markdown-pdf-export/releases/tag/v0.1.0
