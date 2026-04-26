# BookmarkShelf

> 🇯🇵 [日本語版 README はこちら](README.ja.md)

Manage browser bookmarks as a plain directory tree — import from Edge/Chrome/Firefox/Safari, reorganize freely in Finder or shell, then export back to browser-ready HTML. Includes a broken-link checker.

> **Note:** This repository was created through vibe coding in collaboration with Claude Code.

## Features

- **Import** — Convert browser bookmark HTML exports into a folder/`.url` file tree
- **Export** — Rebuild Netscape Bookmark HTML from any edited directory structure
- **Link Checker** — Parallel HTTP check across all bookmarks with CSV export

## Install

### macOS

Download the latest `.dmg` from [Releases](https://github.com/your-username/bookmark-shelf/releases).

| File | Target |
|---|---|
| `BookmarkShelf_x.x.x_aarch64.dmg` | Apple Silicon (M1/M2/M3/M4) |
| `BookmarkShelf_x.x.x_x86_64.dmg` | Intel Mac |

1. Mount the `.dmg` and drag `BookmarkShelf.app` to `/Applications`
2. **First-launch Gatekeeper bypass** (unsigned build):
   ```bash
   xattr -dr com.apple.quarantine /Applications/BookmarkShelf.app
   ```
   or right-click → Open → Open in Finder

## Exporting bookmarks from your browser

| Browser | Steps |
|---|---|
| **Edge** | Settings → Favorites → ··· → Export favorites |
| **Chrome** | Bookmark manager → ··· → Export bookmarks |
| **Firefox** | Bookmarks → Manage bookmarks → Import and Backup → Export Bookmarks to HTML |
| **Safari** | File → Export Bookmarks |

## Usage

1. Launch the app
2. **Home** → "Import Bookmarks" — select the exported HTML file
3. Bookmarks expand into `~/Library/Application Support/bookmark-shelf/bookmarks/`
4. Reorganize freely in Finder or Terminal
5. **Home** → "Export to HTML" — generates a file ready to import into any browser
6. **Link Checker** — detect broken links and export results as CSV

## Development

### Requirements

- macOS 10.15+
- Node.js 22+
- Rust (install via `rustup`)

### Setup

```bash
git clone https://github.com/your-username/bookmark-shelf.git
cd bookmark-shelf
npm install
```

### Commands

```bash
make dev          # Start dev server
make test         # Run all tests
make build-debug  # Debug build (fast)
make build        # Release build
make open-dmg     # Open built DMG
```

### Release

```bash
git tag v0.1.0
git push origin v0.1.0
```

GitHub Actions automatically builds `.dmg` files for Apple Silicon and Intel, then creates a Draft Release. Review and publish when ready.

## License

MIT
