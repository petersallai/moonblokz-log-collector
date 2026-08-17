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
The collector is one of four MoonBlokz components (probe, telemetry hub, log collector, CLI). It polls the hub’s `/download` endpoint over HTTPS, appends log lines to a local file, and keeps an in-memory cursor only (no persistence between runs).

### Upstream Network Context (MoonBlokz Series part VII/2-5, Nov 2025)
- Source references (MoonBlokz Medium series):
  - `https://medium.com/moonblokz/moonblokz-series-part-vii-2-mesh-radio-algorithm-3650af3711f3`
  - `https://medium.com/moonblokz/moonblokz-series-part-vii-3-inside-the-radio-module-d92545624d2b`
  - `https://medium.com/moonblokz/moonblokz-series-part-vii-4-radio-network-simulation-5cc86a721e8c`
  - `https://medium.com/moonblokz/moonblokz-series-part-vii-5-field-testing-infrastructure-6be10e18796c`
- MoonBlokz radio networking is intentionally best-effort and delay-tolerant, not guaranteed-delivery.
- Message propagation is adaptive (connection-aware relay delays + random jitter), so hub-side arrivals can be bursty rather than evenly spaced.
- Nodes may receive or recover data out of order (for example, child data before parent data) and later self-heal via explicit request messages.
- Only large blockchain payloads (`add_block`, `add_transaction`) are fragmented across packets; most control/protocol traffic is intentionally small and single-packet.
- Queue-based, bounded buffering is a design goal in the radio stack; overflow is handled by dropping data predictably rather than blocking indefinitely.
- Simulation guidance: topology depth (hop count) and per-node saturation are the dominant scaling bottlenecks, while many random link drops are tolerated if the network remains connected.
- Field testing architecture adds a telemetry feedback loop (`heartbeat`, `upload`, `download`) and validates using derived metrics such as end-to-end message latency and network connectivity ratio.
- Collector behavior should remain append-only and robust to non-uniform arrival timing from the hub.

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
- Requests: `GET {hub-url}/download?last_log_timestamp=<last_timestamp>` with `X-Api-Key`.
- `last_timestamp` starts at Unix epoch each run; update to the newest valid timestamp in the response.
- Hub responses may include an `update_interval` value; collector loops should honor this server-directed pacing when available.
- Log file format is append-only: `<timestamp>:<message>` with UTC ISO 8601 timestamps.
- Do not assume evenly spaced upstream events; polling and append logic must tolerate bursts and sparse periods equally.
- Keep collector logic transport-agnostic: radio/link details are upstream concerns; collector responsibilities are accurate retrieval, cursor advancement, and durable append.

## Commit & Pull Request Guidelines
- Commit messages in history are short, sentence-case summaries without prefixes (e.g., “Refactor configuration …”). Follow that pattern.
- PRs should describe the change, include the relevant config/log format updates, and note any behavioral changes to polling or error handling.

## Security & Configuration Tips
- Do not commit `config.toml` or API keys; use `config.toml.example` as the template.
- Log files are append-only; avoid writing secrets to logs.
