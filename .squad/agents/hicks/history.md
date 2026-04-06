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
- **Baseline (2026-03-21):** Current branch passes all platform validation checks (format, linting, debug build, test suite, release build). 915 unit tests, 2.74 MB release binary on Windows. No platform blockers before parser changes.

## 2026-03-21: Pest.rs Migration Investigation Complete

### Finding: Pest.rs is mechanically feasible and recommended

**Key outcomes from platform & release review:**

1. **Dependency:** pest + pest_derive are compile-only (zero runtime footprint). MSRV 1.83.0 is less strict than current 1.92 (Rust 2024).

2. **Build Impact:** Compilation adds ~2-5s per target (proc-macro expansion). Native (Linux, macOS, Windows) and WASM targets all work. Cargo.lock grows but is already committed.

3. **CI/CD:** test.yml and release.yml run times increase by ~2-5s per job (acceptable). No new workflows or matrix entries needed.

4. **Cross-platform:** All release targets (x86_64-linux, x86_64-windows, x86_64-darwin, aarch64-darwin) work unchanged. WASM target verified locally.

5. **Distribution:** install.sh/install.ps1 unchanged. Docker unchanged. Binary size unaffected. Version management remains the same.

6. **Grammar File:** Existing `http-file.peg` is comprehensive and PEG-native. Direct translation to pest `.pest` syntax required. Grammar becomes source of truth (improves contributor onboarding).

7. **Gating Criteria:** Must run full test suite, end-to-end flows, and compile-time benchmark before merge.

**Detailed report:** See `.squad/agents/hicks/pest-migration-report.md`

**Recommendation to Christian Helle:**
- Assign to Bishop (grammar migration + parser rewrite)
- Assign to Lambert (expanded test coverage)
- Track compile-time impact post-merge
- Validate against gating criteria before release

📌 **Team update (2026-04-06T10:40:50Z):** Pest migration approved by squad. Hicks' platform validation findings confirmed: no build, CI, distribution, or artifact impact. Compilation +2-5s per target acceptable. Grammar file will be source of truth. Decisions merged and inbox cleared. Ready for Phase 1 grammar authoring. — decided by Ripley, Bishop, Lambert, Hicks

## 2026-03-20: Audit Findings - Platform & Release Path Issues

Four platform and release findings reported to GitHub:

1. **P2: install.sh uses fragile JSON parsing** — Uses grep/regex instead of jq to parse GitHub API. Breaks silently on API format changes.

2. **P2: release.yml mutates main without PR approval** — Commits snapcraft.yaml version directly to main with no review gates or audit trail.

3. **P2: TUI Docker variant lacks integration testing** — publish-docker-tui.yml exists but release.yml's publish-tui-container job has no explicit verification that binary is in artifact or build succeeds.

4. **P3: Rust 1.92+ edition 2024 not documented as MSRV** — No README mention that edition 2024 requires very recent Rust. Contributes contributor friction.
