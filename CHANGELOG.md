# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0](https://github.com/naom1029/skill-browser/releases/tag/v0.3.0) - 2026-07-11

### Added

- add --version, --help with keybindings, -p shorthand
- dual filter, install scope/agent, update, and UX polish
- agent detection, security indicators, and metadata-based source identification
- backend trait and implementations
- skim TUI with Level 1/2 navigation and grep mode
- preview renderer with description header and resource footer
- skill scanner with multi-directory support
- SKILL.md frontmatter parser
- project setup with Skill data model

### Fixed

- use synchronous preview fetch with prefetch for speed
- restore full SKILL.md fetch in preview, cache on first view

### Other

- auto-fix formatting and clippy warnings
- automate versioning with release-plz and streamline release workflow
- README rewrite and CI/CD fixes
- add CI/CD workflows and README
- initial empty commit
