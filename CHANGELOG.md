# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [[0.10.1](https://github.com/Devolutions/mongodm-rs/compare/v0.10.0...v0.10.1)] - 2025-12-16

### <!-- 1 -->Features

- Add bson3 feature ([#36](https://github.com/Devolutions/mongodm-rs/issues/36)) ([61d422e43f](https://github.com/Devolutions/mongodm-rs/commit/61d422e43ff251338bfdfd6caa11afcf6fff17b0)) 

  [mongo 3.4.1](https://docs.rs/crate/mongodb/3.4.1/features) uses bson 2
  by default via the `compat-3-0-0` feature. However, there is support for
  bson 3 in `compat-3-3-0`.
  
  This adds the ability to use bson 3 just like the upstream mongodb
  driver.

- Add openssl feature ([#37](https://github.com/Devolutions/mongodm-rs/issues/37)) ([044b904d93](https://github.com/Devolutions/mongodm-rs/commit/044b904d9351cf16858bc84caa4706f4012e4b97)) 

- Add chrono feature for bson ([#45](https://github.com/Devolutions/mongodm-rs/issues/45)) ([3fcd61b6ca](https://github.com/Devolutions/mongodm-rs/commit/3fcd61b6ca03cef4cf4c729a14ec909578e5206f)) 

  This is on top of #44.

### <!-- 4 -->Bug Fixes

- Separate bson-3 from compat-3-3-0 ([#44](https://github.com/Devolutions/mongodm-rs/issues/44)) ([65b11f9f46](https://github.com/Devolutions/mongodm-rs/commit/65b11f9f46a893e7a12abfb6aad106f363d15222)) 

  This is a correction to split `bson-3` into its own feature. Upstream
  `compat-3-3-0` does not actually imply `bson-3`.



## [0.9.0] 2022-05-02

### Changed

- **BREAKING:** Re-organized the public api surface to provide a cleaner documentation page

## [0.8.2] 2022-05-02

### Changed

- Expose `bulk_update` function to the native `mongodb::Collection` via an Extension Trait `CollectionExt`

### Fixed

- Fixed a typo in the `$subtract` operator

## [0.8.1] 2022-03-31

### Fixed

- Fixed an issue with selection_criteria on the `Repository::bulk_update` function

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
