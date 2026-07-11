# skill-browser

A TUI for managing AI agent skills across multiple sources. Browse, search, install, update, and delete skills from one place.

## Why?

AI coding agents (Claude Code, Codex, Gemini CLI, etc.) use SKILL.md files to extend their capabilities. But skills are scattered across multiple directories and installed via different tools (`gh skill`, `npx skills`, manual placement). **skill-browser** gives you a single, unified view of everything — with fuzzy search, full-text grep, live preview, and one-key install/update/delete.

## Features

### Browse & Search
- **Unified skill list** — Scans `~/.claude/skills/`, `~/.agents/skills/`, and project-scoped directories
- **Fuzzy search** — Type to filter skills by name
- **Full-text grep** (`Ctrl-G`) — Search inside SKILL.md content with match highlighting
- **Source filter** (`Ctrl-S`) — Cycle through: all / gh / npx / local
- **Live preview** — Description, metadata, and full SKILL.md body

### Install & Manage
- **Search & install** (`Ctrl-N`) — Search skills from GitHub and skills.sh, preview before installing
- **Update** (`Ctrl-R`) — Update the selected skill via its backend
- **Delete** (`Ctrl-X`) — Remove skills with confirmation prompt
- **Scope selection** — Choose user or project scope when installing
- **Multi-backend** — Installs via `gh skill` or `npx skills` depending on source

### Security & Metadata
- **Scripts detection** — Warns if a skill contains executable files (`.sh`, `.py`, etc.)
- **Pinning status** — Shows whether a skill is pinned to a specific version
- **Agent info** — Shows which agents the skill supports

### File Browser
- **Drill into skills** (`Enter`) — Browse SKILL.md, references, and supplementary files
- **Open in editor** (`Enter`) — Launch `$EDITOR` on any file

## Install

### Pre-built binaries

Download from [Releases](https://github.com/naom1029/skill-browser/releases).

**Linux (amd64)**
```sh
curl -L https://github.com/naom1029/skill-browser/releases/latest/download/skill-browser-linux-amd64 -o skill-browser
chmod +x skill-browser
sudo mv skill-browser /usr/local/bin/
```

**macOS (Apple Silicon)**
```sh
curl -L https://github.com/naom1029/skill-browser/releases/latest/download/skill-browser-macos-arm64 -o skill-browser
chmod +x skill-browser
sudo mv skill-browser /usr/local/bin/
```

**macOS (Intel)**
```sh
curl -L https://github.com/naom1029/skill-browser/releases/latest/download/skill-browser-macos-amd64 -o skill-browser
chmod +x skill-browser
sudo mv skill-browser /usr/local/bin/
```

### From source

```sh
git clone https://github.com/naom1029/skill-browser.git
cd skill-browser
cargo install --path .
```

## Usage

```sh
skill-browser                          # browse skills (project = cwd)
skill-browser --project /path/to/repo  # specify project directory
```

## Keybindings

| Key | Action |
|-----|--------|
| Type | Fuzzy search by name |
| `Enter` | Browse skill files |
| `Ctrl-G` | Grep mode (full-text search) |
| `Ctrl-N` | Search & install new skill |
| `Ctrl-R` | Update selected skill |
| `Ctrl-S` | Cycle source filter |
| `Ctrl-X` | Delete selected skill |
| `Ctrl-D` / `Ctrl-F` | Preview page down |
| `Ctrl-U` / `Ctrl-B` | Preview page up |
| `Esc` | Back / Quit |

## Scanned Directories

| Directory | Source | Scope |
|-----------|--------|-------|
| `~/.claude/skills/` | gh | user |
| `~/.agents/skills/` | npx | user |
| `.claude/skills/` | local | project |
| `.agents/skills/` | local | project |
| `.github/skills/` | local | project |

## Optional Dependencies

skill-browser works standalone — browse, search, grep, and preview all work without any external tool. The following CLIs unlock additional features:

| Tool | Enables |
|------|---------|
| `gh` CLI | Install, update, search, remote preview via `gh skill` |
| `npx` | Install, search via `npx skills` |

Without `gh` or `npx`, skill-browser still provides full read-only access to all locally installed skills.

## License

MIT
