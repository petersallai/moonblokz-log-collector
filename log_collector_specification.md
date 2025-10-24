# Specification of MoonBlokz log collector

The purpose is this application is to periodically download new log items from the MoonBlokz Telemetry HUB.

Architecture: It is a Rust command line application (use Tokio if it neccessary).

Command line parameters:
- api-key: The API key that must sent with all requestst as a header
- moonblokz-telemetry-hub url: The base of moonblokz-telemetry-hub
- log filename
- request time: the time interval between two requests (in seconds)

Working model:
- at startup the app creates the log files (if not exist) or append it (if it exits)



- The moonblokz-telemetry-hub is accessible via HTTPS get requests
- in every 'request time' seconds  the application call the server url with a get request. It sends one query parameter (last_log_message_id). This parameter is 0 for the first request. After the first request it is the maximal identifier that arrived from the server.
- the responde is a json file. In this json there is an array of log_items. For every log item the data is:
  - log_message_id (u64)
  - timestamp
  - log_line: The content of the log line
- the application write the log lines to a file (first the timestamp, next the log line, the identifier is not logged)