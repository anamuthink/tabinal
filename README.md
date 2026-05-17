# tabinal

A general-purpose TUI terminal multiplexer with split panes, tabs, and a file tree sidebar.

## Features

- **Multi-pane terminal** — Split vertically/horizontally, run independent PTY shells
- **Tab workspaces** — Multiple project tabs with click-to-switch
- **File tree sidebar** — Browse project files with icons, navigate directories ranger-style
- **File preview** — Syntax-highlighted text files and inline image rendering (PNG, JPEG, GIF, BMP, WebP)
- **cd tracking** — File tree and tab name auto-update when you change directories
- **Mouse support** — Click to focus, drag borders to resize, drag to select text, scroll history
- **Scrollback** — 10,000 lines of terminal history per pane
- **Dark theme** — Clean color scheme
- **Cross-platform** — Windows, macOS, Linux
- **Single binary** — ~1MB, no runtime dependencies

## Requirements

File tree icons require a Nerd Font.

If icons appear as □ or ?, your terminal is not using a Nerd Font.

**Option A:** Install a Nerd Font from https://www.nerdfonts.com/ and configure
your terminal to use it. JetBrainsMono Nerd Font is a good default choice.

**Option B:** Create or edit `~/.config/tabinal/config.toml` as follows. Icons
will be shown as plain Unicode symbols (`•`) instead of Nerd Font glyphs:

```toml
[ui]
icons = "plain"
```

## Install

### Via npm (recommended)

```bash
npm install -g tabinal-cli
```

### Download binary

Download the latest binary from [Releases](https://github.com/anamuthink/tabinal/releases):

| Platform | File |
|----------|------|
| Windows (x64) | `tabinal-windows-x64.exe` |
| macOS (Apple Silicon) | `tabinal-macos-arm64` |
| macOS (Intel) | `tabinal-macos-x64` |
| Linux (x64) | `tabinal-linux-x64` |

> **macOS/Linux:** After downloading, make the binary executable: `chmod +x tabinal-*`

### From source

```bash
git clone https://github.com/anamuthink/tabinal.git
cd tabinal
cargo build --release
# Binary at target/release/tabinal (or tabinal.exe on Windows)
```

Requires [Rust](https://rustup.rs/) toolchain.

## Usage

```bash
tabinal
```

Launch from any directory. The file tree shows the current working directory.

## Keybindings

### Pane mode (default)

| Key | Action |
|-----|--------|
| `Ctrl+Shift+→` | Split right (vertical) |
| `Ctrl+Shift+↓` | Split down (horizontal) |
| `Ctrl+W` | Close pane / tab |
| `Alt+T` / `Ctrl+T` | New tab |
| `Alt+1..9` | Jump to tab N |
| `Alt+Left/Right` | Previous / next tab |
| `Alt+N` | Rename tab (session only) |
| `Ctrl+F` | Toggle file tree |
| `Ctrl+Right` | Focus right pane |
| `Ctrl+Left` | Focus left pane |
| `Ctrl+Up` | Focus pane above |
| `Ctrl+Down` | Focus pane below |
| `Ctrl+Q` | Quit |
| `Alt+S` | Open config file |

### File tree mode (after `Ctrl+F`)

| Key | Action |
|-----|--------|
| `↑` / `↓` | Move selection |
| `→` | Navigate into directory |
| `←` | Navigate to parent directory |
| `Enter` | Open file / navigate into directory |
| `.` | Toggle hidden files |
| `Esc` | Return to pane |

### Preview mode (after focusing preview)

| Key | Action |
|-----|--------|
| `↑` / `↓` | Scroll vertically |
| `←` / `→` | Scroll horizontally |
| `Ctrl+W` | Close preview |
| `Esc` | Return to pane |

### Mouse

| Action | Effect |
|--------|--------|
| Click pane | Focus pane |
| Click tab | Switch tab |
| Double-click tab | Rename tab |
| Click `+` | New tab |
| Drag border | Resize panels |
| Scroll wheel | Scroll file tree / preview / terminal history |

## Architecture

```
src/
├── main.rs       # Entry point, event loop, panic hook
├── app.rs        # Workspace/tab state, layout tree, key/mouse handling
├── config.rs     # Keybinding config (~/.config/tabinal/config.toml)
├── pane.rs       # PTY management, vt100 emulation, shell detection
├── ui.rs         # ratatui rendering, theme, layout
├── filetree.rs   # File tree scanning, navigation
└── preview.rs    # File preview with syntax highlighting
```

**Key design decisions:**
- `vt100` crate for terminal emulation (not ANSI stripping) — needed for interactive TUI apps
- Binary tree layout for recursive pane splitting with variable ratios
- Per-PTY reader threads with mpsc channel to main event loop
- OSC 7 detection for automatic cd tracking
- Dirty-flag rendering for minimal CPU usage when idle

## Tech Stack

- [ratatui](https://ratatui.rs/) + [crossterm](https://github.com/crossterm-rs/crossterm) — TUI framework
- [portable-pty](https://github.com/nickelc/portable-pty) — PTY abstraction (ConPTY on Windows)
- [vt100](https://crates.io/crates/vt100) — Terminal emulation
- [syntect](https://github.com/trishume/syntect) — Syntax highlighting

## License

MIT
