import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { useConfig, Config } from "../hooks/useConfig";

export default function Settings() {
  const { config, save } = useConfig();
  const [form, setForm] = useState<Config | null>(null);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    if (config) setForm(structuredClone(config));
  }, [config]);

  if (!form) return <p style={{ color: "var(--text-muted)" }}>読み込み中...</p>;

  const update = (path: string[], value: unknown) => {
    setForm((prev) => {
      if (!prev) return prev;
      const next = structuredClone(prev) as unknown as Record<string, unknown>;
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      let cur: any = next;
      for (let i = 0; i < path.length - 1; i++) cur = cur[path[i]];
      cur[path[path.length - 1]] = value;
      return next as unknown as Config;
    });
    setSaved(false);
  };

  async function selectDir() {
    const selected = await open({ directory: true, title: "ブックマークディレクトリを選択" });
    if (selected) update(["general", "bookmark_dir"], selected as string);
  }

  async function handleSave() {
    if (!form) return;
    await save(form);
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  }

  return (
    <>
      <h1 className="page-title">Settings</h1>

      {saved && <div className="alert alert-success">設定を保存しました</div>}

      <div className="card" style={{ marginBottom: 20 }}>
        <div className="section-title">一般</div>

        <div className="form-group">
          <label>ブックマークディレクトリ</label>
          <div style={{ display: "flex", gap: 8 }}>
            <input
              value={form.general.bookmark_dir}
              onChange={(e) => update(["general", "bookmark_dir"], e.target.value)}
            />
            <button className="btn-ghost" style={{ whiteSpace: "nowrap" }} onClick={selectDir}>
              選択
            </button>
          </div>
        </div>
      </div>

      <div className="card" style={{ marginBottom: 20 }}>
        <div className="section-title">リンクチェッカー</div>

        <div className="form-group">
          <label>並列チェック数</label>
          <input
            type="number"
            min={1}
            max={100}
            value={form.checker.concurrency}
            onChange={(e) => update(["checker", "concurrency"], parseInt(e.target.value) || 1)}
          />
        </div>

        <div className="form-group">
          <label>タイムアウト (秒)</label>
          <input
            type="number"
            min={1}
            max={120}
            value={form.checker.timeout_secs}
            onChange={(e) => update(["checker", "timeout_secs"], parseInt(e.target.value) || 10)}
          />
        </div>

        <div className="form-group">
          <label style={{ display: "flex", alignItems: "center", gap: 8, cursor: "pointer" }}>
            <input
              type="checkbox"
              style={{ width: "auto" }}
              checked={form.checker.follow_redirects}
              onChange={(e) => update(["checker", "follow_redirects"], e.target.checked)}
            />
            リダイレクトを追跡する
          </label>
        </div>
      </div>

      <button className="btn-primary" onClick={handleSave}>
        💾 設定を保存
      </button>
    </>
  );
}
