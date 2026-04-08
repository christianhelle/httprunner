# Changelog

## [Unreleased Changes]

### Merged Pull Requests
- chore(deps): update docker/login-action action to v4 ([#207](https://github.com/christianhelle/httprunner/pull/207)) (@renovate[bot])
- chore(deps): update docker/setup-buildx-action action to v4 ([#208](https://github.com/christianhelle/httprunner/pull/208)) (@renovate[bot])
- chore(deps): update jetli/trunk-action action to v0.5.1 ([#242](https://github.com/christianhelle/httprunner/pull/242)) (@renovate[bot])
- chore(deps): update actions/deploy-pages action to v5 ([#237](https://github.com/christianhelle/httprunner/pull/237)) (@renovate[bot])
- chore(deps): update actions/checkout action to v6 ([#247](https://github.com/christianhelle/httprunner/pull/247)) (@renovate[bot])
- Update actions/github-script action to v8 ([#253](https://github.com/christianhelle/httprunner/pull/253)) (@renovate[bot])
- Update codecov/codecov-action action to v6 ([#254](https://github.com/christianhelle/httprunner/pull/254)) (@renovate[bot])
- Add PEG Grammer File ([#251](https://github.com/christianhelle/httprunner/pull/251)) ([@christianhelle](https://github.com/christianhelle/))
- Expand docs home meta description ([#250](https://github.com/christianhelle/httprunner/pull/250)) ([@christianhelle](https://github.com/christianhelle/))
- Setup agent squad ([#241](https://github.com/christianhelle/httprunner/pull/241)) ([@christianhelle](https://github.com/christianhelle/))

### Features
- Add deterministic CLI smoke tests ([#257](https://github.com/christianhelle/httprunner/pull/257)) ([@christianhelle](https://github.com/christianhelle/))
- Refactor .http file parsing to be PEG grammar based using pest ([#256](https://github.com/christianhelle/httprunner/pull/256)) ([@christianhelle](https://github.com/christianhelle/))
- Refresh Rust crates and adapt workspace code to current APIs ([#252](https://github.com/christianhelle/httprunner/pull/252)) (@Copilot)
- Harden installs and release publishing ([#249](https://github.com/christianhelle/httprunner/pull/249)) ([@christianhelle](https://github.com/christianhelle/))
- Harden runtime caching and output redaction ([#245](https://github.com/christianhelle/httprunner/pull/245)) ([@christianhelle](https://github.com/christianhelle/))
- Fix UI run lifecycle and request round-tripping ([#246](https://github.com/christianhelle/httprunner/pull/246)) ([@christianhelle](https://github.com/christianhelle/))
- Fix core parser and request failures ([#244](https://github.com/christianhelle/httprunner/pull/244)) ([@christianhelle](https://github.com/christianhelle/))
- Match WASM execution and trim GUI persistence overhead ([#248](https://github.com/christianhelle/httprunner/pull/248)) ([@christianhelle](https://github.com/christianhelle/))


## [0.9.51](https://github.com/christianhelle/httprunner/releases/tag/0.9.51) (2026-03-24)

### Merged Pull Requests
- Update actions/checkout action to v6 ([#236](https://github.com/christianhelle/httprunner/pull/236)) (@renovate[bot])
- Update docker/build-push-action action to v7 ([#210](https://github.com/christianhelle/httprunner/pull/210)) (@renovate[bot])
- Update docker/metadata-action action to v6 ([#209](https://github.com/christianhelle/httprunner/pull/209)) (@renovate[bot])
- Update GitHub Artifact Actions (major) ([#204](https://github.com/christianhelle/httprunner/pull/204)) (@renovate[bot])
- Fix build failure: add missing `quotes` field to `Syntax` initializer ([#206](https://github.com/christianhelle/httprunner/pull/206)) (@Copilot)
- Update Rust crate windows-sys to 0.61 ([#191](https://github.com/christianhelle/httprunner/pull/191)) (@renovate[bot])
- Sync static website docs with README - add TUI, timeouts, and missing CLI options ([#183](https://github.com/christianhelle/httprunner/pull/183)) (@Copilot)

### Features
- Add JSON export support for execution results (`--export-json`) ([#205](https://github.com/christianhelle/httprunner/pull/205)) (@Copilot)
- Hide support key when telemetry collection is disabled ([#203](https://github.com/christianhelle/httprunner/pull/203)) ([@christianhelle](https://github.com/christianhelle/))
- Fix missing windows icon on TUI and GUI apps ([#202](https://github.com/christianhelle/httprunner/pull/202)) ([@christianhelle](https://github.com/christianhelle/))
- Add scroll bounds validation to PageDown in environment editor ([#201](https://github.com/christianhelle/httprunner/pull/201)) (@Copilot)
- Add vertical scrolling support in the Environment Editor for the TUI app ([#200](https://github.com/christianhelle/httprunner/pull/200)) ([@christianhelle](https://github.com/christianhelle/))
- Introduce lorem_ipsum() built-in function ([#199](https://github.com/christianhelle/httprunner/pull/199)) ([@christianhelle](https://github.com/christianhelle/))
- Make Results view vertically resizable in GUI app ([#196](https://github.com/christianhelle/httprunner/pull/196)) ([@christianhelle](https://github.com/christianhelle/))
- Environment Editing from TUI app ([#198](https://github.com/christianhelle/httprunner/pull/198)) ([@christianhelle](https://github.com/christianhelle/))
- Use assertion results to determine request success when assertions exist ([#197](https://github.com/christianhelle/httprunner/pull/197)) ([@christianhelle](https://github.com/christianhelle/))
- Change GUI and TUI layout to have Results below Requests ([#195](https://github.com/christianhelle/httprunner/pull/195)) ([@christianhelle](https://github.com/christianhelle/))
- Disable Results view auto-resizing in GUI app ([#194](https://github.com/christianhelle/httprunner/pull/194)) ([@christianhelle](https://github.com/christianhelle/))
- Hide "Working Directory" in WASM GUI build ([#185](https://github.com/christianhelle/httprunner/pull/185)) (@Copilot)
- Separate TUI Docker Hub publishing from CLI ([#184](https://github.com/christianhelle/httprunner/pull/184)) (@Copilot)
- Add .http environment file editing to GUI and TUI apps ([#187](https://github.com/christianhelle/httprunner/pull/187)) ([@christianhelle](https://github.com/christianhelle/))
- Fix --upgrade fails on Linux and MacOS due to captured stdio ([#193](https://github.com/christianhelle/httprunner/pull/193)) ([@christianhelle](https://github.com/christianhelle/))
- Add support to shutdown CLI app using CTRL+C ([#190](https://github.com/christianhelle/httprunner/pull/190)) ([@christianhelle](https://github.com/christianhelle/))
- Fix request duration calculation to only show the actual HTTP request duration ([#188](https://github.com/christianhelle/httprunner/pull/188)) ([@christianhelle](https://github.com/christianhelle/))


## [0.8.48](https://github.com/christianhelle/httprunner/releases/tag/0.8.48) (2026-02-07)

### Features
- Fix --upgrade CLI argument being ignored ([#178](https://github.com/christianhelle/httprunner/pull/178)) ([@christianhelle](https://github.com/christianhelle/))


## [0.8.47](https://github.com/christianhelle/httprunner/releases/tag/0.8.47) (2026-02-05)

### Merged Pull Requests
- Rename lib project to core ([#177](https://github.com/christianhelle/httprunner/pull/177)) (@Copilot)

### Features
- Update TUI app keyboard shortcuts for request delay to [ and ] ([#176](https://github.com/christianhelle/httprunner/pull/176)) ([@christianhelle](https://github.com/christianhelle/))
- Add request delay features with global and per-request controls ([#175](https://github.com/christianhelle/httprunner/pull/175)) ([@christianhelle](https://github.com/christianhelle/))
- Show Request Body in GUI results when in Verbose Mode ([#174](https://github.com/christianhelle/httprunner/pull/174)) ([@christianhelle](https://github.com/christianhelle/))
- Implement telemetry correlation ([#172](https://github.com/christianhelle/httprunner/pull/172)) ([@christianhelle](https://github.com/christianhelle/))
- Introduce anonymous telemetry data collection to Azure App Insights ([#171](https://github.com/christianhelle/httprunner/pull/171)) ([@christianhelle](https://github.com/christianhelle/))
- Introduce support key ([#167](https://github.com/christianhelle/httprunner/pull/167)) ([@christianhelle](https://github.com/christianhelle/))
- Implement toggling file list view (CTRL+B) in TUI app ([#166](https://github.com/christianhelle/httprunner/pull/166)) ([@christianhelle](https://github.com/christianhelle/))
- Fix double press issue in TUI app when running in Windows ([#165](https://github.com/christianhelle/httprunner/pull/165)) ([@christianhelle](https://github.com/christianhelle/))
- Added Q and R shortcuts (in addition to existing Ctrl+Q and Ctrl+R/F5) in TUI app ([#164](https://github.com/christianhelle/httprunner/pull/164)) ([@christianhelle](https://github.com/christianhelle/))


## [0.7.46](https://github.com/christianhelle/httprunner/releases/tag/0.7.46) (2026-01-27)

### Merged Pull Requests
- Add Clippy verification to build workflows ([#142](https://github.com/christianhelle/httprunner/pull/142)) ([@christianhelle](https://github.com/christianhelle/))

### Features
- Introduce --export CLI argument to export request/responses to files ([#162](https://github.com/christianhelle/httprunner/pull/162)) ([@christianhelle](https://github.com/christianhelle/))
- Introduce new built-in transformation functions: upper() and lower() ([#159](https://github.com/christianhelle/httprunner/pull/159)) ([@christianhelle](https://github.com/christianhelle/))
- Async file discovery and request execution for GUI/TUI ([#158](https://github.com/christianhelle/httprunner/pull/158)) ([@christianhelle](https://github.com/christianhelle/))
- Introduce built-in date functions: getdate(), gettime(), getdatetime(), getutcdatetime() ([#157](https://github.com/christianhelle/httprunner/pull/157)) ([@christianhelle](https://github.com/christianhelle/))
- Introduce new built-in functions: name(), first_name(), last_name(), address(), email(), job_title() ([#155](https://github.com/christianhelle/httprunner/pull/155)) ([@christianhelle](https://github.com/christianhelle/))
- Add Terminal UI (TUI) application using Ratatui ([#151](https://github.com/christianhelle/httprunner/pull/151)) (@Copilot)
- Fix GUI app on Ubuntu ([#152](https://github.com/christianhelle/httprunner/pull/152)) ([@christianhelle](https://github.com/christianhelle/))
- Resize text editor in GUI app to vertically fill the area ([#150](https://github.com/christianhelle/httprunner/pull/150)) ([@christianhelle](https://github.com/christianhelle/))
- Fix text editor theme adaptation to system theme ([#149](https://github.com/christianhelle/httprunner/pull/149)) (@Copilot)
- Strip ANSI color codes from log file output ([#147](https://github.com/christianhelle/httprunner/pull/147)) ([@christianhelle](https://github.com/christianhelle/))
- Optimize GUI for WASM by hiding file system UI elements ([#146](https://github.com/christianhelle/httprunner/pull/146)) ([@christianhelle](https://github.com/christianhelle/))
- Add WebAssembly (WASM) support for GUI app ([#143](https://github.com/christianhelle/httprunner/pull/143)) ([@christianhelle](https://github.com/christianhelle/))


## [0.6.45](https://github.com/christianhelle/httprunner/releases/tag/0.6.45) (2026-01-18)

### Merged Pull Requests
- Add full editing capabilities to GUI for .http files ([#117](https://github.com/christianhelle/httprunner/pull/117)) (@Copilot)
- Add README to library package for crates.io ([#140](https://github.com/christianhelle/httprunner/pull/140)) (@Copilot)
- Fix cargo publish failure: add version requirement to httprunner-lib dependency ([#139](https://github.com/christianhelle/httprunner/pull/139)) (@Copilot)
- Add text editor with syntax highlighting for .http files ([#132](https://github.com/christianhelle/httprunner/pull/132)) (@Copilot)
- Move GUI Results panel from bottom to right side ([#125](https://github.com/christianhelle/httprunner/pull/125)) (@Copilot)
- Add CTRL+R keyboard shortcut to run all requests ([#126](https://github.com/christianhelle/httprunner/pull/126)) (@Copilot)
- Refactor GUI state persistence code and fix Zed configuration ([#122](https://github.com/christianhelle/httprunner/pull/122)) (@Copilot)
- Apply code review fixes from PR #118 ([#119](https://github.com/christianhelle/httprunner/pull/119)) (@Copilot)
- Update dependency ruby to v4.0.1 ([#110](https://github.com/christianhelle/httprunner/pull/110)) (@renovate[bot])
- Include GUI binary in build and release workflow artifacts ([#111](https://github.com/christianhelle/httprunner/pull/111)) (@Copilot)
- Upgrade to egui v0.33 ([#109](https://github.com/christianhelle/httprunner/pull/109)) ([@christianhelle](https://github.com/christianhelle/))
- Update Rust crate rfd to 0.17 ([#104](https://github.com/christianhelle/httprunner/pull/104)) (@renovate[bot])
- Exclude GUI package from code coverage ([#106](https://github.com/christianhelle/httprunner/pull/106)) (@Copilot)
- Add comprehensive processor executor tests with idiomatic Rust patterns ([#99](https://github.com/christianhelle/httprunner/pull/99)) ([@christianhelle](https://github.com/christianhelle/))
- Add unit tests to increase code coverage from 69.7% to 74.93% ([#98](https://github.com/christianhelle/httprunner/pull/98)) (@Copilot)
- Update codecov/codecov-action action to v5 ([#97](https://github.com/christianhelle/httprunner/pull/97)) (@renovate[bot])
- Add code coverage workflow with Codecov integration ([#95](https://github.com/christianhelle/httprunner/pull/95)) (@Copilot)

### Features
- Move Results View toggle shortcut help text in GUI app ([#138](https://github.com/christianhelle/httprunner/pull/138)) ([@christianhelle](https://github.com/christianhelle/))
- Remove Run First Request button from Text Editor ([#137](https://github.com/christianhelle/httprunner/pull/137)) ([@christianhelle](https://github.com/christianhelle/))
- Add compact/verbose mode toggle in GUI app Results section ([#135](https://github.com/christianhelle/httprunner/pull/135)) ([@christianhelle](https://github.com/christianhelle/))
- Persist file tree visibility preference across application restarts ([#134](https://github.com/christianhelle/httprunner/pull/134)) (@Copilot)
- Add keyboard shortcuts: Ctrl+B to toggle file tree, Ctrl+S to save ([#133](https://github.com/christianhelle/httprunner/pull/133)) ([@christianhelle](https://github.com/christianhelle/))
- Add state persistence to GUI application ([#121](https://github.com/christianhelle/httprunner/pull/121)) ([@christianhelle](https://github.com/christianhelle/))
- Add keyboard shortcuts to GUI app (F5, Ctrl+O, Ctrl+Q, Ctrl+E) ([#118](https://github.com/christianhelle/httprunner/pull/118)) ([@christianhelle](https://github.com/christianhelle/))
- Fix GUI window icon to use custom icon instead of default egui icon ([#114](https://github.com/christianhelle/httprunner/pull/114)) (@Copilot)
- Fix Windows GUI app opening terminal window on launch ([#113](https://github.com/christianhelle/httprunner/pull/113)) (@Copilot)
- Show assertion results in GUI ([#112](https://github.com/christianhelle/httprunner/pull/112)) ([@christianhelle](https://github.com/christianhelle/))
- Add support for running individual requests from GUI app ([#115](https://github.com/christianhelle/httprunner/pull/115)) (@Copilot)
- Align GUI and CLI behavior in regards to .http file processing ([#108](https://github.com/christianhelle/httprunner/pull/108)) ([@christianhelle](https://github.com/christianhelle/))
- Update GUI layout to have Results below Request Details ([#107](https://github.com/christianhelle/httprunner/pull/107)) ([@christianhelle](https://github.com/christianhelle/))
- Add keyboard shortcuts for font size adjustment in GUI ([#105](https://github.com/christianhelle/httprunner/pull/105)) ([@christianhelle](https://github.com/christianhelle/))
- Add Native Rust GUI Application ([#101](https://github.com/christianhelle/httprunner/pull/101)) ([@christianhelle](https://github.com/christianhelle/))


## [0.5.36](https://github.com/christianhelle/httprunner/releases/tag/0.5.36) (2026-01-10)

### Merged Pull Requests
- Optimize binary size: remove uuid/chrono, minimize dependency features ([#94](https://github.com/christianhelle/httprunner/pull/94)) (@Copilot)
- Refactor datetime formatting: eliminate duplication and fix UTC/Local inconsistency ([#93](https://github.com/christianhelle/httprunner/pull/93)) (@Copilot)
- Document Docker localhost networking for host service access ([#91](https://github.com/christianhelle/httprunner/pull/91)) (@Copilot)

### Features
- Add HTML report format support with comprehensive tests ([#92](https://github.com/christianhelle/httprunner/pull/92)) ([@christianhelle](https://github.com/christianhelle/))


## [0.5.35](https://github.com/christianhelle/httprunner/releases/tag/0.5.35) (2026-01-09)

### Merged Pull Requests
- Update dependency ruby to v4 - autoclosed ([#83](https://github.com/christianhelle/httprunner/pull/83)) (@renovate[bot])
- Update Rust crate reqwest to 0.13 ([#84](https://github.com/christianhelle/httprunner/pull/84)) (@renovate[bot])
- Update GitHub Artifact Actions (major) ([#79](https://github.com/christianhelle/httprunner/pull/79)) (@renovate[bot])

### Features
- Add Built-in Functions Support ([#88](https://github.com/christianhelle/httprunner/pull/88)) ([@christianhelle](https://github.com/christianhelle/))
- Generate report in markdown format using --report ([#81](https://github.com/christianhelle/httprunner/pull/81)) ([@christianhelle](https://github.com/christianhelle/))


## [0.4.34](https://github.com/christianhelle/httprunner/releases/tag/0.4.34) (2025-12-11)

### Merged Pull Requests
- Add comprehensive documentation for --pretty-json feature ([#78](https://github.com/christianhelle/httprunner/pull/78)) (@Copilot)
- Fix multiple parser and runtime bugs ([#75](https://github.com/christianhelle/httprunner/pull/75)) (@Copilot)
- Update actions/checkout action to v6 ([#74](https://github.com/christianhelle/httprunner/pull/74)) (@renovate[bot])
- Add Git commit strategy guidelines for coding agents ([#71](https://github.com/christianhelle/httprunner/pull/71)) ([@christianhelle](https://github.com/christianhelle/))
- 📝 Add docstrings to `if-not-condition` ([#70](https://github.com/christianhelle/httprunner/pull/70)) (@coderabbitai[bot])

### Features
- Introduce --pretty-json CLI argument ([#76](https://github.com/christianhelle/httprunner/pull/76)) ([@christianhelle](https://github.com/christianhelle/))
- Setup Docker Hub and Crates.io deployment environments ([#73](https://github.com/christianhelle/httprunner/pull/73)) ([@christianhelle](https://github.com/christianhelle/))
- Add comprehensive unit tests and dev tooling for coverage ([#72](https://github.com/christianhelle/httprunner/pull/72)) ([@christianhelle](https://github.com/christianhelle/))
- Introduce `@if-not` directive for negated conditions ([#69](https://github.com/christianhelle/httprunner/pull/69)) ([@christianhelle](https://github.com/christianhelle/))


## [0.3.33](https://github.com/christianhelle/httprunner/releases/tag/0.3.33) (2025-11-16)

### Merged Pull Requests
- Refactor conditional execution code based on PR#66 review comments ([#67](https://github.com/christianhelle/httprunner/pull/67)) (@Copilot)
- Add customizable HTTP request timeouts with millisecond precision ([#65](https://github.com/christianhelle/httprunner/pull/65)) (@Copilot)

### Features
- Improve summary output with separate skipped request tracking ([#68](https://github.com/christianhelle/httprunner/pull/68)) ([@christianhelle](https://github.com/christianhelle/))
- Add conditional request execution with @dependsOn and @if directives ([#66](https://github.com/christianhelle/httprunner/pull/66)) (@Copilot)


## [0.3.28](https://github.com/christianhelle/httprunner/releases/tag/0.3.28) (2025-11-12)

### Merged Pull Requests
- Restore critical snapcraft dependencies removed in PR #63 ([#64](https://github.com/christianhelle/httprunner/pull/64)) (@Copilot)

### Features
- Simplify snapcraft.yaml ([#63](https://github.com/christianhelle/httprunner/pull/63)) ([@christianhelle](https://github.com/christianhelle/))


## [v0.1.9](https://github.com/christianhelle/httprunner/releases/tag/v0.1.9) (2025-11-12)


## [0.3.26](https://github.com/christianhelle/httprunner/releases/tag/0.3.26) (2025-11-11)


## [0.3.25](https://github.com/christianhelle/httprunner/releases/tag/0.3.25) (2025-11-11)

### Merged Pull Requests
- Reduce binary output size ([#60](https://github.com/christianhelle/httprunner/pull/60)) ([@christianhelle](https://github.com/christianhelle/))

### Features
- Add --no-banner flag to suppress donation banner ([#62](https://github.com/christianhelle/httprunner/pull/62)) ([@christianhelle](https://github.com/christianhelle/))
- Use Rust 2024 ([#61](https://github.com/christianhelle/httprunner/pull/61)) ([@christianhelle](https://github.com/christianhelle/))
- Show same output when using -V and --version arguments ([#59](https://github.com/christianhelle/httprunner/pull/59)) ([@christianhelle](https://github.com/christianhelle/))


## [0.3.24](https://github.com/christianhelle/httprunner/releases/tag/0.3.24) (2025-11-01)


## [v0.3.22](https://github.com/christianhelle/httprunner/releases/tag/v0.3.22) (2025-10-30)

### Merged Pull Requests
- Add support for '> ' prefix on assertion keywords ([#55](https://github.com/christianhelle/httprunner/pull/55)) ([@christianhelle](https://github.com/christianhelle/))
- Add variable substitution support for assertions ([#54](https://github.com/christianhelle/httprunner/pull/54)) ([@christianhelle](https://github.com/christianhelle/))


## [v0.3.21](https://github.com/christianhelle/httprunner/releases/tag/v0.3.21) (2025-10-28)

### Merged Pull Requests
- Add JSONPath array indexing support for variable substitution ([#53](https://github.com/christianhelle/httprunner/pull/53)) ([@christianhelle](https://github.com/christianhelle/))


## [v0.3.20](https://github.com/christianhelle/httprunner/releases/tag/v0.3.20) (2025-10-27)

### Merged Pull Requests
- Add support for ignoring IntelliJ HTTP Client syntax ([#52](https://github.com/christianhelle/httprunner/pull/52)) ([@christianhelle](https://github.com/christianhelle/))


## [v0.3.19](https://github.com/christianhelle/httprunner/releases/tag/v0.3.19) (2025-10-27)

### Merged Pull Requests
- Fix git commit showing 'unknown' in GitHub Actions builds ([#50](https://github.com/christianhelle/httprunner/pull/50)) ([@christianhelle](https://github.com/christianhelle/))

### Features
- Fix clippy warnings ([#51](https://github.com/christianhelle/httprunner/pull/51)) ([@christianhelle](https://github.com/christianhelle/))


## [v0.3.18](https://github.com/christianhelle/httprunner/releases/tag/v0.3.18) (2025-10-27)

### Merged Pull Requests
- Implement --insecure flag as opt-in for HTTPS certificate validation bypass ([#49](https://github.com/christianhelle/httprunner/pull/49)) ([@christianhelle](https://github.com/christianhelle/))
- Fix version detection in GitHub Actions and cargo installs ([#47](https://github.com/christianhelle/httprunner/pull/47)) ([@christianhelle](https://github.com/christianhelle/))

### Features
- Add insecure HTTPS support for development environments ([#48](https://github.com/christianhelle/httprunner/pull/48)) ([@christianhelle](https://github.com/christianhelle/))


## [v0.3.17](https://github.com/christianhelle/httprunner/releases/tag/v0.3.17) (2025-10-26)

### Merged Pull Requests
- Add crates.io optimized README and application icons ([#46](https://github.com/christianhelle/httprunner/pull/46)) ([@christianhelle](https://github.com/christianhelle/))

### Features
- Show help text when no arguments are passed ([#44](https://github.com/christianhelle/httprunner/pull/44)) ([@christianhelle](https://github.com/christianhelle/))
- Add application icon ([#45](https://github.com/christianhelle/httprunner/pull/45)) ([@christianhelle](https://github.com/christianhelle/))


## [v0.2.0](https://github.com/christianhelle/httprunner/releases/tag/v0.2.0) (2025-10-26)

### Merged Pull Requests
- Update GitHub Artifact Actions (major) ([#41](https://github.com/christianhelle/httprunner/pull/41)) (@renovate[bot])
- Add Makefile ([#40](https://github.com/christianhelle/httprunner/pull/40)) ([@christianhelle](https://github.com/christianhelle/))
- Update actions/upload-pages-artifact action to v4 ([#37](https://github.com/christianhelle/httprunner/pull/37)) (@renovate[bot])
- Update actions/download-artifact action to v5 ([#33](https://github.com/christianhelle/httprunner/pull/33)) (@renovate[bot])
- Update actions/checkout action to v5 ([#36](https://github.com/christianhelle/httprunner/pull/36)) (@renovate[bot])
- Add comprehensive GitHub Copilot instructions for HTTP File Runner development ([#35](https://github.com/christianhelle/httprunner/pull/35)) (@Copilot)
- Revert "Add dev container configuration" ([#29](https://github.com/christianhelle/httprunner/pull/29)) ([@christianhelle](https://github.com/christianhelle/))
- Update actions/configure-pages action to v5 ([#18](https://github.com/christianhelle/httprunner/pull/18)) (@renovate[bot])

### Features
- Re-write HTTP File Runner to Rust (from Zig) ([#43](https://github.com/christianhelle/httprunner/pull/43)) ([@christianhelle](https://github.com/christianhelle/))
- Upgrade to Zig 0.15.1 ([#38](https://github.com/christianhelle/httprunner/pull/38)) ([@christianhelle](https://github.com/christianhelle/))
- Remove Powershell dependency for Git Versioning from build.zig ([#32](https://github.com/christianhelle/httprunner/pull/32)) ([@christianhelle](https://github.com/christianhelle/))
- Add dev container configuration ([#28](https://github.com/christianhelle/httprunner/pull/28)) ([@christianhelle](https://github.com/christianhelle/))
- Add support for request/response chaining and request variables ([#27](https://github.com/christianhelle/httprunner/pull/27)) ([@christianhelle](https://github.com/christianhelle/))
- Custom HTTP Headers ([#24](https://github.com/christianhelle/httprunner/pull/24)) ([@christianhelle](https://github.com/christianhelle/))


## [v0.1.8](https://github.com/christianhelle/httprunner/releases/tag/v0.1.8) (2025-06-20)

### Merged Pull Requests
- Add comprehensive dark mode support to documentation website ([#21](https://github.com/christianhelle/httprunner/pull/21)) (@Copilot)

### Features
- Show donation banner after running .http files ([#23](https://github.com/christianhelle/httprunner/pull/23)) ([@christianhelle](https://github.com/christianhelle/))
- Introduce --upgrade argument ([#22](https://github.com/christianhelle/httprunner/pull/22)) ([@christianhelle](https://github.com/christianhelle/))


## [v0.1.7](https://github.com/christianhelle/httprunner/releases/tag/v0.1.7) (2025-06-19)

### Merged Pull Requests
- Quick Install Script ([#19](https://github.com/christianhelle/httprunner/pull/19)) ([@christianhelle](https://github.com/christianhelle/))


## [v0.1.6](https://github.com/christianhelle/httprunner/releases/tag/v0.1.6) (2025-06-19)

### Features
- Semantic Versioning ([#17](https://github.com/christianhelle/httprunner/pull/17)) ([@christianhelle](https://github.com/christianhelle/))
- Improve error handling in HTTP file processing ([#16](https://github.com/christianhelle/httprunner/pull/16)) ([@christianhelle](https://github.com/christianhelle/))
- Add support for Environment files ([#15](https://github.com/christianhelle/httprunner/pull/15)) ([@christianhelle](https://github.com/christianhelle/))
- Add variable support in .http files ([#14](https://github.com/christianhelle/httprunner/pull/14)) ([@christianhelle](https://github.com/christianhelle/))


## [v0.1.5](https://github.com/christianhelle/httprunner/releases/tag/v0.1.5) (2025-06-17)

### Merged Pull Requests
- Update docker/build-push-action action to v6 ([#8](https://github.com/christianhelle/httprunner/pull/8)) (@renovate[bot])
- Update softprops/action-gh-release action to v2 ([#9](https://github.com/christianhelle/httprunner/pull/9)) (@renovate[bot])
- Configure Renovate ([#7](https://github.com/christianhelle/httprunner/pull/7)) (@renovate[bot])

### Features
- Introduce Assertions ([#13](https://github.com/christianhelle/httprunner/pull/13)) ([@christianhelle](https://github.com/christianhelle/))


## [v0.1.4](https://github.com/christianhelle/httprunner/releases/tag/v0.1.4) (2025-06-14)


## [v0.1.3](https://github.com/christianhelle/httprunner/releases/tag/v0.1.3) (2025-06-14)

### Merged Pull Requests
- Fix memory leak in discovery mode when ArrayList.append() fails ([#6](https://github.com/christianhelle/httprunner/pull/6)) (@Copilot)
- Add verbose mode details to README ([#4](https://github.com/christianhelle/httprunner/pull/4)) (@Copilot)


## [v0.1.2](https://github.com/christianhelle/httprunner/releases/tag/v0.1.2) (2025-06-14)


## [v0.1.1](https://github.com/christianhelle/httprunner/releases/tag/v0.1.1) (2025-06-14)


## [v0.1.0](https://github.com/christianhelle/httprunner/releases/tag/v0.1.0) (2025-06-13)


