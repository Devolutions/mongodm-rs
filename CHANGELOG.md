# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
