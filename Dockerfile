# Use a minimal base image
FROM ubuntu:24.04

# Install minimal dependencies including OpenSSL
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the pre-built binary from the artifacts directory
COPY artifacts/httprunner /usr/local/bin/httprunner
COPY artifacts/httprunner-tui /usr/local/bin/httprunner-tui

# Make it executable
RUN chmod +x /usr/local/bin/httprunner

# Set the entrypoint
ENTRYPOINT ["/usr/local/bin/httprunner"]

# Default command (can be overridden)
CMD ["--help"]
