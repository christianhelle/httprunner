# Docker Troubleshooting Guide

This guide helps you solve common issues when using httprunner with Docker.

## Table of Contents

- [Connection Issues](#connection-issues)
- [Localhost/Networking Problems](#localhostnetworking-problems)
- [File Access Issues](#file-access-issues)
- [Performance Issues](#performance-issues)

## Connection Issues

### Issue: "Connection refused" when accessing localhost

**Symptoms:**
```
❌ GET http://localhost:8080/api - Connection refused
```

**Cause:**
Inside a Docker container, `localhost` refers to the container itself, not your host machine.

**Solutions:**

#### macOS and Windows (Docker Desktop)

Replace `localhost` with `host.docker.internal`:

```http
# Instead of:
GET http://localhost:8080/api

# Use:
GET http://host.docker.internal:8080/api
```

#### Linux

**Option 1: Use host networking (recommended for simplicity)**

```bash
docker run -it --network=host \
  --mount "type=bind,source=${PWD},target=/app,readonly" \
  christianhelle/httprunner test.http
```

Your `.http` files can use `localhost` normally.

**Option 2: Use host gateway (Docker 20.10+)**

```bash
docker run -it --add-host=host.docker.internal:host-gateway \
  --mount "type=bind,source=${PWD},target=/app,readonly" \
  christianhelle/httprunner test.http
```

Then use `host.docker.internal` in your `.http` files.

**Option 3: Use your host's IP address**

Find your host's IP:
```bash
# Linux
hostname -I | awk '{print $1}'

# macOS
ipconfig getifaddr en0
```

Then use it in your `.http` files:
```http
GET http://192.168.1.100:8080/api
```

### Issue: DNS resolution failures

**Symptoms:**
```
❌ GET http://myservice:8080/api - Could not resolve host
```

**Cause:**
The container cannot resolve custom hostnames.

**Solutions:**

1. **Add custom DNS entries:**
```bash
docker run -it --add-host=myservice:192.168.1.100 \
  --mount "type=bind,source=${PWD},target=/app,readonly" \
  christianhelle/httprunner test.http
```

2. **Use docker-compose with service names:**
```yaml
version: '3'
services:
  api:
    image: my-api:latest
    ports:
      - "8080:8080"
  
  httprunner:
    image: christianhelle/httprunner:latest
    volumes:
      - ./:/app:ro
    command: /app/test.http
    depends_on:
      - api
```

In your `.http` file, use the service name:
```http
GET http://api:8080/users
```

## Localhost/Networking Problems

### Using Environment Variables for Portability

**Problem:** Your `.http` files only work on the host or only in Docker, not both.

**Solution:** Use environment files for different scenarios.

1. **Create `http-client.env.json`:**

```json
{
  "local": {
    "baseUrl": "http://localhost:8080"
  },
  "docker": {
    "baseUrl": "http://host.docker.internal:8080"
  },
  "production": {
    "baseUrl": "https://api.example.com"
  }
}
```

2. **Update your `.http` files to use variables:**

```http
@baseUrl=http://localhost:8080

GET {{baseUrl}}/api/users
```

3. **Run with the appropriate environment:**

```bash
# On host
httprunner test.http --env local

# In Docker
docker run -it --mount "type=bind,source=${PWD},target=/app,readonly" \
  christianhelle/httprunner test.http --env docker
```

### Testing Docker Container Networking

To verify your Docker networking setup works:

1. **Start a simple test server on your host:**

```bash
# Using Python
python3 -m http.server 8080

# Using Node.js (if you have http-server)
npx http-server -p 8080
```

2. **Create a test file:**

```http
# test-docker-network.http
GET http://host.docker.internal:8080/
```

3. **Run from Docker:**

```bash
# macOS/Windows
docker run -it --mount "type=bind,source=${PWD},target=/app,readonly" \
  christianhelle/httprunner test-docker-network.http

# Linux
docker run -it --network=host \
  --mount "type=bind,source=${PWD},target=/app,readonly" \
  christianhelle/httprunner test-docker-network.http
```

If you see `✅ GET http://host.docker.internal:8080/ - Status: 200`, networking is working!

## File Access Issues

### Issue: "File not found" errors

**Symptoms:**
```
Error: Unable to read file: /app/test.http
```

**Cause:**
The file path in the container doesn't match the mounted directory.

**Solutions:**

1. **Verify your mount is correct:**
```bash
# The source should be where your .http files are
docker run -it --mount "type=bind,source=${PWD},target=/app,readonly" \
  christianhelle/httprunner /app/yourfile.http
```

2. **Use absolute paths in the container:**
```bash
# If your file is in a subdirectory
docker run -it --mount "type=bind,source=${PWD},target=/app,readonly" \
  christianhelle/httprunner /app/tests/api.http
```

3. **Use discovery mode:**
```bash
# Let httprunner find all .http files
docker run -it --mount "type=bind,source=${PWD},target=/app,readonly" \
  christianhelle/httprunner --discover
```

### Issue: Cannot write log files

**Symptoms:**
```
Error: Permission denied when writing log file
```

**Cause:**
The container is running in read-only mode.

**Solution:**

Allow write access to the mount:
```bash
# Remove 'readonly' from mount
docker run -it --mount "type=bind,source=${PWD},target=/app" \
  christianhelle/httprunner test.http --log results.log
```

Or mount a specific writable directory for logs:
```bash
docker run -it \
  --mount "type=bind,source=${PWD},target=/app,readonly" \
  --mount "type=bind,source=${PWD}/logs,target=/logs" \
  christianhelle/httprunner /app/test.http --log /logs/results.log
```

## Performance Issues

### Issue: Slow request execution in Docker

**Cause:**
DNS resolution or network overhead in containerized environments.

**Solutions:**

1. **Use IP addresses instead of hostnames** (when possible)
2. **Use host networking on Linux** for better performance:
```bash
docker run -it --network=host \
  --mount "type=bind,source=${PWD},target=/app,readonly" \
  christianhelle/httprunner test.http
```

3. **Add DNS servers:**
```bash
docker run -it --dns 8.8.8.8 --dns 8.8.4.4 \
  --mount "type=bind,source=${PWD},target=/app,readonly" \
  christianhelle/httprunner test.http
```

## Getting More Help

If you're still experiencing issues:

1. **Enable verbose mode** to see detailed request/response information:
```bash
docker run -it --mount "type=bind,source=${PWD},target=/app,readonly" \
  christianhelle/httprunner test.http --verbose
```

2. **Check the Docker container logs:**
```bash
docker run -it --mount "type=bind,source=${PWD},target=/app,readonly" \
  christianhelle/httprunner test.http --verbose --log debug.log
```

3. **Test connectivity from inside the container:**
```bash
# Start an interactive shell in the container
docker run -it --entrypoint /bin/bash christianhelle/httprunner

# Then test connectivity
curl http://host.docker.internal:8080
```

4. **Report an issue** on GitHub with:
   - Your operating system
   - Docker version
   - The command you're running
   - Verbose output or logs
   - Example `.http` file (if possible)

## Additional Resources

- [Docker Networking Documentation](https://docs.docker.com/network/)
- [Docker Desktop Networking](https://docs.docker.com/desktop/networking/)
- [httprunner GitHub Issues](https://github.com/christianhelle/httprunner/issues)
