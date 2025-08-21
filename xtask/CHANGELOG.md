# Changelog - xtask

## [Unreleased] - exchange-crate branch

### Added
- New `CHANGELOG.md` file to track changes in the branch.
- `hello_exchanges` function in `exchanges` for integration testing.
- Temporary stubs for `CallbackEvent`, `OrderSide`, and `ProfitError` in `exchanges::temp::profitdll` to allow compilation during refactor.

### Changed
- Workspace modularization: created the `exchanges` crate to house concrete exchange integrations.
- Removed concrete implementations and generics from `toucan-data`, leaving only abstractions.
- Updated imports and dependencies in all crates to reflect the new architecture.
- Added `pub mod temp;` in `exchanges` to expose temporary integrations.
- Updated examples and binaries to use the new import paths.

### Removed
- Removed all modules and references to old `brokers` and `profitdll`.
- Removed all generics and trait bounds from builders in `toucan-data`.
- Removed concrete exchange implementations from `toucan-data`.

### Fixed
- Fixed import errors and cyclic dependencies after the migration.
- Fixed trait and type conflicts after crate separation.
- Adjusted stubs to eliminate temporary compilation errors.

---

> This changelog covers all changes made in the `exchange-crate` branch since the last main commit. Update as needed before merging.
