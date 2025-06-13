# CI/CD Pipeline Setup for HTTP Runner

## Overview

This document describes the complete CI/CD pipeline setup for the HTTP Runner project using GitHub Actions. The pipeline provides automated building, testing, security scanning, and deployment across multiple platforms.

## Pipeline Components

### 1. Continuous Integration (`.github/workflows/build.yml`)

**Triggers:**
- Push to `main` and `develop` branches
- Pull requests to `main` branch

**Jobs:**
- **Test Job**: Runs on Ubuntu, Windows, and macOS
  - Sets up Zig 0.14.0
  - Caches build artifacts
  - Checks code formatting with `zig fmt --check`
  - Builds project in debug and release modes
  - Runs unit tests

- **Lint Job**: Additional code quality checks
  - Code formatting validation
  - Static analysis

- **Security Job**: Security vulnerability scanning
  - Trivy filesystem scanner
  - Results uploaded to GitHub Security tab

### 2. Release Pipeline (`.github/workflows/release.yml`)

**Triggers:**
- Git tags matching `v*` pattern
- Manual workflow dispatch

**Jobs:**
- **Build Job**: Cross-platform binary compilation
  - Linux x86_64
  - Windows x86_64
  - macOS x86_64 and ARM64
  - Creates compressed archives for each platform

- **Release Job**: Automated GitHub releases
  - Downloads all build artifacts
  - Generates changelog
  - Creates GitHub release with binaries
  - Supports both stable and pre-release versions

- **Container Job**: Docker image publishing
  - Builds container image with Linux binary
  - Publishes to GitHub Container Registry
  - Tagged with version and metadata

### 3. Security Analysis (`.github/workflows/codeql.yml`)

**Triggers:**
- Push to `main` branch
- Pull requests to `main` branch
- Weekly schedule (Mondays at 2 AM)

**Features:**
- CodeQL static analysis for Zig code
- Automated security issue detection
- Integration with GitHub Security tab

### 4. Dependency Management (`.github/workflows/dependency-update.yml`)

**Triggers:**
- Weekly schedule (Mondays at 4 AM)
- Manual workflow dispatch

**Features:**
- Automated Zig version checking
- Updates workflow files with latest Zig version
- Creates pull requests for updates
- Validates builds with new versions

### 5. Test Workflow (`.github/workflows/test.yml`)

**Manual testing workflow for validating CI/CD setup:**
- Selective testing (build, test, format, or all)
- Multi-platform validation
- Integration testing with example files
- Comprehensive test result summary

## Development Tools

### Development Setup Script (`dev-setup.ps1`)

PowerShell script for local development:

```powershell
.\dev-setup.ps1 -Install -Build -Test  # Full setup
.\dev-setup.ps1 -Format                # Format code
.\dev-setup.ps1 -Clean -Build          # Clean build
```

**Features:**
- Zig installation validation
- Project building and testing
- Code formatting
- Build artifact cleanup
- Colored output and progress indicators

### Docker Support

**Dockerfile features:**
- Alpine Linux base for minimal size
- Non-root user execution
- CA certificates for HTTPS requests
- Single binary deployment

**Container usage:**
```bash
# Build image
docker build -t httprunner .

# Run container
docker run --rm httprunner examples/basic.http
```

## GitHub Repository Configuration

### Issue Templates
- **Bug Report** (`.github/ISSUE_TEMPLATE/bug_report.yml`)
- **Feature Request** (`.github/ISSUE_TEMPLATE/feature_request.yml`)

### Pull Request Template
- **PR Template** (`.github/pull_request_template.md`)
- Standardized change categorization
- Testing checklist
- Code quality verification

### Documentation
- **CONTRIBUTING.md**: Contribution guidelines
- **SECURITY.md**: Security policy and reporting
- **CHANGELOG.md**: Version history and changes

## Status Badges

The README includes status badges for:
- Build pipeline status
- Release pipeline status
- Security scanning status

```markdown
[![Build](https://github.com/christianhelle/httprunner/actions/workflows/build.yaml/badge.svg)](https://github.com/christianhelle/httprunner/actions/workflows/build.yaml)
[![Release](https://github.com/christianhelle/httprunner/actions/workflows/release.yml/badge.svg)](https://github.com/christianhelle/httprunner/actions/workflows/release.yml)
[![Security](https://github.com/christianhelle/httprunner/actions/workflows/codeql.yml/badge.svg)](https://github.com/christianhelle/httprunner/actions/workflows/codeql.yml)
```

## Usage Workflows

### For Contributors

1. **Setup Development Environment:**
   ```powershell
   .\dev-setup.ps1 -Install
   ```

2. **Make Changes:**
   ```powershell
   # Make code changes
   .\dev-setup.ps1 -Format -Build -Test
   ```

3. **Submit PR:**
   - Create feature branch
   - Push changes
   - CI automatically runs on PR

### For Maintainers

1. **Create Release:**
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. **Monitor Pipelines:**
   - Check GitHub Actions tab
   - Review security alerts
   - Merge dependency updates

### For Users

1. **Download Binaries:**
   - Visit GitHub Releases page
   - Download platform-specific archive
   - Extract and run

2. **Use Container:**
   ```bash
   docker run ghcr.io/christianhelle/httprunner:latest --help
   ```

## Pipeline Benefits

1. **Quality Assurance:**
   - Automated testing on multiple platforms
   - Code formatting enforcement
   - Security vulnerability detection

2. **Automated Releases:**
   - Cross-platform binary generation
   - Semantic versioning support
   - Container image publishing

3. **Developer Experience:**
   - Fast feedback on changes
   - Standardized development workflow
   - Automated dependency management

4. **Security:**
   - Regular security scanning
   - Automated vulnerability detection
   - Secure container images

## Troubleshooting

### Common Issues

1. **Build Failures:**
   - Check Zig version compatibility
   - Verify code formatting with `zig fmt`
   - Review build logs in Actions tab

2. **Test Failures:**
   - Run tests locally: `zig build test`
   - Check for platform-specific issues
   - Review test output in CI logs

3. **Release Issues:**
   - Ensure proper tag format (`v*`)
   - Check binary compilation across platforms
   - Verify container build process

### Getting Help

- Check existing issues and documentation
- Create issue with appropriate template
- Review CI logs for detailed error information
- Use development setup script for local testing

This CI/CD pipeline provides a robust foundation for developing, testing, and releasing the HTTP Runner project while maintaining high code quality and security standards.
