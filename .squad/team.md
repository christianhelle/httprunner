# Squad Team

> Rust workspace for parsing and executing `.http` files through a shared core, CLI, TUI, native GUI, and WASM GUI.

## Coordinator

| Name | Role | Notes |
|------|------|-------|
| Squad | Coordinator | Routes work, enforces handoffs and reviewer gates. |

## Members

| Name | Role | Charter | Status |
|------|------|---------|--------|
| Ripley | Lead / Architect | `.squad/agents/ripley/charter.md` | ✅ Active |
| Bishop | Core & CLI Engineer | `.squad/agents/bishop/charter.md` | ✅ Active |
| Vasquez | GUI / TUI / WASM Engineer | `.squad/agents/vasquez/charter.md` | ✅ Active |
| Ash | Security Engineer | `.squad/agents/ash/charter.md` | ✅ Active |
| Lambert | Tester & Performance Reviewer | `.squad/agents/lambert/charter.md` | ✅ Active |
| Hicks | Platform & Release Engineer | `.squad/agents/hicks/charter.md` | ✅ Active |
| Scribe | Session Logger | `.squad/agents/scribe/charter.md` | 📋 Silent |
| Ralph | Work Monitor | — | 🔄 Monitor |

## Coding Agent

<!-- copilot-auto-assign: false -->

| Name | Role | Charter | Status |
|------|------|---------|--------|
| @copilot | Coding Agent | — | 🤖 Coding Agent |

### Capabilities

**🟢 Good fit — auto-route when enabled:**
- Bug fixes with clear reproduction steps
- Test coverage additions or fixes
- Dependency bumps and routine maintenance
- Small isolated features with clear acceptance criteria
- README and documentation improvements

**🟡 Needs review — route with squad oversight:**
- Medium features with clear specs
- Refactors with good existing coverage
- Mechanical multi-file updates that follow an established pattern

**🔴 Not suitable — keep with squad members:**
- Architecture decisions
- Security-sensitive changes
- Performance-critical hot paths
- Multi-surface changes touching core plus UI or release flows

## Project Context

- **Owner:** Christian Helle
- **Project:** httprunner
- **Stack:** Rust 2024 workspace, reqwest, clap, tokio, egui/eframe, ratatui/crossterm, wasm32, GitHub Actions
- **Description:** Multi-surface HTTP file runner with parsing, request chaining, assertions, reporting, and interactive execution.
- **Created:** 2026-03-20

## Issue Source

- **Repository:** `christianhelle/httprunner`
- **Connection:** GitHub CLI (`gh`) authenticated as `christianhelle`
- **Default target:** New review findings should become prioritized GitHub issues assigned to Christian Helle
