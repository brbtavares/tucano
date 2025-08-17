
# Contributing to Tucano

Thank you for your interest in contributing! Some quick guidelines:


## Workflow
1. Fork & branch from `main`.
2. Use descriptive branches: `feat/`, `fix/`, `chore/`, `docs/`.
3. Run locally (or use pre-commit): `cargo fmt && cargo clippy -- -D warnings && cargo test`.
4. Open a PR with a clear description (context, motivation, changes).
5. Keep commits cohesive; squash if necessary.


## Code Style
- Formatting: `rustfmt` (config in `rustfmt.toml`).
- Lints: clippy with no warnings.
- Avoid unwrap/expect outside tests or initialization.


## Commits
Suggested (not strict):
```
feat(core): add support for X
fix(execution): fix bug in Y
chore(ci): update workflow
```


## Tests
- Add cases for new features or fixes.
- Prefer unit tests + minimal integration.


## Security
- Do not include real credentials (API keys, etc.).
- Report vulnerabilities privately first.


## Crate Publishing
- Coordinated via `./scripts/release.sh` script.
- Versions follow SemVer.
- The `tucano` façade aggregates stable releases.


## Discussions
Open issues for major proposals before implementing.


## Automating Local Checks

To avoid CI failures, enable the pre-commit hook:

```
ln -sf ../../scripts/pre_commit.sh .git/hooks/pre-commit
```

Runs on every commit:
- rustfmt (check)
- clippy (no warnings)
- cargo-deny (if installed)
- quick tests (`cargo test --lib`)

Run manually:
```
./scripts/pre_commit.sh
```

Install cargo-deny (once):
```
cargo install cargo-deny --locked
```

To speed up development: use only libs with `cargo test --workspace --lib`; before opening a PR, run full tests if necessary.

Welcome!
