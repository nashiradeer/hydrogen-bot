# Hydrogen // TODO list

## 0.0.1-alpha.13

- [x] Update dependencies. (if needed)
- [x] Create a feature for Lavalink can use simd-json instead of serde_json.
- [x] Migrate to `dynfmt` crate instead of a lot of `String::replace`.
- [x] Migrate to responses using plain text instead of embeds.
- [x] Delete Player Message when there is no player in the
  guild. ([#10](https://github.com/nashiradeer/hydrogen-bot/issues/10))
- [x] Rework seek to show current time and total time when no arguments are provided.
- [x] Rename seek to time.
- [x] Rework join to have templates for the Music Player.
- [x] Rework play to choose if the music should be played now, next or at the end of the queue.
- [x] Remove PlayTogether from the Music Player.
- [x] Filter Lavalink's Stats and PlayerUpdate messages from logs.
- [x] Use songbird ConnectionInfo instead of store VoiceState.

## 0.0.1-alpha.14

- [ ] Update dependencies. (if needed)
- [ ] Create the shuffle component.
- [ ] Create commands for the Music Player's components.
- [ ] Create the about command.
- [ ] Create the donate command.
- [ ] Create a command to set Music Player language.
- [ ] Implement auto-play loop mode.

## 0.0.1-alpha.15

- [ ] Update dependencies. (if needed)
- [ ] Create the queue-view command.
- [ ] Create the queue-select command.
- [ ] Create the queue-clear command.
- [ ] Create the queue-remove command.

## 0.0.1-alpha.16

- [ ] Update dependencies. (if needed)
- [ ] Implement a database system (using Diesel) for the collection system.
- [ ] Create the collection-create command.
- [ ] Create the collection-view command.
- [ ] Create the collection-delete command.
- [ ] Create the collection-select command.
