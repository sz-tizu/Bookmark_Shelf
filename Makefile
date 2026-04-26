.PHONY: dev build build-debug test test-rust test-fe clean open-dmg

# ── 開発 ────────────────────────────────────────────────────────────────
dev:
	npm run tauri dev

# ── ビルド ──────────────────────────────────────────────────────────────
## リリースビルド（LTO有効・サイズ最適化）
build:
	npm run tauri build

## デバッグビルド（高速・シンボル付き）
build-debug:
	npm run tauri build -- --debug

## Apple Silicon 専用リリースビルド
build-aarch64:
	npm run tauri build -- --target aarch64-apple-darwin

## Intel Mac 専用リリースビルド
build-x86_64:
	npm run tauri build -- --target x86_64-apple-darwin

# ── テスト ──────────────────────────────────────────────────────────────
test: test-fe test-rust

test-fe:
	npm test

test-rust:
	cd src-tauri && cargo test --lib --tests

# ── クリーン ────────────────────────────────────────────────────────────
clean:
	rm -rf dist
	cd src-tauri && cargo clean

# ── 確認用 ──────────────────────────────────────────────────────────────
## デバッグビルドの DMG をマウントして確認
open-dmg:
	open src-tauri/target/debug/bundle/dmg/BookmarkShelf_*.dmg
