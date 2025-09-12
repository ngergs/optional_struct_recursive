# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3](https://github.com/ngergs/optionable/compare/optionable_derive-v0.1.2...optionable_derive-v0.1.3) - 2025-09-12

### Fixed

- adjust Optionable-impl for Option

## [0.1.2](https://github.com/ngergs/optionable/compare/optionable_derive-v0.1.1...optionable_derive-v0.1.2) - 2025-09-10

### Added

- 'required' helper attribute

### Fixed

- docs

## [0.1.1](https://github.com/ngergs/optionable/compare/optionable_derive-v0.1.0...optionable_derive-v0.1.1) - 2025-09-09

### Fixed

- derive readme url typo

## [0.1.0](https://github.com/ngergs/optionable/releases/tag/optionable_derive-v0.1.0) - 2025-09-09

### Added

- add serde helper attributes to derived structs to skip serializing Option::None

### Fixed

- clippy
- use darling default attribute for derive macro implementation
- keep visibility same for derived optional structs/enums
- handle visibility modifier in derive

### Other

- readme
- prepare publish
- document similar crates
- docs
- rename to optionable
