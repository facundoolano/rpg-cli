# Changelog

## Unreleased
### Added
* a `rpg reset --hard` flag to remove data files and forget information from previous plays #46
* Quest system #47
* Tutorial quests #49
* `rpg ls` command to look for chests at the current location #51

### Changed
* Tombstones are found with `rpg ls` instead of automatically #52

## [0.4.0](https://github.com/facundoolano/rpg-cli/releases/tag/0.4.0) - 2021-06-05
### Added
* This Changelog
* `rpg cd -f` sets the hero location without initiating battles, intended for custom shell integrations
* `rpg battle` initiates a battle (with a probability) at the hero's current location.
* --quiet,-q option to reduce output while changing directories and printing the hero status.
* --plain to facilitate scripting around the hero stats.
* Documentation for shell integrations.

### Changed
* General command overhaul, now all actions are done via a subcommand: `rpg cd`, `rpg stat`, etc., with status printing being the default.
* `rpg cd` without args moves the hero to home and `rpg cd -` moves it to `$OLDPWD` (when present) to match the `cd` behavior 4ba4c59
* --shop,-s renamed to buy,b and --inventory,-i renamed to use,u f737a81
* Removed most empty lines from output.

## [0.3.0](https://github.com/facundoolano/rpg-cli/releases/tag/0.3.0) - 2021-05-28
### Added
* Binary upload from travis on GitHub releases #36
* Experimental support for windows #35
* Different OS tests in travis 3a7eb6b

### Changed
* Print version number in help 8efdead
* Rebalancing of character stats to prevent overgrowth #33
* Several updates to the README instructions

### Fixed
* Prevent overflow bug at high levels #33
* Keep items sorted when printing the character status #15
* Missing Cargo.lock checked into the repository #26

## [0.2.0](https://github.com/facundoolano/rpg-cli/releases/tag/0.2.0) - 2021-05-23

## [0.1.0](https://github.com/facundoolano/rpg-cli/releases/tag/0.1.0) - 2021-05-06
