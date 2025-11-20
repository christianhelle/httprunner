# HTTP File Runner (Docker Image)

HTTP File Runner executes REST, GraphQL, and gRPC requests defined in `.http` files (the same format used by VS Code REST Client). The Docker image packages the compiled binary so you can run your collections anywhere without installing Rust or additional tooling.

## Image Details

- **Registry**: `docker pull christianhelle/httprunner[:tag]`
- **Default tag**: `latest` (points to the newest release); semantic version tags such as `v1.2.3` are also published
- **Architectures**: Multi-arch images for both `linux/amd64` and `linux/arm64`
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

## Common Commands

```bash
# Verbose output
docker run -it --rm -v "${PWD}:/app:ro" christianhelle/httprunner your-file.http --verbose

# Accept self-signed certificates (development only)
docker run -it --rm -v "${PWD}:/app:ro" christianhelle/httprunner your-file.http --insecure

# Save execution logs to a file
docker run -it --rm -v "${PWD}:/app:ro" christianhelle/httprunner your-file.http --log results.txt

# Run multiple files in one go
docker run -it --rm -v "${PWD}:/app:ro" christianhelle/httprunner tests/*.http

# Automatically discover every .http file under the mount point
docker run -it --rm -v "${PWD}:/app:ro" christianhelle/httprunner --discover
```

## Create a Local Alias

```bash
alias httprunner='docker run -it --rm -v "${PWD}:/app:ro" christianhelle/httprunner'
httprunner --discover --log discovery.log
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
- Ensure required environment variables or secrets are provided via `docker run --env` when your `.http` files reference them.
- Network access comes from the Docker host; confirm the container can reach any services under test.
