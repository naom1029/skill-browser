# skill-browser

A skim-powered TUI for browsing, searching, and managing AI agent skills.

## Features

- **Unified view** — Scans `~/.claude/skills/`, `~/.agents/skills/`, plugin cache, and project-scoped directories in one list
- **Fuzzy search** — Filter skills by name with skim's fuzzy matching
- **Full-text grep** — `Ctrl-G` to search SKILL.md content with match highlighting in preview
- **Live preview** — Description header + SKILL.md body in the preview pane
- **File browser** — `Enter` to drill into a skill's files (SKILL.md, references, etc.), open in `$EDITOR`
- **Install** — `Ctrl-N` to search and install skills from GitHub (`gh skill`) and skills.sh (`npx skills`)
- **Delete** — `Ctrl-X` to remove skills with confirmation
- **Source filter** — `Ctrl-S` to cycle through source types (gh / plugin / npx / local)
- **Plugin dedup** — Reads `installed_plugins.json` to show only active plugin versions

## Install

### From source

```sh
cargo install --path .
```

### Pre-built binaries

See [Releases](https://github.com/naom1029/skill-browser/releases).

## Usage

```sh
skill-browser                        # browse skills in current project
skill-browser --project /path/to/repo  # specify project directory
```

## Keybindings

| Key | Action |
|-----|--------|
| `Enter` | Browse files (Level 2) |
| `Ctrl-G` | Grep mode (full-text search) |
| `Ctrl-N` | Install new skill |
| `Ctrl-S` | Cycle source filter |
| `Ctrl-X` | Delete skill |
| `Ctrl-D` | Preview page down |
| `Ctrl-U` | Preview page up |
| `Esc` | Back / Quit |

## Requirements

- [skim](https://github.com/lotabout/skim) is bundled as a library (no external dependency)
- `gh` CLI (optional, for install/search via `gh skill`)
- `npx` (optional, for install/search via `npx skills`)

## Tech Stack

- Rust (edition 2024)
- [skim](https://github.com/lotabout/skim) — fuzzy finder
- Single binary, no runtime dependencies

## License

MIT
