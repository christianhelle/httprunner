# Use a minimal base image
FROM alpine:latest

# Install necessary runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    curl

# Create a non-root user
RUN addgroup -g 1001 -S httprunner && \
    adduser -S -D -H -u 1001 -s /sbin/nologin httprunner -G httprunner

# Copy the binary from the build artifacts
COPY artifacts/httprunner /usr/local/bin/httprunner

# Make the binary executable
RUN chmod +x /usr/local/bin/httprunner

# Switch to non-root user
USER httprunner

# Set the working directory
WORKDIR /app

# Expose a port if needed (adjust based on your application)
# EXPOSE 8080

# Set the entrypoint
ENTRYPOINT ["/usr/local/bin/httprunner"]

# Default command
CMD ["--help"]
