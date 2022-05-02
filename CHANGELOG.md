# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.8.2] 2022-05-02

### Changed

- Expose `bulk_update` function to the native `mongodb::Collection` via an Extension Trait `CollectionExt`

## [0.8.1] 2022-04-30

### Changed

- Fixed a typo in the `$subtract` operator

## [0.8.0] 2021-12-09

### Changed

- **BREAKING:** Update `mongodb` to `2.x`

## [0.7.0] 2021-05-01

### Changed

- **BREAKING:** `Lookup`, `Map` and `Cond` operators are now strongly typed

- `field!` macro now expand to `&'static str`even for nested fields

### Added

- `pipeline!` macro to conveniently build Vec of pipeline stages.
  Allow to more ergonomically use the new strongly typed operators (see **changed**)
  
- Added `LookupPipeline` operator, a strongly typed version of the `$lookup` operator using pipeline

## [0.4.4] 2020-11-26

### Added

- `field!` macro can insert `$` signs by prefixing with `@` (ie: `field!(@foo in Bar)`)

## [0.4.3] 2020-11-16

### Added

- Support to string operator $replaceOne and $replaceAll
