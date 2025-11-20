# HTTP File Runner (Docker Image)

HTTP File Runner executes basic HTTP requests defined in `.http` files (the same format used by VS Code REST Client). Package your test suites in a portable Docker image to run them anywhere without installing Rust or additional tooling.

## Feature Highlights

- ✅ Run plain HTTP collections (REST-style requests, headers, query params, bodies)
- 🔁 Execute multiple `.http` files in a single invocation
- 🔍 `--discover` finds and runs every `.http` file under the mounted directory
- 🔐 `--insecure` toggle for self-signed certificates in non-production setups
- 📝 Built-in logging with `--log <file>` and banner suppression via `--no-banner`
- 📦 Multi-architecture container images (`linux/amd64`, `linux/arm64`)

## Image Details

- **Registry**: `docker pull christianhelle/httprunner[:tag]`
- **Default tag**: `latest` (always the newest release); semantic tags such as `v1.2.3` are also published
- **Entrypoint**: `httprunner` (container runs the CLI directly)

## Quick Start

```bash
# Pull the latest release
docker pull christianhelle/httprunner:latest

# Run against a .http file in the current directory
docker run -it --rm \
  --mount "type=bind,source=${PWD},target=/app,readonly" \
  christianhelle/httprunner:latest your-file.http
```

### Mounting Your Tests

- Bind-mount the folder that contains your `.http` files into the container (examples below use `/app`).
- A read-only mount is recommended so the container cannot modify your tests or secrets.
- Inside the container you can reference files exactly as they appear in `/app`.

## Create a Local Alias

```bash
alias httprunner='docker run -it --rm -v "${PWD}:/app:ro" christianhelle/httprunner'
```

Keep that alias in your shell profile so all commands below can be run just like the native binary.

## Usage Examples

```bash
# Run a single file
httprunner tests/basic.http

# Verbose output + log file
httprunner tests/basic.http --verbose --log results.txt

# Accept self-signed certificates (development only)
httprunner tests/https.http --insecure

# Run multiple files explicitly
httprunner tests/login.http tests/orders.http

# Discover and run every .http file under the mount point
httprunner --discover

# Discover with verbose logging and custom log destination
httprunner --discover --verbose --log discovery.log
```

## Docker Compose Example

```yaml
version: "3.8"
services:
  api-tests:
    image: christianhelle/httprunner:latest
    volumes:
      - ./tests:/app:ro
    command: ["--discover", "--verbose"]
```

## CI Usage (GitHub Actions Snippet)

```yaml
- name: Run HTTP contract tests
  run: |
    docker run --rm \
      -v ${{ github.workspace }}:/app:ro \
      christianhelle/httprunner:latest \
      --discover --log artifacts/results.log
```

## Tips

- Combine flags like `--verbose`, `--log`, `--no-banner`, or `--insecure` exactly as you would with a native install.
- Supply required environment variables via `docker run --env NAME=value` (or `--env-file`).
- Network access comes from the Docker host; confirm the container can reach any services under test.
