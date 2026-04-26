# BookmarkShelf — Design Document v0.1

> ブラウザのブックマークをファイルシステムで管理し、GUIで操作できるツール

---

## 概要

BookmarkShelf は、ブラウザのブックマークをディレクトリ構造として扱い、Finder・シェルで自由に編集した後、再びブラウザへインポート可能な形式に戻すことを目的としたデスクトップツールです。

```
Browser (Edge / Chrome / Firefox / Safari)
        ↕  import / export
   BookmarkShelf (GUI)
        ↕  sync
   ~/bookmarks/  (普通のフォルダ構造)
        ↕
   Finder / Terminal (自由に編集)
```

---

## 機能一覧

### F-1: ブックマークインポート

| 項目 | 内容 |
|------|------|
| 対応ブラウザ | Edge / Chrome / Firefox / Safari |
| 入力形式 | Netscape Bookmark HTML (全ブラウザ共通のエクスポート形式) |
| 出力 | ローカルディレクトリへのフォルダ・`.url` ファイル展開 |
| フォルダ構造 | ブックマークバーの階層をそのままディレクトリ階層へ変換 |
| ブックマーク表現 | 各ブックマーク → `<title>.url` ファイル (Windows INI 互換形式) |

**`.url` ファイルの内部構造例:**
```ini
[InternetShortcut]
URL=https://example.com
ICONINDEX=0
```

---

### F-2: ディレクトリ → ブックマークファイルへの逆変換 (エクスポート)

| 項目 | 内容 |
|------|------|
| 入力 | ローカルディレクトリ構造 (F-1 で生成したもの、または手動編集済み) |
| 出力形式 | Netscape Bookmark HTML (`bookmarks_YYYYMMDD_HHmmss.html`) |
| 対応操作 | フォルダ移動・名前変更・削除・新規追加 をすべて反映 |
| ソート | ディレクトリ内の名前順（アルファベット/Unicode順）を保持 |

---

### F-3: リンク切れチェッカー

| 項目 | 内容 |
|------|------|
| 対象 | ディレクトリ内の全 `.url` ファイル |
| 判定基準 | HTTP ステータスコード 4xx / 5xx、またはタイムアウト (デフォルト 10 秒) |
| 並列数 | デフォルト 20 並列（設定変更可） |
| 結果出力 | GUI 上のリスト表示 + CSV エクスポート |
| リダイレクト | 最終 URL を記録し、元 URL と異なる場合は "要確認" フラグ |

---

### F-4: GUI

| 項目 | 内容 |
|------|------|
| フレームワーク | Tauri v2 (Rust バックエンド + React/TypeScript フロントエンド) |
| 配布形式 | macOS: `.app` / Windows: `.exe` (インストーラ不要のポータブル版も提供) |
| 起動 | ダブルクリック一発、またはメニューバー常駐アイコン |

**主要画面:**

1. **ホーム** — ブックマークディレクトリのパス表示、インポート/エクスポートボタン
2. **ブラウザ選択ダイアログ** — Edge / Chrome / Firefox / Safari からブックマーク HTML ファイルを選択
3. **ツリービュー** — 現在のディレクトリ構造をリアルタイム表示
4. **リンク切れチェック** — チェック開始ボタン、進捗バー、結果リスト
5. **設定** — ターゲットディレクトリ・並列数・タイムアウト

---

## アーキテクチャ

```
bookmark-shelf/
├── src-tauri/          # Rust バックエンド
│   ├── src/
│   │   ├── main.rs
│   │   ├── importer.rs     # HTML パース → ディレクトリ生成
│   │   ├── exporter.rs     # ディレクトリ → Netscape HTML 生成
│   │   ├── checker.rs      # リンク切れチェック (reqwest 非同期)
│   │   └── watcher.rs      # ディレクトリ変更監視 (notify)
│   └── Cargo.toml
├── src/                # React/TypeScript フロントエンド
│   ├── App.tsx
│   ├── pages/
│   │   ├── Home.tsx
│   │   ├── TreeView.tsx
│   │   ├── LinkChecker.tsx
│   │   └── Settings.tsx
│   └── components/
├── package.json
└── DESIGN.md
```

### バックエンド (Rust) の主要クレート

| クレート | 用途 |
|----------|------|
| `tauri` | デスクトップアプリフレームワーク |
| `scraper` | Netscape Bookmark HTML パース |
| `reqwest` | HTTP リクエスト (リンクチェック) |
| `tokio` | 非同期ランタイム |
| `notify` | ファイルシステム変更監視 |
| `serde_json` | Tauri コマンドとのデータ交換 |

---

## データフロー

### インポートフロー

```
ユーザーが HTML ファイルを選択
        ↓
importer::parse_html()
  - <DL><DT><A> をツリー構造としてパース
  - フォルダ = <DT><H3> → ディレクトリ作成
  - ブックマーク = <DT><A> → <title>.url ファイル作成
        ↓
ターゲットディレクトリに書き出し
        ↓
GUI のツリービューを更新
```

### エクスポートフロー

```
ユーザーが「エクスポート」をクリック
        ↓
exporter::dir_to_html()
  - ディレクトリを再帰探索
  - フォルダ → <DT><H3>...</DL>
  - .url ファイル → <DT><A HREF="...">title</A>
        ↓
Netscape HTML ファイルとして保存
        ↓
ブラウザへインポート可能な状態
```

### リンクチェックフロー

```
checker::check_all(urls, concurrency, timeout)
        ↓
tokio::spawn × N 並列タスク
  - reqwest::get(url).await
  - ステータスコード / エラー種別を返却
        ↓
GUI へ StreamEvent で逐次結果を送信
        ↓
完了後、CSV エクスポート可能
```

---

## ファイル命名規則

| 対象 | 規則 |
|------|------|
| ブックマークタイトル | ファイルシステム禁止文字 (`/\:*?"<>|`) を `_` に置換 |
| 最大ファイル名長 | 200 文字 (macOS/Windows 共通の安全値) |
| フォルダ名 | 同上 |
| 重複 | `title (2).url`, `title (3).url` のようにサフィックスで連番 |

---

## 設定ファイル

`~/.config/bookmark-shelf/config.toml`

```toml
[general]
bookmark_dir = "~/bookmarks"

[checker]
concurrency = 20
timeout_secs = 10
follow_redirects = true
```

---

## MVP スコープ (v0.1)

- [ ] Netscape Bookmark HTML のインポート（Edge/Chrome 優先）
- [ ] ディレクトリ → Netscape HTML エクスポート
- [ ] リンク切れチェック（基本機能）
- [ ] Tauri GUI（ホーム・ブラウザ選択・エクスポート・リンクチェック画面）
- [ ] macOS ビルド

## 将来対応 (v0.2 以降)

- ブックマークディレクトリのリアルタイム変更監視
- Safari の独自形式 (`Bookmarks.plist`) 直接読み込み
- ブックマークの重複検出
- タグ管理（`.url` ファイルへのメタデータ拡張）
- Windows / Linux ビルド

---

## ライセンス

MIT

---

*Last updated: 2026-04-26*
