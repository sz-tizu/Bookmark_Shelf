import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { useConfig } from "../hooks/useConfig";

interface Stats { folders: number; bookmarks: number }
type Alert = { type: "success" | "error" | "info"; msg: string } | null;

export default function Home() {
  const { config } = useConfig();
  const [alert, setAlert] = useState<Alert>(null);
  const [importing, setImporting] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [lastImport, setLastImport] = useState<Stats | null>(null);
  const [lastExport, setLastExport] = useState<Stats | null>(null);

  const bookmarkDir = config?.general.bookmark_dir ?? "~/bookmarks";

  async function handleImport() {
    const selected = await open({
      title: "ブックマーク HTML ファイルを選択",
      filters: [{ name: "Bookmark HTML", extensions: ["html", "htm"] }],
    });
    if (!selected) return;

    setImporting(true);
    setAlert(null);
    try {
      const stats = await invoke<Stats>("import_bookmarks", {
        htmlPath: selected as string,
        destDir: bookmarkDir,
      });
      setLastImport(stats);
      setAlert({ type: "success", msg: `インポート完了：フォルダ ${stats.folders} 件、ブックマーク ${stats.bookmarks} 件` });
    } catch (e) {
      setAlert({ type: "error", msg: String(e) });
    } finally {
      setImporting(false);
    }
  }

  async function handleExport() {
    const outputPath = await save({
      title: "保存先を選択",
      defaultPath: `bookmarks_${new Date().toISOString().slice(0,10)}.html`,
      filters: [{ name: "HTML", extensions: ["html"] }],
    });
    if (!outputPath) return;

    setExporting(true);
    setAlert(null);
    try {
      const stats = await invoke<Stats>("export_bookmarks", {
        srcDir: bookmarkDir,
        outputPath,
      });
      setLastExport(stats);
      setAlert({ type: "success", msg: `エクスポート完了：フォルダ ${stats.folders} 件、ブックマーク ${stats.bookmarks} 件` });
    } catch (e) {
      setAlert({ type: "error", msg: String(e) });
    } finally {
      setExporting(false);
    }
  }

  async function handleOpenFinder() {
    await invoke("open_in_finder", { path: bookmarkDir }).catch(console.error);
  }

  return (
    <>
      <h1 className="page-title">ホーム</h1>

      {alert && (
        <div className={`alert alert-${alert.type}`}>{alert.msg}</div>
      )}

      <div className="card" style={{ marginBottom: 20 }}>
        <div className="section-title">ブックマークディレクトリ</div>
        <div className="path-display">{bookmarkDir}</div>
        <div className="btn-row">
          <button className="btn-ghost" onClick={handleOpenFinder}>Finder で開く</button>
        </div>
      </div>

      <div className="grid-2" style={{ marginBottom: 24 }}>
        <div className="card">
          <div className="section-title">インポート</div>
          <p style={{ color: "var(--text-muted)", marginBottom: 12, lineHeight: 1.6 }}>
            Edge / Chrome / Firefox / Safari からエクスポートした HTML ファイルを選択してディレクトリに変換します。
          </p>
          <button className="btn-primary" onClick={handleImport} disabled={importing}>
            {importing && <span className="spinner" />}
            {importing ? "インポート中..." : "📥 ブックマークをインポート"}
          </button>
          {lastImport && (
            <p style={{ marginTop: 12, fontSize: 12, color: "var(--text-muted)" }}>
              前回: フォルダ {lastImport.folders} / ブックマーク {lastImport.bookmarks}
            </p>
          )}
        </div>

        <div className="card">
          <div className="section-title">エクスポート</div>
          <p style={{ color: "var(--text-muted)", marginBottom: 12, lineHeight: 1.6 }}>
            現在のディレクトリ構造を Netscape Bookmark HTML に変換します。ブラウザに直接インポートできます。
          </p>
          <button className="btn-secondary" onClick={handleExport} disabled={exporting}>
            {exporting && <span className="spinner" />}
            {exporting ? "エクスポート中..." : "📤 HTML にエクスポート"}
          </button>
          {lastExport && (
            <p style={{ marginTop: 12, fontSize: 12, color: "var(--text-muted)" }}>
              前回: フォルダ {lastExport.folders} / ブックマーク {lastExport.bookmarks}
            </p>
          )}
        </div>
      </div>

      <div className="card">
        <div className="section-title">対応ブラウザ</div>
        <div style={{ display: "flex", gap: 16, flexWrap: "wrap", marginTop: 8 }}>
          {["Edge", "Chrome", "Firefox", "Safari"].map((b) => (
            <div key={b} style={{ background: "var(--surface2)", borderRadius: 6, padding: "6px 14px", fontSize: 13 }}>
              {b}
            </div>
          ))}
        </div>
        <p style={{ marginTop: 12, color: "var(--text-muted)", fontSize: 12 }}>
          各ブラウザの「ブックマークのエクスポート」機能で書き出した HTML ファイルを使用してください。
        </p>
      </div>
    </>
  );
}
