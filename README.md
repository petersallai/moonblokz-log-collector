# MoonBlokz Log Collector

A command-line application that periodically downloads log entries from the MoonBlokz Telemetry Hub and writes them to a local file for analysis.

## Overview

The Log Collector is part of the MoonBlokz Test Infrastructure. It connects to the Telemetry Hub via HTTPS and downloads new log entries at regular intervals. The logs are appended to a local file in a simple text format suitable for offline or online analysis.

## Features

- **Periodic polling**: Downloads logs from the hub at configurable intervals (default: 60 seconds)
- **Incremental downloads**: Maintains state to only download new logs since the last request
- **Append-only log file**: Writes logs in a simple `timestamp:message` format
- **Error handling**: 
  - Terminates on authentication failures (401)
  - Retries on temporary errors (400, 5xx)
  - Handles network timeouts and DNS errors
- **HTTPS support**: All communication uses secure HTTPS connections
- **Configurable**: Settings loaded from a TOML configuration file

## Installation

### Prerequisites

- Rust 1.70 or later
- Internet connection to download dependencies

### Building from source

```bash
# Clone the repository (if not already done)
git clone <repository-url>
cd moonblokz-log-collector

# Build the application
cargo build --release

# The binary will be available at ./target/release/moonblokz-log-collector
```

## Configuration

Create a `config.toml` file in the same directory as the executable (or use the `--config` flag to specify a different location):

```toml
# API key for authenticating with the telemetry hub
api-key = "your-api-key-here"

# Base URL of the telemetry hub (without /download suffix)
hub-url = "https://your-hub-url.com"

# Path to the output log file
log-file = "moonblokz-logs.txt"

# Polling interval in seconds (default: 60)
interval = 60

# Optional: Maximum number of items to retrieve per request
# max-items = 10000
```

### Configuration Options

| Option | Type | Required | Default | Description |
|--------|------|----------|---------|-------------|
| `api-key` | string | Yes | - | API key for hub authentication (X-Api-Key header) |
| `hub-url` | string | Yes | - | Base URL of the telemetry hub |
| `log-file` | string | Yes | - | Path to the output log file |
| `interval` | integer | No | 60 | Seconds between download attempts |
| `max-items` | integer | No | - | Maximum items per request (hub-dependent) |

## Usage

### Basic usage

```bash
# Run with default config file (config.toml in current directory)
./moonblokz-log-collector

# Specify a custom config file
./moonblokz-log-collector --config /path/to/config.toml
```

### Command-line options

```bash
moonblokz-log-collector [OPTIONS]

Options:
  -c, --config <FILE>    Path to configuration file [default: config.toml]
  -h, --help            Print help information
```

## Log File Format

The collector writes logs to a plain text file with one log entry per line:

```
2025-10-23T18:00:00Z:[INFO] Node initialised
2025-10-23T18:00:05Z:[DEBUG] Packet received from peer
2025-10-23T18:00:10Z:[WARN] Signal strength low
```

Each line has two fields separated by a colon:
- **timestamp**: ISO 8601 UTC timestamp
- **message**: Original log message including level prefix

## Protocol

The collector communicates with the Telemetry Hub using the following protocol:

### Download Request

```
GET /download?last_log_message_id=<id> HTTP/1.1
Host: <hub-url>
X-Api-Key: <api-key>
```

### Download Response (200 OK)

```json
{
  "logs": [
    {
      "item_id": 42,
      "timestamp": "2025-10-23T18:00:00Z",
      "message": "[INFO] Node initialised"
    },
    {
      "item_id": 43,
      "timestamp": "2025-10-23T18:00:05Z",
      "message": "[DEBUG] Packet received from peer"
    }
  ]
}
```

## Error Handling

The collector handles various error conditions:

- **401 Unauthorized**: Logs error and terminates (indicates misconfigured API key)
- **400 Bad Request**: Logs error and retries after the configured interval
- **5xx Server Error**: Logs error and retries after the configured interval
- **Network errors**: Logs error and retries on next interval
- **Malformed JSON**: Logs error and ignores the response
- **File write errors**: Logs error and terminates

## State Management

The collector maintains a `last_id` variable in memory that tracks the highest `item_id` received from the hub. This ensures that only new logs are downloaded on subsequent requests. 

**Note**: The `last_id` is not persisted to disk. When the collector is restarted, it begins from `last_id = 0`, which means it will re-download all available logs from the hub.

## Development

### Project Structure

```
moonblokz-log-collector/
├── Cargo.toml              # Rust dependencies and project metadata
├── config.toml             # Configuration file (not in git)
├── config.toml.example     # Example configuration
├── src/
│   ├── main.rs            # Main application logic
│   └── config.rs          # Configuration parsing
└── README.md              # This file
```

### Running in development mode

```bash
cargo run -- --config config.toml
```

### Running tests

```bash
cargo test
```

## Dependencies

- **tokio**: Async runtime
- **reqwest**: HTTP client with JSON support
- **serde**: Serialization/deserialization
- **serde_json**: JSON parsing
- **toml**: TOML configuration parsing
- **clap**: Command-line argument parsing
- **anyhow**: Error handling
- **thiserror**: Custom error types

## License

See LICENSE file for details.

## Related Components

This log collector is part of the MoonBlokz Test Infrastructure, which includes:

- **MoonBlokz Probe**: Rust daemon running on Raspberry Pi Zero that collects logs from RP2040 nodes
- **Telemetry Hub**: WASI/WASM service that receives logs from probes and serves them to collectors
- **Telemetry CLI**: Command-line tool for sending commands to probes via the hub
- **Log Collector**: This component (downloads logs from hub to local file)

## Support

For issues, questions, or contributions, please refer to the main project repository.
A log collector command line application that connects to the moonblokz-telemetry-hub
