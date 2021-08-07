# Changelog

## Unreleased

### Added
* Quest to beat your own shadow #86
* Easter egg quest  #87

### Fixed
* Reach level 50 and 100 unlock and reward 4128f75

## [0.6.0](https://github.com/facundoolano/rpg-cli/releases/tag/0.6.0) - 2021-08-04
### Added
* Customizable classes file #76
* Thief class and command to select player class #77
* Mage class, magic attacks and ether item #78
* Quests to raise 5 levels on each available player class #81
* Reach level 50 and level 100 quests #81
* Items rewarded on battle won #82

### Removed
* Backwards compatibility code for binary game data from v0.4.0 #75

### Changed
* `rpg reset --hard` removes datafile instead of entire .rpg dir 5adfb87
* Character speed contributes to run away success probability 4d6e1a3
* Initial stats are randomized 50af983
* Use GitHub actions instead of travis for CI and release building #80
* Change xp gained based on enemy class category #83
* Accept multiple items in buy and use commands #84

### Fixed
* Find chest quest not rewarded when finding a tombstone c0d62aa

## [0.5.0](https://github.com/facundoolano/rpg-cli/releases/tag/0.5.0) - 2021-06-26
### Added
* a `rpg reset --hard` flag to remove data files and forget information from previous plays #46
* Quest system #47
* Tutorial quests #49
* `rpg ls` command to look for chests at the current location #51
* Example sh file #54
* Poisoned and burning status effects #48

### Changed
* Tombstones are found with `rpg ls` instead of automatically #52

### Fixed
* When hero dies twice in the same location, tombstone chest contents
are merged instead of overridden #73

## [0.4.1](https://github.com/facundoolano/rpg-cli/releases/tag/0.4.1) - 2021-06-14
### Changed
* Game data is now serialized to JSON to allow extending it without breaking backwards compatibility.

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
