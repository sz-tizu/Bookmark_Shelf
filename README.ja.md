# BookmarkShelf

> 🇺🇸 [English README](README.md)

ブラウザのブックマークをファイルシステムで管理するデスクトップツール。
Edge / Chrome / Firefox / Safari からインポートし、Finder やシェルで自由に整理して、再びブラウザへ戻せます。

> **注意:** このリポジトリはバイブコーディングによって Claude Code との協業で作られました。

## 機能

- **インポート** — ブラウザのブックマーク HTML をフォルダ・`.url` ファイルのツリーに変換
- **エクスポート** — 編集済みのディレクトリ構造を Netscape Bookmark HTML に再変換
- **リンクチェック** — 全ブックマークを並列 HTTP チェック・CSV エクスポート

## インストール

### macOS

[Releases](https://github.com/your-username/bookmark-shelf/releases) から最新の `.dmg` をダウンロード。

| ファイル名 | 対象 |
|---|---|
| `BookmarkShelf_x.x.x_aarch64.dmg` | Apple Silicon (M1/M2/M3/M4) |
| `BookmarkShelf_x.x.x_x86_64.dmg` | Intel Mac |

1. `.dmg` をマウントして `BookmarkShelf.app` を `/Applications` にドラッグ
2. **初回起動の Gatekeeper 解除**（未署名のため必要）:
   ```bash
   xattr -dr com.apple.quarantine /Applications/BookmarkShelf.app
   ```
   または Finder で右クリック →「開く」→「開く」をクリック

## ブックマークのエクスポート方法

| ブラウザ | 手順 |
|---|---|
| **Edge** | 設定 → お気に入り → ··· → お気に入りのエクスポート |
| **Chrome** | ブックマークマネージャー → ··· → ブックマークをエクスポート |
| **Firefox** | ブックマーク → すべてのブックマークを管理 → インポートとバックアップ → HTML でエクスポート |
| **Safari** | ファイル → ブックマークをエクスポート |

## 使い方

1. アプリを起動
2. **ホーム** → 「ブックマークをインポート」でエクスポート済み HTML を選択
3. `~/Library/Application Support/bookmark-shelf/bookmarks/` にフォルダ・`.url` ファイルとして展開
4. Finder やターミナルで自由に整理
5. **ホーム** → 「HTML にエクスポート」でブラウザにインポートできる HTML を生成
6. **Link Checker** でリンク切れを確認・CSV 出力

## 開発

### 必要環境

- macOS 10.15+
- Node.js 22+
- Rust（`rustup` でインストール）

### セットアップ

```bash
git clone https://github.com/your-username/bookmark-shelf.git
cd bookmark-shelf
npm install
```

### コマンド

```bash
make dev          # 開発サーバー起動
make test         # 全テスト実行
make build-debug  # デバッグビルド（高速）
make build        # リリースビルド
make open-dmg     # ビルドした DMG を開く
```

### リリース

```bash
git tag v0.1.0
git push origin v0.1.0
```

GitHub Actions が自動で Apple Silicon / Intel 両対応の `.dmg` をビルドし、Draft Release として作成します。内容を確認後、Publish してください。

## ライセンス

MIT
