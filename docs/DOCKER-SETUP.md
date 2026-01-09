# Docker Container Publishing Setup

This project automatically publishes Docker containers to both GitHub Container Registry (GHCR) and DockerHub when a new release is created.

## Container Registries

The container is published to:

- **GitHub Container Registry**: `ghcr.io/christianhelle/httprunner`
- **DockerHub**: `christianhelle/httprunner`

## Required Secrets

To enable DockerHub publishing, you need to configure the following repository secrets:

### DockerHub Secrets

1. **DOCKERHUB_USERNAME**: Your DockerHub username
2. **DOCKERHUB_TOKEN**: A DockerHub access token (not your password)

#### Creating a DockerHub Access Token

1. Log in to [DockerHub](https://hub.docker.com)
2. Go to Account Settings → Security
3. Click "New Access Token"
4. Give it a descriptive name (e.g., "GitHub Actions httprunner")
5. Select appropriate permissions (Read, Write, Delete)
6. Copy the generated token

#### Adding Secrets to GitHub Repository

1. Go to your GitHub repository
2. Click Settings → Secrets and variables → Actions
3. Click "New repository secret"
4. Add `DOCKERHUB_USERNAME` with your DockerHub username
5. Add `DOCKERHUB_TOKEN` with your DockerHub access token

## Container Tags

The workflow automatically creates the following tags:

- `latest` - Points to the most recent release
- `v1.2.3` - Exact version tag
- `v1.2` - Major.minor version
- `v1` - Major version only

## Usage

### Pull from GitHub Container Registry
```bash
docker pull ghcr.io/christianhelle/httprunner:latest
```

### Pull from DockerHub
```bash
docker pull christianhelle/httprunner:latest
```

### Run the container
```bash
docker run --rm -v $(pwd):/app christianhelle/httprunner:latest /app/examples/basic.http
```

## Networking Considerations

### Accessing Services on the Host Machine

When running httprunner in a Docker container, `localhost` refers to the container itself, not the host machine. To access services running on your host:

**macOS/Windows (Docker Desktop):**
```bash
# Use host.docker.internal in your .http files
# GET http://host.docker.internal:8080/api
docker run --rm -v $(pwd):/app christianhelle/httprunner:latest /app/test.http
```

**Linux:**

Option 1 - Use host networking:
```bash
docker run --rm --network=host -v $(pwd):/app christianhelle/httprunner:latest /app/test.http
```

Option 2 - Use host gateway (Docker 20.10+):
```bash
docker run --rm --add-host=host.docker.internal:host-gateway \
  -v $(pwd):/app christianhelle/httprunner:latest /app/test.http
```

### Example .http File for Host Services

```http
# For cross-platform compatibility, use variables
@hostname=host.docker.internal
@port=8080

GET http://{{hostname}}:{{port}}/api/users
```

For more networking details, see the main README.

## Workflow Trigger

Container publishing is triggered by:
- Creating a new release tag (e.g., `v1.0.0`)
- Manual workflow dispatch with version input

The workflow will:
1. Build the Linux binary
2. Create a Docker container with the binary
3. Push to both GHCR and DockerHub with appropriate tags
