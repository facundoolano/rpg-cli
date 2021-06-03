# Changelog

## Unreleased
### Added
* --battle initiates a battle (with a probability) at the hero's current location b349362
* --mv sets the hero location without initiating battles, intended for custom shell integrations 7ab2401
* --stat,-s prints the hero status 8da1090
*  This Changelog

### Changed
* `rpg` without args moves the hero to home and `rpg -` moves it to `$OLDPWD` (when present) to match the `cd` behavior 4ba4c59
* --shop,-s renamed to --buy,-b and --inventory,-i renamed to --use,-u f737a81
* always print hero location on cd command 05b661e

## [0.3.0](https://github.com/facundoolano/rpg-cli/releases/tag/0.2.0) - 2021-05-28
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
