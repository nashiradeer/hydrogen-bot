# Hydrogen // Changelog

## [Unreleased]

### Fixed

- Typo in Severity enum (Suspicous -> Suspicious).
- Play command is only returning the current track instead of the new one.

### Changed

- Avoid recalculations on pause component.

## [0.0.1-alpha.12] - 2025-01-18

### Added

- Add a config option to disable multithreading.

### Changed

- Change Lavalink configuration to the new database URL-like format.
- Implement a database URL-like configuration for Lavalink.
- Implement cluster support in the Lavalink client.
- Refactor the player and it manager.
- Reimplement the Lavalink client using v4 API.
- Reimplement the Lavalink client to use Enum-returning functions (like [tungstenite](https://docs.rs/tungstenite/latest/tungstenite/protocol/struct.WebSocket.html#method.read)) instead of Handler trait (like [serenity](https://docs.rs/serenity/latest/serenity/client/trait.EventHandler.html)).
- Update to Alpine 3.21.
- Update to Rust 1.84.0.
- Update the dependencies.

### Fixed

- Wrong play command message for playing = true and count > 1. ([#12](https://github.com/nashiradeer/hydrogen-bot/issues/12))

### Removed

- Remove the Random loop mode.

## [0.0.1-alpha.11] - 2024-11-06

### Fixed

- Panic, expect, and unwrap doesn't finalize the program. ([#5](https://github.com/nashiradeer/hydrogen-bot/issues/5))
- Player Manager is using the wrong translation key. ([#6](https://github.com/nashiradeer/hydrogen-bot/issues/6))

## [0.0.1-alpha.10] - 2024-11-03

### Changed

- Change how commands and components are registered and handled.
- Change how Hydrogen loads the Lavalink configuration.
- Implement `Default` in `LavalinkUpdatePlayer`.
- Implement `Display` instead of `ToString` in roll module.
- Implement `Debug` to some structs in `lavalink` module.
- Move to a static context.
- Move to static i18n translations.
- Refactor manager module to remove deprecated `Cache::channel` function.
- Refactor project structure.

### Removed

- Remove Deutch (de) translation.
- Remove `builtin-language` feature.
- Remove config handler (also removes config from files and command line).
- Remove Latin American Spanish (es-419) translation.
- Remove `LavalinkUpdatePlayer::new()`.
- Remove multi Lavalink node support.
- Remove public instance warning.
- Remove Spanish (es-ES) translation.

## [0.0.1-alpha.9] - 2024-09-25

### Added

- Add Debian 12 (Bookworm) support.

### Changed

- Update dependencies.
- Update to Rust 1.81.0.
- Update to Alpine 3.20.

## [0.0.1-alpha.8] - 2024-04-16

### Added

- Add an option to force auto-roll from messages.
- Add a warning about the public instance ending.
- Add Portuguese Brazil translation to the public instance warning.
- Disable auto-roll from messages when another roll bot is detected. ([#40](https://github.com/nashiradeer/hydrogen/issues/40))

### Fixed

- Avoid accidental rolls inside messages with a prefix. ([#39](https://github.com/nashiradeer/hydrogen/issues/39))
- Fix the embed footer icon URL.
- Panic doesn't finalize the program. ([#42](https://github.com/nashiradeer/hydrogen/issues/42))

## [0.0.1-alpha.7] - 2024-04-15

### Added

- Add a config option to enable public instance-only features.
- New project icon.

### Changed

- Change `lavalink-openj9` tag on the compose file to `v3-update-lp`.
- Change the default search provider from SoundCloud to YouTube.
- Change use the local `Dockerfile` on the compose file instead of pulling the image.
- Update CONTRIBUTING.md file
- Update the embed footer icon to the new one.

### Fixed

- Fix typos in the English translations. (Thanks to [@LemonyOwO](https://github.com/LemonyOwO))
- Fix typos in the README.md. (Thanks to [@LemonyOwO](https://github.com/LemonyOwO))

## [0.0.1-alpha.6] - 2024-03-30

### Added

- Create the Track Stuck event for Lavalink.
- Create the Track Exception event for Lavalink.
- Implement logging handlers for Track Stuck and Track Exception.

### Changed

- Change the default search provider from YouTube to SoundCloud.
- Change the debug level from error to warn for errors during roll result send.

## [0.0.1-alpha.5] - 2024-03-12

### Changed

- Change 'rustls' to 'native-tls'. (Issue #38)

### Fixed

- LavalinkConfig not parsing hostname as address.

## [0.0.1-alpha.4] - 2024-03-12

### Added

- Create a link from ES-419 (LATAM) to ES-ES.
- Create a message handler to roll dices.
- Create a roll engine.
- Create a roll syntax parser.
- Create '/about' command.
- Create '/roll' command.
- New default 'builtin-language' feature to include 'assets/langs/en-US.json' in the binary as default language.
- New config parser.
- Old component message auto remover.

### Changed

- Decrease update_voice_server and update_voice_state logs spamming, ignoring them when nothing has been occurred.
- Replace internal i18n with 'hydrogen-i18n'.
- Resume when Player::play() is called when Lavalink is playing nothing.
- New command register and handler.
- New component handler.

### Fixed

- Change from '\d' to '[0-9]' to avoid Regex match non-ASCII digits.

## [0.0.1-alpha.3] - 2023-01-08

### Added

- Create de-DE translation.
- Create es-ES translation.
- Create a HashMap with command's IDs.

### Changed

- Refactor en-US translation.
- Refactor error messages.
- Refactor log messages.
- Refactor translation keys.
- Update pt-BR translation.

### Fixed

- Missing variable value in JoinCommand. (Issue #15)
- Wrong translation key in LoopComponent. (Issue #16)
- Wrong translations variables in HydrogenManager. (Issue #16)
