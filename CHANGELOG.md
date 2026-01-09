# Changelog

## [Unreleased](https://github.com/christianhelle/httprunner/tree/HEAD)

[Full Changelog](https://github.com/christianhelle/httprunner/compare/0.5.35...HEAD)

**Fixed bugs:**

- Cannot connect to Docker container from localhost [\#90](https://github.com/christianhelle/httprunner/issues/90)

**Merged pull requests:**

- Document Docker localhost networking for host service access [\#91](https://github.com/christianhelle/httprunner/pull/91) ([Copilot](https://github.com/apps/copilot-swe-agent))

## [0.5.35](https://github.com/christianhelle/httprunner/tree/0.5.35) (2026-01-09)

[Full Changelog](https://github.com/christianhelle/httprunner/compare/0.4.34...0.5.35)

**Implemented enhancements:**

- Add Built-in Functions Support [\#88](https://github.com/christianhelle/httprunner/pull/88) ([christianhelle](https://github.com/christianhelle))
- Generate report in markdown format using --report [\#81](https://github.com/christianhelle/httprunner/pull/81) ([christianhelle](https://github.com/christianhelle))

**Merged pull requests:**

- Update Rust crate reqwest to 0.13 [\#84](https://github.com/christianhelle/httprunner/pull/84) ([renovate[bot]](https://github.com/apps/renovate))
- Update dependency ruby to v4 - autoclosed [\#83](https://github.com/christianhelle/httprunner/pull/83) ([renovate[bot]](https://github.com/apps/renovate))
- Update GitHub Artifact Actions \(major\) [\#79](https://github.com/christianhelle/httprunner/pull/79) ([renovate[bot]](https://github.com/apps/renovate))

## [0.4.34](https://github.com/christianhelle/httprunner/tree/0.4.34) (2025-12-11)

[Full Changelog](https://github.com/christianhelle/httprunner/compare/0.3.33...0.4.34)

**Implemented enhancements:**

- Introduce --pretty-json CLI argument [\#76](https://github.com/christianhelle/httprunner/pull/76) ([christianhelle](https://github.com/christianhelle))
- Setup Docker Hub and Crates.io deployment environments [\#73](https://github.com/christianhelle/httprunner/pull/73) ([christianhelle](https://github.com/christianhelle))
- Add comprehensive unit tests and dev tooling for coverage [\#72](https://github.com/christianhelle/httprunner/pull/72) ([christianhelle](https://github.com/christianhelle))
- Introduce `@if-not` directive for negated conditions [\#69](https://github.com/christianhelle/httprunner/pull/69) ([christianhelle](https://github.com/christianhelle))

**Merged pull requests:**

- Fix multiple parser and runtime bugs [\#75](https://github.com/christianhelle/httprunner/pull/75) ([Copilot](https://github.com/apps/copilot-swe-agent))
- Update actions/checkout action to v6 [\#74](https://github.com/christianhelle/httprunner/pull/74) ([renovate[bot]](https://github.com/apps/renovate))
- Add Git commit strategy guidelines for coding agents [\#71](https://github.com/christianhelle/httprunner/pull/71) ([christianhelle](https://github.com/christianhelle))

## [0.3.33](https://github.com/christianhelle/httprunner/tree/0.3.33) (2025-11-16)

[Full Changelog](https://github.com/christianhelle/httprunner/compare/0.3.28...0.3.33)

**Implemented enhancements:**

- Improve summary output with separate skipped request tracking [\#68](https://github.com/christianhelle/httprunner/pull/68) ([christianhelle](https://github.com/christianhelle))
- Add conditional request execution with @dependsOn and @if directives [\#66](https://github.com/christianhelle/httprunner/pull/66) ([Copilot](https://github.com/apps/copilot-swe-agent))

**Merged pull requests:**

- Refactor conditional execution code based on PR\#66 review comments [\#67](https://github.com/christianhelle/httprunner/pull/67) ([Copilot](https://github.com/apps/copilot-swe-agent))
- Add customizable HTTP request timeouts with millisecond precision [\#65](https://github.com/christianhelle/httprunner/pull/65) ([Copilot](https://github.com/apps/copilot-swe-agent))

## [0.3.28](https://github.com/christianhelle/httprunner/tree/0.3.28) (2025-11-12)

[Full Changelog](https://github.com/christianhelle/httprunner/compare/0.3.26...0.3.28)

**Implemented enhancements:**

- Simplify snapcraft.yaml [\#63](https://github.com/christianhelle/httprunner/pull/63) ([christianhelle](https://github.com/christianhelle))

**Merged pull requests:**

- Restore critical snapcraft dependencies removed in PR \#63 [\#64](https://github.com/christianhelle/httprunner/pull/64) ([Copilot](https://github.com/apps/copilot-swe-agent))

## [0.3.26](https://github.com/christianhelle/httprunner/tree/0.3.26) (2025-11-11)

[Full Changelog](https://github.com/christianhelle/httprunner/compare/0.3.25...0.3.26)

## [0.3.25](https://github.com/christianhelle/httprunner/tree/0.3.25) (2025-11-11)

[Full Changelog](https://github.com/christianhelle/httprunner/compare/0.3.24...0.3.25)

**Implemented enhancements:**

- Add --no-banner flag to suppress donation banner [\#62](https://github.com/christianhelle/httprunner/pull/62) ([christianhelle](https://github.com/christianhelle))
- Use Rust 2024 [\#61](https://github.com/christianhelle/httprunner/pull/61) ([christianhelle](https://github.com/christianhelle))
- Show same output when using -V and --version arguments [\#59](https://github.com/christianhelle/httprunner/pull/59) ([christianhelle](https://github.com/christianhelle))

**Merged pull requests:**

- Reduce binary output size [\#60](https://github.com/christianhelle/httprunner/pull/60) ([christianhelle](https://github.com/christianhelle))

## [0.3.24](https://github.com/christianhelle/httprunner/tree/0.3.24) (2025-10-31)

[Full Changelog](https://github.com/christianhelle/httprunner/compare/v0.3.22...0.3.24)

## [v0.3.22](https://github.com/christianhelle/httprunner/tree/v0.3.22) (2025-10-30)

[Full Changelog](https://github.com/christianhelle/httprunner/compare/v0.3.21...v0.3.22)

**Merged pull requests:**

- Add support for '\> ' prefix on assertion keywords [\#55](https://github.com/christianhelle/httprunner/pull/55) ([christianhelle](https://github.com/christianhelle))
- Add variable substitution support for assertions [\#54](https://github.com/christianhelle/httprunner/pull/54) ([christianhelle](https://github.com/christianhelle))

## [v0.3.21](https://github.com/christianhelle/httprunner/tree/v0.3.21) (2025-10-28)

[Full Changelog](https://github.com/christianhelle/httprunner/compare/v0.3.20...v0.3.21)

**Merged pull requests:**

- Add JSONPath array indexing support for variable substitution [\#53](https://github.com/christianhelle/httprunner/pull/53) ([christianhelle](https://github.com/christianhelle))

## [v0.3.20](https://github.com/christianhelle/httprunner/tree/v0.3.20) (2025-10-27)

[Full Changelog](https://github.com/christianhelle/httprunner/compare/v0.3.19...v0.3.20)

**Merged pull requests:**

- Add support for ignoring IntelliJ HTTP Client syntax [\#52](https://github.com/christianhelle/httprunner/pull/52) ([christianhelle](https://github.com/christianhelle))

## [v0.3.19](https://github.com/christianhelle/httprunner/tree/v0.3.19) (2025-10-27)

[Full Changelog](https://github.com/christianhelle/httprunner/compare/v0.3.18...v0.3.19)

**Implemented enhancements:**

- Fix clippy warnings [\#51](https://github.com/christianhelle/httprunner/pull/51) ([christianhelle](https://github.com/christianhelle))

**Merged pull requests:**

- Fix git commit showing 'unknown' in GitHub Actions builds [\#50](https://github.com/christianhelle/httprunner/pull/50) ([christianhelle](https://github.com/christianhelle))

## [v0.3.18](https://github.com/christianhelle/httprunner/tree/v0.3.18) (2025-10-27)

[Full Changelog](https://github.com/christianhelle/httprunner/compare/v0.3.17...v0.3.18)

**Implemented enhancements:**

- Add insecure HTTPS support for development environments [\#48](https://github.com/christianhelle/httprunner/pull/48) ([christianhelle](https://github.com/christianhelle))

**Merged pull requests:**

- Implement --insecure flag as opt-in for HTTPS certificate validation bypass [\#49](https://github.com/christianhelle/httprunner/pull/49) ([christianhelle](https://github.com/christianhelle))
- Fix version detection in GitHub Actions and cargo installs [\#47](https://github.com/christianhelle/httprunner/pull/47) ([christianhelle](https://github.com/christianhelle))

## [v0.3.17](https://github.com/christianhelle/httprunner/tree/v0.3.17) (2025-10-26)

[Full Changelog](https://github.com/christianhelle/httprunner/compare/v0.2.0...v0.3.17)

**Implemented enhancements:**

- Add application icon [\#45](https://github.com/christianhelle/httprunner/pull/45) ([christianhelle](https://github.com/christianhelle))
- Show help text when no arguments are passed [\#44](https://github.com/christianhelle/httprunner/pull/44) ([christianhelle](https://github.com/christianhelle))

**Merged pull requests:**

- Add crates.io optimized README and application icons [\#46](https://github.com/christianhelle/httprunner/pull/46) ([christianhelle](https://github.com/christianhelle))

## [v0.2.0](https://github.com/christianhelle/httprunner/tree/v0.2.0) (2025-10-26)

[Full Changelog](https://github.com/christianhelle/httprunner/compare/v0.1.9...v0.2.0)

**Implemented enhancements:**

- Re-write HTTP File Runner to Rust \(from Zig\) [\#43](https://github.com/christianhelle/httprunner/pull/43) ([christianhelle](https://github.com/christianhelle))
- Upgrade to Zig 0.15.1 [\#38](https://github.com/christianhelle/httprunner/pull/38) ([christianhelle](https://github.com/christianhelle))
- Remove Powershell dependency for Git Versioning from build.zig [\#32](https://github.com/christianhelle/httprunner/pull/32) ([christianhelle](https://github.com/christianhelle))
- Add dev container configuration [\#28](https://github.com/christianhelle/httprunner/pull/28) ([christianhelle](https://github.com/christianhelle))
- Add support for request/response chaining and request variables [\#27](https://github.com/christianhelle/httprunner/pull/27) ([christianhelle](https://github.com/christianhelle))

**Closed issues:**

- Setup CoPilot Instructions  [\#34](https://github.com/christianhelle/httprunner/issues/34)
- Setup Dev Containers [\#30](https://github.com/christianhelle/httprunner/issues/30)
- Snapcraft Description [\#25](https://github.com/christianhelle/httprunner/issues/25)

**Merged pull requests:**

- Update GitHub Artifact Actions \(major\) [\#41](https://github.com/christianhelle/httprunner/pull/41) ([renovate[bot]](https://github.com/apps/renovate))
- Add Makefile [\#40](https://github.com/christianhelle/httprunner/pull/40) ([christianhelle](https://github.com/christianhelle))
- Update actions/upload-pages-artifact action to v4 [\#37](https://github.com/christianhelle/httprunner/pull/37) ([renovate[bot]](https://github.com/apps/renovate))
- Update actions/checkout action to v5 [\#36](https://github.com/christianhelle/httprunner/pull/36) ([renovate[bot]](https://github.com/apps/renovate))
- Add comprehensive GitHub Copilot instructions for HTTP File Runner development [\#35](https://github.com/christianhelle/httprunner/pull/35) ([Copilot](https://github.com/apps/copilot-swe-agent))
- Update actions/download-artifact action to v5 [\#33](https://github.com/christianhelle/httprunner/pull/33) ([renovate[bot]](https://github.com/apps/renovate))
- Revert "Add dev container configuration" [\#29](https://github.com/christianhelle/httprunner/pull/29) ([christianhelle](https://github.com/christianhelle))
- Update actions/configure-pages action to v5 [\#18](https://github.com/christianhelle/httprunner/pull/18) ([renovate[bot]](https://github.com/apps/renovate))

## [v0.1.9](https://github.com/christianhelle/httprunner/tree/v0.1.9) (2025-06-22)

[Full Changelog](https://github.com/christianhelle/httprunner/compare/v0.1.8...v0.1.9)

**Implemented enhancements:**

- Custom HTTP Headers [\#24](https://github.com/christianhelle/httprunner/pull/24) ([christianhelle](https://github.com/christianhelle))

## [v0.1.8](https://github.com/christianhelle/httprunner/tree/v0.1.8) (2025-06-20)

[Full Changelog](https://github.com/christianhelle/httprunner/compare/v0.1.7...v0.1.8)

**Implemented enhancements:**

- Show donation banner after running .http files [\#23](https://github.com/christianhelle/httprunner/pull/23) ([christianhelle](https://github.com/christianhelle))
- Introduce --upgrade argument [\#22](https://github.com/christianhelle/httprunner/pull/22) ([christianhelle](https://github.com/christianhelle))

**Closed issues:**

- Dark Mode support in documentation static website [\#20](https://github.com/christianhelle/httprunner/issues/20)

**Merged pull requests:**

- Add comprehensive dark mode support to documentation website [\#21](https://github.com/christianhelle/httprunner/pull/21) ([Copilot](https://github.com/apps/copilot-swe-agent))

## [v0.1.7](https://github.com/christianhelle/httprunner/tree/v0.1.7) (2025-06-19)

[Full Changelog](https://github.com/christianhelle/httprunner/compare/v0.1.6...v0.1.7)

**Merged pull requests:**

- Quick Install Script [\#19](https://github.com/christianhelle/httprunner/pull/19) ([christianhelle](https://github.com/christianhelle))

## [v0.1.6](https://github.com/christianhelle/httprunner/tree/v0.1.6) (2025-06-19)

[Full Changelog](https://github.com/christianhelle/httprunner/compare/v0.1.5...v0.1.6)

**Implemented enhancements:**

- Semantic Versioning [\#17](https://github.com/christianhelle/httprunner/pull/17) ([christianhelle](https://github.com/christianhelle))
- Improve error handling in HTTP file processing [\#16](https://github.com/christianhelle/httprunner/pull/16) ([christianhelle](https://github.com/christianhelle))
- Add support for Environment files [\#15](https://github.com/christianhelle/httprunner/pull/15) ([christianhelle](https://github.com/christianhelle))
- Add variable support in .http files [\#14](https://github.com/christianhelle/httprunner/pull/14) ([christianhelle](https://github.com/christianhelle))

## [v0.1.5](https://github.com/christianhelle/httprunner/tree/v0.1.5) (2025-06-17)

[Full Changelog](https://github.com/christianhelle/httprunner/compare/v0.1.4...v0.1.5)

**Implemented enhancements:**

- Introduce Assertions [\#13](https://github.com/christianhelle/httprunner/pull/13) ([christianhelle](https://github.com/christianhelle))

**Closed issues:**

- Re-write everything in Rust [\#11](https://github.com/christianhelle/httprunner/issues/11)

**Merged pull requests:**

- Update softprops/action-gh-release action to v2 [\#9](https://github.com/christianhelle/httprunner/pull/9) ([renovate[bot]](https://github.com/apps/renovate))
- Update docker/build-push-action action to v6 [\#8](https://github.com/christianhelle/httprunner/pull/8) ([renovate[bot]](https://github.com/apps/renovate))
- Configure Renovate [\#7](https://github.com/christianhelle/httprunner/pull/7) ([renovate[bot]](https://github.com/apps/renovate))

## [v0.1.4](https://github.com/christianhelle/httprunner/tree/v0.1.4) (2025-06-14)

[Full Changelog](https://github.com/christianhelle/httprunner/compare/v0.1.3...v0.1.4)

## [v0.1.3](https://github.com/christianhelle/httprunner/tree/v0.1.3) (2025-06-14)

[Full Changelog](https://github.com/christianhelle/httprunner/compare/v0.1.2...v0.1.3)

**Closed issues:**

- Crashes when using --discover [\#5](https://github.com/christianhelle/httprunner/issues/5)
- Add verbose mode details to README [\#3](https://github.com/christianhelle/httprunner/issues/3)
- Fix snapcraft build [\#1](https://github.com/christianhelle/httprunner/issues/1)

**Merged pull requests:**

- Fix memory leak in discovery mode when ArrayList.append\(\) fails [\#6](https://github.com/christianhelle/httprunner/pull/6) ([Copilot](https://github.com/apps/copilot-swe-agent))

## [v0.1.2](https://github.com/christianhelle/httprunner/tree/v0.1.2) (2025-06-14)

[Full Changelog](https://github.com/christianhelle/httprunner/compare/v0.1.1...v0.1.2)

## [v0.1.1](https://github.com/christianhelle/httprunner/tree/v0.1.1) (2025-06-14)

[Full Changelog](https://github.com/christianhelle/httprunner/compare/v0.1.0...v0.1.1)

## [v0.1.0](https://github.com/christianhelle/httprunner/tree/v0.1.0) (2025-06-13)

[Full Changelog](https://github.com/christianhelle/httprunner/compare/020fea5b6718bc22ce57176914a09a550c9bd6ad...v0.1.0)



\* *This Changelog was automatically generated by [github_changelog_generator](https://github.com/github-changelog-generator/github-changelog-generator)*
