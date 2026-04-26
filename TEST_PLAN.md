# BookmarkShelf — Test Plan v0.1

## SLO 定義

> 対象: 個人利用からコミュニティ配布（OSS 小規模リリース相当）

| 品質軸 | 目標 | 根拠 |
|--------|------|------|
| **データ完全性** | インポート→エクスポート往復でブックマーク損失 0% | ユーザーの大切なブックマークを壊してはならない |
| **クラッシュ耐性** | 不正な HTML 入力でパニック・プロセス終了しない | 野生の Chrome/Edge/Firefox エクスポートは汚い |
| **リンクチェック精度** | HTTP 200/4xx/5xx/タイムアウトを正しく分類 | 誤検知・見逃しが多いとツールとして無意味 |
| **設定永続性** | 設定の保存→読み込みが完全一致 | 起動のたびに設定がリセットされるのは使えない |

### 明示的スコープ外

- ブラウザ固有の拡張フィールド（favicon, tags 等）の完全保持
- 10 万件超の大規模ブックマーク性能
- ネットワーク障害シナリオの完全網羅
- UI のビジュアルリグレッション
- macOS 以外のプラットフォーム（v0.1 時点）

---

## テスト戦略（3 層構造）

```
┌─────────────────────────────────────────────┐
│  Layer 3: Manual Smoke Test (release gate)  │  ← リリース前に人手で確認
├─────────────────────────────────────────────┤
│  Layer 2: Integration Test (Rust)           │  ← 往復一貫性・実ファイルI/O
├─────────────────────────────────────────────┤
│  Layer 1: Unit Test (Rust + Vitest)         │  ← 純粋関数・モジュール単体
└─────────────────────────────────────────────┘
```

### Layer 1 — ユニットテスト

| 対象 | ツール | 観点 |
|------|--------|------|
| `importer.rs` | `cargo test` | HTML パース、ファイル名サニタイズ、重複処理 |
| `exporter.rs` | `cargo test` | HTML 生成フォーマット、エスケープ、ディレクトリ再帰 |
| `checker.rs` | `cargo test` + `wiremock` | HTTP 200/404/5xx/タイムアウト分類、URL 収集 |
| `config.rs` | `cargo test` | デフォルト値、TOML 往復シリアライズ |
| Frontend utils | Vitest | Tauri コマンドのモック、型境界 |

### Layer 2 — 統合テスト

| テスト | 内容 |
|--------|------|
| `roundtrip::import_then_export` | HTML → dir → HTML で URL・タイトルが全件一致 |
| `roundtrip::nested_folders` | 3 階層フォルダ構造が正しく往復する |
| `roundtrip::dirty_titles` | 禁則文字を含むタイトルがサニタイズされて往復する |
| `roundtrip::empty_dir` | 空ディレクトリで export してもクラッシュしない |

### Layer 3 — 手動スモークテスト（リリースチェックリスト）

リリース前に以下を実機確認する。

- [ ] Edge の実エクスポート HTML をインポートできる
- [ ] Chrome の実エクスポート HTML をインポートできる
- [ ] Finder でフォルダを移動・リネームした後エクスポートして HTML が正しい
- [ ] リンクチェックで既知の 404 URL が broken と表示される
- [ ] 設定を変更→再起動して設定が保持されている
- [ ] 空のブックマークディレクトリでクラッシュしない

---

## 実行方法

```bash
# Rust ユニット + 統合テスト
cd src-tauri && cargo test

# フロントエンド ユニットテスト
npm test

# 全体（CI 相当）
npm run test:all
```

---

## カバレッジ目標

| モジュール | 行カバレッジ目標 |
|------------|-----------------|
| `importer.rs` | ≥ 80% |
| `exporter.rs` | ≥ 80% |
| `checker.rs` | ≥ 70% (HTTP 部分はモック) |
| `config.rs` | ≥ 90% |
| Frontend | スモークのみ（数値目標なし） |

> カバレッジは `cargo llvm-cov` で計測（CI には含めない、ローカル確認用）

---

## CI 方針（GitHub Actions 想定）

```yaml
# .github/workflows/test.yml のイメージ
on: [push, pull_request]
jobs:
  test:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cd src-tauri && cargo test --lib --tests
      - uses: actions/setup-node@v4
        with: { node-version: 22 }
      - run: npm ci && npm test
```

*注: Tauri の E2E（WebdriverIO）は今バージョンのスコープ外。*

---

*Last updated: 2026-04-26*
