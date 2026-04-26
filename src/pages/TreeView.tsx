import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useConfig } from "../hooks/useConfig";

interface TreeNode {
  name: string;
  path: string;
  type: "folder" | "bookmark";
  url?: string;
  children?: TreeNode[];
}

function TreeNodeItem({ node, depth = 0 }: { node: TreeNode; depth?: number }) {
  const [open, setOpen] = useState(depth < 2);

  if (node.type === "folder") {
    return (
      <li className="tree-item">
        <div className="tree-folder" onClick={() => setOpen(!open)}>
          <span>{open ? "📂" : "📁"}</span>
          <span>{node.name}</span>
          {node.children && (
            <span style={{ fontSize: 11, color: "var(--text-muted)", marginLeft: "auto" }}>
              {node.children.length}
            </span>
          )}
        </div>
        {open && node.children && node.children.length > 0 && (
          <div className="tree-children">
            <ul className="tree">
              {node.children.map((child, i) => (
                <TreeNodeItem key={i} node={child} depth={depth + 1} />
              ))}
            </ul>
          </div>
        )}
      </li>
    );
  }

  return (
    <li className="tree-item">
      <div className="tree-bookmark">
        <span>🔗</span>
        <a
          href={node.url}
          target="_blank"
          rel="noreferrer"
          title={node.url}
          style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}
        >
          {node.name}
        </a>
      </div>
    </li>
  );
}

export default function TreeView() {
  const { config } = useConfig();
  const [tree, setTree] = useState<TreeNode | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const bookmarkDir = config?.general.bookmark_dir;

  async function refresh() {
    if (!bookmarkDir) return;
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<TreeNode>("read_dir_tree", { dir: bookmarkDir });
      setTree(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    if (bookmarkDir) refresh();
  }, [bookmarkDir]);

  const countAll = (node: TreeNode | null): { folders: number; bookmarks: number } => {
    if (!node) return { folders: 0, bookmarks: 0 };
    if (node.type === "bookmark") return { folders: 0, bookmarks: 1 };
    const child = (node.children ?? []).reduce(
      (acc, c) => {
        const r = countAll(c);
        return { folders: acc.folders + r.folders, bookmarks: acc.bookmarks + r.bookmarks };
      },
      { folders: 0, bookmarks: 0 }
    );
    return { folders: child.folders + 1, bookmarks: child.bookmarks };
  };

  const counts = countAll(tree);

  return (
    <>
      <h1 className="page-title">Tree View</h1>

      <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 20 }}>
        <button className="btn-secondary" onClick={refresh} disabled={loading}>
          {loading ? <><span className="spinner" />読み込み中...</> : "🔄 更新"}
        </button>
        {tree && (
          <span style={{ color: "var(--text-muted)", fontSize: 13 }}>
            フォルダ {counts.folders} / ブックマーク {counts.bookmarks}
          </span>
        )}
      </div>

      {error && <div className="alert alert-error">{error}</div>}

      {!tree && !loading && !error && (
        <div className="empty-state">
          <div className="icon">🌳</div>
          <p>ブックマークディレクトリが見つかりません。<br />先にインポートを実行してください。</p>
        </div>
      )}

      {tree && (
        <div className="card">
          <ul className="tree">
            {(tree.children ?? []).map((child, i) => (
              <TreeNodeItem key={i} node={child} depth={0} />
            ))}
          </ul>
        </div>
      )}
    </>
  );
}
