# Repository Guidelines

## Project Structure & Module Organization
- `src/main.rs`: CLI entry point, HTTP polling loop, and log file append logic.
- `src/config.rs`: Config loading/validation from TOML.
- `config.toml.example`: Sample configuration to copy from.
- `config.toml`: Local config (gitignored; contains secrets).
- `moonblokz_test_infrastructure_full_spec.md`: system spec for the probe, hub, collector, and CLI (treat as source of truth).
- `IMPLEMENTATION.md`: implementation notes for this repo.
- `target/`: Cargo build output (ignored).

## Architecture Context
The collector is one of four MoonBlokz components (probe, telemetry hub, log collector, CLI). It polls the hub’s `/download` endpoint over HTTPS, appends log lines to a local file, and keeps `last_id` in memory only (no persistence between runs).

## Build, Test, and Development Commands
- `cargo build --release`: Build optimized binary at `target/release/moonblokz-log-collector`.
- `cargo run -- --config config.toml`: Run locally with a specific config file.
- `cargo test`: Run the Rust test suite (if/when tests exist).

## Coding Style & Naming Conventions
- Rust standard style and formatting (prefer `cargo fmt` if available).
- Module/file names are lowercase (`config.rs`), types use `PascalCase`, functions/vars use `snake_case`.
- Prefer idiomatic Rust and avoid unnecessary cloning where possible (per spec).
- Keep logging and error messages explicit; use the existing `log_info!`/`log_error!` macros for consistency.

## Testing Guidelines
- No dedicated `tests/` directory currently. Add unit tests in `src/*` with `#[cfg(test)]` or create `tests/` for integration tests.
- Name tests descriptively (e.g., `parses_config_with_missing_fields`).
- Run `cargo test` before submitting changes.

## Configuration & Protocol Notes
- `config.toml` keys (spec): `api-key`, `hub-url`, `log-file`, optional `interval` (seconds), optional `max-items` (<= 10,000 recommended).
- Requests: `GET {hub-url}/download?last_log_message_id=<last_id>` with `X-Api-Key`.
- `last_id` starts at 0 each run; update to highest `item_id` in the response.
- Log file format is append-only: `<timestamp>:<message>` with UTC ISO 8601 timestamps.

## Commit & Pull Request Guidelines
- Commit messages in history are short, sentence-case summaries without prefixes (e.g., “Refactor configuration …”). Follow that pattern.
- PRs should describe the change, include the relevant config/log format updates, and note any behavioral changes to polling or error handling.

## Security & Configuration Tips
- Do not commit `config.toml` or API keys; use `config.toml.example` as the template.
- Log files are append-only; avoid writing secrets to logs.
