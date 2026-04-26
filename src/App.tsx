import { NavLink, Route, Routes } from "react-router-dom";
import Home from "./pages/Home";
import TreeView from "./pages/TreeView";
import LinkChecker from "./pages/LinkChecker";
import Settings from "./pages/Settings";

const nav = [
  { to: "/", label: "Home", icon: "🏠" },
  { to: "/tree", label: "Tree View", icon: "🌳" },
  { to: "/checker", label: "Link Checker", icon: "🔍" },
  { to: "/settings", label: "Settings", icon: "⚙️" },
];

export default function App() {
  return (
    <div className="layout">
      <aside className="sidebar">
        <div className="sidebar-logo">📚 BookmarkShelf</div>
        <ul className="sidebar-nav">
          {nav.map((n) => (
            <li key={n.to}>
              <NavLink
                to={n.to}
                end={n.to === "/"}
                className={({ isActive }) => (isActive ? "active" : "")}
              >
                <span>{n.icon}</span>
                <span>{n.label}</span>
              </NavLink>
            </li>
          ))}
        </ul>
      </aside>
      <main className="main-content">
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/tree" element={<TreeView />} />
          <Route path="/checker" element={<LinkChecker />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
      </main>
    </div>
  );
}
