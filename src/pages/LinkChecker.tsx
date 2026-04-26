import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { useConfig } from "../hooks/useConfig";

type CheckStatus = "ok" | "redirect" | "broken" | "timeout" | "error";

interface CheckResult {
  title: string;
  url: string;
  status: CheckStatus;
  final_url?: string;
  error?: string;
}

const STATUS_LABEL: Record<CheckStatus, { label: string; cls: string }> = {
  ok: { label: "OK", cls: "badge-ok" },
  redirect: { label: "リダイレクト", cls: "badge-warn" },
  broken: { label: "リンク切れ", cls: "badge-err" },
  timeout: { label: "タイムアウト", cls: "badge-err" },
  error: { label: "エラー", cls: "badge-err" },
};

export default function LinkChecker() {
  const { config } = useConfig();
  const [results, setResults] = useState<CheckResult[]>([]);
  const [checking, setChecking] = useState(false);
  const [filter, setFilter] = useState<CheckStatus | "all">("all");

  const bookmarkDir = config?.general.bookmark_dir ?? "";
  const concurrency = config?.checker.concurrency ?? 20;
  const timeoutSecs = config?.checker.timeout_secs ?? 10;

  async function handleCheck() {
    setChecking(true);
    setResults([]);
    try {
      const res = await invoke<CheckResult[]>("check_links", {
        dir: bookmarkDir,
        concurrency,
        timeoutSecs,
      });
      setResults(res);
    } catch (e) {
      console.error(e);
    } finally {
      setChecking(false);
    }
  }

  async function handleExportCsv() {
    const path = await save({
      title: "CSV として保存",
      defaultPath: `link_check_${new Date().toISOString().slice(0, 10)}.csv`,
      filters: [{ name: "CSV", extensions: ["csv"] }],
    });
    if (!path) return;

    const rows = [
      ["Title", "URL", "Status", "Final URL", "Error"],
      ...results.map((r) => [
        r.title,
        r.url,
        r.status,
        r.final_url ?? "",
        r.error ?? "",
      ]),
    ];
    const csv = rows.map((r) => r.map((v) => `"${v.replace(/"/g, '""')}"`).join(",")).join("\n");
    await invoke("save_config", {}).catch(() => {}); // just to satisfy no-op; use fs plugin instead
    const { writeTextFile } = await import("@tauri-apps/plugin-fs");
    await writeTextFile(path, csv);
  }

  const filtered = filter === "all" ? results : results.filter((r) => r.status === filter);

  const counts = results.reduce(
    (acc, r) => ({ ...acc, [r.status]: (acc[r.status as keyof typeof acc] ?? 0) + 1 }),
    { ok: 0, redirect: 0, broken: 0, timeout: 0, error: 0 }
  );

  return (
    <>
      <h1 className="page-title">Link Checker</h1>

      <div className="card" style={{ marginBottom: 20 }}>
        <div style={{ display: "flex", gap: 12, alignItems: "center", flexWrap: "wrap" }}>
          <button className="btn-primary" onClick={handleCheck} disabled={checking || !bookmarkDir}>
            {checking ? <><span className="spinner" />チェック中...</> : "🔍 リンクチェック開始"}
          </button>
          {results.length > 0 && (
            <button className="btn-ghost" onClick={handleExportCsv}>📋 CSV でエクスポート</button>
          )}
          <span style={{ color: "var(--text-muted)", fontSize: 12 }}>
            並列数: {concurrency} / タイムアウト: {timeoutSecs}s
          </span>
        </div>
      </div>

      {results.length > 0 && (
        <>
          <div className="grid-2" style={{ marginBottom: 20 }}>
            {(["ok", "redirect", "broken", "error"] as const).map((s) => (
              <div key={s} className="stat-card" style={{ cursor: "pointer" }} onClick={() => setFilter(s)}>
                <div className="stat-value" style={{ fontSize: 24 }}>{counts[s]}</div>
                <div className="stat-label">{STATUS_LABEL[s].label}</div>
              </div>
            ))}
          </div>

          <div style={{ display: "flex", gap: 8, marginBottom: 12 }}>
            {(["all", "ok", "redirect", "broken", "timeout", "error"] as const).map((s) => (
              <button
                key={s}
                className={filter === s ? "btn-primary" : "btn-ghost"}
                style={{ padding: "5px 12px", fontSize: 12 }}
                onClick={() => setFilter(s)}
              >
                {s === "all" ? `すべて (${results.length})` : `${STATUS_LABEL[s].label} (${counts[s as CheckStatus] ?? 0})`}
              </button>
            ))}
          </div>

          <div className="card" style={{ padding: 0, overflow: "hidden" }}>
            <table className="result-table">
              <thead>
                <tr>
                  <th>タイトル</th>
                  <th>URL</th>
                  <th>ステータス</th>
                  <th>詳細</th>
                </tr>
              </thead>
              <tbody>
                {filtered.map((r, i) => (
                  <tr key={i}>
                    <td style={{ maxWidth: 200, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                      {r.title}
                    </td>
                    <td>
                      <a href={r.url} target="_blank" rel="noreferrer" className="url-cell" title={r.url}>
                        {r.url}
                      </a>
                    </td>
                    <td>
                      <span className={`badge ${STATUS_LABEL[r.status].cls}`}>
                        {STATUS_LABEL[r.status].label}
                      </span>
                    </td>
                    <td style={{ fontSize: 12, color: "var(--text-muted)" }}>
                      {r.final_url ? (
                        <a href={r.final_url} target="_blank" rel="noreferrer" className="url-cell" title={r.final_url}>
                          → {r.final_url}
                        </a>
                      ) : r.error ?? ""}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </>
      )}

      {!checking && results.length === 0 && (
        <div className="empty-state">
          <div className="icon">🔍</div>
          <p>「リンクチェック開始」ボタンを押すと、ブックマークディレクトリ内の全リンクをチェックします。</p>
        </div>
      )}
    </>
  );
}
