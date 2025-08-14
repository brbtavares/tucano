<!--
  Aggregated changelog for the tucano workspace.
  Conventions:
  - Keep an Unreleased section at the top.
  - Group notable changes under each crate/version.
  - Use imperative mood (Add, Fix, Change, Remove).
-->

# Changelog

## [Unreleased]
### Added
- (placeholder) Add new features here.
### Changed
- (placeholder)
### Fixed
- (placeholder)
### Security
- (placeholder)

---

## Published Releases

### tucano (facade)
- 0.1.0 – Initial facade crate providing unified re-exports & prelude.

### tucano-core
- 0.12.2 – Post-refactor baseline after crate renames and lint hardening.

### tucano-markets
- 0.3.1 – Published with renamed workspace prefix and metadata.

### tucano-integration
- 0.9.1 – Integration protocols & streaming published.

### tucano-data
- 0.10.1 – Data events & instruments under unified prefix.

### tucano-execution
- 0.5.3 – Execution layer publication after dependency alignment.

### tucano-macros
- 0.2.0 – Procedural macros adjusted to new naming scheme.

### tucano-analytics
- 0.1.0 – Initial analytics crate (algorithms, metrics, timing utilities).

### tucano-risk
- 0.1.0 – Initial risk checks crate.

### tucano-strategies
- 0.1.0 – Initial collection of reference trading strategy implementations.

### tucano-trader
- 0.1.0 – Trader orchestration crate published.

---

## Guidelines
1. Add entries under Unreleased while developing.
2. When releasing a crate, move relevant entries into a new version line under that crate with the release date (YYYY-MM-DD).
3. Keep entries concise and user-facing (avoid internal-only refactors unless impactful).
4. Always record security fixes in a Security subsection.

## Format Example
### tucano-core
- 0.12.3 (2025-08-14)
  - Add XYZ feature improving latency by N%.
  - Fix panic in ABC edge case.

---

<!-- End of file -->
