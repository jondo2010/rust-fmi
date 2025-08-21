# Changelog

All notable changes to the `fmi-test-data` crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-08-21

### Changed
- Upgraded Reference FMUs from version 0.0.29 to 0.0.39
- Refactored version management to use constants for easier future updates
- Made version constants public for external use

### Added
- New method `list_available_fmus()` to enumerate all available FMUs in the archive
- New method `version()` to get the current Reference FMUs version being used
- Comprehensive documentation with examples for all public methods
- Integration tests for multi-FMU access, version compatibility, and archive consistency
- Support for const-time string concatenation using `const_format` crate

### Improved
- Better error messages with more context
- Enhanced documentation for all public methods
- More robust test coverage including edge cases
- Cleaner code structure with centralized version management

## Previous Versions

Changes before this refactoring were not systematically tracked.
