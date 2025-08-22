# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2025-08-22

### Added

- GitHub Copilot instructions with validated commands and timing (#106)
- GitHub Actions workflow for code coverage (#99)
- Additional model description and variable support in fmi-schema (#98)
- FMI2 model description structures and serialization
- Enhanced FMI3 variable definitions with additional attributes
- Codecov badge to README.md

### Changed

- **BREAKING**: Unified versioning across all crates to 0.5.0
- All crates (fmi, fmi-schema, fmi-sim, fmi-sys, fmi-test-data) now use synchronized versioning
- Simplified dependency management and release process
- Upgraded to Rust edition 2024
- Refactored FMI 3.0 traits and instance interfaces (#102)
  - Updated `Fmi3Model` trait with methods for instantiating ME, CS, and SE instances
  - Introduced new `traits` module for FMI 3.0 with `Common`, `ModelExchange`, `CoSimulation`, and `ScheduledExecution` interfaces
  - Extracted `GetSet` trait from `Common`
  - Reworked string and binary interfaces
  - Updated `get_event_indicators` method signatures to return Result types
- Updated dependencies:
  - bindgen to 0.72
  - built to 0.8  
  - cc to v1.2.33
  - zip to v4.3.0
  - itertools to 0.14
  - libloading to v0.8.6
  - fetch-data to 0.2
  - yaserde to 0.12
  - actions/checkout to v5
  - codecov/codecov-action to v5

### Fixed

- Outdated documentation example in fmi crate (#104)

### Removed

- Unnecessary `Import` type from `FmiInstance` trait

## [0.4.1] - 2024-10-30

### Added

- Add Windows and MacOS builds to GH CI (#66)
- Add renovate.json (#10)

### Changed

- Add PR Conventional Commit Validation workflow (#68)
- Update Cargo.lock (#64)
- Update Rust crate float-cmp to 0.10 (#60)
- Replace in-repo copies of fmi-standard header files with git submodules. (#62)
- Update Rust crate built to v0.7.4 (#50)
- Update Rust crate bindgen to 0.70 (#53)
- Update Rust crate libloading to v0.8.5 (#48)
- Update Rust crate url to v2.5.2 (#47)
- Update Rust crate cc to v1.1.21 (#46)
- Update Rust crate zip to v2 (#43)
- Use correctly represented resource paths in fmi2 and fmi3. (#54)
- Update Rust crate dependencies (#44)
- Update Rust crate rstest to 0.21 (#42)
- Update Rust crate anyhow to v1.0.86 (#36)
- Update Rust crate cc to v1.0.98 (#35)
- Update Rust crate semver to v1.0.23 (#37)
- Update Rust crate thiserror to v1.0.61 (#38)
- Update Rust crate libc to v0.2.155 (#40)
- Update Rust crate built to v0.7.3 (#41)
- Update Rust crate rstest to 0.19 (#30)
- Update Rust crate thiserror to v1.0.59 (#29)
- Update Rust crate test-log to v0.2.16 (#28)
- Update Rust crate libc to v0.2.154 (#27)
- Update Rust crate chrono to v0.4.38 (#26)
- Update Rust crate cc to v1.0.96 (#25)
- Update Rust crate built to v0.7.2 (#24)
- Update Rust crate assert_cmd to 2.0.14 (#12)
- Update Rust crate anyhow to 1.0.82 (#11)

## [0.4.0] - 2024-04-16

### Added

- Support FMI2.0 in `fmi-sim` (#9)
- Support output files in fmi-sim.
- Add functions to query number of continous state and event indicator values
- Add thiserror to crate root

### Changed

- Prepare fmi-sim for release, added bouncing_ball example
- Prepare for release
- Sim mostly working (#8)
- Initial ScheduledExecution interface
- Refactoring and error cleanup
- Almost there
- Almost there
- Switch to clap, ME work-in-progress
- Traits refactor (#7)
- Initial reference testing (#6)
- Fmi-check (#4)
- Total Refactor, support for FMI3 (#3)
- Use lfs in ci checkout
- Install lapack3 in ci workflow

### Fixed

- Fix ci workflow branch

## [0.2.2] - 2023-11-02

### Added

- Added workflows, devcontainer, cargo-dist
- Added gitpod config, fix gitlab-ci
- Added rustfmt.toml, applied
- Add CoSim doStep, var getters/setters, enumeration type

### Changed

- 0.2.2
- Bumped deps, removed gitlab config
- Update README.md
- Merge branch 'fixLog' into 'master'
- Don't reuse va_args
- * Determine FMI_PLATFORM path at compile-time, as done in FMILibrary.
- Merge branch '2023-update' into 'master'
- Updates for rust2021 edition, bump deps
- Fixed misnamed .gitpod.dockerfile
- Merge branch 'gitpod-config' into 'master'
- - Moved fmi_check into it's own crate
- Got rid of unecessary use of Rc
- Patch release 0.2.1
- Merge branch 'hugwijst-master-patch-99090' into 'master'
- Fix buffer overflow on large log messages.
- Bump version to 0.2.0
- Merge branch 'wip' into 'master'
- * Additional CS support in fmi_check example
- Ran rustfmt
- Merge branch '1-casting-error-prevent-compilation' into 'master'
- Resolve "casting error prevent compilation"
- Merge branch 'fix_build' into 'master'
- * Fix codecov from copy/paste
- * Added lfs to gitlab ci
- * Added .gitlab-ci.yml
- * Initial Gitlab import

[0.4.1]: https://github.com///compare/v0.4.0..v0.4.1
[0.4.0]: https://github.com///compare/v0.2.2..v0.4.0

<!-- generated by git-cliff -->
