# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - 2026-03-08

### Added

- Discord Slash-Command (owner-limited): `/reboot`
- Forward Announcement Messages by LiShogi

### Changed

- Embed Colors:
  - Move: `#ff8000`
  - Game End: `#004E00`
  - Announcement: `#8000ff`

## [0.2.0] - 2026-03-04

### Changed

- Improve Time Information in Game Move Embed
- Make Embed Titles clickable

## [0.1.0] - 2026-03-02

### Added

- Rust-based Discord Bot Binary
- Discord Slash-Command: `/shogi user`
- Discord Slash-Command: `/shogi track`
- Discord Slash-Command: `/changelog`
- Discord (Owner) Slash-Command: `/debug`
- Rust-based LiShogi library
- Multi-Threaded WebSocket Listeners (per game)

[Unreleased]: https://github.com/rshchekotov/wwbda/compare/v0.2.0...HEAD

[0.2.0]: https://github.com/rshchekotov/wwbda/releases/tag/v0.2.0

[0.1.0]: https://github.com/rshchekotov/wwbda/releases/tag/v0.1.0
