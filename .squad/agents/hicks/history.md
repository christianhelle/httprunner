# Project Context

- **Owner:** Christian Helle
- **Project:** httprunner
- **Stack:** Rust 1.92+ workspace (edition 2024), reqwest, clap, tokio, egui/eframe, ratatui/crossterm, wasm32, GitHub Actions
- **Created:** 2026-03-20T17:28:26.715Z

## Learnings

- Hicks owns CI, packaging, installers, Docker, and release path review.
- The project ships CLI, GUI, and TUI binaries and also supports a WASM GUI deployment path.
- Platform review should watch for brittle workflows, missing prerequisites, and release automation gaps.
- Rust 1.92+ edition 2024 is not widely documented—creates contributor friction.

## 2026-03-20: Audit Findings - Platform & Release Path Issues

Four platform and release findings reported to GitHub:

1. **P2: install.sh uses fragile JSON parsing** — Uses grep/regex instead of jq to parse GitHub API. Breaks silently on API format changes.

2. **P2: release.yml mutates main without PR approval** — Commits snapcraft.yaml version directly to main with no review gates or audit trail.

3. **P2: TUI Docker variant lacks integration testing** — publish-docker-tui.yml exists but release.yml's publish-tui-container job has no explicit verification that binary is in artifact or build succeeds.

4. **P3: Rust 1.92+ edition 2024 not documented as MSRV** — No README mention that edition 2024 requires very recent Rust. Contributes contributor friction.
