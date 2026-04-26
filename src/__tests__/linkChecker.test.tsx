import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { vi, describe, it, expect, beforeEach } from "vitest";
import { HashRouter } from "react-router-dom";
import LinkChecker from "../pages/LinkChecker";
import { useConfig } from "../hooks/useConfig";

vi.mock("../hooks/useConfig");

const mockInvoke = vi.mocked(invoke);
const mockUseConfig = vi.mocked(useConfig);

const BASE_CONFIG = {
  general: { bookmark_dir: "/tmp/bookmarks" },
  checker: { concurrency: 5, timeout_secs: 5, follow_redirects: true },
};

function renderLinkChecker() {
  return render(
    <HashRouter>
      <LinkChecker />
    </HashRouter>
  );
}

describe("LinkChecker page", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseConfig.mockReturnValue({ config: BASE_CONFIG, save: vi.fn() });
  });

  const getCheckButton = () => screen.getByRole("button", { name: /リンクチェック開始/ });

  it("renders check button", () => {
    renderLinkChecker();
    expect(getCheckButton()).toBeInTheDocument();
  });

  it("shows empty state before any check", () => {
    renderLinkChecker();
    expect(getCheckButton()).toBeInTheDocument();
    expect(screen.queryByRole("table")).not.toBeInTheDocument();
  });

  it("calls check_links command with correct args on click", async () => {
    mockInvoke.mockResolvedValueOnce([]);
    renderLinkChecker();

    fireEvent.click(getCheckButton());

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("check_links", {
        dir: "/tmp/bookmarks",
        concurrency: 5,
        timeoutSecs: 5,
      });
    });
  });

  it("renders result rows after check completes", async () => {
    const results = [
      { title: "Rust", url: "https://rust-lang.org", status: "ok", final_url: null, error: null },
      { title: "Gone", url: "https://gone.example.com", status: "broken", final_url: null, error: "HTTP 404" },
    ];
    mockInvoke.mockResolvedValueOnce(results);
    renderLinkChecker();

    fireEvent.click(getCheckButton());

    await waitFor(() => {
      expect(screen.getByText("Rust")).toBeInTheDocument();
      expect(screen.getByText("Gone")).toBeInTheDocument();
    });
  });

  it("shows OK badge for successful URLs", async () => {
    mockInvoke.mockResolvedValueOnce([
      { title: "Site", url: "https://ok.example.com", status: "ok", final_url: null, error: null },
    ]);
    renderLinkChecker();
    fireEvent.click(getCheckButton());

    await waitFor(() => {
      // The badge element specifically (not the stat-card label)
      const badges = screen.getAllByText("OK");
      expect(badges.some((el) => el.classList.contains("badge"))).toBe(true);
    });
  });

  it("shows リンク切れ badge for 4xx/5xx URLs", async () => {
    mockInvoke.mockResolvedValueOnce([
      { title: "Dead", url: "https://dead.example.com", status: "broken", final_url: null, error: "HTTP 404" },
    ]);
    renderLinkChecker();
    fireEvent.click(getCheckButton());

    await waitFor(() => {
      const badges = screen.getAllByText("リンク切れ");
      expect(badges.some((el) => el.classList.contains("badge"))).toBe(true);
    });
  });
});
