# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - 2026-03-11

## [0.4.0] - 2026-03-11

### Added

- Shogi Board Visualization

## [0.3.1] - 2026-03-11

### Fixed

- Changelog Splicing Bug

## [0.3.0] - 2026-03-11

### Added

- Discord Slash-Command (owner-limited): `/reboot`
- Forward Announcement Messages by LiShogi

### Changed

- Embed Colors:
  - Move: `#ff8000`
  - Game End: `#004E00`
  - Announcement: `#8000ff`

### Fixed

- Turn Time Delta now references the latest move
  not the first move of the game

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

[Unreleased]: https://github.com/rshchekotov/wwbda/compare/v0.4.0...HEAD

[0.4.0]: https://github.com/rshchekotov/wwbda/releases/tag/v0.4.0

[0.3.1]: https://github.com/rshchekotov/wwbda/releases/tag/v0.3.1

[0.3.0]: https://github.com/rshchekotov/wwbda/releases/tag/v0.3.0

[0.2.0]: https://github.com/rshchekotov/wwbda/releases/tag/v0.2.0

[0.1.0]: https://github.com/rshchekotov/wwbda/releases/tag/v0.1.0
