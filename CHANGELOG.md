# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/) 

<!---
## [Template]
### Added
 - for new features.
### Changed
 - for changes in existing functionality.
### Deprecated
 - for once-stable features removed in upcoming releases.
### Removed
 - for deprecated features removed in this release.
### Fixed
 - for any bug fixes.
### Security 
 - to invite users to upgrade in case of vulnerabilities.
-->

## [Unreleased]
### Added
 - Changelog

## Fixed
 - IPv6 incompability

## [0.1.2] - 2016-08-23
### Added
 - Readme
 - Show latency in the interface
 - Travis integration for testing in different rust versions
 - Clock countdown between move updates
 - Ability to make moves in games
   - Only supports four character coordinates like e2e4

### Changed
 - Display opponent name in games list instead of id

## Fixed
 - Sending the correct useragent everywhere
 - Real fix for latency, now 30ms as expected from here
   - Replaced crate rust-websocket, using ws instead

## [0.1.1] - 2016-08-10
### Added
 - Internal ping measuring (sends it to logfile)
 - Handling of socket message versions
 - Logging to file with correct log levels, stdout is used for rendering
 - Session handling
   - Menu lists your current games if logged into a named account
   - Username/password is entered from stdin
   - Can use anonymous with blank username
 - Menu for opening multiple tv:s
   - All the different tv chanlles can now be accessed without recompiling

### Fixed
 - One issue with the latency
   - Real ping is 30ms, but liru got 100ms, now 70ms by using `set_nodelay(true)`
 - Board orientation, pieces and clock on correct side
 - Decode error, moves is not available anymore
 - All compile warnings

## [0.1.0] - 2015-07-13
### Added
 - Keyboard shortcut to quit: q
 - Render player name and rating on game
 - Render clock updates on moves
 - Simple watch mode, where you can follow moves live
   - Hardcoded game/tv url, recompile to watch something else
 - MIT license


[Unreleased]: https://github.com/flugsio/liru/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/flugsio/liru/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/flugsio/liru/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/flugsio/liru/compare/a1443b908a...v0.1.0

