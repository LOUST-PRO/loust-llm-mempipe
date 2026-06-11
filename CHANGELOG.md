# Changelog

All notable changes to loust-llm-mempipe are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/) and the project adheres to
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- F1: Project skeleton, Cargo.toml with full SEO metadata, contract types
  (`NormalizedMessage`, `Role`, `PipelineConfig`, `OutputFormat`, `SecretKind`,
  `Adapter`, `AdapterKind`).
- F1: Stub adapters for `chatgpt`, `claude_web`, `gemini`, `claude_code`.
- F1: Stub pipeline modules (`parser`, `scrubber`, `normalizer`, `dedup`,
  `signals`, `writer`).
- F1: 7 unit tests covering hash determinism, slugify, role serde, config
  defaults, and secret pattern coverage.
- F1: Makefile with build / test / clippy / fmt / release targets.
- F1: README with project status table and build instructions.

## [0.1.0] — 2026-06-10

Initial skeleton release. CLI is a placeholder (`--version` and `--info`
work; the full surface lands in F4). Library types are stable and ready
for downstream consumption.

[Unreleased]: https://github.com/LOUST-PRO/loust-llm-mempipe/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/LOUST-PRO/loust-llm-mempipe/releases/tag/v0.1.0
