# BookmarkShelf

ブラウザのブックマークをファイルシステムで管理するデスクトップツール。

- **インポート** — Edge / Chrome / Firefox / Safari のエクスポート HTML をディレクトリに変換
- **エクスポート** — Finder やシェルで編集したフォルダ構造をブラウザへ戻せる HTML に変換
- **リンクチェック** — ブックマーク内のリンク切れを並列検出・CSV 出力

## インストール

### macOS（推奨）

[Releases](https://github.com/your-username/bookmark-shelf/releases) から最新の `.dmg` をダウンロード。

| ファイル名 | 対象 |
|---|---|
| `BookmarkShelf_x.x.x_aarch64.dmg` | Apple Silicon (M1/M2/M3/M4) |
| `BookmarkShelf_x.x.x_x86_64.dmg` | Intel Mac |

1. `.dmg` をダブルクリックしてマウント
2. `BookmarkShelf.app` を `/Applications` にドラッグ
3. **初回起動の Gatekeeper 解除**（未署名のため必要）:
   ```bash
   xattr -dr com.apple.quarantine /Applications/BookmarkShelf.app
   ```
   または Finder で右クリック →「開く」→「開く」をクリック

## ブックマークのエクスポート方法

| ブラウザ | 手順 |
|---|---|
| **Edge** | 設定 → お気に入り → … → お気に入りのエクスポート |
| **Chrome** | ブックマークマネージャー → … → ブックマークをエクスポート |
| **Firefox** | ブックマーク → すべてのブックマークを管理 → インポートとバックアップ → HTMLでエクスポート |
| **Safari** | ファイル → ブックマークをエクスポート |

## 使い方

1. アプリを起動
2. **ホーム** → 「ブックマークをインポート」でエクスポート済み HTML を選択
3. `~/bookmarks/` にフォルダ・`.url` ファイルとして展開される
4. Finder やターミナルで自由に整理
5. **ホーム** → 「HTML にエクスポート」でブラウザにインポートできる HTML を生成
6. **Link Checker** でリンク切れを確認

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

GitHub Actions が自動で Apple Silicon / Intel 両対応の `.dmg` をビルドし、
Draft Release として作成します。内容を確認後、Publish してください。

## ライセンス

MIT
