# ohaconf

> A YAML-driven CLI wrapper for [oha](https://github.com/hatoo/oha) — the HTTP load testing tool.

`ohaconf` lets you define multiple load tests in a single YAML file and run them interactively or by name, with support for environment variables, weighted distribution, and comparative reports.

> 🤖 **This project was built with AI assistance ([Oz by Warp](https://www.warp.dev)).**

---

## Why?

`oha` is a great load testing tool but lacks a configuration file. You end up copy-pasting long commands or maintaining shell scripts. `ohaconf` solves this by letting you declare all your tests in a YAML file:

```yaml
tests:
  - name: dashboard
    url: "${BASE_URL}/web/clients/232/dashboard"
    requests: 100
    connections: 50
    headers:
      Authorization: "Bearer ${TOKEN}"
```

Then just run:

```sh
ohaconf run --test dashboard
```

---

## Requirements

- [oha](https://github.com/hatoo/oha) installed and available in `$PATH`
- Rust toolchain (to build from source)

---

## Installation

```sh
git clone https://github.com/AntonioHReyes/ohaconf.git
cd ohaconf
cargo install --path .
```

The `ohaconf` binary will be installed to `~/.cargo/bin/ohaconf`.

---

## Quick Start

**1. Create your config file (`ohaconf.yaml`):**

```yaml
env:
  BASE_URL: http://localhost:8080
  TOKEN: ${API_TOKEN}           # reads from your shell environment

defaults:
  requests: 200
  connections: 50
  headers:
    Authorization: "Bearer ${TOKEN}"
    Accept: application/json

tests:
  - name: health_check
    url: "${BASE_URL}/health"
    tags: [health]

  - name: dashboard
    description: Monthly client dashboard
    method: GET
    url: "${BASE_URL}/web/clients/232/dashboard"
    query_params:
      startDate: "2025-01-01"
      endDate:   "2025-01-31"
    requests: 100
    connections: 50
    weight: 70
    tags: [dashboard, read]

  - name: create_session
    method: POST
    url: "${BASE_URL}/api/sessions"
    headers:
      Content-Type: application/json
    body: |
      {"clientId": 232, "type": "standard"}
    requests: 50
    connections: 10
    weight: 30
    tags: [write]
```

**2. Export your token and run:**

```sh
export API_TOKEN=your-jwt-token

ohaconf list                        # see all tests
ohaconf run                         # interactive menu
ohaconf run --test dashboard        # run by name
```

---

## Commands

### `list`
Display all tests defined in the config file.

```sh
ohaconf list
ohaconf -f my-tests.yaml list
```

---

### `validate`
Validate the YAML file and environment variable expansion without executing anything.

```sh
ohaconf validate
```

---

### `run`
Run a single test. If no `--test` is given, an interactive menu is shown.

```sh
ohaconf run                              # interactive selector
ohaconf run --test dashboard             # by name
ohaconf run --tag read                   # filter by tag (interactive if multiple)
ohaconf run --test dashboard --dry-run   # print oha command without running
ohaconf run --test dashboard --pass-tui  # show oha's live TUI instead of capturing output
ohaconf run --test dashboard -o results.json
```

---

### `run-all`
Run all tests (or filtered by tag), sequentially or in parallel.

```sh
ohaconf run-all                          # sequential
ohaconf run-all --parallel               # parallel
ohaconf run-all --tag read               # only tests tagged "read"
ohaconf run-all --parallel -o results.json
```

---

### `run-weighted`
Run all tests in **parallel**, distributing requests proportionally according to each test's `weight` field.

```sh
ohaconf run-weighted -n 1000             # 1000 total requests distributed by weight
ohaconf run-weighted -z 60s              # all tests run for 60 seconds in parallel
ohaconf run-weighted -n 1000 -o results.json
```

---

### `export`
Print the equivalent `oha` command without executing it (dry-run). Sensitive headers are redacted by default.

```sh
ohaconf export --test dashboard
ohaconf export --test dashboard --show-secrets
```

---

### `report`
Read a saved JSON results file and generate a comparative report.

```sh
ohaconf report --input results.json
ohaconf report --input results.json --format csv -o report.csv
ohaconf report --input results.json --format json
```

---

## Configuration Reference

### Top-level fields

| Field      | Type              | Description                                      |
|------------|-------------------|--------------------------------------------------|
| `env`      | `map<string>`     | Variables used in `${VAR}` expansion             |
| `defaults` | object            | Default values applied to all tests              |
| `tests`    | list of TestCase  | The load tests to define                         |

### `defaults` fields

| Field               | Type     | Description                            |
|---------------------|----------|----------------------------------------|
| `requests`          | integer  | Default `-n` for oha (default: 200)    |
| `connections`       | integer  | Default `-c` for oha (default: 50)     |
| `duration`          | string   | Default `-z` (e.g. `"30s"`, `"2m"`)   |
| `headers`           | map      | Default headers merged into all tests  |
| `query_per_second`  | float    | Default `-q` for oha                   |
| `timeout`           | string   | Default request timeout                |
| `http2`             | bool     | Enable HTTP/2 by default               |
| `disable_keepalive` | bool     | Disable keep-alive by default          |
| `insecure`          | bool     | Skip TLS verification by default       |

### `TestCase` fields

| Field               | Type     | Required | Description                                                 |
|---------------------|----------|----------|-------------------------------------------------------------|
| `name`              | string   | ✅       | Unique identifier                                           |
| `url`               | string   | ✅       | Target URL (supports `${VAR}`)                              |
| `description`       | string   |          | Human-readable description                                  |
| `method`            | string   |          | HTTP method (default: `GET`)                                |
| `query_params`      | map      |          | Query parameters appended to the URL                        |
| `headers`           | map      |          | HTTP headers (merged on top of `defaults.headers`)          |
| `body`              | string   |          | Request body as inline string                               |
| `body_file`         | string   |          | Path to file used as request body (`-D` in oha)             |
| `requests`          | integer  |          | Number of requests (`-n`)                                   |
| `connections`       | integer  |          | Concurrent connections (`-c`)                               |
| `duration`          | string   |          | Test duration (`-z`), e.g. `"30s"`. Overrides `requests`   |
| `query_per_second`  | float    |          | Rate limit in QPS (`-q`)                                    |
| `timeout`           | string   |          | Per-request timeout (`-t`)                                  |
| `weight`            | float    |          | Weight for `run-weighted` distribution (default: `1.0`)     |
| `tags`              | list     |          | Tags for filtering with `--tag`                             |
| `http2`             | bool     |          | Use HTTP/2                                                  |
| `disable_keepalive` | bool     |          | Disable keep-alive                                          |
| `insecure`          | bool     |          | Accept invalid TLS certificates                             |

### Variable expansion

`${VAR}` is supported in: `url`, `headers` (values), `body`, `body_file`, `query_params` (values).

**Precedence (highest first):**
1. Shell environment variables (`export FOO=bar`)
2. Variables defined in `env:` block of the YAML

---

## Output example

When running multiple tests, `ohaconf` shows a comparative table:

```
────────────────────────────────────────────────────────────────────────────────────────────
TEST                           REQUESTS   SUCCESS%        RPS        AVG        P50        P95        P99     ERRORS
────────────────────────────────────────────────────────────────────────────────────────────
dashboard                           100     100.0%       1423     12.3ms     11.8ms     18.2ms     24.5ms          0
create_session                       50     100.0%        621     45.1ms     42.0ms     78.3ms    102.1ms          0
health_check                        500     100.0%       8201      3.1ms      2.9ms      5.2ms      8.4ms          0
────────────────────────────────────────────────────────────────────────────────────────────
```

---

## Saving and reading results

```sh
# Save results to JSON
ohaconf run-all --parallel -o results.json

# Read and display later
ohaconf report --input results.json

# Export as CSV
ohaconf report --input results.json --format csv -o report.csv
```

---

## Global flags

| Flag              | Short | Description                                    |
|-------------------|-------|------------------------------------------------|
| `--config <FILE>` | `-f`  | Config file path (default: `./ohaconf.yaml`)   |
| `--help`          | `-h`  | Show help                                      |
| `--version`       | `-V`  | Show version                                   |

---

## Security

- Headers containing `Authorization`, `Token`, `x-api-key` are **automatically redacted** in logs and `export` output.
- Use `--show-secrets` to reveal them when needed.
- Never hardcode tokens in YAML — use `${ENV_VAR}` references instead.

---

## Roadmap

- [ ] `--env KEY=VALUE` CLI overrides for variables
- [ ] Per-URL headers support in `--urls-from-file` mode (upstream oha feature request)
- [ ] Shell completion (`ohaconf completions bash/zsh/fish`)
- [ ] `ohaconf init` wizard to generate a starter config
- [ ] Contribute YAML config support upstream to [hatoo/oha](https://github.com/hatoo/oha)

---

## License

MIT
